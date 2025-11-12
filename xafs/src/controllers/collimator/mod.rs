use std::sync::Arc;

use crate::{
    command_executor::sensors::command_sender::SensorsCommandSender,
    controllers::collimator::{axis::CollimatorAxis, controller::CollimatorController},
};

pub mod axis;
pub mod config;
pub mod controller;

pub fn create_controller(
    // config: CollimatorControllerConfig,
    sensors_command_sender: SensorsCommandSender,
) -> CollimatorController {
    let axis1 = CollimatorAxis::new(
        "TemperatureInput".to_string(),
        9,
        sensors_command_sender.clone(),
    );
    let axis2 = CollimatorAxis::new("TemperatureOutput".to_string(), 10, sensors_command_sender);

    let mut controller = CollimatorController::new();
    controller.add_axis(Arc::new(axis1));
    controller.add_axis(Arc::new(axis2));

    controller
}
