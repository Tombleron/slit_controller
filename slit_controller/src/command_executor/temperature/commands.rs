use std::io;

use tokio::sync::oneshot::Sender;

pub enum GetTridAttribute {
    Temperature,
}

pub enum TridCommandType {
    Get(GetTridAttribute),
    Reconnect,
}

pub struct TridCommand {
    trid_id: u8,
    command_type: TridCommandType,
    response_ch: Sender<io::Result<CommandResponse>>,
}

impl TridCommand {
    pub fn new(
        trid_id: u8,
        command_type: TridCommandType,
        response_ch: Sender<io::Result<CommandResponse>>,
    ) -> Self {
        Self {
            trid_id,
            command_type,
            response_ch,
        }
    }

    pub fn trid_id(&self) -> u8 {
        self.trid_id
    }

    pub fn command_type(&self) -> &TridCommandType {
        &self.command_type
    }

    pub fn response_ch(self) -> Sender<io::Result<CommandResponse>> {
        self.response_ch
    }
}

#[derive(Debug)]
pub enum CommandResponse {
    None,
    Temperature(f32),
    Ok,
}
