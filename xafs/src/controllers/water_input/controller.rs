use std::sync::Arc;

use motarem::{axis::Axis, motor_controller::MotorController};

pub struct WaterInputController {
    axis: Arc<dyn Axis>,
}

impl WaterInputController {
    pub fn new(axis: Arc<dyn Axis>) -> Self {
        Self { axis }
    }
}

#[async_trait::async_trait]
impl MotorController for WaterInputController {
    fn name(&self) -> &str {
        "WaterInputController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        vec![self.axis.clone()]
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.axis.stop().await?;

        Ok(())
    }
}
