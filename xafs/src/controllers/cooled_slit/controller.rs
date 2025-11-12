use std::sync::Arc;

use motarem::{axis::Axis, motor_controller::MotorController};

pub struct CooledSlitController {
    axes: Vec<Arc<dyn Axis>>,
}

impl CooledSlitController {
    pub fn new() -> Self {
        Self { axes: Vec::new() }
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

        Ok(())
    }
}
