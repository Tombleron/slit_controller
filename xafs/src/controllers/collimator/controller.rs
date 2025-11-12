use std::sync::Arc;

use motarem::{axis::Axis, motor_controller::MotorController};

pub struct CollimatorController {
    axes: Vec<Arc<dyn Axis>>,
}

impl CollimatorController {
    pub fn new() -> Self {
        Self { axes: Vec::new() }
    }

    pub fn add_axis(&mut self, axis: Arc<dyn Axis>) {
        self.axes.push(axis);
    }
}

#[async_trait::async_trait]
impl MotorController for CollimatorController {
    fn name(&self) -> &str {
        "CollimatorController"
    }

    fn axes(&self) -> Vec<Arc<dyn Axis>> {
        self.axes.clone()
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        for axis in &self.axes {
            axis.stop().await?;
        }

        Ok(())
    }
}
