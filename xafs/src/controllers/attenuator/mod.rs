use std::sync::Arc;

use crate::{
    command_executor::{
        motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
    },
    controllers::attenuator::{
        axis::AttenuatorAxis, config::AttenuatorControllerConfig, controller::AttenuatorController,
    },
};

pub mod axis;
pub mod config;
pub mod controller;
pub mod motor;
pub mod params;

pub fn create_controller(
    config: &AttenuatorControllerConfig,
    em2rs_command_sender: Em2rsCommandSender,
    sensors_command_sender: SensorsCommandSender,
) -> AttenuatorController {
    let axis = AttenuatorAxis::new(
        "AttenuatorAxis".to_string(),
        5,
        sensors_command_sender,
        em2rs_command_sender,
        config.axis.steps_per_mm,
    );

    AttenuatorController::new(Arc::new(axis))
}
