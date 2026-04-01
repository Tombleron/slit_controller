use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};

use tokio::sync::oneshot;

pub trait Command: Send {
    type Response: Send;
    type Handler: DeviceHandler<Command = Self>;

    fn execute(self, handler: &mut Self::Handler) -> io::Result<Self::Response>;
}

pub trait DeviceHandler {
    type Command: Command<Handler = Self>;
}

pub struct GenericCommand<C: Command> {
    command: C,
    response_ch: oneshot::Sender<io::Result<C::Response>>,
}

impl<C: Command> GenericCommand<C> {
    pub fn new(command: C, response_ch: oneshot::Sender<io::Result<C::Response>>) -> Self {
        Self {
            command,
            response_ch,
        }
    }

    pub fn execute(self, handler: &mut C::Handler) -> io::Result<()> {
        let result = self.command.execute(handler);

        self.response_ch
            .send(result)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to send response"))?;

        Ok(())
    }
}

pub struct CommandExecutor<H: DeviceHandler + Send + 'static> {
    handler: H,
    commands_ch: Receiver<GenericCommand<H::Command>>,
    sender: Sender<GenericCommand<H::Command>>,
}

impl<H: DeviceHandler + Send> CommandExecutor<H> {
    pub fn new(handler: H) -> Self {
        let (sender, commands_ch) = std::sync::mpsc::channel();

        Self {
            handler,
            commands_ch,
            sender,
        }
    }

    pub fn sender(&self) -> CommandSender<H::Command> {
        CommandSender::new(self.sender.clone())
    }

    pub fn run(&mut self) -> io::Result<()> {
        while let Ok(command) = self.commands_ch.recv() {
            if let Err(_) = command.execute(&mut self.handler) {
                // TODO: atleast log the error
                continue;
            }
        }

        Ok(())
    }

    pub fn spawn(mut self) -> tokio::task::JoinHandle<io::Result<()>> {
        tokio::task::spawn_blocking(move || self.run())
    }
}

#[derive(Clone)]
pub struct CommandSender<T: Command> {
    commands_ch: Sender<GenericCommand<T>>,
}

impl<C: Command> CommandSender<C> {
    pub fn new(commands_ch: Sender<GenericCommand<C>>) -> Self {
        Self { commands_ch }
    }

    pub async fn send_command(&self, command: C) -> io::Result<C::Response> {
        let (response_ch, response_rx) = oneshot::channel();
        let command = GenericCommand::new(command, response_ch);

        self.commands_ch
            .send(command)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to send command"))?;

        response_rx
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to receive response"))?
    }
}
