use std::io;

use standa::command::state::StateParams;
use tokio::sync::oneshot::Sender;

pub enum GetMotorAttribute {
    State,
}

pub enum SetMotorAttribute {
    Velocity(u32),
    Acceleration(u16),
    Deceleration(u16),
}

pub enum MotorCommandType {
    Get(GetMotorAttribute),
    Set(SetMotorAttribute),
    Stop,
    Move { steps: i32, substeps: i16 },
    Reconnect,
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
    Ok,
}
