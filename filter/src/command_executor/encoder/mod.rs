use crate::command_executor::encoder::commands::EncoderCommand;
use lir::LIR;
use std::io;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream, modbus::ModbusError};
pub mod command_sender;
pub mod commands;

pub struct EncoderHandler {
    tcp_stream: LazyTcpStream,
    encoder: LIR,
}

impl EncoderHandler {
    pub fn new(tcp_stream: LazyTcpStream, encoder: LIR) -> Self {
        Self {
            tcp_stream,
            encoder,
        }
    }

    fn get_position(&mut self) -> io::Result<f32> {
        self.encoder
            .get_current_measurement(&mut self.tcp_stream, 3)
            .map_err(|e| match e {
                ModbusError::IoError(error) => io::Error::from(error),
                _ => io::Error::new(io::ErrorKind::Other, format!("{e}")),
            })
    }
}

impl DeviceHandler for EncoderHandler {
    type Command = EncoderCommand;
}
