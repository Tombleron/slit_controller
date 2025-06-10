use standa::command::state::StateParams;
use std::fmt;
use tokio::sync::oneshot;

pub type CommandResult = Result<CommandResponse, CommandError>;

#[derive(Debug, Clone)]
pub struct CommandError {
    pub code: u16,
    pub message: String,
}

impl From<String> for CommandError {
    fn from(message: String) -> Self {
        CommandError { code: 500, message }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
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
    Velocity(u32),
    Acceleration(u16),
    Deceleration(u16),
    PositionWindow(f32),
    Error(String),
}

#[derive(Debug)]
pub enum CommandParams {
    Position(f32),
    Velocity(u32),
    Acceleration(u16),
    Deceleration(u16),
    PositionWindow(f32),
    None,
}

#[derive(Debug)]
pub enum Command {
    Move {
        axis: usize,
        position: f32,
    },
    Stop {
        axis: usize,
    },

    Get {
        axis: usize,
        property: AxisProperty,
    },

    Set {
        axis: usize,
        property: AxisProperty,
        value: CommandParams,
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
    Velocity,
    Acceleration,
    Deceleration,
    PositionWindow,
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
    pub state: AxisStateValue<StateParams>,
    pub is_moving: AxisStateValue<bool>,
    pub velocity: AxisStateValue<u32>,
    pub acceleration: AxisStateValue<u16>,
    pub deceleration: AxisStateValue<u16>,
    pub position_window: AxisStateValue<f32>,
}

#[derive(Debug)]
pub struct SharedState {
    pub axes: [Option<AxisState>; 4],
}
