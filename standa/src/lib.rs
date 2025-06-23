use command::{
    r#move::{MOVEParameters, MOVR, STOP},
    state::StateParams,
    StandaCommand, StandaGetSetCommand,
};
use std::io::{Read, Result, Write};
pub mod async_client;
pub mod command;

#[derive(Default)]
pub struct Standa;

impl Standa {
    pub fn new() -> Self {
        Standa {}
    }

    pub fn get_velocity(&self, sender: &mut (impl Write + Read)) -> Result<u32> {
        Ok(MOVEParameters::get(sender)?.speed)
    }

    pub fn set_velocity(&self, sender: &mut (impl Write + Read), velocity: u32) -> Result<()> {
        let mut move_params = MOVEParameters::get(sender)?;
        move_params.speed = velocity;
        move_params.set(sender)
    }

    pub fn get_acceleration(&self, sender: &mut (impl Write + Read)) -> Result<u16> {
        Ok(MOVEParameters::get(sender)?.accel)
    }

    pub fn set_acceleration(
        &self,
        sender: &mut (impl Write + Read),
        acceleration: u16,
    ) -> Result<()> {
        let mut move_params = MOVEParameters::get(sender)?;
        move_params.accel = acceleration;
        move_params.set(sender)
    }

    pub fn get_deceleration(&self, sender: &mut (impl Write + Read)) -> Result<u16> {
        Ok(MOVEParameters::get(sender)?.decel)
    }

    pub fn set_deceleration(
        &self,
        sender: &mut (impl Write + Read),
        deceleration: u16,
    ) -> Result<()> {
        let mut move_params = MOVEParameters::get(sender)?;
        move_params.decel = deceleration;
        move_params.set(sender)
    }

    pub fn get_state(&self, sender: &mut (impl Write + Read)) -> Result<StateParams> {
        StateParams::get(sender)
    }

    pub fn move_relative(
        &self,
        sender: &mut (impl Write + Read),
        steps: i32,
        sub_steps: i16,
    ) -> Result<()> {
        MOVR {
            position: steps,
            u_position: sub_steps,
        }
        .send(sender)
    }

    pub fn stop(&self, sender: &mut (impl Write + Read)) -> Result<()> {
        STOP.send(sender)
    }
}
