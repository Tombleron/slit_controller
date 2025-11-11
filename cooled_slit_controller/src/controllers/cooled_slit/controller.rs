use std::{io, sync::Arc};

use motarem::{axis::Axis, motor_controller::MotorController};
use tokio::{sync::Mutex, task::JoinHandle};
use utilities::command_executor::CommandExecutor;

use crate::command_executor::{motor::Em2rsHandler, sensors::SensorsHandler};

pub struct CooledSlitController {
    axes: Vec<Arc<dyn Axis>>,

    sensors_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
    em2rs_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
}

impl CooledSlitController {
    pub fn new(
        // axes: Vec<Arc<dyn Axis>>,
        mut sensors_command_executor: CommandExecutor<SensorsHandler>,
        mut em2rs_command_executor: CommandExecutor<Em2rsHandler>,
    ) -> Self {
        let sensors_handle = tokio::task::spawn_blocking(move || sensors_command_executor.run());
        let em2rs_handle = tokio::task::spawn_blocking(move || em2rs_command_executor.run());

        Self {
            axes: Vec::new(),
            sensors_join_handle: Arc::new(Mutex::new(sensors_handle)),
            em2rs_join_handle: Arc::new(Mutex::new(em2rs_handle)),
        }
    }

    pub fn add_axis(&mut self, axis: Arc<dyn Axis>) {
        self.axes.push(axis);
    }
}

#[async_trait::async_trait]
impl MotorController for CooledSlitController {
    fn name(&self) -> &str {
        "CooledSlitController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        self.axes.clone()
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        for axis in self.axes() {
            axis.stop().await?;
        }

        // FIXME: Is it really ok?
        let sensors_handle = self.sensors_join_handle.lock().await;
        sensors_handle.abort();
        let em2rs_handle = self.em2rs_join_handle.lock().await;
        em2rs_handle.abort();

        Ok(())
    }
}
