use std::error::Error;
use std::fmt;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum ModbusError {
    IoError(std::io::Error),
    InvalidCrc {
        expected: u16,
        received: u16,
    },
    InvalidResponseLength {
        expected: usize,
        received: usize,
    },
    InvalidSlaveId {
        expected: u8,
        received: u8,
    },
    InvalidFunctionCode {
        expected: u8,
        received: u8,
    },
    ExceptionResponse {
        function_code: u8,
        exception_code: u8,
    },
    Timeout,
    ProtocolError(String),
}

impl fmt::Display for ModbusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModbusError::IoError(err) => write!(f, "IO error: {}", err),
            ModbusError::InvalidCrc { expected, received } => {
                write!(
                    f,
                    "CRC error: expected 0x{:04X}, received 0x{:04X}",
                    expected, received
                )
            }
            ModbusError::InvalidResponseLength { expected, received } => {
                write!(
                    f,
                    "Invalid response length: expected {}, received {}",
                    expected, received
                )
            }
            ModbusError::InvalidSlaveId { expected, received } => {
                write!(
                    f,
                    "Invalid slave ID: expected {}, received {}",
                    expected, received
                )
            }
            ModbusError::InvalidFunctionCode { expected, received } => {
                write!(
                    f,
                    "Invalid function code: expected 0x{:02X}, received 0x{:02X}",
                    expected, received
                )
            }
            ModbusError::ExceptionResponse {
                function_code,
                exception_code,
            } => {
                let exception_msg = match exception_code {
                    0x01 => "Illegal Function",
                    0x02 => "Illegal Data Address",
                    0x03 => "Illegal Data Value",
                    0x04 => "Slave Device Failure",
                    0x05 => "Acknowledge",
                    0x06 => "Slave Device Busy",
                    0x07 => "Negative Acknowledge",
                    0x08 => "Memory Parity Error",
                    0x0A => "Gateway Path Unavailable",
                    0x0B => "Gateway Target Device Failed To Respond",
                    _ => "Unknown Exception",
                };
                write!(
                    f,
                    "Modbus exception (function 0x{:02X}): {} (0x{:02X})",
                    function_code, exception_msg, exception_code
                )
            }
            ModbusError::Timeout => write!(f, "Request timed out"),
            ModbusError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl Error for ModbusError {}

impl From<ModbusError> for std::io::Error {
    fn from(error: ModbusError) -> Self {
        match error {
            ModbusError::IoError(error) => std::io::Error::from(error),
            _ => std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
        }
    }
}

impl From<std::io::Error> for ModbusError {
    fn from(error: std::io::Error) -> Self {
        ModbusError::IoError(error)
    }
}

pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

pub fn calculate_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;

    for &byte in data {
        crc ^= byte as u16;
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

#[derive(Clone)]
pub struct Modbus {
    id: u8,
}

impl Modbus {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn set_id(&mut self, id: u8) -> &mut Self {
        self.id = id;
        self
    }

    fn send_receive<T: Read + Write>(
        &self,
        client: &mut T,
        request: &[u8],
        min_response_len: usize,
    ) -> Result<Vec<u8>, ModbusError> {
        client.write_all(request)?;

        let mut buffer = vec![0; 256];
        let mut bytes_read = 0;

        client.read_exact(&mut buffer[0..1])?;
        bytes_read += 1;

        client.read_exact(&mut buffer[1..2])?;
        bytes_read += 1;

        if buffer[1] & 0x80 == 0x80 {
            client.read_exact(&mut buffer[2..3])?;

            client.read_exact(&mut buffer[3..5])?;

            let received_crc = ((buffer[4] as u16) << 8) | (buffer[3] as u16);
            let calculated_crc = calculate_crc16(&buffer[0..3]);

            if calculated_crc != received_crc {
                return Err(ModbusError::InvalidCrc {
                    expected: calculated_crc,
                    received: received_crc,
                });
            }

            return Err(ModbusError::ExceptionResponse {
                function_code: buffer[1] & 0x7F,
                exception_code: buffer[2],
            });
        }

        let remaining_bytes = if buffer[1] == FunctionCode::ReadCoils as u8
            || buffer[1] == FunctionCode::ReadDiscreteInputs as u8
            || buffer[1] == FunctionCode::ReadHoldingRegisters as u8
            || buffer[1] == FunctionCode::ReadInputRegisters as u8
        {
            client.read_exact(&mut buffer[2..3])?;
            bytes_read += 1;

            buffer[2] as usize + 2
        } else {
            min_response_len.saturating_sub(bytes_read)
        };

        if remaining_bytes > 0 {
            client.read_exact(&mut buffer[bytes_read..(bytes_read + remaining_bytes)])?;
            bytes_read += remaining_bytes;
        }

        buffer.truncate(bytes_read);

        if buffer.len() < min_response_len {
            return Err(ModbusError::InvalidResponseLength {
                expected: min_response_len,
                received: buffer.len(),
            });
        }

        if buffer[0] != self.id {
            return Err(ModbusError::InvalidSlaveId {
                expected: self.id,
                received: buffer[0],
            });
        }

        let data_len = buffer.len() - 2;
        let received_crc = ((buffer[data_len + 1] as u16) << 8) | (buffer[data_len] as u16);
        let calculated_crc = calculate_crc16(&buffer[0..data_len]);

        if calculated_crc != received_crc {
            return Err(ModbusError::InvalidCrc {
                expected: calculated_crc,
                received: received_crc,
            });
        }

        Ok(buffer)
    }

    pub fn read_holding_registers<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        count: u16,
    ) -> Result<Vec<u16>, ModbusError> {
        if count == 0 || count > 125 {
            return Err(ModbusError::ProtocolError(
                "Invalid register count. Must be between 1 and 125".to_string(),
            ));
        }

        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::ReadHoldingRegisters as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count >> 8) as u8);
        request.push(count as u8);

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 5 + (count * 2) as usize;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::ReadHoldingRegisters as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::ReadHoldingRegisters as u8,
                received: response[1],
            });
        }

        let byte_count = response[2] as usize;
        if byte_count != (count * 2) as usize {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected byte count. Expected {}, received {}",
                count * 2,
                byte_count
            )));
        }

        let mut registers = Vec::with_capacity(count as usize);
        let data_offset = 3;

        for i in 0..count as usize {
            let high_byte = response[data_offset + (i * 2)] as u16;
            let low_byte = response[data_offset + (i * 2) + 1] as u16;
            registers.push((high_byte << 8) | low_byte);
        }

        Ok(registers)
    }

    pub fn read_holding_register<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
    ) -> Result<u16, ModbusError> {
        let registers = self.read_holding_registers(client, address, 1)?;
        Ok(registers[0])
    }

    pub fn read_input_registers<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        count: u16,
    ) -> Result<Vec<u16>, ModbusError> {
        if count == 0 || count > 125 {
            return Err(ModbusError::ProtocolError(
                "Invalid register count. Must be between 1 and 125".to_string(),
            ));
        }

        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::ReadInputRegisters as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count >> 8) as u8);
        request.push(count as u8);

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 5 + (count * 2) as usize;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::ReadInputRegisters as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::ReadInputRegisters as u8,
                received: response[1],
            });
        }

        let byte_count = response[2] as usize;
        if byte_count != (count * 2) as usize {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected byte count. Expected {}, received {}",
                count * 2,
                byte_count
            )));
        }

        let mut registers = Vec::with_capacity(count as usize);
        let data_offset = 3; // Skip slave id, function code, and byte count

        for i in 0..count as usize {
            let high_byte = response[data_offset + (i * 2)] as u16;
            let low_byte = response[data_offset + (i * 2) + 1] as u16;
            registers.push((high_byte << 8) | low_byte);
        }

        Ok(registers)
    }

    pub fn read_input_register<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
    ) -> Result<u16, ModbusError> {
        let registers = self.read_input_registers(client, address, 1)?;
        Ok(registers[0])
    }

    pub fn read_coils<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        count: u16,
    ) -> Result<Vec<bool>, ModbusError> {
        if count == 0 || count > 2000 {
            return Err(ModbusError::ProtocolError(
                "Invalid coil count. Must be between 1 and 2000".to_string(),
            ));
        }

        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::ReadCoils as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count >> 8) as u8);
        request.push(count as u8);

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let byte_count = (count + 7) / 8;
        let expected_response_len = 5 + byte_count as usize;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::ReadCoils as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::ReadCoils as u8,
                received: response[1],
            });
        }

        if response[2] as u16 != byte_count {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected byte count. Expected {}, received {}",
                byte_count, response[2]
            )));
        }

        let mut coils = Vec::with_capacity(count as usize);
        let data_offset = 3;

        for i in 0..count as usize {
            let byte_index = i / 8;
            let bit_index = i % 8;
            let byte = response[data_offset + byte_index];
            let coil_value = (byte & (1 << bit_index)) != 0;
            coils.push(coil_value);
        }

        Ok(coils)
    }

    pub fn read_coil<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
    ) -> Result<bool, ModbusError> {
        let coils = self.read_coils(client, address, 1)?;
        Ok(coils[0])
    }

    pub fn read_discrete_inputs<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        count: u16,
    ) -> Result<Vec<bool>, ModbusError> {
        if count == 0 || count > 2000 {
            return Err(ModbusError::ProtocolError(
                "Invalid input count. Must be between 1 and 2000".to_string(),
            ));
        }

        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::ReadDiscreteInputs as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count >> 8) as u8);
        request.push(count as u8);

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let byte_count = (count + 7) / 8;
        let expected_response_len = 5 + byte_count as usize;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::ReadDiscreteInputs as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::ReadDiscreteInputs as u8,
                received: response[1],
            });
        }

        if response[2] as u16 != byte_count {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected byte count. Expected {}, received {}",
                byte_count, response[2]
            )));
        }

        let mut inputs = Vec::with_capacity(count as usize);
        let data_offset = 3;

        for i in 0..count as usize {
            let byte_index = i / 8;
            let bit_index = i % 8;
            let byte = response[data_offset + byte_index];
            let input_value = (byte & (1 << bit_index)) != 0;
            inputs.push(input_value);
        }

        Ok(inputs)
    }

    pub fn read_discrete_input<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
    ) -> Result<bool, ModbusError> {
        let inputs = self.read_discrete_inputs(client, address, 1)?;
        Ok(inputs[0])
    }

    pub fn write_single_register<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        value: u16,
    ) -> Result<(), ModbusError> {
        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::WriteSingleRegister as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((value >> 8) as u8);
        request.push(value as u8);

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 8;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::WriteSingleRegister as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::WriteSingleRegister as u8,
                received: response[1],
            });
        }

        let resp_address = ((response[2] as u16) << 8) | (response[3] as u16);
        if resp_address != address {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected register address in response. Expected {}, received {}",
                address, resp_address
            )));
        }

        let resp_value = ((response[4] as u16) << 8) | (response[5] as u16);
        if resp_value != value {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected register value in response. Expected {}, received {}",
                value, resp_value
            )));
        }

        Ok(())
    }

    pub fn write_single_coil<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        value: bool,
    ) -> Result<(), ModbusError> {
        let mut request = Vec::with_capacity(8);
        request.push(self.id);
        request.push(FunctionCode::WriteSingleCoil as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);

        if value {
            request.push(0xFF);
            request.push(0x00);
        } else {
            request.push(0x00);
            request.push(0x00);
        }

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 8;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::WriteSingleCoil as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::WriteSingleCoil as u8,
                received: response[1],
            });
        }

        let resp_address = ((response[2] as u16) << 8) | (response[3] as u16);
        if resp_address != address {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected coil address in response. Expected {}, received {}",
                address, resp_address
            )));
        }

        let resp_value = response[4] == 0xFF && response[5] == 0x00;
        if resp_value != value {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected coil value in response. Expected {}, received {}",
                value, resp_value
            )));
        }

        Ok(())
    }

    pub fn write_multiple_registers<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        values: &[u16],
    ) -> Result<(), ModbusError> {
        let count = values.len();
        if count == 0 || count > 123 {
            return Err(ModbusError::ProtocolError(
                "Invalid register count. Must be between 1 and 123".to_string(),
            ));
        }

        let mut request = Vec::with_capacity(9 + count * 2);
        request.push(self.id);
        request.push(FunctionCode::WriteMultipleRegisters as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count as u16 >> 8) as u8);
        request.push(count as u8);
        request.push((count * 2) as u8);

        for &value in values {
            request.push((value >> 8) as u8);
            request.push(value as u8);
        }

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 8;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::WriteMultipleRegisters as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::WriteMultipleRegisters as u8,
                received: response[1],
            });
        }

        let resp_address = ((response[2] as u16) << 8) | (response[3] as u16);
        if resp_address != address {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected starting address in response. Expected {}, received {}",
                address, resp_address
            )));
        }

        let resp_count = ((response[4] as u16) << 8) | (response[5] as u16);
        if resp_count as usize != count {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected register count in response. Expected {}, received {}",
                count, resp_count
            )));
        }

        Ok(())
    }

    pub fn write_multiple_coils<T: Read + Write>(
        &self,
        client: &mut T,
        address: u16,
        values: &[bool],
    ) -> Result<(), ModbusError> {
        let count = values.len();
        if count == 0 || count > 1968 {
            return Err(ModbusError::ProtocolError(
                "Invalid coil count. Must be between 1 and 1968".to_string(),
            ));
        }

        let byte_count = (count + 7) / 8;

        let mut request = Vec::with_capacity(9 + byte_count);
        request.push(self.id);
        request.push(FunctionCode::WriteMultipleCoils as u8);
        request.push((address >> 8) as u8);
        request.push(address as u8);
        request.push((count as u16 >> 8) as u8);
        request.push(count as u8);
        request.push(byte_count as u8);

        for byte_index in 0..byte_count {
            let mut byte: u8 = 0;
            for bit_index in 0..8 {
                let coil_index = byte_index * 8 + bit_index;
                if coil_index < count && values[coil_index] {
                    byte |= 1 << bit_index;
                }
            }
            request.push(byte);
        }

        let crc = calculate_crc16(&request);
        request.push((crc & 0xFF) as u8);
        request.push((crc >> 8) as u8);

        let expected_response_len = 8;

        let response = self.send_receive(client, &request, expected_response_len)?;

        if response[1] != FunctionCode::WriteMultipleCoils as u8 {
            return Err(ModbusError::InvalidFunctionCode {
                expected: FunctionCode::WriteMultipleCoils as u8,
                received: response[1],
            });
        }

        let resp_address = ((response[2] as u16) << 8) | (response[3] as u16);
        if resp_address != address {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected starting address in response. Expected {}, received {}",
                address, resp_address
            )));
        }

        let resp_count = ((response[4] as u16) << 8) | (response[5] as u16);
        if resp_count as usize != count {
            return Err(ModbusError::ProtocolError(format!(
                "Unexpected coil count in response. Expected {}, received {}",
                count, resp_count
            )));
        }

        Ok(())
    }
}
