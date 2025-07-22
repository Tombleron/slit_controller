use std::{io, sync::mpsc::Sender};

use tokio::sync::oneshot;

use super::commands::{CommandResponse, EncoderCommand, EncoderCommandType, GetEncoderAttribute};

#[derive(Debug, Clone)]
pub struct Rf256CommandSender {
    commands_ch: Sender<EncoderCommand>,
}

impl Rf256CommandSender {
    pub fn new(commands_ch: Sender<EncoderCommand>) -> Self {
        Self { commands_ch }
    }

    async fn send_command(
        &self,
        axis: u8,
        command_type: EncoderCommandType,
    ) -> io::Result<CommandResponse> {
        let (tx, rx) = oneshot::channel();
        let command = EncoderCommand::new(axis, command_type, tx);

        self.commands_ch.send(command).unwrap();

        rx.await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to receive response"))?
    }

    pub fn read_position(
        &self,
        axis: u8,
    ) -> impl std::future::Future<Output = io::Result<f32>> + '_ {
        let command_type = EncoderCommandType::Get(GetEncoderAttribute::Position);
        async move {
            let response = self.send_command(axis, command_type).await?;
            match response {
                CommandResponse::Position(position) => Ok(position),
                _ => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unexpected response type",
                )),
            }
        }
    }

    pub fn read_position_with_retries(
        &self,
        axis: u8,
        retries: u8,
    ) -> impl std::future::Future<Output = io::Result<f32>> + '_ {
        let command_type =
            EncoderCommandType::Get(GetEncoderAttribute::PositionWithRetries(retries));
        async move {
            let response = self.send_command(axis, command_type).await?;
            match response {
                CommandResponse::Position(position) => Ok(position),
                _ => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unexpected response type",
                )),
            }
        }
    }

    pub fn read_id(&self, axis: u8) -> impl std::future::Future<Output = io::Result<u8>> + '_ {
        let command_type = EncoderCommandType::Get(GetEncoderAttribute::Id);
        async move {
            let response = self.send_command(axis, command_type).await?;
            match response {
                CommandResponse::Id(id) => Ok(id),
                _ => Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unexpected response type",
                )),
            }
        }
    }

    pub async fn reconnect(&self) -> io::Result<()> {
        let command_type = EncoderCommandType::Reconnect;

        let response = self.send_command(0, command_type).await?;

        match response {
            CommandResponse::Ok => Ok(()),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
