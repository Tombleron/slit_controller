use std::time::Duration;

use tokio::sync::oneshot;

use crate::{
    communication::{AxisProperty, Command, CommandEnvelope, CommandResult},
    controller::single_axis::MoveArgs,
};

pub fn parse_command(cmd_str: &str) -> Option<(CommandEnvelope, oneshot::Receiver<CommandResult>)> {
    let parts: Vec<&str> = cmd_str.trim().split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let (tx, rx) = oneshot::channel();

    let command = match parts[0] {
        "move" => {
            // Two variants: "move:axis:position" or "move:axis:position:velocity:acceleration:deceleration:position_window:time_limit"
            if parts.len() != 3 && parts.len() != 8 {
                return None;
            }
            let axis = parts[1].parse::<usize>().ok()?;
            let position = parts[2].parse::<f32>().ok()?;

            let params = if parts.len() == 8 {
                let velocity = parts[3].parse::<u16>().ok()?;
                let acceleration = parts[4].parse::<u16>().ok()?;
                let deceleration = parts[5].parse::<u16>().ok()?;
                let position_window = parts[6].parse::<f32>().ok()?;
                let time_limit = parts[7].parse::<u64>().ok()?;

                Some(MoveArgs {
                    velocity,
                    acceleration,
                    deceleration,
                    position_window,
                    time_limit: Duration::from_secs(time_limit),
                })
            } else {
                None
            };

            Command::Move {
                axis,
                position,
                params,
            }
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
                "is_moving" => AxisProperty::Moving,
                "temperature" => AxisProperty::Temperature,
                _ => return None,
            };

            Command::Get { axis, property }
        }
        _ => return None,
    };

    Some((
        CommandEnvelope {
            command,
            sender: tx,
        },
        rx,
    ))
}
