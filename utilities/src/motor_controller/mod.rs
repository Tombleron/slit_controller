#![allow(async_fn_in_trait)]

use std::time::{Duration, Instant};

pub trait MotorState {
    fn start_switch(&self) -> bool;
    fn end_switch(&self) -> bool;
    fn is_moving(&self) -> bool;
}

pub trait MotorController {
    type MovementParameters;
    type MotorState;

    async fn stop(&mut self) -> Result<(), String>;
    async fn update_parameters(
        &mut self,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String>;
    async fn get_state(&self) -> Result<Self::MotorState, String>;
    async fn get_position(&self) -> Result<f32, String>;

    fn is_moving(&self) -> bool;
    fn set_moving(&mut self, is_moving: bool);
    fn init_motion(
        &mut self,
        target: f32,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String>;

    async fn move_to(
        &mut self,
        target: f32,
        parameters: Self::MovementParameters,
    ) -> Result<(), String> {
        if self.is_moving() {
            return Err("Motor is already in motion".to_string());
        }

        self.update_parameters(&parameters).await?;

        self.set_moving(true);

        self.init_motion(target, &parameters)?;

        Ok(())
    }
}

pub trait Motor {
    async fn position(&self) -> Result<f32, String>;
    async fn state(&self) -> Result<impl MotorState, String>;
    async fn move_relative(&mut self, error: f32) -> Result<(), String>;

    fn get_position_window(&self) -> f32;
    fn get_time_limit(&self) -> Duration;
    fn get_start_time(&self) -> Instant;
    fn get_target_position(&self) -> f32;

    fn add_error(&mut self, error: f32);
    fn get_rms(&self) -> f32;

    fn is_moving(&self) -> bool;
    fn set_moving(&mut self, is_moving: bool);

    fn is_time_limit_exceeded(&self) -> bool {
        self.get_start_time().elapsed() > self.get_time_limit()
    }

    async fn run(&mut self) -> Result<(), String> {
        while self.is_moving() && !self.is_time_limit_exceeded() {
            let current_position = self.position().await?;
            let target_position = self.get_target_position();

            let error = current_position - target_position;

            self.add_error(error);

            if self.get_rms() <= self.get_position_window() {
                break;
            }

            self.move_relative(error).await?;

            let state = self.state().await?;

            if state.start_switch() && error < 0.0 {
                break;
            }
            if state.end_switch() && error > 0.0 {
                break;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }
}
