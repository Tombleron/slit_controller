use std::sync::Arc;

use crate::command_executor::motor::command_sender::Em2rsCommandSender;
use crate::controllers::SensorsCommandSender;
use crate::controllers::cooled_slit::config::CooledSlitControllerConfig;
use crate::controllers::cooled_slit::{axis::CooledSlitAxis, controller::CooledSlitController};

pub mod axis;
pub mod config;
pub mod controller;
pub mod motor;
pub mod params;

// The order of TRIDs and EM2RS matters since Controller will have logic to determine which sensors axis to use.
//
// TRID
// 1-4 is for Slit knifes temperature
// 5-8 is for water output of knifes cooling
// 9 is for water cooling input
// 10 is for receiver temperature
//
// EM2RS
// 1-4 is for Slit control
// 5 is for atenuator control
pub fn create_controller(
    config: &CooledSlitControllerConfig,
    em2rs_command_sender: Em2rsCommandSender,
    sensors_command_sender: SensorsCommandSender,
) -> CooledSlitController {
    let upper_axis = CooledSlitAxis::new(
        "Y_Up".to_string(),
        0,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
        config.upper_axis.steps_per_mm,
    );
    let lower_axis = CooledSlitAxis::new(
        "Y_Down".to_string(),
        1,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
        config.lower_axis.steps_per_mm,
    );
    let left_axis = CooledSlitAxis::new(
        "X_Left".to_string(),
        2,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
        config.left_axis.steps_per_mm,
    );
    let right_axis = CooledSlitAxis::new(
        "X_Right".to_string(),
        3,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
        config.right_axis.steps_per_mm,
    );

    let mut controller = CooledSlitController::new();
    controller.add_axis(Arc::new(upper_axis));
    controller.add_axis(Arc::new(lower_axis));
    controller.add_axis(Arc::new(left_axis));
    controller.add_axis(Arc::new(right_axis));

    controller
}
