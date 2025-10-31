use crate::command_executor::sensors::commands::SensorsCommand;
use lir::LIR;
use std::io;
use trid::Trid;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream, modbus::ModbusError};
pub mod command_sender;
pub mod commands;

pub struct SensorsHandler {
    tcp_stream: LazyTcpStream,
    encoders: Vec<LIR>,
    temperature: Vec<Trid>,
}

impl SensorsHandler {
    pub fn new(tcp_stream: LazyTcpStream, encoders: Vec<LIR>, temperature: Vec<Trid>) -> Self {
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
        let trid = self.temperature.get(axis as usize).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Trid ID: {}", axis),
            )
        })?;

        trid.read_data(&mut self.tcp_stream)
    }
}

impl DeviceHandler for SensorsHandler {
    type Command = SensorsCommand;
}
