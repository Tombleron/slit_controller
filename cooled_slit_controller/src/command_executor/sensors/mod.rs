use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use icpcon::M7015;
use lir::LIR;
use utilities::{lazy_tcp::LazyTcpStream, modbus::ModbusError};

use crate::command_executor::sensors::{
    command_sender::SensorsCommandSender, commands::GetSensorsAttribute,
};

pub mod command_sender;
pub mod commands;

struct Inner {
    tcp_stream: LazyTcpStream,
    encoders: Vec<LIR>,
    temperature: M7015,
}

impl Inner {
    fn get_position(&mut self, axis: u8) -> io::Result<f32> {
        self.encoders[axis as usize]
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

    fn reconnect(&mut self) -> io::Result<()> {
        self.tcp_stream.reconnect()
    }

    pub fn handle_get_command(
        &mut self,
        axis: u8,
        attribute: &GetSensorsAttribute,
    ) -> io::Result<commands::CommandResponse> {
        match attribute {
            GetSensorsAttribute::Position => {
                let position = self.get_position(axis)?;
                Ok(commands::CommandResponse::Position(position))
            }
            GetSensorsAttribute::Temperature => {
                let temperature = self.get_temperature(axis)?;
                Ok(commands::CommandResponse::Temperature(temperature))
            }
        }
    }
}

pub struct SensorsCommandExecutor {
    inner: Inner,

    commands_ch: Receiver<commands::SensorsCommand>,
    sender: Sender<commands::SensorsCommand>,
}

impl SensorsCommandExecutor {
    pub fn new(tcp_stream: LazyTcpStream, encoders: Vec<LIR>, temperature: M7015) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        let inner = Inner {
            tcp_stream,
            encoders,
            temperature,
        };

        Self {
            inner,
            commands_ch,
            sender,
        }
    }

    pub fn command_sender(&self) -> SensorsCommandSender {
        SensorsCommandSender::new(self.sender.clone())
    }

    pub fn run(&mut self) -> io::Result<()> {
        while let Ok(command) = self.commands_ch.recv() {
            let result = match command.command_type() {
                commands::SensorsCommandType::Get(attribute) => {
                    self.inner.handle_get_command(command.axis(), &attribute)
                }
                commands::SensorsCommandType::Reconnect => self
                    .inner
                    .reconnect()
                    .map(|_| commands::CommandResponse::Ok),
            };

            if let Err(e) = command.response_ch().send(result) {
                eprintln!("Failed to send response: {:?}", e);
            }
        }

        Ok(())
    }
}
