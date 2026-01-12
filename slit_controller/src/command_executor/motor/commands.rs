use std::io;

use standa::command::state::StateParams;
use utilities::command_executor::Command;

use crate::command_executor::motor::StandaHandler;

#[derive(Clone)]
pub enum MotorCommand {
    GetState,
    SetVelocity(u32),
    SetAcceleration(u16),
    SetDeceleration(u16),
    Stop,
    Move { steps: i32, substeps: i16 },
    Reconnect,
}

#[derive(Debug)]
pub enum MotorResponse {
    None,
    State(StateParams),
    Ok,
}

impl Command for MotorCommand {
    type Response = MotorResponse;
    type Handler = StandaHandler;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response> {
        match self {
            MotorCommand::GetState => {
                let state = handler.get_state()?;
                Ok(MotorResponse::State(state))
            }
            MotorCommand::SetVelocity(velocity) => {
                handler.set_velocity(velocity)?;
                Ok(MotorResponse::Ok)
            }
            MotorCommand::SetAcceleration(acceleration) => {
                handler.set_acceleration(acceleration)?;
                Ok(MotorResponse::Ok)
            }
            MotorCommand::SetDeceleration(deceleration) => {
                handler.set_deceleration(deceleration)?;
                Ok(MotorResponse::Ok)
            }
            MotorCommand::Stop => {
                handler.stop()?;
                Ok(MotorResponse::Ok)
            }
            MotorCommand::Move { steps, substeps } => {
                handler.move_relative(steps, substeps)?;
                Ok(MotorResponse::Ok)
            }
            MotorCommand::Reconnect => {
                handler.reconnect()?;
                Ok(MotorResponse::Ok)
            }
        }
    }
}
