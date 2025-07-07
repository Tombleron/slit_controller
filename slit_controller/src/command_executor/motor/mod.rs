use std::{
    io,
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

use commands::{CommandResponse, MotorCommand, MotorCommandType};
use standa::{command::state::StateParams, Standa};

pub mod commands;

pub struct StandaControlExecutor {
    tcp_stream: TcpStream,
    standa: Standa,

    commands_ch: Receiver<MotorCommand>,
    sender: Sender<MotorCommand>,
}

impl StandaControlExecutor {
    pub fn new(tcp_stream: TcpStream, standa: Standa) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        Self {
            tcp_stream,
            standa,
            commands_ch,
            sender,
        }
    }

    fn get_state(&mut self) -> io::Result<StateParams> {
        self.standa.get_state(&mut self.tcp_stream)
    }

    fn handle_get_command(
        &mut self,
        attribute: &commands::MotorAttribute,
    ) -> io::Result<CommandResponse> {
        match attribute {
            commands::MotorAttribute::Velocity => Ok(CommandResponse::State(self.get_state()?)),
            commands::MotorAttribute::Acceleration => {
                Ok(CommandResponse::None) // Replace with actual response
            }
            commands::MotorAttribute::Deceleration => {
                Ok(CommandResponse::None) // Replace with actual response
            }
            commands::MotorAttribute::State => {
                Ok(CommandResponse::None) // Replace with actual response
            }
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        for command in self.commands_ch.iter() {
            match command.command_type() {
                MotorCommandType::Get(attribute) => {
                    let result = self.handle_get_command(attribute);
                    command.response_ch().send(result).unwrap();
                }
                MotorCommandType::Set() => command
                    .response_ch()
                    .send(Ok(CommandResponse::None))
                    .unwrap(),
            }
        }

        Ok(())
    }

    pub fn sender(&self) -> Sender<MotorCommand> {
        self.sender.clone()
    }
}
