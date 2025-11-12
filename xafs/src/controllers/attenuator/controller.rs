use std::sync::Arc;

use motarem::{axis::Axis, motor_controller::MotorController};

pub struct AttenuatorController {
    axis: Arc<dyn Axis>,
}

impl AttenuatorController {
    pub fn new(axis: Arc<dyn Axis>) -> Self {
        Self { axis }
    }
}

#[async_trait::async_trait]
impl MotorController for AttenuatorController {
    fn name(&self) -> &str {
        "AttenuatorController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        vec![self.axis.clone()]
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        self.axis.stop().await?;

        Ok(())
    }
}
