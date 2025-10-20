use tokio::sync::oneshot;

use crate::controller::single_axis::MoveArgs;

pub mod commands;
pub mod service;

pub enum AxisProperty {
    Position,
    State,
    Moving,
    Temperature,
}

pub enum Command {
    Move {
        axis: usize,
        position: f32,
        params: Option<MoveArgs>,
    },
    Stop {
        axis: usize,
    },

    Get {
        axis: usize,
        property: AxisProperty,
    },
}

pub struct CommandError {
    pub message: String,
}

pub type CommandResult = Result<CommandResponse, CommandError>;

pub enum CommandResponse {
    Success,
    Position(f32),
    State(),
    Moving(bool),
    Temperature(f32),
    Error(String),
}

pub struct CommandEnvelope {
    pub command: Command,
    pub sender: oneshot::Sender<CommandResult>,
}
