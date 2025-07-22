pub mod command_sender;
pub mod commands;

use std::io::{ErrorKind, Read as _};

use command_sender::Rf256CommandSender;
use commands::{CommandResponse, EncoderCommand, EncoderCommandType, GetEncoderAttribute};
use rf256::Rf256;
use std::sync::mpsc::{Receiver, Sender};

use crate::lazy_tcp::LazyTcpStream;

struct Inner {
    tcp_stream: LazyTcpStream,
    rf256: [Rf256; 4],
}

impl Inner {
    fn get_position(&mut self, axis: u8) -> std::io::Result<f32> {
        match self.verify_id(axis) {
            Ok(_) => {}
            Err(_) => {
                self.clear_buffer()?;
                self.verify_id(axis)?;
            }
        }

        self.rf256[axis as usize].read_data(&mut self.tcp_stream)
    }

    fn get_id(&mut self) -> std::io::Result<u8> {
        self.rf256[0].read_id(&mut self.tcp_stream)
    }

    fn verify_id(&mut self, axis: u8) -> std::io::Result<()> {
        let id = self.rf256[axis as usize].get_device_id();

        if id != self.rf256[0].read_id(&mut self.tcp_stream)? {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Device ID mismatch: expected {}, got {}", axis, id),
            ));
        }
        Ok(())
    }

    fn clear_buffer(&mut self) -> std::io::Result<()> {
        let mut buf = [0; 1024];

        match self.tcp_stream.read(&mut buf) {
            Ok(_) => Ok(()),
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn reconnect(&mut self) -> std::io::Result<()> {
        self.tcp_stream.reconnect()
    }

    pub fn handle_get_command(
        &mut self,
        axis: u8,
        attribute: &GetEncoderAttribute,
    ) -> std::io::Result<CommandResponse> {
        match attribute {
            GetEncoderAttribute::Position => {
                let position = self.get_position(axis)?;
                Ok(CommandResponse::Position(position))
            }
            GetEncoderAttribute::Id => {
                let id = self.get_id()?;
                Ok(CommandResponse::Id(id))
            }
            GetEncoderAttribute::PositionWithRetries(retries) => {
                let mut attempts = 0;
                loop {
                    match self.get_position(axis) {
                        Ok(position) => return Ok(CommandResponse::Position(position)),
                        Err(_) if attempts < *retries => {
                            attempts += 1;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
}

pub struct Rf256CommandExecutor {
    inner: Inner,

    commands_ch: Receiver<EncoderCommand>,
    sender: Sender<EncoderCommand>,
}

impl Rf256CommandExecutor {
    pub fn new(tcp_stream: LazyTcpStream, rf256: [Rf256; 4]) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        let inner = Inner { tcp_stream, rf256 };

        Self {
            inner,
            commands_ch,
            sender,
        }
    }

    pub fn sender(&self) -> Rf256CommandSender {
        Rf256CommandSender::new(self.sender.clone())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        for command in self.commands_ch.iter() {
            match command.command_type() {
                EncoderCommandType::Get(attribute) => {
                    let response = self.inner.handle_get_command(command.axis(), attribute);

                    match response {
                        Ok(resp) => {
                            command.response_ch().send(Ok(resp)).unwrap();
                        }
                        Err(e) => {
                            command.response_ch().send(Err(e)).unwrap();
                        }
                    }
                }
                EncoderCommandType::Reconnect => {
                    let response = self.inner.reconnect();

                    match response {
                        Ok(_) => {
                            command.response_ch().send(Ok(CommandResponse::Ok)).unwrap();
                        }
                        Err(e) => {
                            command.response_ch().send(Err(e)).unwrap();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
