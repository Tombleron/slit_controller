use em2rs::StateParams;
use std::io;
use utilities::command_executor::Command;

use crate::command_executor::motor::Em2rsHandler;

#[derive(Clone)]
pub enum MotorCommand {
    GetState,
    SetVelocity { velocity: u16 },
    SetAcceleration { acceleration: u16 },
    SetDeceleration { deceleration: u16 },
    Stop,
    Move { steps: i32 },
}

pub enum CommandResponse {
    State(StateParams),
    Ok,
}

impl Command for MotorCommand {
    type Response = CommandResponse;
    type Handler = Em2rsHandler;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response> {
        match self {
            MotorCommand::GetState => handler.get_state(),
            MotorCommand::SetVelocity { velocity } => handler.set_velocity(velocity),
            MotorCommand::SetAcceleration { acceleration } => {
                handler.set_acceleration(acceleration)
            }
            MotorCommand::SetDeceleration { deceleration } => {
                handler.set_deceleration(deceleration)
            }
            MotorCommand::Stop => handler.stop(),
            MotorCommand::Move { steps } => handler.move_relative(steps),
        }
    }
}
