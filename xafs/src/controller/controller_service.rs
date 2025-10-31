use anyhow::Result;
use em2rs::Em2rs;
use lir::LIR;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use trid::Trid;
use utilities::{command_executor::CommandExecutor, lazy_tcp::LazyTcpStream};

use tokio::sync::{Mutex, mpsc};

use crate::{
    command_executor::{
        motor::{Em2rsHandler, command_sender::Em2rsCommandSender},
        sensors::{SensorsHandler, command_sender::SensorsCommandSender},
    },
    controller::{
        multi_axis::{Axes, MultiAxis, MultiAxisConfig},
        single_axis::SingleAxis,
    },
    models::{Command, CommandEnvelope, CommandError, CommandResponse},
};

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

const READ_TIMEOUT: Duration = Duration::from_millis(100);
const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
const CONNECT_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_RETRIES: u32 = 3;

pub fn create_controller(
    config: MultiAxisConfig,
) -> (
    CommandExecutor<SensorsHandler>,
    CommandExecutor<Em2rsHandler>,
    MultiAxis,
) {
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
        vec![
            Trid::new(2, 0),
            Trid::new(2, 1),
            Trid::new(2, 2),
            Trid::new(2, 3),
        ],
    );
    // M7015::new(config.icpcon_id),

    let sensors_command_executor = CommandExecutor::new(sensors_handler);
    let sensors_command_sender = SensorsCommandSender::new(sensors_command_executor.sender());

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

    let upper_axis = SingleAxis::new(
        0,
        config.upper_axis.steps_per_mm,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
    );
    let lower_axis = SingleAxis::new(
        1,
        config.lower_axis.steps_per_mm,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
    );
    let left_axis = SingleAxis::new(
        2,
        config.left_axis.steps_per_mm,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
    );
    let right_axis = SingleAxis::new(
        3,
        config.right_axis.steps_per_mm,
        sensors_command_sender.clone(),
        em2rs_command_sender.clone(),
    );

    let axes = Axes::new(upper_axis, lower_axis, left_axis, right_axis);
    let multi_axis = MultiAxis::new(axes);

    (sensors_command_executor, em2rs_command_executor, multi_axis)
}
