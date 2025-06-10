use crate::models::{AxisProperty, Command, CommandEnvelope, CommandParams, CommandResult};
use tokio::sync::oneshot;

pub fn parse_command(cmd_str: &str) -> Option<(CommandEnvelope, oneshot::Receiver<CommandResult>)> {
    let parts: Vec<&str> = cmd_str.trim().split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let (tx, rx) = oneshot::channel();

    let command = match parts[0] {
        "move" => {
            if parts.len() != 3 {
                return None;
            }
            let axis = parts[1].parse::<usize>().ok()?;
            let position = parts[2].parse::<f32>().ok()?;

            Command::Move { axis, position }
        }
        "stop" => {
            if parts.len() != 2 {
                return None;
            }
            let axis = parts[1].parse::<usize>().ok()?;

            Command::Stop { axis }
        }
        "get" => {
            if parts.len() != 3 {
                return None;
            }
            let axis = parts[1].parse::<usize>().ok()?;
            let property = match parts[2] {
                "position" => AxisProperty::Position,
                "state" => AxisProperty::State,
                "velocity" => AxisProperty::Velocity,
                "acceleration" => AxisProperty::Acceleration,
                "deceleration" => AxisProperty::Deceleration,
                "position_window" => AxisProperty::PositionWindow,
                _ => return None,
            };

            Command::Get { axis, property }
        }
        "set" => {
            if parts.len() != 4 {
                return None;
            }
            let axis = parts[1].parse::<usize>().ok()?;
            let (property, value) = match parts[2] {
                "velocity" => {
                    let val = parts[3].parse::<u32>().ok()?;
                    (AxisProperty::Velocity, CommandParams::Velocity(val))
                }
                "acceleration" => {
                    let val = parts[3].parse::<u16>().ok()?;
                    (AxisProperty::Acceleration, CommandParams::Acceleration(val))
                }
                "deceleration" => {
                    let val = parts[3].parse::<u16>().ok()?;
                    (AxisProperty::Deceleration, CommandParams::Deceleration(val))
                }
                "position_window" => {
                    let val = parts[3].parse::<f32>().ok()?;
                    (
                        AxisProperty::PositionWindow,
                        CommandParams::PositionWindow(val),
                    )
                }
                _ => return None,
            };

            Command::Set {
                axis,
                property,
                value,
            }
        }
        _ => return None,
    };

    Some((
        CommandEnvelope {
            command,
            response: tx,
        },
        rx,
    ))
}
