use std::{fmt, sync::Arc};

use em2rs::StateParams;
use tokio::sync::{Mutex, oneshot};

use crate::controller::single_axis::MoveArgs;

type AxisStateValue<T> = Result<T, String>;

#[derive(Debug)]
pub struct AxisState {
    pub position: AxisStateValue<f32>,
    pub temperature: AxisStateValue<f32>,
    pub state: AxisStateValue<StateParams>,
    pub is_moving: AxisStateValue<bool>,
}

#[derive(Debug, Clone, Copy)]
pub enum AxisProperty {
    Position,
    State,
    Moving,
    Temperature,
}

pub struct CSlitState {
    upper: Arc<Mutex<AxisState>>,
    lower: Arc<Mutex<AxisState>>,
    left: Arc<Mutex<AxisState>>,
    right: Arc<Mutex<AxisState>>,
}

impl CSlitState {
    pub fn get_axis_state(&self, axis: usize) -> Option<Arc<Mutex<AxisState>>> {
        match axis {
            0 => Some(self.upper.clone()),
            1 => Some(self.lower.clone()),
            2 => Some(self.right.clone()),
            3 => Some(self.left.clone()),
            _ => None,
        }
    }
}

impl Default for CSlitState {
    fn default() -> Self {
        Self {
            upper: Arc::new(Mutex::new(AxisState {
                position: Err("Not initialized".to_string()),
                temperature: Err("Not initialized".to_string()),
                state: Err("Not initialized".to_string()),
                is_moving: Err("Not initialized".to_string()),
            })),
            lower: Arc::new(Mutex::new(AxisState {
                position: Err("Not initialized".to_string()),
                temperature: Err("Not initialized".to_string()),
                state: Err("Not initialized".to_string()),
                is_moving: Err("Not initialized".to_string()),
            })),
            left: Arc::new(Mutex::new(AxisState {
                position: Err("Not initialized".to_string()),
                temperature: Err("Not initialized".to_string()),
                state: Err("Not initialized".to_string()),
                is_moving: Err("Not initialized".to_string()),
            })),
            right: Arc::new(Mutex::new(AxisState {
                position: Err("Not initialized".to_string()),
                temperature: Err("Not initialized".to_string()),
                state: Err("Not initialized".to_string()),
                is_moving: Err("Not initialized".to_string()),
            })),
        }
    }
}

pub struct SharedState {
    pub cslit: CSlitState,
}

#[derive(Debug)]
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

impl Command {
    pub fn is_get(&self) -> bool {
        matches!(self, Command::Get { .. })
    }
}

#[derive(Debug)]
pub struct CommandEnvelope {
    pub command: Command,
    pub response: oneshot::Sender<CommandResult>,
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

pub type CommandResult = Result<CommandResponse, CommandError>;
