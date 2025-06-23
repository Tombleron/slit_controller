use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::State;

#[derive(Debug, Clone, Copy)]
pub struct Rf256 {
    device_id: u8,
}

impl Rf256 {
    pub fn new(device_id: u8) -> Self {
        Rf256 { device_id }
    }

    pub fn get_device_id(&self) -> u8 {
        self.device_id
    }

    pub fn set_device_id(&mut self, device_id: u8) {
        self.device_id = device_id;
    }

    fn convert_bytes_to_float(&self, data: &[u8]) -> f32 {
        if data.len() != 4 {
            panic!("Data must be exactly 4 bytes long");
        }

        let raw_value = i32::from_le_bytes(data.try_into().unwrap());

        raw_value as f32 / 10000.0
    }

    async fn send_command(
        &self,
        sender: &mut (impl AsyncWrite + Unpin),
        command: u8,
        msg: Option<&[u8]>,
    ) -> std::io::Result<()> {
        let mut packet = Vec::new();

        packet.push(self.device_id);
        packet.push(command | 0x80);

        if let Some(msg) = msg {
            for &byte in msg {
                packet.push(0x80 | (byte & 0x0F));
                packet.push(0x80 | ((byte >> 4) & 0x0F));
            }
        }

        sender.write_all(&packet).await?;

        Ok(())
    }

    pub async fn read_response(
        &self,
        sender: &mut (impl AsyncRead + Unpin),
        expected_len: usize,
    ) -> std::io::Result<Vec<u8>> {
        let mut raw = vec![0; expected_len * 2];

        match sender.read_exact(&mut raw).await {
            Ok(_) => {}
            Err(e) => return Err(e),
        }

        let mut decoded = vec![];
        let mut counters = vec![];

        for chunk in raw.chunks(2) {
            if chunk.len() != 2 || chunk[0] & 0x80 == 0 || chunk[1] & 0x80 == 0 {
                let mut buf = vec![0; 256];
                let _ = sender.read_to_end(&mut buf).await;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid response format",
                ));
            }

            let low = chunk[0] & 0x0F;
            let high = chunk[1] & 0x0F;

            let low_counter = chunk[0] >> 4;
            let high_counter = chunk[1] >> 4;

            decoded.push(low | (high << 4));
            counters.push(low_counter);
            counters.push(high_counter);
        }

        // all counters must be the same
        if !counters.windows(2).all(|w| w[0] == w[1]) {
            let mut buf = vec![0; 256];
            let _ = sender.read_to_end(&mut buf).await;
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Counters do not match",
            ));
        }

        Ok(decoded)
    }

    pub async fn read_data(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::io::Result<f32> {
        self.send_command(sender, 0x06, None).await?;
        let response = self.read_response(sender, 4).await?;

        Ok(self.convert_bytes_to_float(&response))
    }

    async fn read_parameter(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
        parameter: u8,
    ) -> std::io::Result<u8> {
        self.send_command(sender, 0x02, Some(&[parameter])).await?;

        let response = self.read_response(sender, 1).await?;

        Ok(response[0])
    }

    async fn write_parameter(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
        parameter: u8,
        value: u8,
    ) -> std::io::Result<()> {
        self.send_command(sender, 0x03, Some(&[parameter, value]))
            .await?;
        Ok(())
    }

    pub async fn read_state(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::io::Result<State> {
        let value = self.read_parameter(sender, 0x00).await?;

        bincode::deserialize::<State>(&[value])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn read_id(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::io::Result<u8> {
        self.read_parameter(sender, 0x02).await
    }

    pub async fn set_id(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
        id: u8,
    ) -> std::io::Result<()> {
        self.write_parameter(sender, 0x02, id).await
    }

    pub async fn read_baudrate(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::io::Result<u32> {
        self.read_parameter(sender, 0x03)
            .await
            .map(|v| v as u32 * 2400)
    }

    pub async fn set_baudrate(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
        baudrate: u32,
    ) -> std::io::Result<()> {
        let value = (baudrate / 2400) as u8;
        self.write_parameter(sender, 0x03, value).await
    }

    pub async fn save_to_flash(
        &self,
        sender: &mut (impl AsyncRead + AsyncWrite + Unpin),
    ) -> std::io::Result<()> {
        self.send_command(sender, 0x04, Some(&[0xAA])).await?;

        let response = self.read_response(sender, 1).await?;

        if response.is_empty() || response[0] != 0xAA {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to save to flash",
            ));
        }

        Ok(())
    }
}
