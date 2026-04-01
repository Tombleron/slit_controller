use serde::{Deserialize, Serialize};

use super::{StandaCommand, StandaGetSetCommand};

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MOVEParameters {
    pub speed: u32,
    pub u_speed: u8,
    pub accel: u16,
    pub decel: u16,
    pub antiplay_speed: u32,
    pub u_antiplay_speed: u8,
    pub move_flags: u8,
}

impl<'a> StandaCommand<'a, 9> for MOVEParameters {}
impl<'a> StandaGetSetCommand<'a, 9> for MOVEParameters {
    const GET_CMD_NAME: &'static str = "gmov";
    const SET_CMD_NAME: &'static str = "smov";
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug)]
pub struct MOVR {
    pub position: i32,
    pub u_position: i16,
}
impl<'a> StandaCommand<'a, 6, true> for MOVR {
    const CMD_NAME: &'static str = "movr";
}

#[repr(C, packed)]
#[derive(Serialize, Deserialize, Debug)]
pub struct STOP;
impl<'a> StandaCommand<'a, 0, false> for STOP {
    const CMD_NAME: &'static str = "stop";
}
