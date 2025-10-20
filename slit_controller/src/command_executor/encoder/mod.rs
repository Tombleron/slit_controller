use std::io::{self, ErrorKind, Read as _};

use rf256::Rf256;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream};

use crate::command_executor::encoder::commands::EncoderCommand;

pub mod command_sender;
pub mod commands;

pub struct Rf256Handler {
    tcp_stream: LazyTcpStream,
    rf256: [Rf256; 4],
}

impl Rf256Handler {
    pub fn new(tcp_stream: LazyTcpStream, rf256: [Rf256; 4]) -> Self {
        Self { tcp_stream, rf256 }
    }

    fn get_position(&mut self, axis: u8) -> io::Result<f32> {
        match self.verify_id(axis) {
            Ok(_) => {}
            Err(_) => {
                self.clear_buffer()?;
                self.verify_id(axis)?;
            }
        }

        self.rf256
            .get(axis as usize)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid axis"))?
            .read_data(&mut self.tcp_stream)
    }

    fn verify_id(&mut self, axis: u8) -> io::Result<()> {
        let id = self
            .rf256
            .get(axis as usize)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid axis"))?
            .get_device_id();

        let requested_id = self.rf256[axis as usize].read_id(&mut self.tcp_stream)?;

        if id != requested_id {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("Device ID mismatch: expected {}, got {}", id, requested_id),
            ));
        }
        Ok(())
    }

    fn clear_buffer(&mut self) -> io::Result<()> {
        let mut buf = [0; 1024];

        match self.tcp_stream.read(&mut buf) {
            Ok(_) => Ok(()),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl DeviceHandler for Rf256Handler {
    type Command = EncoderCommand;
}
