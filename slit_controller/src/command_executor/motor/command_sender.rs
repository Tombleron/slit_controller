use std::{io, sync::mpsc::Sender};

use standa::command::state::StateParams;
use tokio::sync::oneshot;

use super::commands::{
    CommandResponse, GetMotorAttribute, MotorCommand, MotorCommandType, SetMotorAttribute,
};

#[derive(Debug, Clone)]
pub struct StandaCommandSender {
    commands_ch: Sender<MotorCommand>,
}

impl StandaCommandSender {
    pub fn new(commands_ch: Sender<MotorCommand>) -> Self {
        Self { commands_ch }
    }

    async fn send_command(&self, command_type: MotorCommandType) -> io::Result<CommandResponse> {
        let (tx, rx) = oneshot::channel();
        let command = MotorCommand::new(command_type, tx);

        self.commands_ch.send(command).unwrap();

        rx.await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to receive response"))?
    }

    pub async fn get_state(&self) -> io::Result<StateParams> {
        let response = self
            .send_command(MotorCommandType::Get(GetMotorAttribute::State))
            .await?;

        match response {
            CommandResponse::State(state) => Ok(state),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn set_velocity(&self, velocity: u32) -> io::Result<()> {
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

    pub async fn send_steps(&self, steps: i32, substeps: i16) -> io::Result<()> {
        let response = self
            .send_command(MotorCommandType::Move { steps, substeps })
            .await?;

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
                "Unexpected response type for reconnect",
            )),
        }
    }
}
