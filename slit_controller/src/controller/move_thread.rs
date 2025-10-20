use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use standa::command::state::StateParams;
use utilities::motor_controller::{Motor, MotorState};

use crate::command_executor::{
    encoder::command_sender::EncoderCommandSender, motor::command_sender::StandaCommandSender,
};

const STEPS_PER_MM: u32 = 800;

pub struct MoveThread {
    rf256_cs: EncoderCommandSender,
    rf256_axis: u8,
    standa_cs: StandaCommandSender,

    target_position: f32,
    position_window: f32,
    time_limit: Duration,

    filter: MovingAverage,

    moving: Arc<AtomicBool>,
    start_time: Instant,
}

impl MoveThread {
    pub fn new(
        rf256_cs: EncoderCommandSender,
        standa_cs: StandaCommandSender,
        rf256_axis: u8,
        target_position: f32,
        position_window: f32,
        time_limit: Duration,
        moving: Arc<AtomicBool>,
    ) -> Self {
        Self {
            rf256_cs,
            standa_cs,
            rf256_axis,

            filter: MovingAverage::new(20),

            target_position,
            position_window,
            time_limit,

            moving,
            start_time: Instant::now(),
        }
    }

    async fn send_steps(&self, steps: i32, sub_steps: i16) -> io::Result<()> {
        self.standa_cs.send_steps(steps, sub_steps).await
    }
}

impl Drop for MoveThread {
    fn drop(&mut self) {
        self.moving.store(false, Ordering::SeqCst);
    }
}

impl Motor for MoveThread {
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
            ((error * STEPS_PER_MM as f32) as i32, 0)
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
        self.moving.load(Ordering::Relaxed)
    }

    fn set_moving(&mut self, is_moving: bool) {
        self.moving.store(is_moving, Ordering::Relaxed);
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

struct MovingAverage {
    values: Vec<f32>,
    max_size: usize,
}

impl MovingAverage {
    pub fn new(max_size: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add(&mut self, value: f32) {
        if self.values.len() >= self.max_size {
            self.values.remove(0);
        }
        self.values.push(value);
    }

    pub fn get_rms(&self) -> f32 {
        if self.values.is_empty() {
            0.0
        } else {
            let sum_of_squares: f32 = self.values.iter().map(|&v| v * v).sum();
            (sum_of_squares / self.values.len() as f32).sqrt()
        }
    }
}
