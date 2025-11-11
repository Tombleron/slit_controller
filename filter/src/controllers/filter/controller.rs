use std::{io, sync::Arc};

use motarem::{axis::Axis, motor_controller::MotorController};
use tokio::{sync::Mutex, task::JoinHandle};
use utilities::command_executor::CommandExecutor;

use crate::command_executor::{encoder::EncoderHandler, motor::Em2rsHandler};

pub struct FilterController {
    axis: Arc<dyn Axis>,

    sensors_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
    em2rs_join_handle: Arc<Mutex<JoinHandle<io::Result<()>>>>,
}

impl FilterController {
    pub fn new(
        axis: Arc<dyn Axis>,

        mut sensors_command_executor: CommandExecutor<EncoderHandler>,
        mut em2rs_command_executor: CommandExecutor<Em2rsHandler>,
    ) -> Self {
        let sensors_handle = tokio::task::spawn_blocking(move || sensors_command_executor.run());
        let em2rs_handle = tokio::task::spawn_blocking(move || em2rs_command_executor.run());

        Self {
            axis,
            sensors_join_handle: Arc::new(Mutex::new(sensors_handle)),
            em2rs_join_handle: Arc::new(Mutex::new(em2rs_handle)),
        }
    }
}

#[async_trait::async_trait]
impl MotorController for FilterController {
    fn name(&self) -> &str {
        "FilterController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        vec![self.axis.clone()]
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
