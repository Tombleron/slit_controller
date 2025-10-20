use tokio::sync::oneshot::Sender;

pub enum GetSensorsAttribute {
    Position,
    Temperature,
}

pub enum SensorsCommandType {
    Get(GetSensorsAttribute),
    Reconnect,
}

#[derive(Debug)]
pub enum CommandResponse {
    None,
    Temperature(f32),
    Position(f32),
    Ok,
}

pub struct SensorsCommand {
    axis: u8,
    command_type: SensorsCommandType,
    response_ch: Sender<std::io::Result<CommandResponse>>,
}

impl SensorsCommand {
    pub fn new(
        axis: u8,
        command_type: SensorsCommandType,
        response_ch: Sender<std::io::Result<CommandResponse>>,
    ) -> Self {
        Self {
            axis,
            command_type,
            response_ch,
        }
    }

    pub fn command_type(&self) -> &SensorsCommandType {
        &self.command_type
    }

    pub fn response_ch(self) -> Sender<std::io::Result<CommandResponse>> {
        self.response_ch
    }

    pub fn axis(&self) -> u8 {
        self.axis
    }
}
