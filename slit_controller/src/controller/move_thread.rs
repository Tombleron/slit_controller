use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use standa::command::state::StateParams;
use tracing::{error, info, warn};

use crate::command_executor::{
    encoder::command_sender::Rf256CommandSender, motor::command_sender::StandaCommandSender,
};

const STEPS_PER_MM: u32 = 800;

pub struct MoveThread {
    rf256_cs: Rf256CommandSender,
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
        rf256_cs: Rf256CommandSender,
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

    async fn position_with_retries(&self, retries: u8) -> io::Result<f32> {
        self.rf256_cs
            .read_position_with_retries(self.rf256_axis, retries)
            .await
    }

    fn is_moving(&self) -> bool {
        let moving = self.moving.load(Ordering::SeqCst);
        moving
    }

    fn time_limit_exceeded(&self) -> bool {
        let elapsed = self.start_time.elapsed();
        let exceeded = elapsed > self.time_limit;
        if exceeded {
            warn!(
                elapsed_ms = elapsed.as_millis(),
                limit_ms = self.time_limit.as_millis(),
                "Time limit exceeded"
            );
        }
        exceeded
    }

    async fn get_state(&self) -> io::Result<StateParams> {
        let result = self.standa_cs.get_state().await;
        if let Err(ref e) = result {
            error!(error = %e, "Failed to get Standa state");
        }
        result
    }

    async fn stop(&self) -> io::Result<()> {
        let result = self.standa_cs.stop().await;
        if let Err(ref e) = result {
            error!(error = %e, "Failed to stop Standa movement");
        }
        result
    }

    async fn send_steps(&self, steps: i32, sub_steps: i16) -> io::Result<()> {
        info!(steps, sub_steps, "Sending move command to Standa");
        let result = self.standa_cs.send_steps(steps, sub_steps).await;
        if let Err(ref e) = result {
            error!(error = %e, "Failed to send steps to Standa");
        }
        result
    }

    async fn move_relative(&self, error: f32) -> io::Result<()> {
        let (steps, sub_steps) = if error.abs() == 0.0 {
            (0, 0)
        } else if error.abs() < 0.001 {
            (0, if error > 0.0 { 5 } else { -5 })
            // (if error > 0.0 { 1 } else { -1 }, 0)
        } else {
            ((error * STEPS_PER_MM as f32) as i32, 0)
        };

        let result = self.send_steps(steps, sub_steps).await;
        if let Err(ref e) = result {
            error!(error = %e, "Failed to move relative");
            return result;
        }

        // Wait until motion completes or stop is requested
        let mut wait_count = 0;
        while self.is_moving() && self.get_state().await?.is_moving() && !self.time_limit_exceeded()
        {
            wait_count += 1;
            if wait_count % 50 == 0 {
                // Log every ~500ms
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    pub async fn run(&mut self) {
        info!(
            target_position = self.target_position,
            position_window = self.position_window,
            "Starting move thread"
        );

        while self.is_moving() && !self.time_limit_exceeded() {
            let current_position = match self.position_with_retries(5).await {
                Ok(pos) => pos,
                Err(e) => {
                    error!(error = %e, "Error reading position, aborting move thread");
                    return;
                }
            };

            let error = current_position - self.target_position;
            self.filter.add(error);
            info!(
                current_position,
                target_position = self.target_position,
                error,
                position_window = self.position_window,
                "Position status"
            );

            // Check if we are within the position window
            if self.filter.get_rms() <= self.position_window {
                break;
            }

            if let Err(e) = self.move_relative(error).await {
                error!(error = %e, "Error moving Standa, aborting move thread");
                return;
            }

            let state = match self.get_state().await {
                Ok(state) => state,
                Err(e) => {
                    error!(error = %e, "Error getting state, aborting move thread");
                    return;
                }
            };

            if state.right_switch() && error > 0.0 {
                tracing::warn!(
                    error,
                    right_switch = state.right_switch(),
                    "Reached right switch, stopping movement"
                );
                break;
            } else if state.left_switch() && error < 0.0 {
                tracing::warn!(
                    error,
                    left_switch = state.left_switch(),
                    "Reached left switch, stopping movement"
                );
                break;
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for MoveThread {
    fn drop(&mut self) {
        info!("Stopping move thread");
        self.moving.store(false, Ordering::SeqCst);
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
