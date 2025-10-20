use std::io::{Read, Write};

use utilities::modbus::{Modbus, ModbusError};

pub struct M7015 {
    client: Modbus,
}

impl M7015 {
    pub fn new(id: u8) -> Self {
        let modbus = Modbus::new(id);
        Self { client: modbus }
    }

    pub fn get_current_measurement(
        &self,
        client: &mut (impl Write + Read),
        channel: u8,
        retries: u8,
    ) -> Result<f32, ModbusError> {
        for t in 0..retries {
            match self.client.read_input_registers(client, 0x00, 6) {
                Ok(response) => {
                    if response.len() != 6 {
                        return Err(ModbusError::InvalidResponseLength {
                            expected: 6,
                            received: response.len(),
                        });
                    }

                    return Ok(response[channel as usize] as f32 / 10.0);
                }
                Err(e) => {
                    if t == retries - 1 {
                        return Err(e);
                    }
                }
            }
        }

        unreachable!()
    }
}
