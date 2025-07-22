use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use command_sender::StandaCommandSender;
use commands::{CommandResponse, MotorCommand, MotorCommandType, SetMotorAttribute};
use standa::Standa;

use crate::lazy_tcp::LazyTcpStream;

pub mod command_sender;
pub mod commands;

struct Inner {
    tcp_stream: LazyTcpStream,
    standa: Standa,
}

impl Inner {
    pub fn stop(&mut self) -> io::Result<CommandResponse> {
        match self.standa.stop(&mut self.tcp_stream) {
            Ok(_) => Ok(CommandResponse::Ok),
            Err(e) => Err(e),
        }
    }

    pub fn move_relative(&mut self, steps: i32, substeps: i16) -> io::Result<CommandResponse> {
        match self
            .standa
            .move_relative(&mut self.tcp_stream, steps, substeps)
        {
            Ok(_) => Ok(CommandResponse::Ok),
            Err(e) => Err(e),
        }
    }

    pub fn handle_get_command(
        &mut self,
        attribute: &commands::GetMotorAttribute,
    ) -> io::Result<CommandResponse> {
        match attribute {
            commands::GetMotorAttribute::State => {
                let state = self.standa.get_state(&mut self.tcp_stream)?;
                Ok(CommandResponse::State(state))
            }
        }
    }

    pub fn handle_set_command(
        &mut self,
        attribute: &SetMotorAttribute,
    ) -> io::Result<CommandResponse> {
        match attribute {
            SetMotorAttribute::Velocity(velocity) => {
                self.standa.set_velocity(&mut self.tcp_stream, *velocity)?;
                Ok(CommandResponse::Ok)
            }
            SetMotorAttribute::Acceleration(acceleration) => {
                self.standa
                    .set_acceleration(&mut self.tcp_stream, *acceleration)?;
                Ok(CommandResponse::Ok)
            }
            SetMotorAttribute::Deceleration(deceleration) => {
                self.standa
                    .set_deceleration(&mut self.tcp_stream, *deceleration)?;
                Ok(CommandResponse::Ok)
            }
        }
    }

    pub fn reconnect(&mut self) -> io::Result<()> {
        self.tcp_stream.reconnect()
    }
}

pub struct StandaCommandExecutor {
    inner: Inner,

    commands_ch: Receiver<MotorCommand>,
    sender: Sender<MotorCommand>,
}

impl StandaCommandExecutor {
    pub fn new(tcp_stream: LazyTcpStream, standa: Standa) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        let inner = Inner { tcp_stream, standa };

        Self {
            inner,
            commands_ch,
            sender,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        for command in self.commands_ch.iter() {
            match command.command_type() {
                MotorCommandType::Get(attribute) => {
                    let response = self.inner.handle_get_command(attribute);

                    command.response_ch().send(response).unwrap();
                }
                MotorCommandType::Set(attribute) => {
                    let response = self.inner.handle_set_command(attribute);

                    command.response_ch().send(response).unwrap()
                }
                MotorCommandType::Stop => {
                    let response = self.inner.stop();

                    command.response_ch().send(response).unwrap();
                }
                MotorCommandType::Move { steps, substeps } => {
                    let response = self.inner.move_relative(*steps, *substeps);

                    command.response_ch().send(response).unwrap();
                }
                MotorCommandType::Reconnect => {
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

    pub fn sender(&self) -> StandaCommandSender {
        StandaCommandSender::new(self.sender.clone())
    }
}
