use std::{net::SocketAddr, sync::Arc, time::Duration};

use rf256::Rf256;
use standa::Standa;
use trid::Trid;
use utilities::{command_executor::CommandExecutor, lazy_tcp::LazyTcpStream};

use crate::{
    command_executor::{
        encoder::{command_sender::EncoderCommandSender, Rf256Handler},
        motor::{command_sender::StandaCommandSender, StandaHandler},
        temperature::{command_sender::TridCommandSender, TridHandler},
    },
    controllers::slit_controller::{
        axis::SlitAxis, config::SlitControllerConfig, controller::SlitController,
    },
};

pub mod axis;
pub mod config;
pub mod controller;
pub mod motor;
pub mod params;

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

pub fn create_encoder(
    config: &SlitControllerConfig,
) -> (CommandExecutor<Rf256Handler>, EncoderCommandSender) {
    let rf256_scoket_addr = SocketAddr::new(config.rf256_ip.parse().unwrap(), config.rf256_port);

    let rf256_tcp_stream = LazyTcpStream::new(
        rf256_scoket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let rf256_handler = Rf256Handler::new(
        rf256_tcp_stream,
        [
            Rf256::new(config.upper_axis.rf256_id),
            Rf256::new(config.lower_axis.rf256_id),
            Rf256::new(config.right_axis.rf256_id),
            Rf256::new(config.left_axis.rf256_id),
        ],
    );

    let rf256_command_executor = CommandExecutor::new(rf256_handler);
    let rf256_command_sender = EncoderCommandSender::new(rf256_command_executor.sender());

    (rf256_command_executor, rf256_command_sender)
}

pub fn create_trid(
    config: &SlitControllerConfig,
) -> (CommandExecutor<TridHandler>, TridCommandSender) {
    let trid_socket_addr = SocketAddr::new(config.trid_ip.parse().unwrap(), config.trid_port);

    let trid_tcp_stream = LazyTcpStream::new(
        trid_socket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let trid_handler = TridHandler::new(
        trid_tcp_stream,
        [
            Trid::new(config.trid_device_id, config.upper_axis.trid_id),
            Trid::new(config.trid_device_id, config.lower_axis.trid_id),
            Trid::new(config.trid_device_id, config.right_axis.trid_id),
            Trid::new(config.trid_device_id, config.left_axis.trid_id),
        ],
    );

    let trid_command_executor = CommandExecutor::new(trid_handler);
    let trid_command_sender = TridCommandSender::new(trid_command_executor.sender());

    (trid_command_executor, trid_command_sender)
}

fn create_standa_command_executor(
    standa_ip: &str,
    standa_port: u16,
) -> CommandExecutor<StandaHandler> {
    let tcp_stream = LazyTcpStream::new(
        SocketAddr::new(standa_ip.parse().unwrap(), standa_port),
        1,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let standa = Standa::new();
    let handler = StandaHandler::new(standa, tcp_stream);

    CommandExecutor::new(handler)
}

pub fn create_standas(
    config: &SlitControllerConfig,
) -> Vec<(CommandExecutor<StandaHandler>, StandaCommandSender)> {
    let upper_standa_executor =
        create_standa_command_executor(&config.upper_axis.standa_ip, config.upper_axis.standa_port);
    let lower_standa_executor =
        create_standa_command_executor(&config.lower_axis.standa_ip, config.lower_axis.standa_port);
    let right_standa_executor =
        create_standa_command_executor(&config.right_axis.standa_ip, config.right_axis.standa_port);
    let left_standa_executor =
        create_standa_command_executor(&config.left_axis.standa_ip, config.left_axis.standa_port);

    let upper_standa_command_sender = StandaCommandSender::new(upper_standa_executor.sender());
    let lower_standa_command_sender = StandaCommandSender::new(lower_standa_executor.sender());
    let right_standa_command_sender = StandaCommandSender::new(right_standa_executor.sender());
    let left_standa_command_sender = StandaCommandSender::new(left_standa_executor.sender());

    vec![
        (upper_standa_executor, upper_standa_command_sender),
        (lower_standa_executor, lower_standa_command_sender),
        (right_standa_executor, right_standa_command_sender),
        (left_standa_executor, left_standa_command_sender),
    ]
}

pub fn create_controller(config: &SlitControllerConfig) -> SlitController {
    let (rf256_command_executor, rf256_command_sender) = create_encoder(config);
    let (trid_command_executor, trid_command_sender) = create_trid(config);
    let standas = create_standas(config);

    let upper_axis = SlitAxis::new(
        "Y_Up".to_string(),
        0,
        rf256_command_sender.clone(),
        trid_command_sender.clone(),
        standas[0].1.clone(),
        config.upper_axis.steps_per_mm,
    );
    let lower_axis = SlitAxis::new(
        "Y_Down".to_string(),
        1,
        rf256_command_sender.clone(),
        trid_command_sender.clone(),
        standas[1].1.clone(),
        config.lower_axis.steps_per_mm,
    );
    let left_axis = SlitAxis::new(
        "X_Left".to_string(),
        2,
        rf256_command_sender.clone(),
        trid_command_sender.clone(),
        standas[2].1.clone(),
        config.left_axis.steps_per_mm,
    );
    let right_axis = SlitAxis::new(
        "X_Right".to_string(),
        3,
        rf256_command_sender.clone(),
        trid_command_sender.clone(),
        standas[3].1.clone(),
        config.right_axis.steps_per_mm,
    );

    let mut controller = SlitController::new(
        rf256_command_executor,
        trid_command_executor,
        standas
            .into_iter()
            .map(|(executor, _sender)| executor)
            .collect(),
    );

    controller.add_axis(Arc::new(upper_axis));
    controller.add_axis(Arc::new(lower_axis));
    controller.add_axis(Arc::new(left_axis));
    controller.add_axis(Arc::new(right_axis));

    controller
}
