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

    pub async fn get_state(&self, axis: usize) -> io::Result<StateParams> {
        let response = self
            .sender
            .send_command(MotorCommand::GetState { axis })
            .await?;

        match response {
            CommandResponse::State(state) => Ok(state),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_velocity(&self, axis: usize, velocity: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetVelocity { axis, velocity })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_acceleration(&self, axis: usize, acceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetAcceleration { axis, acceleration })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_deceleration(&self, axis: usize, deceleration: u16) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::SetDeceleration { axis, deceleration })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn stop(&self, axis: usize) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::Stop { axis })
            .await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn send_steps(&self, axis: usize, steps: i32) -> io::Result<()> {
        let response = self
            .sender
            .send_command(MotorCommand::Move { axis, steps })
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
