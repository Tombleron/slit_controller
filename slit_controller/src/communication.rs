use anyhow::{anyhow, Result};
use standa::command::state::StateParams;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex};

use crate::commands::parse_command;
use crate::models::{
    Command, CommandEnvelope, CommandError, CommandResponse, Limit, SharedState, State,
};

fn state_params_to_state(state_params: &StateParams) -> (State, Limit) {
    let state = match (state_params.is_moving(), state_params.is_error()) {
        (true, false) => State::Moving,
        (false, true) => State::Fault,
        (false, false) => State::On,
        (true, true) => State::Fault, // If moving and error, treat as fault
    };

    let limit = match (state_params.left_switch(), state_params.right_switch()) {
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
            crate::models::AxisProperty::Position => {
                axis_state.position.clone().map(CommandResponse::Position)
            }
            crate::models::AxisProperty::State => axis_state
                .state
                .clone()
                .map(|state| CommandResponse::State(state_params_to_state(&state))),
            crate::models::AxisProperty::Velocity => {
                axis_state.velocity.clone().map(CommandResponse::Velocity)
            }
            crate::models::AxisProperty::Acceleration => axis_state
                .acceleration
                .clone()
                .map(CommandResponse::Acceleration),
            crate::models::AxisProperty::Deceleration => axis_state
                .deceleration
                .clone()
                .map(CommandResponse::Deceleration),
            crate::models::AxisProperty::PositionWindow => axis_state
                .position_window
                .clone()
                .map(CommandResponse::PositionWindow),
        };

        respose.map_err(|e| CommandError {
            code: 500,
            message: e.to_string(),
        })
    } else {
        Err(CommandError {
            code: 404,
            message: format!("Axis {} not found", axis),
        })
    };

    envelop.response.send(response).unwrap_or_else(|e| {
        eprintln!("Failed to send response: {:#?}", e);
    });
}

pub async fn run_communication_layer(
    command_tx: mpsc::Sender<CommandEnvelope>,
    shared_state: Arc<Mutex<SharedState>>,
) -> Result<()> {
    // Remove existing socket file if it exists to prevent "Address already in use" error
    if Path::new("/tmp/slit_controller.sock").exists() {
        std::fs::remove_file("/tmp/slit_controller.sock")
            .map_err(|e| anyhow!("Failed to remove existing socket file: {}", e))?;
    }

    let listener = tokio::net::UnixListener::bind("/tmp/slit_controller.sock")
        .map_err(|e| anyhow!("Failed to bind to socket: {}", e))?;

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
                                            format!("Position: {}\n", pos)
                                        }
                                        CommandResponse::State(state) => {
                                            format!("State: {:?}\n", state)
                                        }
                                        CommandResponse::Velocity(vel) => {
                                            format!("Velocity: {}\n", vel)
                                        }
                                        CommandResponse::Acceleration(acc) => {
                                            format!("Acceleration: {}\n", acc)
                                        }
                                        CommandResponse::Deceleration(dec) => {
                                            format!("Deceleration: {}\n", dec)
                                        }
                                        CommandResponse::PositionWindow(win) => {
                                            format!("Position Window: {}\n", win)
                                        }
                                        CommandResponse::Error(e) => {
                                            format!("Error: {}\n", e)
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
