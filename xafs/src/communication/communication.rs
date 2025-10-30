use anyhow::Result;
use anyhow::anyhow;
use em2rs::StateParams;
use std::os::unix::fs::PermissionsExt;
use std::{path::Path, sync::Arc};
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::{Mutex, mpsc};

use crate::communication::commands::parse_command;
use crate::models::{
    AxisProperty, Command, CommandEnvelope, CommandError, CommandResponse, Limit, SharedState,
    State,
};

fn state_params_to_state(state_params: &StateParams) -> (State, Limit) {
    let state = if state_params.is_moving() {
        State::Moving
    } else {
        State::On
    };

    let limit = match (
        state_params.low_limit_triggered(),
        state_params.high_limit_triggered(),
    ) {
        (true, true) => Limit::Both,
        (true, false) => Limit::Lower,
        (false, true) => Limit::Upper,
        (false, false) => Limit::None,
    };

    (state, limit)
}

async fn handle_get_command(envelop: CommandEnvelope, shared_state: Arc<Mutex<SharedState>>) {
    let CommandEnvelope {
        command: Command::Get { axis, property },
        response: _,
    } = envelop
    else {
        unreachable!("Only GET commands should reach this handler");
    };

    let shared_state = shared_state.lock().await;
    let response = if let Some(axis_state) = &shared_state.axes[axis] {
        let respose = match property {
            AxisProperty::Position => axis_state.position.clone().map(CommandResponse::Position),
            AxisProperty::State => axis_state
                .state
                .clone()
                .map(|state| CommandResponse::State(state_params_to_state(&state))),
            AxisProperty::Moving => axis_state.is_moving.clone().map(CommandResponse::Moving),
            // AxisProperty::Temperature => axis_state
            //     .temperature
            //     .clone()
            //     .map(CommandResponse::Temperature),
        };

        respose.map_err(|e| CommandError {
            message: e.to_string(),
        })
    } else {
        Err(CommandError {
            message: format!("Axis {} not found", axis),
        })
    };

    envelop.response.send(response).unwrap_or_else(|e| {
        eprintln!("Failed to send response: {:#?}", e);
    });
}

const SOCKET_NAME: &str = "/tmp/xafs.sock";

pub async fn run_communication_layer(
    command_tx: mpsc::Sender<CommandEnvelope>,
    shared_state: Arc<Mutex<SharedState>>,
) -> Result<()> {
    // Remove existing socket file if it exists to prevent "Address already in use" error
    if Path::new(SOCKET_NAME).exists() {
        std::fs::remove_file(SOCKET_NAME)
            .map_err(|e| anyhow!("Failed to remove existing socket file: {}", e))?;
    }

    let listener = tokio::net::UnixListener::bind(SOCKET_NAME)
        .map_err(|e| anyhow!("Failed to bind to socket: {}", e))?;
    let permissions = std::fs::Permissions::from_mode(0o666);
    std::fs::set_permissions(SOCKET_NAME, permissions)
        .map_err(|e| anyhow!("Failed to set permissions: {}", e))?;

    loop {
        let shared_state = shared_state.clone();
        let (mut socket, _) = listener
            .accept()
            .await
            .map_err(|e| anyhow!("Failed to accept connection: {}", e))?;

        let command_tx = command_tx.clone();

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let shared_state = shared_state.clone();

            loop {
                match socket.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let command_str = String::from_utf8_lossy(&buffer[..n]);

                        if let Some((envelope, receiver)) = parse_command(&command_str) {
                            if envelope.command.is_get() {
                                handle_get_command(envelope, shared_state.clone()).await;
                            } else if command_tx.send(envelope).await.is_err() {
                                let _ = socket
                                    .write_all(b"Error: Failed to process command\n")
                                    .await;
                                continue;
                            }

                            // Wait for response and send it back to client
                            match receiver.await {
                                Ok(Ok(response)) => {
                                    let response_str = match response {
                                        CommandResponse::Success => "OK\n".to_string(),
                                        CommandResponse::Position(pos) => {
                                            format!("{}\n", pos)
                                        }
                                        CommandResponse::State(state) => {
                                            format!("{:?}\n", state)
                                        }
                                        CommandResponse::Moving(is_moving) => {
                                            format!("{}\n", is_moving)
                                        }
                                        CommandResponse::Error(e) => {
                                            format!("Error: {}\n", e)
                                        }
                                        CommandResponse::Temperature(temp) => {
                                            format!("{}\n", temp)
                                        }
                                    };
                                    let _ = socket.write_all(response_str.as_bytes()).await;
                                }
                                Ok(Err(err)) => {
                                    let _ = socket
                                        .write_all(format!("Error: {}\n", err.message).as_bytes())
                                        .await;
                                }
                                Err(_) => {
                                    let _ = socket
                                        .write_all(b"Error: Failed to receive response\n")
                                        .await;
                                }
                            }
                        } else {
                            let _ = socket.write_all(b"Error: Invalid command format\n").await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
