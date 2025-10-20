use std::io;

use commands::MotorCommand;
use standa::{command::state::StateParams, Standa};

use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream};

pub mod command_sender;
pub mod commands;

pub struct StandaHandler {
    tcp_stream: LazyTcpStream,
    standa: Standa,
}

impl DeviceHandler for StandaHandler {
    type Command = MotorCommand;
}

impl StandaHandler {
    pub fn new(standa: Standa, tcp_stream: LazyTcpStream) -> Self {
        Self { tcp_stream, standa }
    }

    pub fn stop(&mut self) -> io::Result<()> {
        self.standa.stop(&mut self.tcp_stream)
    }

    pub fn move_relative(&mut self, steps: i32, substeps: i16) -> io::Result<()> {
        self.standa
            .move_relative(&mut self.tcp_stream, steps, substeps)
    }

    pub fn get_state(&mut self) -> io::Result<StateParams> {
        self.standa.get_state(&mut self.tcp_stream)
    }

    pub fn set_velocity(&mut self, velocity: u32) -> io::Result<()> {
        self.standa.set_velocity(&mut self.tcp_stream, velocity)
    }

    pub fn set_acceleration(&mut self, acceleration: u16) -> io::Result<()> {
        self.standa
            .set_acceleration(&mut self.tcp_stream, acceleration)
    }

    pub fn set_deceleration(&mut self, deceleration: u16) -> io::Result<()> {
        self.standa
            .set_deceleration(&mut self.tcp_stream, deceleration)
    }

    pub fn reconnect(&mut self) -> io::Result<()> {
        self.tcp_stream.reconnect()
    }
}
