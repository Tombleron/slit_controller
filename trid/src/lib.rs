use std::io::{Read, Write};

#[derive(Debug, Clone, Copy)]
pub struct Trid {
    device_id: u8,
    axis: u16,
}

impl Trid {
    pub fn new(device_id: u8, axis: u16) -> Self {
        Trid { device_id, axis }
    }

    pub fn get_device_id(&self) -> u8 {
        self.device_id
    }

    pub fn set_device_id(&mut self, device_id: u8) {
        self.device_id = device_id;
    }

    pub fn read_holding_register(
        &self,
        sender: &mut (impl Write + Read),
        register_address: u16,
    ) -> std::io::Result<Vec<u8>> {
        let register_count = 1;

        let mut request = vec![
            self.device_id,
            0x03,
            (register_address >> 8) as u8,
            (register_address & 0xFF) as u8,
            (register_count >> 8) as u8,
            (register_count & 0xFF) as u8,
        ];

        let crc = self.calculate_crc(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        sender.write_all(&request)?;

        let mut response_header = vec![0; 3];
        sender.read_exact(&mut response_header)?;

        if response_header[0] != self.device_id || response_header[1] != 0x03 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid response header",
            ));
        }

        let byte_count = response_header[2] as usize;
        let mut response_data = vec![0; byte_count + 2];
        sender.read_exact(&mut response_data)?;

        let mut full_response = response_header.clone();
        full_response.extend_from_slice(&response_data);

        let received_crc =
            ((response_data[byte_count + 1] as u16) << 8) | (response_data[byte_count] as u16);
        let calculated_crc = self.calculate_crc(&full_response[0..full_response.len() - 2]);

        if received_crc != calculated_crc {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CRC check failed",
            ));
        }

        Ok(response_data[0..byte_count].to_vec())
    }

    fn calculate_crc(&self, data: &[u8]) -> u16 {
        let mut crc = 0xFFFF;

        for byte in data {
            crc ^= *byte as u16;

            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc >>= 1;
                    crc ^= 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc
    }

    pub fn read_data(&self, sender: &mut (impl Write + Read)) -> std::io::Result<f32> {
        let result = self.read_holding_register(sender, self.axis)?;
        if result.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Response too short",
            ));
        }

        let value = (((result[0] as u16) << 8) | (result[1] as u16)) as f32 / 10.0;

        if value < 0.0 || value > 200.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Temp sensors are missing",
            ));
        }

        Ok(value)
    }
}
