use std::io;

use tokio::sync::oneshot::Sender;

pub enum GetEncoderAttribute {
    PositionWithRetries(u8),
    Position,
    Id,
}

pub enum EncoderCommandType {
    Get(GetEncoderAttribute),
    Reconnect,
}

pub struct EncoderCommand {
    axis: u8,
    command_type: EncoderCommandType,
    response_ch: Sender<io::Result<CommandResponse>>,
}

impl EncoderCommand {
    pub fn new(
        axis: u8,
        command_type: EncoderCommandType,
        response_ch: Sender<io::Result<CommandResponse>>,
    ) -> Self {
        Self {
            axis,
            command_type,
            response_ch,
        }
    }

    pub fn command_type(&self) -> &EncoderCommandType {
        &self.command_type
    }

    pub fn response_ch(self) -> Sender<io::Result<CommandResponse>> {
        self.response_ch
    }

    pub fn axis(&self) -> u8 {
        self.axis
    }
}

#[derive(Debug)]
pub enum CommandResponse {
    None,
    Position(f32),
    Id(u8),
    Ok,
}
