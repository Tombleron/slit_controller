use crate::command_executor::motor::commands::{CommandResponse, MotorCommand};
use em2rs::Em2rs;
use std::io;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream};
pub mod command_sender;
pub mod commands;

pub struct Em2rsHandler {
    tcp_stream: LazyTcpStream,
    em2rs: Em2rs,
}

impl DeviceHandler for Em2rsHandler {
    type Command = MotorCommand;
}

impl Em2rsHandler {
    pub fn new(tcp_stream: LazyTcpStream, em2rs: Em2rs) -> Self {
        Self { tcp_stream, em2rs }
    }

    pub fn stop(&mut self) -> io::Result<CommandResponse> {
        match self.em2rs.stop(&mut self.tcp_stream) {
            Ok(_) => Ok(CommandResponse::Ok),
            Err(e) => Err(e.into()),
        }
    }

    pub fn move_relative(&mut self, steps: i32) -> io::Result<CommandResponse> {
        match self.em2rs.move_relative(&mut self.tcp_stream, steps) {
            Ok(_) => Ok(CommandResponse::Ok),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_state(&mut self) -> io::Result<CommandResponse> {
        let state = self.em2rs.get_state(&mut self.tcp_stream)?;
        Ok(CommandResponse::State(state))
    }

    pub fn set_velocity(&mut self, velocity: u16) -> io::Result<CommandResponse> {
        self.em2rs.set_velocity(&mut self.tcp_stream, velocity)?;
        Ok(CommandResponse::Ok)
    }

    pub fn set_acceleration(&mut self, acceleration: u16) -> io::Result<CommandResponse> {
        self.em2rs
            .set_acceleration(&mut self.tcp_stream, acceleration)?;
        Ok(CommandResponse::Ok)
    }

    pub fn set_deceleration(&mut self, deceleration: u16) -> io::Result<CommandResponse> {
        self.em2rs
            .set_deceleration(&mut self.tcp_stream, deceleration)?;
        Ok(CommandResponse::Ok)
    }
}
