use std::{io, sync::mpsc::Sender};

use em2rs::Em2rsState;

use crate::command_executor::motor::commands::{CommandResponse, MotorCommand, MotorCommandType};

use super::commands::SetMotorAttribute;

#[derive(Clone)]
pub struct Em2rsCommandSender {
    commands_ch: Sender<MotorCommand>,
}

impl Em2rsCommandSender {
    pub fn new(commands_ch: Sender<MotorCommand>) -> Self {
        Self { commands_ch }
    }

    async fn send_command(&self, command_type: MotorCommandType) -> io::Result<CommandResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let command = MotorCommand::new(command_type, tx);

        self.commands_ch
            .send(command)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to send command"))?;

        rx.await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to receive response"))?
    }

    pub async fn get_state(&self) -> io::Result<Em2rsState> {
        let response = self
            .send_command(MotorCommandType::Get(
                super::commands::GetMotorAttribute::State,
            ))
            .await?;

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
            .send_command(MotorCommandType::Set(SetMotorAttribute::Velocity(velocity)))
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
            .send_command(MotorCommandType::Set(SetMotorAttribute::Acceleration(
                acceleration,
            )))
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
            .send_command(MotorCommandType::Set(SetMotorAttribute::Deceleration(
                deceleration,
            )))
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
        let response = self.send_command(MotorCommandType::Stop).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn send_steps(&self, steps: i32) -> io::Result<()> {
        let response = self.send_command(MotorCommandType::Move { steps }).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn reconnect(&self) -> io::Result<()> {
        let response = self.send_command(MotorCommandType::Reconnect).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
