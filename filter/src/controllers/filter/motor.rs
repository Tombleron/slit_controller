use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use em2rs::StateParams;
use utilities::{
    motor_controller::{Motor, MotorState},
    moving_average::MovingAverage,
};

use crate::command_executor::{
    encoder::command_sender::EncoderCommandSender, motor::command_sender::Em2rsCommandSender,
};

pub struct FilterMotor {
    encoder_cs: EncoderCommandSender,
    em2rs_cs: Em2rsCommandSender,

    target_position: f32,
    position_window: f32,
    time_limit: Duration,

    filter: MovingAverage,

    is_moving: Arc<AtomicBool>,
    start_time: Instant,
    steps_per_mm: u32,
}

impl Drop for FilterMotor {
    fn drop(&mut self) {
        self.is_moving.store(false, Ordering::SeqCst);
    }
}

impl FilterMotor {
    pub fn new(
        encoder_cs: EncoderCommandSender,
        em2rs_cs: Em2rsCommandSender,
        target_position: f32,
        position_window: f32,
        time_limit: Duration,
        moving: Arc<AtomicBool>,
        steps_per_mm: u32,
    ) -> Self {
        Self {
            encoder_cs,
            em2rs_cs,

            filter: MovingAverage::new(20),

            target_position,
            position_window,
            time_limit,

            is_moving: moving,
            start_time: Instant::now(),
            steps_per_mm,
        }
    }

    async fn send_steps(&self, steps: i32) -> std::io::Result<()> {
        self.em2rs_cs.send_steps(steps).await
    }
}

impl Motor for FilterMotor {
    async fn position(&self) -> Result<f32, String> {
        self.encoder_cs
            .get_position()
            .await
            .map_err(|err| err.to_string())
    }

    async fn state(&self) -> Result<impl utilities::motor_controller::MotorState, String> {
        self.em2rs_cs
            .get_state()
            .await
            .map(|state| Em2rsState(state))
            .map_err(|err| err.to_string())
    }

    async fn move_relative(&mut self, error: f32) -> Result<(), String> {
        let steps = if error.abs() == 0.0 {
            0
        } else if error.abs() < 0.001 {
            if error > 0.0 { 100 } else { -100 }
        } else {
            (error * self.steps_per_mm as f32) as i32
        };

        let _result = self
            .send_steps(steps)
            .await
            .map_err(|e| format!("Failed to move relative: {}", e))?;

        while self.is_moving() && self.state().await?.is_moving() && !self.is_time_limit_exceeded()
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }

    fn get_position_window(&self) -> f32 {
        self.position_window
    }

    fn get_time_limit(&self) -> Duration {
        self.time_limit
    }

    fn get_start_time(&self) -> Instant {
        self.start_time
    }

    fn get_target_position(&self) -> f32 {
        self.target_position
    }

    fn add_error(&mut self, error: f32) {
        self.filter.add(error);
    }

    fn get_rms(&self) -> f32 {
        self.filter.get_rms()
    }

    fn is_moving(&self) -> bool {
        self.is_moving.load(Ordering::Relaxed)
    }

    fn set_moving(&mut self, is_moving: bool) {
        self.is_moving.store(is_moving, Ordering::Relaxed);
    }
}

struct Em2rsState(StateParams);

impl MotorState for Em2rsState {
    fn start_switch(&self) -> bool {
        self.0.low_limit_triggered()
    }

    fn end_switch(&self) -> bool {
        self.0.high_limit_triggered()
    }

    fn is_moving(&self) -> bool {
        self.0.is_moving()
    }
}
