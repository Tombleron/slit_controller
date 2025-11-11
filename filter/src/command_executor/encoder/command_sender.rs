use crate::command_executor::encoder::commands::{CommandResponse, EncoderCommand};
use std::io;
use utilities::command_executor::CommandSender;

#[derive(Clone)]
pub struct EncoderCommandSender {
    sender: CommandSender<EncoderCommand>,
}

impl EncoderCommandSender {
    pub fn new(sender: CommandSender<EncoderCommand>) -> Self {
        Self { sender }
    }

    pub async fn get_position(&self) -> io::Result<f32> {
        let response = self.sender.send_command(EncoderCommand::Position).await?;

        match response {
            CommandResponse::Position(position) => Ok(position),
        }
    }
}
