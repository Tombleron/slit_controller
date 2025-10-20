use std::io;

use standa::command::state::StateParams;
use utilities::command_executor::CommandSender;

use crate::command_executor::motor::commands::MotorResponse;

use super::commands::MotorCommand;

#[derive(Clone)]
pub struct StandaCommandSender {
    sender: CommandSender<MotorCommand>,
}

impl StandaCommandSender {
    pub fn new(sender: CommandSender<MotorCommand>) -> Self {
        Self { sender }
    }

    pub async fn get_state(&self) -> io::Result<StateParams> {
        let response = self.sender.send_command(MotorCommand::GetState).await?;

        match response {
            MotorResponse::State(state) => Ok(state),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_velocity(&self, velocity: u32) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetVelocity(velocity))
            .await?;

        match response {
            MotorResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_acceleration(&self, acceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetAcceleration(acceleration))
            .await?;

        match response {
            MotorResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_deceleration(&self, deceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetDeceleration(deceleration))
            .await?;

        match response {
            MotorResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn stop(&self) -> io::Result<()> {
        let response = self.sender.send_command(MotorCommand::Stop).await?;

        match response {
            MotorResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn send_steps(&self, steps: i32, substeps: i16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::Move { steps, substeps })
            .await?;

        match response {
            MotorResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
