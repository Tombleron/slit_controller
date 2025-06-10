use std::{
    io::{self, Read, Write},
    ops::DerefMut as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use rf256::Rf256;
use standa::{command::state::StateParams, Standa};
use tracing::{info, instrument};

const STEPS_PER_MM: u32 = 800;

pub struct MoveThread<T: Write + Read + Send> {
    rf256_client: Arc<Mutex<T>>,
    standa_client: Arc<Mutex<T>>,

    rf256: Rf256,
    standa: Standa,

    target_position: f32,
    position_window: f32,

    moving: Arc<AtomicBool>,
}

impl<T: Write + Read + Send> MoveThread<T> {
    #[instrument(skip(rf256_client, standa_client, moving), fields(rf256_id))]
    pub fn new(
        rf256_client: Arc<Mutex<T>>,
        standa_client: Arc<Mutex<T>>,
        rf256_id: u8,
        target_position: f32,
        position_window: f32,
        moving: Arc<AtomicBool>,
    ) -> Self {
        info!(
            rf256_id,
            target_position, position_window, "Creating new MoveThread"
        );
        Self {
            rf256_client,
            standa_client,

            rf256: Rf256::new(rf256_id),
            standa: Standa::new(),

            target_position,
            position_window,
            moving,
        }
    }

    #[instrument(skip(self), name = "get_position")]
    fn position(&self) -> io::Result<f32> {
        info!("Attempting to read current position");
        let mut client = self.rf256_client.lock().unwrap();
        let result = self.rf256.read_data(client.deref_mut());
        if let Ok(pos) = result {
            info!(position = pos, "Read current position successfully");
        } else if let Err(ref e) = result {
            tracing::error!(error = %e, "Failed to read position");
        }
        result
    }

    #[instrument(skip(self), name = "check_movement_status")]
    fn is_moving(&self) -> bool {
        let moving = self.moving.load(Ordering::SeqCst);
        info!(moving, "Checking movement status");
        moving
    }

    #[instrument(skip(self), name = "get_standa_state")]
    fn get_state(&self) -> io::Result<StateParams> {
        info!("Requesting Standa state");
        let mut client = self.standa_client.lock().unwrap();
        let result = self.standa.get_state(client.deref_mut());
        if let Ok(ref state) = result {
            info!(
                is_moving = state.is_moving(),
                left_switch = state.left_switch(),
                right_switch = state.right_switch(),
                "Got Standa state successfully"
            );
        } else if let Err(ref e) = result {
            tracing::error!(error = %e, "Failed to get Standa state");
        }
        result
    }

    #[instrument(skip(self), fields(steps, sub_steps))]
    fn send_steps(&self, steps: i32, sub_steps: i16) -> io::Result<()> {
        info!(steps, sub_steps, "Sending move command to Standa");
        let mut client = self.standa_client.lock().unwrap();
        let result = self
            .standa
            .move_relative(client.deref_mut(), steps, sub_steps);
        if let Err(ref e) = result {
            tracing::error!(error = %e, "Failed to send steps to Standa");
        } else {
            info!("Successfully sent move command to Standa");
        }
        result
    }

    #[instrument(skip(self), fields(error, steps, sub_steps))]
    fn move_relative(&self, error: f32) -> io::Result<()> {
        let (steps, sub_steps) = if error.abs() == 0.0 {
            (0, 0)
        } else if error.abs() < 0.001 {
            (0, if error > 0.0 { 1 } else { -1 })
        } else {
            ((error * STEPS_PER_MM as f32) as i32, 0)
        };

        info!(error, steps, sub_steps, "Moving relative");
        let result = self.send_steps(steps, sub_steps);
        if let Err(ref e) = result {
            tracing::error!(error = %e, "Failed to move relative");
            return result;
        }

        // Wait until motion completes or stop is requested
        info!("Waiting for motion to complete");
        let mut wait_count = 0;
        while self.is_moving() && self.get_state()?.is_moving() {
            wait_count += 1;
            if wait_count % 50 == 0 {
                // Log every ~500ms
                info!(
                    wait_time_ms = wait_count * 10,
                    "Still waiting for motion to complete"
                );
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        info!("Motion completed after waiting for ~{}ms", wait_count * 10);

        Ok(())
    }

    #[instrument(skip(self), fields(target_position, position_window))]
    pub fn run(&self) {
        info!(
            target_position = self.target_position,
            position_window = self.position_window,
            "Starting move thread"
        );

        let mut loop_count = 0;
        while self.is_moving() {
            loop_count += 1;
            info!(
                iteration = loop_count,
                "Starting positioning loop iteration"
            );

            let current_position = match self.position() {
                Ok(pos) => pos,
                Err(e) => {
                    tracing::error!(error = %e, "Error reading position, aborting move thread");
                    return;
                }
            };

            let error = current_position - self.target_position;
            info!(
                current_position,
                target_position = self.target_position,
                error,
                position_window = self.position_window,
                "Position status"
            );

            // Check if we are within the position window
            if error.abs() <= self.position_window {
                info!(
                    error_abs = error.abs(),
                    "Target position reached within window"
                );
                break;
            }

            info!(error, "Moving to correct position error");
            if let Err(e) = self.move_relative(error) {
                tracing::error!(error = %e, "Error moving Standa, aborting move thread");
                return;
            }

            let state = match self.get_state() {
                Ok(state) => state,
                Err(e) => {
                    tracing::error!(error = %e, "Error getting state, aborting move thread");
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

            info!("Sleeping before next positioning iteration");
            std::thread::sleep(Duration::from_millis(10));
        }

        info!(iterations = loop_count, "Move thread completed");
    }
}

impl<T: Write + Read + Send> Drop for MoveThread<T> {
    fn drop(&mut self) {
        println!("Stopping move thread");
        self.moving.store(false, Ordering::SeqCst);
    }
}
