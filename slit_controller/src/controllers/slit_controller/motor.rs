use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use standa::command::state::StateParams;
use utilities::{
    motor_controller::{Motor, MotorState},
    moving_average::MovingAverage,
};

use crate::command_executor::{
    encoder::command_sender::EncoderCommandSender, motor::command_sender::StandaCommandSender,
};

pub struct SlitMotor {
    rf256_cs: EncoderCommandSender,
    rf256_axis: u8,
    standa_cs: StandaCommandSender,

    target_position: f32,
    position_window: f32,
    time_limit: Duration,

    filter: MovingAverage,

    is_moving: Arc<AtomicBool>,
    start_time: Instant,
    steps_per_mm: i32,
}

impl Drop for SlitMotor {
    fn drop(&mut self) {
        self.is_moving.store(false, Ordering::Relaxed);
    }
}

impl SlitMotor {
    pub fn new(
        rf256_cs: EncoderCommandSender,
        rf256_axis: u8,
        standa_cs: StandaCommandSender,
        target_position: f32,
        position_window: f32,
        time_limit: Duration,
        is_moving: Arc<AtomicBool>,
        steps_per_mm: i32,
    ) -> Self {
        let start_time = Instant::now();
        let filter = MovingAverage::new(10);

        SlitMotor {
            rf256_cs,
            rf256_axis,
            standa_cs,

            target_position,
            position_window,
            time_limit,

            filter,

            is_moving,
            start_time,
            steps_per_mm,
        }
    }

    async fn send_steps(&self, steps: i32, sub_steps: i16) -> io::Result<()> {
        self.standa_cs.send_steps(steps, sub_steps).await
    }
}

impl Motor for SlitMotor {
    async fn position(&self) -> Result<f32, String> {
        self.rf256_cs
            .get_position(self.rf256_axis)
            .await
            .map_err(|e| format!("Failed to read position: {}", e))
    }

    async fn state(&self) -> Result<impl MotorState, String> {
        self.standa_cs
            .get_state()
            .await
            .map(StandaState)
            .map_err(|e| format!("Failed to get Standa state: {}", e))
    }

    async fn move_relative(&mut self, error: f32) -> Result<(), String> {
        let (steps, sub_steps) = if error.abs() == 0.0 {
            (0, 0)
        } else if error.abs() < 0.001 {
            (0, if error > 0.0 { 5 } else { -5 })
        } else {
            ((error * self.steps_per_mm as f32) as i32, 0)
        };

        let _result = self
            .send_steps(steps, sub_steps)
            .await
            .map_err(|e| format!("Failed to send steps: {}", e))?;

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

struct StandaState(StateParams);

impl MotorState for StandaState {
    fn start_switch(&self) -> bool {
        self.0.left_switch()
    }

    fn end_switch(&self) -> bool {
        self.0.right_switch()
    }

    fn is_moving(&self) -> bool {
        self.0.is_moving()
    }
}
