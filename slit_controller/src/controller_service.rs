use anyhow::Result;
use rf256::Rf256;
use standa::Standa;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use trid::Trid;

use crate::command_executor::encoder::Rf256CommandExecutor;
use crate::command_executor::motor::StandaCommandExecutor;
use crate::command_executor::temperature::TridCommandExecutor;
use crate::command_executor::{CONNECT_TIMEOUT, READ_TIMEOUT, WRITE_TIMEOUT};
use crate::controller::multi_axis::{MultiAxis, MultiAxisConfig};
use crate::controller::single_axis::SingleAxis;
use crate::lazy_tcp::LazyTcpStream;
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
            } => {
                // GET commands should not reach the controller
                // This is an error condition
                Err(CommandError {
                    message: "GET commands should not be handled by the controller".to_string(),
                })
            }
        };

        let _ = response.send(result);
    }

    Ok(())
}

fn create_standa_command_executor(standa_ip: &str, standa_port: u16) -> StandaCommandExecutor {
    let tcp_stream = LazyTcpStream::new(
        SocketAddr::new(standa_ip.parse().unwrap(), standa_port),
        1,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );

    let standa = Standa::new();

    StandaCommandExecutor::new(tcp_stream, standa)
}

pub fn create_controller(
    config: MultiAxisConfig,
) -> (
    Rf256CommandExecutor,
    TridCommandExecutor,
    StandaCommandExecutor,
    StandaCommandExecutor,
    StandaCommandExecutor,
    StandaCommandExecutor,
    MultiAxis,
) {
    let rf256_socket_addr = SocketAddr::new(config.rf256_ip.parse().unwrap(), config.rf256_port);
    let rf256_tcp_steam = LazyTcpStream::new(
        rf256_socket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );
    let rf256_command_executor = Rf256CommandExecutor::new(
        rf256_tcp_steam,
        [
            Rf256::new(config.upper_rf256_id),
            Rf256::new(config.lower_rf256_id),
            Rf256::new(config.right_rf256_id),
            Rf256::new(config.left_rf256_id),
        ],
    );

    let trid_socket_addr = SocketAddr::new(config.trid_ip.parse().unwrap(), config.trid_port);
    let trid_tcp_stream = LazyTcpStream::new(
        trid_socket_addr,
        MAX_RETRIES,
        READ_TIMEOUT,
        WRITE_TIMEOUT,
        CONNECT_TIMEOUT,
    );
    let trid_command_executor = TridCommandExecutor::new(
        trid_tcp_stream,
        [
            Trid::new(config.upper_trid_id),
            Trid::new(config.lower_trid_id),
            Trid::new(config.right_trid_id),
            Trid::new(config.left_trid_id),
        ],
    );

    let upper_standa_executor =
        create_standa_command_executor(&config.upper_standa_ip, config.upper_standa_port);
    let lower_standa_executor =
        create_standa_command_executor(&config.lower_standa_ip, config.lower_standa_port);
    let right_standa_executor =
        create_standa_command_executor(&config.right_standa_ip, config.right_standa_port);
    let left_standa_executor =
        create_standa_command_executor(&config.left_standa_ip, config.left_standa_port);

    let upper_axis = SingleAxis::new(
        0,
        rf256_command_executor.sender(),
        trid_command_executor.sender(),
        upper_standa_executor.sender(),
    );

    let lower_axis = SingleAxis::new(
        1,
        rf256_command_executor.sender(),
        trid_command_executor.sender(),
        lower_standa_executor.sender(),
    );

    let right_axis = SingleAxis::new(
        2,
        rf256_command_executor.sender(),
        trid_command_executor.sender(),
        right_standa_executor.sender(),
    );

    let left_axis = SingleAxis::new(
        3,
        rf256_command_executor.sender(),
        trid_command_executor.sender(),
        left_standa_executor.sender(),
    );

    let multi_axis = MultiAxis::new(
        [upper_axis, lower_axis, right_axis, left_axis],
        rf256_command_executor.sender(),
        trid_command_executor.sender(),
    );

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
