pub mod command_sender;
pub mod commands;
use commands::TridCommand;
use trid::Trid;
use utilities::{command_executor::DeviceHandler, lazy_tcp::LazyTcpStream};

pub struct TridHandler {
    tcp_stream: LazyTcpStream,
    // FIXME: There's only one TRID
    trid: [Trid; 4],
}

impl DeviceHandler for TridHandler {
    type Command = TridCommand;
}

impl TridHandler {
    pub fn new(tcp_stream: LazyTcpStream, trid: [Trid; 4]) -> Self {
        Self { tcp_stream, trid }
    }

    fn get_temperature(&mut self, axis: u8) -> std::io::Result<f32> {
        let trid = self.trid.get(axis as usize).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Trid ID: {}", axis),
            )
        })?;

        trid.read_data(&mut self.tcp_stream)
    }

    pub fn reconnect(&mut self) -> std::io::Result<()> {
        self.tcp_stream.reconnect()
    }
}
