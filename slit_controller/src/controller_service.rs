use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use crate::controller::multi_axis::{MultiAxis, MultiAxisConfig};
use crate::models::{
    AxisProperty, Command, CommandEnvelope, CommandError, CommandParams, CommandResponse,
};

pub async fn run_controller(
    mut command_rx: mpsc::Receiver<CommandEnvelope>,
    multi_axis_controller: Arc<Mutex<MultiAxis>>,
) -> Result<()> {
    while let Some(envelope) = command_rx.recv().await {
        let CommandEnvelope { command, response } = envelope;
        let mut multi_axis = multi_axis_controller.lock().await;

        let result = match command {
            Command::Move { axis, position } => multi_axis
                .move_to_position(axis, position)
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

            Command::Set {
                axis,
                property,
                value,
            } => match (property, value) {
                (AxisProperty::Velocity, CommandParams::Velocity(velocity)) => multi_axis
                    .set_velocity(axis, velocity)
                    .await
                    .map(|_| CommandResponse::Success)
                    .map_err(|e| e.to_string().into()),
                (AxisProperty::Acceleration, CommandParams::Acceleration(acceleration)) => {
                    multi_axis
                        .set_acceleration(axis, acceleration)
                        .await
                        .map(|_| CommandResponse::Success)
                        .map_err(|e| e.to_string().into())
                }
                (AxisProperty::Deceleration, CommandParams::Deceleration(deceleration)) => {
                    multi_axis
                        .set_deceleration(axis, deceleration)
                        .await
                        .map(|_| CommandResponse::Success)
                        .map_err(|e| e.to_string().into())
                }
                (AxisProperty::PositionWindow, CommandParams::PositionWindow(window)) => multi_axis
                    .set_position_window(axis, window)
                    .await
                    .map(|_| CommandResponse::Success)
                    .map_err(|e| e.to_string().into()),
                (AxisProperty::TimeLimit, CommandParams::TimeLimit(limit)) => multi_axis
                    .set_time_limit(axis, limit)
                    .await
                    .map(|_| CommandResponse::Success)
                    .map_err(|e| e.to_string().into()),
                _ => Err(CommandError {
                    message: "Invalid parameter for property".to_string(),
                }),
            },
        };

        let _ = response.send(result);
    }

    Ok(())
}

pub fn create_controller(config: MultiAxisConfig) -> Arc<Mutex<MultiAxis>> {
    Arc::new(Mutex::new(MultiAxis::from_config(config)))
}
