use standa::command::state::StateParams;
use std::fmt;
use tokio::sync::oneshot;

use crate::controller::single_axis::MovementParams;

pub type CommandResult = Result<CommandResponse, CommandError>;

#[derive(Debug, Clone)]
pub struct CommandError {
    pub message: String,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        CommandError { message }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub enum State {
    On,
    Moving,
    Fault,
}

#[derive(Debug, Clone)]
pub enum Limit {
    Upper,
    Lower,
    Both,
    None,
}

#[derive(Debug, Clone)]
pub enum CommandResponse {
    Success,
    Position(f32),
    State((State, Limit)),
    Moving(bool),
    Temperature(f32),
    Error(String),
}

#[derive(Debug)]
pub enum CommandParams {
    Position(f32),
    Velocity(u32),
    Temperature(u16),
    None,
}

#[derive(Debug)]
pub enum Command {
    Move {
        axis: usize,
        position: f32,
        params: Option<MovementParams>,
    },
    Stop {
        axis: usize,
    },

    Get {
        axis: usize,
        property: AxisProperty,
    },
}

impl Command {
    pub fn is_get(&self) -> bool {
        matches!(self, Command::Get { .. })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AxisProperty {
    Position,
    State,
    Moving,
    Temperature,
}

#[derive(Debug)]
pub struct CommandEnvelope {
    pub command: Command,
    pub response: oneshot::Sender<CommandResult>,
}

type AxisStateValue<T> = Result<T, String>;

#[derive(Debug)]
pub struct AxisState {
    pub position: AxisStateValue<f32>,
    pub temperature: AxisStateValue<f32>,
    pub state: AxisStateValue<StateParams>,
    pub is_moving: AxisStateValue<bool>,
}

#[derive(Debug)]
pub struct SharedState {
    pub axes: [Option<AxisState>; 4],
}
