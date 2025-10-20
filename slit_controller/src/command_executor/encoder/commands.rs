use utilities::command_executor::Command;

use crate::command_executor::encoder::Rf256Handler;

const MAX_RETRIES: u8 = 5;

#[derive(Clone)]
pub enum EncoderCommand {
    GetPosition { axis: u8 },
}

pub enum EncoderResponse {
    Position { axis: u8, position: f32 },
}

impl Command for EncoderCommand {
    type Response = EncoderResponse;
    type Handler = Rf256Handler;

    fn execute(self, handler: &mut Self::Handler) -> std::io::Result<Self::Response> {
        match self {
            EncoderCommand::GetPosition { axis } => {
                let mut attempts = 0;
                loop {
                    match handler.get_position(axis) {
                        Ok(position) => return Ok(EncoderResponse::Position { axis, position }),
                        Err(_) if attempts < MAX_RETRIES => {
                            attempts += 1;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
}
