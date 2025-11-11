use std::io;

use crate::command_executor::sensors::SensorsHandler;
use utilities::command_executor::Command;

#[derive(Clone)]
pub enum SensorsCommand {
    Position { axis: u8 },
    Temperature { axis: u8 },
}

#[derive(Debug)]
pub enum CommandResponse {
    Temperature(f32),
    Position(f32),
}

impl Command for SensorsCommand {
    type Response = CommandResponse;
    type Handler = SensorsHandler;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response> {
        match self {
            SensorsCommand::Position { axis } => handler
                .get_position(axis)
                .map(|position| CommandResponse::Position(position)),
            SensorsCommand::Temperature { axis } => handler
                .get_temperature(axis)
                .map(|temperature| CommandResponse::Temperature(temperature)),
        }
    }
}
