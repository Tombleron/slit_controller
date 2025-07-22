pub mod command_sender;
pub mod commands;

use std::sync::mpsc::{Receiver, Sender};

use command_sender::TridCommandSender;
use commands::{CommandResponse, GetTridAttribute, TridCommand};
use trid::Trid;

use crate::lazy_tcp::LazyTcpStream;

struct Inner {
    tcp_stream: LazyTcpStream,
    trid: [Trid; 4],
}

impl Inner {
    fn get_temperature(&mut self, axis: u16) -> std::io::Result<f32> {
        let trid = self.trid.get(axis as usize).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Trid ID: {}", axis),
            )
        })?;

        trid.read_data(&mut self.tcp_stream, axis)
    }

    pub fn handle_get_command(
        &mut self,
        trid_id: u16,
        command: &commands::GetTridAttribute,
    ) -> std::io::Result<CommandResponse> {
        match command {
            GetTridAttribute::Temperature => {
                let temperature = self.get_temperature(trid_id)?;
                Ok(CommandResponse::Temperature(temperature))
            }
        }
    }

    pub fn reconnect(&mut self) -> std::io::Result<()> {
        self.tcp_stream.reconnect()
    }
}

pub struct TridCommandExecutor {
    inner: Inner,

    commands_ch: Receiver<TridCommand>,
    sender: Sender<TridCommand>,
}

impl TridCommandExecutor {
    pub fn new(tcp_stream: LazyTcpStream, trid: [Trid; 4]) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        let inner = Inner { tcp_stream, trid };

        Self {
            inner,
            commands_ch,
            sender,
        }
    }

    pub fn sender(&self) -> TridCommandSender {
        TridCommandSender::new(self.sender.clone())
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        while let Ok(command) = self.commands_ch.recv() {
            let trid_id = command.trid_id();
            let command_type = command.command_type();

            let response = match command_type {
                commands::TridCommandType::Get(attribute) => {
                    self.inner.handle_get_command(trid_id as u16, attribute)
                }
                commands::TridCommandType::Reconnect => match self.inner.reconnect() {
                    Ok(_) => Ok(CommandResponse::Ok),
                    Err(e) => Err(e),
                },
            };

            command.response_ch().send(response).unwrap()
        }

        Ok(())
    }
}
