use crate::command_executor::motor::commands::{CommandResponse, MotorCommand};
use em2rs::StateParams;
use std::io;
use utilities::command_executor::CommandSender;

#[derive(Clone)]
pub struct Em2rsCommandSender {
    sender: CommandSender<MotorCommand>,
}

impl Em2rsCommandSender {
    pub fn new(sender: CommandSender<MotorCommand>) -> Self {
        Self { sender }
    }

    pub async fn get_state(&self) -> io::Result<StateParams> {
        let response = self.sender.send_command(MotorCommand::GetState).await?;

        match response {
            CommandResponse::State(state) => Ok(state),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_velocity(&self, velocity: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetVelocity { velocity })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_acceleration(&self, acceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetAcceleration { acceleration })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_deceleration(&self, deceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetDeceleration { deceleration })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn stop(&self) -> io::Result<()> {
        let response = self.sender.send_command(MotorCommand::Stop).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn send_steps(&self, steps: i32) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::Move { steps })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
