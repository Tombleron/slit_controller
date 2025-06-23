use command::{
    r#move::{MOVEParameters, MOVR, STOP},
    state::StateParams,
    StandaCommand, StandaGetSetCommand,
};
use std::io::{Read, Result, Write};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::command;

#[derive(Default, Clone)]
pub struct Standa;

impl Standa {
    pub fn new() -> Self {
        Standa {}
    }

    pub async fn get_velocity(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> Result<u32> {
        let params = MOVEParameters::get_async(sender).await?;
        Ok(params.speed)
    }

    pub async fn set_velocity(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
        velocity: u32,
    ) -> Result<()> {
        let mut move_params = MOVEParameters::get_async(sender).await?;
        move_params.speed = velocity;
        move_params.set_async(sender).await
    }

    pub async fn get_acceleration(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> Result<u16> {
        let params = MOVEParameters::get_async(sender).await?;
        Ok(params.accel)
    }

    pub async fn set_acceleration(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
        acceleration: u16,
    ) -> Result<()> {
        let mut move_params = MOVEParameters::get_async(sender).await?;
        move_params.accel = acceleration;
        move_params.set_async(sender).await
    }

    pub async fn get_deceleration(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> Result<u16> {
        let params = MOVEParameters::get_async(sender).await?;
        Ok(params.decel)
    }

    pub async fn set_deceleration(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
        deceleration: u16,
    ) -> Result<()> {
        let mut move_params = MOVEParameters::get_async(sender).await?;
        move_params.decel = deceleration;
        move_params.set_async(sender).await
    }

    pub async fn get_state(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
    ) -> Result<StateParams> {
        StateParams::get_async(sender).await
    }

    pub async fn move_relative(
        &self,
        sender: &mut (impl AsyncWrite + AsyncRead + Unpin),
        steps: i32,
        sub_steps: i16,
    ) -> Result<()> {
        MOVR {
            position: steps,
            u_position: sub_steps,
        }
        .send_async(sender)
        .await
    }

    pub async fn stop(&self, sender: &mut (impl AsyncWrite + AsyncRead + Unpin)) -> Result<()> {
        STOP.send_async(sender).await
    }

    // There still has to be a way to stop motor in non async way
    pub fn stop_non_async(&self, sender: &mut (impl Write + Read)) -> Result<()> {
        STOP.send(sender)
    }
}
