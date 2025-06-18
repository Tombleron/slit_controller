use std::{
    io::{self, Read, Write},
    ops::DerefMut as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use rf256::Rf256;
use standa::{command::state::StateParams, Standa};
use tracing::{error, info, instrument, warn};

const STEPS_PER_MM: u32 = 800;

pub struct MoveThread<T: Write + Read + Send> {
    rf256_client: Arc<Mutex<T>>,
    standa_client: Arc<Mutex<T>>,

    rf256: Rf256,
    standa: Standa,

    target_position: f32,
    position_window: f32,
    time_limit: Duration,

    filter: MovingAverage,

    moving: Arc<AtomicBool>,
    start_time: Instant,
}

impl<T: Write + Read + Send> MoveThread<T> {
    pub fn new(
        rf256_client: Arc<Mutex<T>>,
        standa_client: Arc<Mutex<T>>,
        rf256_id: u8,
        target_position: f32,
        position_window: f32,
        time_limit: Duration,
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

            filter: MovingAverage::new(20),

            target_position,
            position_window,
            time_limit,

            moving,
            start_time: Instant::now(),
        }
    }

    fn position(&self) -> io::Result<f32> {
        let mut client = self.rf256_client.lock().unwrap();

        let id = self.rf256.read_id(client.deref_mut())?;
        if id != self.rf256.get_device_id() {
            warn!(
                "RF256 ID mismatch: expected {}, got {}",
                self.rf256.get_device_id(),
                id
            );

            {
                let mut buf = [0; 1024];
                let _content_in_buffer = client.deref_mut().read(&mut buf);
            }

            // fetch new id
            let new_id = self.rf256.read_id(client.deref_mut())?;

            if new_id != self.rf256.get_device_id() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "RF256 ID mismatch: expected {}, got {}",
                        self.rf256.get_device_id(),
                        id
                    ),
                ));
            }
        }
        let result = self.rf256.read_data(client.deref_mut());

        result
    }

    fn position_with_retries(&self, retries: u8) -> io::Result<f32> {
        let mut attempts = 0;
        loop {
            match self.position() {
                Ok(position) => return Ok(position),
                Err(e) if attempts < retries => {
                    warn!("Failed to read position (attempt {}): {}", attempts + 1, e);
                    attempts += 1;
                }
                Err(e) => {
                    error!("Failed to read position after {} attempts: {}", retries, e);
                    return Err(e);
                }
            }
        }
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

    fn get_state(&self) -> io::Result<StateParams> {
        let mut client = self.standa_client.lock().unwrap();
        let result = self.standa.get_state(client.deref_mut());
        if let Err(ref e) = result {
            error!(error = %e, "Failed to get Standa state");
        }
        result
    }

    fn stop(&self) -> io::Result<()> {
        let mut client = self.standa_client.lock().unwrap();
        let result = self.standa.stop(client.deref_mut());
        if let Err(ref e) = result {
            error!(error = %e, "Failed to stop Standa movement");
        }
        result
    }

    fn send_steps(&self, steps: i32, sub_steps: i16) -> io::Result<()> {
        info!(steps, sub_steps, "Sending move command to Standa");
        let mut client = self.standa_client.lock().unwrap();
        let result = self
            .standa
            .move_relative(client.deref_mut(), steps, sub_steps);
        if let Err(ref e) = result {
            error!(error = %e, "Failed to send steps to Standa");
        }
        result
    }

    fn move_relative(&self, error: f32) -> io::Result<()> {
        let (steps, sub_steps) = if error.abs() == 0.0 {
            (0, 0)
        } else if error.abs() < 0.001 {
            (0, if error > 0.0 { 5 } else { -5 })
            // (if error > 0.0 { 1 } else { -1 }, 0)
        } else {
            ((error * STEPS_PER_MM as f32) as i32, 0)
        };

        let result = self.send_steps(steps, sub_steps);
        if let Err(ref e) = result {
            error!(error = %e, "Failed to move relative");
            return result;
        }

        // Wait until motion completes or stop is requested
        let mut wait_count = 0;
        while self.is_moving() && self.get_state()?.is_moving() && !self.time_limit_exceeded() {
            wait_count += 1;
            if wait_count % 50 == 0 {
                // Log every ~500ms
            }
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }

    #[instrument(skip(self), fields(target_position, position_window))]
    pub fn run(&mut self) {
        info!(
            target_position = self.target_position,
            position_window = self.position_window,
            "Starting move thread"
        );

        while self.is_moving() && !self.time_limit_exceeded() {
            let current_position = match self.position_with_retries(5) {
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

            if let Err(e) = self.move_relative(error) {
                error!(error = %e, "Error moving Standa, aborting move thread");
                return;
            }

            let state = match self.get_state() {
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

impl<T: Write + Read + Send> Drop for MoveThread<T> {
    fn drop(&mut self) {
        info!("Stopping move thread");
        self.moving.store(false, Ordering::SeqCst);

        info!("Stopping Standa movement");
        if let Err(e) = self.stop() {
            error!(error = %e, "Failed to stop Standa movement on drop");
        } else {
            info!("Successfully stopped Standa movement on drop");
        }
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
