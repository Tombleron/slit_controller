use crate::command_executor::sensors::commands::SensorsCommand;
use icpcon::M7015;
use lir::LIR;
use std::io;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream, modbus::ModbusError};
pub mod command_sender;
pub mod commands;

pub struct SensorsHandler {
    tcp_stream: LazyTcpStream,
    encoders: Vec<LIR>,
    temperature: M7015,
}

impl SensorsHandler {
    pub fn new(tcp_stream: LazyTcpStream, encoders: Vec<LIR>, temperature: M7015) -> Self {
        Self {
            tcp_stream,
            encoders,
            temperature,
        }
    }

    fn get_position(&mut self, axis: u8) -> io::Result<f32> {
        self.encoders
            .get(axis as usize)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, format!("Invalid axis")))?
            .get_current_measurement(&mut self.tcp_stream, 3)
            .map_err(|e| match e {
                ModbusError::IoError(error) => io::Error::from(error),
                _ => io::Error::new(io::ErrorKind::Other, format!("{e}")),
            })
    }

    fn get_temperature(&mut self, axis: u8) -> io::Result<f32> {
        self.temperature
            .get_current_measurement(&mut self.tcp_stream, axis, 3)
            .map_err(|e| match e {
                ModbusError::IoError(error) => io::Error::from(error),
                _ => io::Error::new(io::ErrorKind::Other, format!("{e}")),
            })
    }
}

impl DeviceHandler for SensorsHandler {
    type Command = SensorsCommand;
}
