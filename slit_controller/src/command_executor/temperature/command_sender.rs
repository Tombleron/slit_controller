use super::commands::TridCommand;
use crate::command_executor::temperature::commands::TridResponse;
use std::io;
use utilities::command_executor::CommandSender;

#[derive(Clone)]
pub struct TridCommandSender {
    sender: CommandSender<TridCommand>,
}

impl TridCommandSender {
    pub fn new(commands_ch: CommandSender<TridCommand>) -> Self {
        Self {
            sender: commands_ch,
        }
    }

    pub async fn read_temperature(&self, axis: u8) -> io::Result<f32> {
        let response = self
            .sender
            .send_command(TridCommand::GetTemperature { axis })
            .await?;

        match response {
            TridResponse::Temperature(temp) => Ok(temp),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Unexpected response type",
            )),
        }
    }
}
