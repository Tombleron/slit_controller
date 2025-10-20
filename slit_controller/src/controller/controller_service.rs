use anyhow::Result;
use rf256::Rf256;
use standa::Standa;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use trid::Trid;
use utilities::command_executor::CommandExecutor;
use utilities::lazy_tcp::LazyTcpStream;

use crate::command_executor::encoder::{command_sender::EncoderCommandSender, Rf256Handler};
use crate::command_executor::motor::command_sender::StandaCommandSender;
use crate::command_executor::motor::StandaHandler;
use crate::command_executor::temperature::command_sender::TridCommandSender;
use crate::command_executor::temperature::TridHandler;
use crate::command_executor::{CONNECT_TIMEOUT, READ_TIMEOUT, WRITE_TIMEOUT};
use crate::controller::multi_axis::{MultiAxis, MultiAxisConfig};
use crate::controller::single_axis::SingleAxis;
use crate::models::{Command, CommandEnvelope, CommandError, CommandResponse};

const MAX_RETRIES: u32 = 1;

pub async fn run_controller(
    mut command_rx: mpsc::Receiver<CommandEnvelope>,
    multi_axis_controller: Arc<Mutex<MultiAxis>>,
) -> Result<()> {
    while let Some(envelope) = command_rx.recv().await {
        let CommandEnvelope { command, response } = envelope;
        let mut multi_axis = multi_axis_controller.lock().await;

        let result = match command {
            Command::Move {
                axis,
                position,
                params,
            } => multi_axis
                .move_to_position(axis, position, params.unwrap_or_default())
                .await
                .map(|_| CommandResponse::Success)
                .map_err(|e| e.to_string().into()),
            Command::Stop { axis } => multi_axis
                .stop(axis)
                .await
                .map(|_| CommandResponse::Success)
                .map_err(|e| e.to_string().into()),
            Command::Get {
                axis: _,
                property: _,
            } => Err(CommandError {
                message: "GET commands should not be handled by the controller".to_string(),
            }),
        };

        let _ = response.send(result);
    }

    Ok(())
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

pub fn create_controller(
    config: MultiAxisConfig,
) -> (
    CommandExecutor<Rf256Handler>,
    CommandExecutor<TridHandler>,
    CommandExecutor<StandaHandler>,
    CommandExecutor<StandaHandler>,
    CommandExecutor<StandaHandler>,
    CommandExecutor<StandaHandler>,
    MultiAxis,
) {
    let rf256_socket_addr = SocketAddr::new(config.rf256_ip.parse().unwrap(), config.rf256_port);
    let rf256_tcp_stream = LazyTcpStream::new(
        rf256_socket_addr,
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
    let encoder_command_sender = EncoderCommandSender::new(rf256_command_executor.sender());

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
            Trid::new(1, config.upper_axis.trid_id as u16),
            Trid::new(1, config.lower_axis.trid_id as u16),
            Trid::new(1, config.right_axis.trid_id as u16),
            Trid::new(1, config.left_axis.trid_id as u16),
        ],
    );
    let trid_command_executor = CommandExecutor::new(trid_handler);
    let trid_command_sender = TridCommandSender::new(trid_command_executor.sender());

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

    let upper_axis = SingleAxis::new(
        0,
        encoder_command_sender.clone(),
        trid_command_sender.clone(),
        upper_standa_command_sender,
    );

    let lower_axis = SingleAxis::new(
        1,
        encoder_command_sender.clone(),
        trid_command_sender.clone(),
        lower_standa_command_sender,
    );

    let right_axis = SingleAxis::new(
        2,
        encoder_command_sender.clone(),
        trid_command_sender.clone(),
        right_standa_command_sender,
    );

    let left_axis = SingleAxis::new(
        3,
        encoder_command_sender.clone(),
        trid_command_sender.clone(),
        left_standa_command_sender,
    );

    let multi_axis = MultiAxis::new([upper_axis, lower_axis, right_axis, left_axis]);

    return (
        rf256_command_executor,
        trid_command_executor,
        upper_standa_executor,
        lower_standa_executor,
        right_standa_executor,
        left_standa_executor,
        multi_axis,
    );
}
