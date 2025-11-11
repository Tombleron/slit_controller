pub mod axis;
pub mod config;
pub mod controller;
pub mod motor;
pub mod params;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use config::CooledSlitControllerConfig;
use em2rs::Em2rs;
use icpcon::M7015;
use lir::LIR;
use utilities::{command_executor::CommandExecutor, lazy_tcp::LazyTcpStream};

use crate::{
    command_executor::{
        motor::{Em2rsHandler, command_sender::Em2rsCommandSender},
        sensors::{SensorsHandler, command_sender::SensorsCommandSender},
    },
    controllers::cooled_slit::{axis::CooledSlitAxis, controller::CooledSlitController},
};

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

pub fn create_sensors(
    config: &CooledSlitControllerConfig,
) -> (CommandExecutor<SensorsHandler>, SensorsCommandSender) {
    let sensors_scoket_addr =
        SocketAddr::new(config.sensors_ip.parse().unwrap(), config.sensors_port);

    let sensors_tcp_stream = LazyTcpStream::new(
        sensors_scoket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let sensors_handler = SensorsHandler::new(
        sensors_tcp_stream,
        vec![
            LIR::new(config.upper_axis.lir_id, config.upper_axis.lir_step),
            LIR::new(config.lower_axis.lir_id, config.lower_axis.lir_step),
            LIR::new(config.right_axis.lir_id, config.right_axis.lir_step),
            LIR::new(config.left_axis.lir_id, config.left_axis.lir_step),
        ],
        M7015::new(config.icpcon_id),
    );

    let sensors_command_executor = CommandExecutor::new(sensors_handler);
    let sensors_command_sender = SensorsCommandSender::new(sensors_command_executor.sender());

    (sensors_command_executor, sensors_command_sender)
}

pub fn create_em2rs(
    config: &CooledSlitControllerConfig,
) -> (CommandExecutor<Em2rsHandler>, Em2rsCommandSender) {
    let em2rs_socket_addr = SocketAddr::new(config.em2rs_ip.parse().unwrap(), config.em2rs_port);
    let em2rs_tcp_stream = LazyTcpStream::new(
        em2rs_socket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let em2rs_handler = Em2rsHandler::new(
        em2rs_tcp_stream,
        [
            Em2rs::new(
                config.upper_axis.em2rs_id,
                config.upper_axis.em2rs_low_limit,
                config.upper_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.lower_axis.em2rs_id,
                config.lower_axis.em2rs_low_limit,
                config.lower_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.right_axis.em2rs_id,
                config.right_axis.em2rs_low_limit,
                config.right_axis.em2rs_high_limit,
            ),
            Em2rs::new(
                config.left_axis.em2rs_id,
                config.left_axis.em2rs_low_limit,
                config.left_axis.em2rs_high_limit,
            ),
        ],
    );

    let em2rs_command_executor = CommandExecutor::new(em2rs_handler);
    let em2rs_command_sender = Em2rsCommandSender::new(em2rs_command_executor.sender());

    (em2rs_command_executor, em2rs_command_sender)
}

pub fn create_controller(config: &CooledSlitControllerConfig) -> CooledSlitController {
    let (em2rs_command_executor, em2rs_command_sender) = create_em2rs(config);
    let (sensors_command_executor, sensors_command_sender) = create_sensors(config);

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

    let mut controller = CooledSlitController::new(
        // vec![
        //     Arc::new(upper_axis),
        //     Arc::new(lower_axis),
        //     Arc::new(left_axis),
        //     Arc::new(right_axis),
        // ],
        sensors_command_executor,
        em2rs_command_executor,
    );
    controller.add_axis(Arc::new(upper_axis));
    controller.add_axis(Arc::new(lower_axis));
    controller.add_axis(Arc::new(left_axis));
    controller.add_axis(Arc::new(right_axis));

    controller
}
