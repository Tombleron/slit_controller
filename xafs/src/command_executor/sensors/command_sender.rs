use crate::command_executor::sensors::commands::{CommandResponse, SensorsCommand};
use std::io;
use utilities::command_executor::CommandSender;

#[derive(Clone)]
pub struct SensorsCommandSender {
    sender: CommandSender<SensorsCommand>,
}

impl SensorsCommandSender {
    pub fn new(sender: CommandSender<SensorsCommand>) -> Self {
        Self { sender }
    }

    pub async fn get_position(&self, axis: u8) -> io::Result<f32> {
        let response = self
            .sender
            .send_command(SensorsCommand::Position { axis })
            .await?;

        match response {
            CommandResponse::Position(position) => Ok(position),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }

    pub async fn get_temperature(&self, axis: u8) -> io::Result<f32> {
        let response = self
            .sender
            .send_command(SensorsCommand::Temperature { axis })
            .await?;

        match response {
            CommandResponse::Temperature(temperature) => Ok(temperature),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
