use std::{io, sync::mpsc::Sender};

use tokio::sync::oneshot;

use super::commands::{CommandResponse, GetTridAttribute, TridCommand, TridCommandType};

#[derive(Debug, Clone)]
pub struct TridCommandSender {
    commands_ch: Sender<TridCommand>,
}

impl TridCommandSender {
    pub fn new(commands_ch: Sender<TridCommand>) -> Self {
        Self { commands_ch }
    }

    pub async fn send_command(
        &self,
        trid_id: u8,
        command_type: TridCommandType,
    ) -> io::Result<CommandResponse> {
        let (tx, rx) = oneshot::channel();
        let command = TridCommand::new(trid_id, command_type, tx);

        self.commands_ch.send(command).map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Failed to send command to channel")
        })?;

        rx.await.map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Failed to receive response from command",
            )
        })?
    }

    pub async fn read_temperature(&self, trid_id: u8) -> io::Result<f32> {
        let response = self
            .send_command(trid_id, TridCommandType::Get(GetTridAttribute::Temperature))
            .await?;

        match response {
            CommandResponse::Temperature(temp) => Ok(temp),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn reconnect(&self) -> io::Result<()> {
        let command = TridCommandType::Reconnect;

        let response = self.send_command(0, command).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type for reconnect",
            )),
        }
    }
}
