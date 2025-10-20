use std::{
    io,
    sync::mpsc::{Receiver, SyncSender},
};

use em2rs::Em2rs;
use utilities::lazy_tcp::LazyTcpStream;

use crate::command_executor::motor::commands::{CommandResponse, MotorCommand};

pub mod command_sender;
pub mod commands;

struct Inner<const LOW_LIMIT: u8, const HIGH_LIMIT: u8> {
    tcp_stream: LazyTcpStream,
    em2rs: Em2rs<LOW_LIMIT, HIGH_LIMIT>,
}

impl<const LOW_LIMIT: u8, const HIGH_LIMIT: u8> Inner<LOW_LIMIT, HIGH_LIMIT> {
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

    pub fn handle_get_command(
        &mut self,
        attribute: &commands::GetMotorAttribute,
    ) -> io::Result<CommandResponse> {
        match attribute {
            commands::GetMotorAttribute::State => {
                let state = self.em2rs.get_state(&mut self.tcp_stream)?;
                Ok(CommandResponse::State(state))
            }
        }
    }

    pub fn handle_set_command(
        &mut self,
        attribute: &commands::SetMotorAttribute,
    ) -> io::Result<CommandResponse> {
        match attribute {
            commands::SetMotorAttribute::Velocity(velocity) => {
                self.em2rs.set_velocity(&mut self.tcp_stream, *velocity)?;
                Ok(CommandResponse::Ok)
            }
            commands::SetMotorAttribute::Acceleration(acceleration) => {
                self.em2rs
                    .set_acceleration(&mut self.tcp_stream, *acceleration)?;
                Ok(CommandResponse::Ok)
            }
            commands::SetMotorAttribute::Deceleration(deceleration) => {
                self.em2rs
                    .set_deceleration(&mut self.tcp_stream, *deceleration)?;
                Ok(CommandResponse::Ok)
            }
        }
    }

    pub fn reconnect(&mut self) -> io::Result<()> {
        self.tcp_stream.reconnect()
    }
}

pub struct Em2rsCommandExecutor<const LOW_LIMIT: u8, const HIGH_LIMIT: u8> {
    inner: Inner<LOW_LIMIT, HIGH_LIMIT>,

    commands_ch: Receiver<MotorCommand>,
    sender: SyncSender<MotorCommand>,
}

impl<const LOW_LIMIT: u8, const HIGH_LIMIT: u8> Em2rsCommandExecutor<LOW_LIMIT, HIGH_LIMIT> {
    pub fn new(tcp_stream: LazyTcpStream, em2rs: Em2rs<LOW_LIMIT, HIGH_LIMIT>) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::sync_channel(100);

        let inner = Inner { tcp_stream, em2rs };

        Self {
            inner,
            commands_ch,
            sender,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        for command in self.commands_ch.iter() {
            let response = match command.command_type() {
                commands::MotorCommandType::Get(attr) => self.inner.handle_get_command(attr),
                commands::MotorCommandType::Set(attr) => self.inner.handle_set_command(attr),
                commands::MotorCommandType::Stop => self.inner.stop(),
                commands::MotorCommandType::Move { steps } => self.inner.move_relative(*steps),
                commands::MotorCommandType::Reconnect => match self.inner.reconnect() {
                    Ok(_) => Ok(CommandResponse::Ok),
                    Err(e) => Err(e),
                },
            };

            let _ = command.response_ch().send(response);
        }

        Ok(())
    }

    pub fn sender(&self) -> SyncSender<MotorCommand> {
        self.sender.clone()
    }
}
