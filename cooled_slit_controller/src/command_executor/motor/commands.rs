use em2rs::StateParams;
use std::io;
use utilities::command_executor::Command;

use crate::command_executor::motor::Em2rsHandler;

#[derive(Clone)]
pub enum MotorCommand {
    GetState { axis: usize },
    SetVelocity { axis: usize, velocity: u16 },
    SetAcceleration { axis: usize, acceleration: u16 },
    SetDeceleration { axis: usize, deceleration: u16 },
    Stop { axis: usize },
    Move { axis: usize, steps: i32 },
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
            MotorCommand::GetState { axis } => handler.get_state(axis),
            MotorCommand::SetVelocity { axis, velocity } => handler.set_velocity(axis, velocity),
            MotorCommand::SetAcceleration { axis, acceleration } => {
                handler.set_acceleration(axis, acceleration)
            }
            MotorCommand::SetDeceleration { axis, deceleration } => {
                handler.set_deceleration(axis, deceleration)
            }
            MotorCommand::Stop { axis } => handler.stop(axis),
            MotorCommand::Move { axis, steps } => handler.move_relative(axis, steps),
        }
    }
}
