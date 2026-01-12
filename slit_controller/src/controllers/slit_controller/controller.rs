use std::{io, sync::Arc};

use motarem::{axis::Axis, motor_controller::MotorController};
use tokio::{sync::Mutex, task::JoinHandle};
use utilities::command_executor::CommandExecutor;

use crate::command_executor::{
    encoder::Rf256Handler, motor::StandaHandler, temperature::TridHandler,
};

pub struct SlitController {
    axes: Vec<Arc<dyn Axis>>,

    rf256_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
    trid_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
    standas_join_handlers: Arc<Mutex<Vec<JoinHandle<io::Result<()>>>>>,
}

impl SlitController {
    pub fn new(
        mut rf256_command_executor: CommandExecutor<Rf256Handler>,
        mut trid_command_executor: CommandExecutor<TridHandler>,
        standa_command_executors: Vec<CommandExecutor<StandaHandler>>,
    ) -> Self {
        let rf256_handle = tokio::task::spawn_blocking(move || rf256_command_executor.run());
        let trid_handle = tokio::task::spawn_blocking(move || trid_command_executor.run());
        let standas_handles = standa_command_executors
            .into_iter()
            .map(|mut executor| tokio::task::spawn_blocking(move || executor.run()))
            .collect();

        Self {
            axes: Vec::new(),
            rf256_join_handle: Arc::new(Mutex::new(rf256_handle)),
            trid_join_handle: Arc::new(Mutex::new(trid_handle)),
            standas_join_handlers: Arc::new(Mutex::new(standas_handles)),
        }
    }

    pub fn add_axis(&mut self, axis: Arc<dyn Axis>) {
        self.axes.push(axis);
    }
}

#[async_trait::async_trait]
impl MotorController for SlitController {
    fn name(&self) -> &str {
        "SlitController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        self.axes.clone()
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        for axis in self.axes() {
            axis.stop().await?;
        }

        // FIXME: Is it really ok?
        let rf256_handle = self.rf256_join_handle.lock().await;
        rf256_handle.abort();
        let trid_handle = self.trid_join_handle.lock().await;
        trid_handle.abort();
        let standas_handles = self.standas_join_handlers.lock().await;
        for handle in &*standas_handles {
            handle.abort();
        }

        Ok(())
    }
}
