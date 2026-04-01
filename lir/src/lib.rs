use std::io::{Read, Write};

use utilities::modbus::{Modbus, ModbusError};

pub struct LIR {
    client: Modbus,
    step: f32,
}

impl LIR {
    pub fn new(id: u8, step: f32) -> Self {
        let modbus = Modbus::new(id);
        Self {
            client: modbus,
            step,
        }
    }

    pub fn get_current_measurement(
        &self,
        client: &mut (impl Write + Read),
        retries: u8,
    ) -> Result<f32, ModbusError> {
        for t in 0..retries {
            match self.client.read_input_registers(client, 0x00, 5) {
                Ok(response) => {
                    if response.len() != 5 {
                        return Err(ModbusError::InvalidResponseLength {
                            expected: 5,
                            received: response.len(),
                        });
                    }

                    let result = i32::from_le_bytes([
                        (response[1] & 0xFF) as u8,
                        (response[1] >> 8) as u8,
                        (response[2] & 0xFF) as u8,
                        (response[2] >> 8) as u8,
                    ]);
                    return Ok(result as f32 * self.step);
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
