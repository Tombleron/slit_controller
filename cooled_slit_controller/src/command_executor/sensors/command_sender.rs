use std::io;

use std::sync::mpsc::Sender;

use crate::command_executor::sensors::commands::{
    CommandResponse, GetSensorsAttribute, SensorsCommand, SensorsCommandType,
};

#[derive(Clone)]
pub struct SensorsCommandSender {
    command_ch: Sender<SensorsCommand>,
}

impl SensorsCommandSender {
    pub fn new(command_ch: Sender<SensorsCommand>) -> Self {
        Self { command_ch }
    }

    pub async fn send_command(
        &self,
        axis: u8,
        command_type: SensorsCommandType,
    ) -> io::Result<CommandResponse> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let command = SensorsCommand::new(axis, command_type, tx);

        self.command_ch
            .send(command)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to send command"))?;

        rx.await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to receive response"))?
    }

    pub async fn read_position(&self, axis: u8) -> io::Result<f32> {
        let command_type = SensorsCommandType::Get(GetSensorsAttribute::Position);

        let response = self.send_command(axis, command_type).await?;

        match response {
            CommandResponse::Position(position) => Ok(position),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn read_temperature(&self, axis: u8) -> io::Result<f32> {
        let command_type = SensorsCommandType::Get(GetSensorsAttribute::Temperature);

        let response = self.send_command(axis, command_type).await?;

        match response {
            CommandResponse::Temperature(temperature) => Ok(temperature),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn reconnect(&self, axis: u8) -> io::Result<()> {
        let command_type = SensorsCommandType::Reconnect;

        let response = self.send_command(axis, command_type).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
