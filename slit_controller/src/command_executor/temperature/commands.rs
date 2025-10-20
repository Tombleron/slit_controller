use crate::command_executor::temperature::TridHandler;
use std::io;
use utilities::command_executor::Command;

#[derive(Clone)]
pub enum TridCommand {
    GetTemperature { axis: u8 },
}

impl Command for TridCommand {
    type Response = TridResponse;
    type Handler = TridHandler;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response> {
        match self {
            TridCommand::GetTemperature { axis } => handler
                .get_temperature(axis)
                .map(|temperature| TridResponse::Temperature(temperature)),
        }
    }
}

#[derive(Debug)]
pub enum TridResponse {
    Temperature(f32),
    Ok,
}
