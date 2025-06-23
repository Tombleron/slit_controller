#![allow(async_fn_in_trait)]

pub mod home;
pub mod r#move;
pub mod state;

use std::{
    io::{self, Error, ErrorKind, Read, Write},
    mem::size_of,
};

use bincode::deserialize;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

fn crc16(pbuf: &[u8]) -> u16 {
    let mut crc: u16 = 0xffff;
    for &byte in pbuf {
        crc ^= byte as u16;
        for _ in 0..8 {
            let a = crc;
            let carry_flag = a & 0x0001;
            crc >>= 1;
            if carry_flag == 1 {
                crc ^= 0xa001;
            }
        }
    }
    crc
}

#[repr(C, packed)]
#[derive(Deserialize, Debug)]
struct Response<T> {
    cmd: u32,
    #[serde(flatten)]
    data: T,
    crc: u16,
}

pub trait StandaCommand<'a, const RESERVED: usize = 0, const CRC: bool = true>:
    Serialize + Sized
{
    const SIZE: usize = size_of::<Self>() + RESERVED;
    const CMD_NAME: &'static str = "";

    fn as_bytes(&self, cmd_name: &'a str) -> Vec<u8> {
        let command = cmd_name.as_bytes();

        let bytes = bincode::serialize(self).expect("failed to serialize struct.");

        let mut buffer = Vec::with_capacity(4 + Self::SIZE + if CRC { 2 } else { 0 });

        buffer.extend_from_slice(command);
        buffer.extend_from_slice(&bytes);
        buffer.extend_from_slice(&[0; RESERVED]);

        if CRC {
            let crc = crc16(&buffer[4..]);
            buffer.extend_from_slice(&[crc as u8, (crc >> 8) as u8]);
        }

        buffer
    }

    fn send(&self, sender: &mut (impl Write + Read)) -> io::Result<()> {
        let bytes = self.as_bytes(Self::CMD_NAME);

        Self::send_raw(sender, &bytes, 0).map(|_| ())
    }

    async fn send_async(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> io::Result<()> {
        let bytes = self.as_bytes(Self::CMD_NAME);

        Self::send_raw_async(sender, &bytes, 0).await.map(|_| ())
    }

    fn send_raw(
        sender: &mut (impl Write + Read),
        bytes: &[u8],
        payload_size: usize,
    ) -> io::Result<Vec<u8>> {
        sender.write_all(bytes)?;

        // Eat all zeros
        let mut cmd_name_buffer = vec![0; 4];

        while cmd_name_buffer[0] == 0 {
            match sender.read_exact(&mut cmd_name_buffer[..1]) {
                Ok(_) => {}
                // Sync on timeout
                Err(e) if e.kind() == ErrorKind::TimedOut => {
                    return Err(Self::synchronization(sender).unwrap_err());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Read rest
        match sender.read_exact(&mut cmd_name_buffer[1..]) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                return Err(Self::synchronization(sender).unwrap_err())
            }
            Err(e) => return Err(e),
        }

        // Check command name
        if cmd_name_buffer != bytes[0..4] {
            return Err(Self::synchronization(sender).unwrap_err());
        }

        if payload_size == 0 {
            return Ok(Vec::new());
        }

        // Read payload + CRC
        let mut payload = vec![0; payload_size + 2];
        match sender.read_exact(&mut payload) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                return Err(Self::synchronization(sender).unwrap_err())
            }
            Err(e) => return Err(e),
        }

        // Check CRC
        let (payload, crc) = payload.split_at(payload_size);
        let calculated_crc = crc16(payload);
        let received_crc = u16::from_le_bytes([crc[0], crc[1]]);

        if calculated_crc != received_crc {
            return Err(Error::new(ErrorKind::InvalidData, "CRC mismatch"));
        }

        Ok(payload.to_vec())
    }

    async fn send_raw_async(
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
        bytes: &[u8],
        payload_size: usize,
    ) -> io::Result<Vec<u8>> {
        sender.write_all(bytes).await?;

        // Eat all zeros
        let mut cmd_name_buffer = vec![0; 4];

        while cmd_name_buffer[0] == 0 {
            match sender.read_exact(&mut cmd_name_buffer[..1]).await {
                Ok(_) => {}
                // Sync on timeout
                Err(e) if e.kind() == ErrorKind::TimedOut => {
                    return Err(Self::synchronization_async(sender).await.unwrap_err());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Read rest
        match sender.read_exact(&mut cmd_name_buffer[1..]).await {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                return Err(Self::synchronization_async(sender).await.unwrap_err())
            }
            Err(e) => return Err(e),
        }

        // Check command name
        if cmd_name_buffer != bytes[0..4] {
            return Err(Self::synchronization_async(sender).await.unwrap_err());
        }

        if payload_size == 0 {
            return Ok(Vec::new());
        }

        // Read payload + CRC
        let mut payload = vec![0; payload_size + 2];
        match sender.read_exact(&mut payload).await {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                return Err(Self::synchronization_async(sender).await.unwrap_err())
            }
            Err(e) => return Err(e),
        }

        // Check CRC
        let (payload, crc) = payload.split_at(payload_size);
        let calculated_crc = crc16(payload);
        let received_crc = u16::from_le_bytes([crc[0], crc[1]]);

        if calculated_crc != received_crc {
            return Err(Error::new(ErrorKind::InvalidData, "CRC mismatch"));
        }

        Ok(payload.to_vec())
    }

    fn synchronization(sender: &mut (impl Write + Read)) -> io::Result<()> {
        'outer: for _ in 0..3 {
            sender.flush()?;

            sender.write_all(&[0; 64])?;

            for _ in 0..64 {
                let mut buf = [0; 1];
                match sender.read_exact(&mut buf) {
                    Ok(_) => {
                        if buf[0] == 0 {
                            return Err(Error::other("Synchronized with device"));
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::TimedOut => continue 'outer,
                    Err(e) => return Err(e),
                }
            }
        }

        Err(Error::new(
            ErrorKind::HostUnreachable,
            "Device is unreachable or not responding",
        ))
    }

    async fn synchronization_async(
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> io::Result<()> {
        for _ in 0..3 {
            sender.flush().await?;

            sender.write_all(&[0; 64]).await?;

            for _ in 0..64 {
                let mut buf = [0; 1];
                match sender.read_exact(&mut buf).await {
                    Ok(_) => {
                        if buf[0] == 0 {
                            return Err(Error::other("Synchronized with device"));
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::TimedOut => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        Err(Error::new(
            ErrorKind::HostUnreachable,
            "Device is unreachable or not responding",
        ))
    }
}

pub trait StandaGetSetCommand<'a, const RESERVED: usize = 0, const CRC: bool = true>:
    StandaCommand<'a, RESERVED, CRC>
where
    Self: Sized,
{
    const GET_CMD_NAME: &'static str;
    const SET_CMD_NAME: &'static str;

    fn get(sender: &mut (impl Write + Read)) -> io::Result<Self>
    where
        Self: for<'de> Deserialize<'de>,
    {
        let name = Self::GET_CMD_NAME.as_bytes();

        let payload = Self::send_raw(sender, name, Self::SIZE)?;

        let (data, _) = payload.split_at(size_of::<Self>());

        let response = deserialize::<Self>(data).map_err(|_e| {
            Error::new(
                ErrorKind::InvalidData,
                "failed to parse response from serial port.",
            )
        })?;

        Ok(response)
    }

    async fn get_async(sender: &mut (impl AsyncWrite + AsyncRead + Unpin)) -> io::Result<Self>
    where
        Self: for<'de> Deserialize<'de>,
    {
        let name = Self::GET_CMD_NAME.as_bytes();

        let payload = Self::send_raw_async(sender, name, Self::SIZE).await?;

        let (data, _) = payload.split_at(size_of::<Self>());

        let response = deserialize::<Self>(data).map_err(|_e| {
            Error::new(
                ErrorKind::InvalidData,
                "failed to parse response from serial port.",
            )
        })?;

        Ok(response)
    }

    fn set(&self, sender: &mut (impl Write + Read)) -> io::Result<()> {
        let bytes = self.as_bytes(Self::SET_CMD_NAME);

        Self::send_raw(sender, &bytes, 0)?;

        Ok(())
    }

    async fn set_async(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> io::Result<()> {
        let bytes = self.as_bytes(Self::SET_CMD_NAME);

        Self::send_raw_async(sender, &bytes, 0).await?;

        Ok(())
    }
}
