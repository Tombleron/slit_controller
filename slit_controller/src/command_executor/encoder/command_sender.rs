use utilities::command_executor::CommandSender;

use crate::command_executor::encoder::commands::{EncoderCommand, EncoderResponse};

#[derive(Clone)]
pub struct EncoderCommandSender {
    sender: CommandSender<EncoderCommand>,
}

impl EncoderCommandSender {
    pub fn new(sender: CommandSender<EncoderCommand>) -> Self {
        Self { sender }
    }

    pub async fn get_position(&self, axis: u8) -> std::io::Result<f32> {
        let response = self
            .sender
            .send_command(EncoderCommand::GetPosition { axis })
            .await?;
        match response {
            EncoderResponse::Position {
                axis: _axis,
                position,
            } => Ok(position),
        }
    }
}
