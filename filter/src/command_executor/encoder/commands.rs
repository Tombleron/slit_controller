use std::io;

use crate::command_executor::encoder::EncoderHandler;
use utilities::command_executor::Command;

#[derive(Clone)]
pub enum EncoderCommand {
    Position,
}

#[derive(Debug)]
pub enum CommandResponse {
    Position(f32),
}

impl Command for EncoderCommand {
    type Response = CommandResponse;
    type Handler = EncoderHandler;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response> {
        match self {
            EncoderCommand::Position => handler
                .get_position()
                .map(|position| CommandResponse::Position(position)),
        }
    }
}
