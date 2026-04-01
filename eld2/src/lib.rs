use std::{
    io::{Read, Write},
    ops::{Add, AddAssign, Shl},
};
use utilities::modbus::{Modbus, ModbusError};

const MOTION_CONTROL_REG: u16 = 0x6002;
const SI_STATUS_REG: u16 = 0x0179;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitSwitch {
    None,
    Low,
    High,
    Both,
}

impl Add for LimitSwitch {
    type Output = LimitSwitch;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (LimitSwitch::None, LimitSwitch::None) => LimitSwitch::None,
            (LimitSwitch::Low, LimitSwitch::Low) => LimitSwitch::Low,
            (LimitSwitch::High, LimitSwitch::High) => LimitSwitch::High,

            (LimitSwitch::Low, LimitSwitch::High) => LimitSwitch::Both,
            (LimitSwitch::High, LimitSwitch::Low) => LimitSwitch::Both,

            (lhs, LimitSwitch::None) => lhs,
            (LimitSwitch::None, rhs) => rhs,
            (_, LimitSwitch::Both) => LimitSwitch::Both,
            (LimitSwitch::Both, _) => LimitSwitch::Both,
        }
    }
}

impl AddAssign for LimitSwitch {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MotionStatus(u16);

#[derive(Debug, Clone)]
pub struct StateParams {
    motion_status: MotionStatus,
    switches: LimitSwitch,
}

impl StateParams {
    pub fn motion_status(&self) -> MotionStatus {
        self.motion_status
    }

    pub fn limit_switches(&self) -> LimitSwitch {
        self.switches
    }

    pub fn is_moving(&self) -> bool {
        self.motion_status.0 != 0
    }

    pub fn high_limit_triggered(&self) -> bool {
        self.switches == LimitSwitch::High || self.switches == LimitSwitch::Both
    }

    pub fn low_limit_triggered(&self) -> bool {
        self.switches == LimitSwitch::Low || self.switches == LimitSwitch::Both
    }
}

#[derive(Clone)]
pub struct Em2rs {
    client: Modbus,
    low_limit: u8,
    high_limit: u8,
}

impl Em2rs {
    pub fn new(id: u8, low_limit: u8, high_limit: u8) -> Self {
        let modbus = Modbus::new(id);
        Self {
            client: modbus,
            low_limit,
            high_limit,
        }
    }

    pub fn set_velocity(
        &self,
        client: &mut (impl Write + Read),
        velocity: u16,
    ) -> Result<(), ModbusError> {
        self.client.write_single_register(client, 0x6203, velocity)
    }

    pub fn get_velocity(&self, client: &mut (impl Write + Read)) -> Result<u16, ModbusError> {
        self.client.read_holding_register(client, 0x6203)
    }

    pub fn set_acceleration(
        &self,
        client: &mut (impl Write + Read),
        acceleration: u16,
    ) -> Result<(), ModbusError> {
        self.client
            .write_single_register(client, 0x6204, acceleration)
    }

    pub fn get_acceleration(&self, client: &mut (impl Write + Read)) -> Result<u16, ModbusError> {
        self.client.read_holding_register(client, 0x6204)
    }

    pub fn set_deceleration(
        &self,
        client: &mut (impl Write + Read),
        deceleration: u16,
    ) -> Result<(), ModbusError> {
        self.client
            .write_single_register(client, 0x6205, deceleration)
    }

    pub fn get_deceleration(&self, client: &mut (impl Write + Read)) -> Result<u16, ModbusError> {
        self.client.read_holding_register(client, 0x6205)
    }

    pub fn move_relative(
        &self,
        client: &mut (impl Write + Read),
        steps: i32,
    ) -> Result<(), ModbusError> {
        let data = steps.to_be_bytes();

        let high = u16::from_be_bytes([data[0], data[1]]);
        let low = u16::from_be_bytes([data[2], data[3]]);

        self.client
            .write_single_register(client, 0x6200, 0b1000001)?; // Set move type to relative position
        self.client.write_single_register(client, 0x6201, high)?;
        self.client.write_single_register(client, 0x6202, low)?;
        self.client
            .write_single_register(client, MOTION_CONTROL_REG, 0x10)
    }

    pub fn get_speed(&self, client: &mut (impl Write + Read)) -> Result<u16, ModbusError> {
        let speed = self.client.read_holding_register(client, 0x0B09)?;
        Ok(speed)
    }

    pub fn stop(&self, client: &mut (impl Write + Read)) -> Result<(), ModbusError> {
        self.client
            .write_single_register(client, MOTION_CONTROL_REG, 0x40)
    }

    pub fn get_si_status(
        &self,
        index: u8,
        client: &mut (impl Write + Read),
    ) -> Result<bool, ModbusError> {
        if !(0..8).contains(&index) {
            return Err(ModbusError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Index must be between 0 and 7",
            )));
        }

        let ret = self
            .client
            .read_holding_register(client, SI_STATUS_REG)?
            .to_be_bytes()[1];

        let status = (ret & 1u8.shl(index)) > 0;

        Ok(status)
    }

    pub fn get_limit_switch_state(
        &self,
        client: &mut (impl Write + Read),
    ) -> Result<LimitSwitch, ModbusError> {
        let mut switch = LimitSwitch::None;

        self.get_si_status(self.low_limit, client)?
            .then(|| switch += LimitSwitch::Low);
        self.get_si_status(self.high_limit, client)?
            .then(|| switch += LimitSwitch::High);

        Ok(switch)
    }

    pub fn get_state(&self, client: &mut (impl Write + Read)) -> Result<StateParams, ModbusError> {
        let speed = self.get_speed(client)?;
        let switches = self.get_limit_switch_state(client)?;

        Ok(StateParams {
            motion_status: MotionStatus(speed),
            switches,
        })
    }
}
