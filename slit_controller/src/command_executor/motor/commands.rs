use std::io;

use standa::command::state::StateParams;
use tokio::sync::oneshot::Sender;

pub enum MotorAttribute {
    Velocity,
    Acceleration,
    Deceleration,
    State,
}

pub enum MotorCommandType {
    Get(MotorAttribute),
    Set(),
}

pub struct MotorCommand {
    command_type: MotorCommandType,
    response_ch: Sender<io::Result<CommandResponse>>,
}

impl MotorCommand {
    pub fn new(
        command_type: MotorCommandType,
        response_ch: Sender<io::Result<CommandResponse>>,
    ) -> Self {
        Self {
            command_type,
            response_ch,
        }
    }

    pub fn command_type(&self) -> &MotorCommandType {
        &self.command_type
    }

    pub fn response_ch(self) -> Sender<io::Result<CommandResponse>> {
        self.response_ch
    }
}

#[derive(Debug)]
pub enum CommandResponse {
    None,
    State(StateParams),
}
