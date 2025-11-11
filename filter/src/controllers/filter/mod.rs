pub mod axis;
pub mod config;
pub mod controller;
pub mod motor;
pub mod params;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use config::FilterControllerConfig;
use em2rs::Em2rs;
use lir::LIR;
use utilities::{command_executor::CommandExecutor, lazy_tcp::LazyTcpStream};

use crate::{
    command_executor::{
        encoder::{EncoderHandler, command_sender::EncoderCommandSender},
        motor::{Em2rsHandler, command_sender::Em2rsCommandSender},
    },
    controllers::filter::{axis::FilterAxis, controller::FilterController},
};

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

pub fn create_sensors(
    config: &FilterControllerConfig,
) -> (CommandExecutor<EncoderHandler>, EncoderCommandSender) {
    let sensors_scoket_addr =
        SocketAddr::new(config.encoder_ip.parse().unwrap(), config.encoder_port);

    let sensors_tcp_stream = LazyTcpStream::new(
        sensors_scoket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let sensors_handler =
        EncoderHandler::new(sensors_tcp_stream, LIR::new(config.lir_id, config.lir_step));

    let sensors_command_executor = CommandExecutor::new(sensors_handler);
    let sensors_command_sender = EncoderCommandSender::new(sensors_command_executor.sender());

    (sensors_command_executor, sensors_command_sender)
}

pub fn create_em2rs(
    config: &FilterControllerConfig,
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
        Em2rs::new(
            config.em2rs_id,
            config.em2rs_low_limit,
            config.em2rs_high_limit,
        ),
    );

    let em2rs_command_executor = CommandExecutor::new(em2rs_handler);
    let em2rs_command_sender = Em2rsCommandSender::new(em2rs_command_executor.sender());

    (em2rs_command_executor, em2rs_command_sender)
}

pub fn create_controller(config: &FilterControllerConfig) -> FilterController {
    let (em2rs_command_executor, em2rs_command_sender) = create_em2rs(config);
    let (sensors_command_executor, sensors_command_sender) = create_sensors(config);

    let axis = FilterAxis::new(
        "Rotational".to_string(),
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
        config.steps_per_mm,
    );

    let controller = FilterController::new(
        Arc::new(axis),
        sensors_command_executor,
        em2rs_command_executor,
    );

    controller
}
