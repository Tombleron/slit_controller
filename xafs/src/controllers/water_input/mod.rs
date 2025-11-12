use std::sync::Arc;

use crate::{
    command_executor::sensors::command_sender::SensorsCommandSender,
    controllers::water_input::{axis::WaterInputAxis, controller::WaterInputController},
};

pub mod axis;
pub mod config;
pub mod controller;

pub fn create_controller(
    // config: &WaterInputControllerConfig,
    sensors_command_sender: SensorsCommandSender,
) -> WaterInputController {
    let axis = WaterInputAxis::new("Temperature".to_string(), 8, sensors_command_sender);

    let controller = WaterInputController::new(Arc::new(axis));

    controller
}
