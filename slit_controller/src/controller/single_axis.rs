use std::{
    io::{self, Read},
    net::TcpStream,
    ops::DerefMut as _,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{controller::move_thread::MoveThread, models::AxisState};
use rf256::Rf256;
use standa::{command::state::StateParams, Standa};
use tracing::{debug, error, field::debug, info, warn};
use trid::Trid;

#[derive(Debug, Clone, Copy)]
pub struct MovementParams {
    pub acceleration: u16,
    pub deceleration: u16,
    pub velocity: u32,
    pub position_window: f32,
    pub time_limit: Duration,
}

impl Default for MovementParams {
    fn default() -> Self {
        MovementParams {
            acceleration: 500,
            deceleration: 500,
            velocity: 2000,
            position_window: 0.0005,
            time_limit: Duration::from_secs(60),
        }
    }
}

pub struct SingleAxis {
    rf256_id: u8,
    trid_id: u8,

    rf256_client: Arc<Mutex<TcpStream>>,
    trid_client: Arc<Mutex<TcpStream>>,
    standa_client: Arc<Mutex<TcpStream>>,

    move_thread: Option<JoinHandle<()>>,
    moving: Arc<AtomicBool>,
}

impl SingleAxis {
    pub fn new(
        rf256_client: Arc<Mutex<TcpStream>>,
        rf256_id: u8,
        trid_client: Arc<Mutex<TcpStream>>,
        trid_id: u8,
        standa_client: Arc<Mutex<TcpStream>>,
    ) -> Self {
        info!("Initializing SingleAxis with id {}", rf256_id);

        Self {
            rf256_id,
            trid_id,
            rf256_client,
            trid_client,
            standa_client,
            move_thread: None,
            moving: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn temperature(&self) -> io::Result<f32> {
        debug("Acquiring lock on TRID client for temperature reading");
        let mut client = match self.trid_client.lock() {
            Ok(client) => client,
            Err(e) => {
                warn!("Failed to acquire lock on TRID client: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire lock",
                ));
            }
        };

        debug!("Reading temperature from id {}", self.trid_id);
        let trid = Trid::new(1);
        let result = trid.read_data(client.deref_mut(), self.trid_id as u16);

        match &result {
            Ok(temperature) => debug!("Successfully read temperature: {}", temperature),
            Err(e) => warn!("Failed to read temperature: {}", e),
        };

        result
    }

    pub fn position_with_retries(&self, retries: u8) -> io::Result<f32> {
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

    pub fn position(&self) -> io::Result<f32> {
        debug!("Acquiring lock on RF256 client for position reading");
        let mut client = match self.rf256_client.lock() {
            Ok(client) => client,
            Err(e) => {
                warn!("Failed to acquire lock on RF256 client: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire lock",
                ));
            }
        };

        debug!("Reading position from id {}", self.rf256_id);

        let id = Rf256::new(self.rf256_id).read_id(client.deref_mut())?;
        if id != self.rf256_id {
            warn!("RF256 ID mismatch: expected {}, got {}", self.rf256_id, id);

            {
                let mut buf = [0; 1024];
                let _content_in_buffer = client.deref_mut().read(&mut buf);
            }

            // fetch new id
            let new_id = Rf256::new(self.rf256_id).read_id(client.deref_mut())?;

            if new_id != self.rf256_id {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("RF256 ID mismatch: expected {}, got {}", self.rf256_id, id),
                ));
            }
        }
        let result = Rf256::new(self.rf256_id).read_data(client.deref_mut());

        match &result {
            Ok(position) => debug!("Successfully read position: {}", position),
            Err(e) => warn!("Failed to read position: {}", e),
        }

        result
    }

    pub fn is_moving(&self) -> bool {
        let moving = self.moving.load(Ordering::SeqCst);
        debug!("Axis {} is moving: {}", self.rf256_id, moving);
        moving
    }

    pub fn state(&self) -> io::Result<StateParams> {
        debug!("Reading state for id {}", self.rf256_id);
        debug!("Acquiring lock on Standa client");
        let mut client = match self.standa_client.lock() {
            Ok(client) => client,
            Err(e) => {
                warn!("Failed to acquire lock on Standa client: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire lock",
                ));
            }
        };

        let result = Standa::new().get_state(client.deref_mut());

        match &result {
            Ok(_state) => debug!("Successfully read state for id {}", self.rf256_id),
            Err(e) => warn!("Failed to read state: {}", e),
        }

        result
    }

    pub fn update_motor_settings(&self, params: MovementParams) -> io::Result<()> {
        debug!("Updating motor settings for id {}", self.rf256_id);
        debug!("Acquiring lock on Standa client for updating settings");
        let mut client = match self.standa_client.lock() {
            Ok(client) => client,
            Err(e) => {
                warn!("Failed to acquire lock on Standa client: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to acquire lock",
                ));
            }
        };

        let standa = Standa::new();

        debug!("Setting velocity to {}", params.velocity);
        if let Err(e) = standa.set_velocity(client.deref_mut(), params.velocity) {
            warn!("Failed to set velocity: {}", e);
            return Err(e);
        }

        debug!("Setting acceleration to {}", params.acceleration);
        if let Err(e) = standa.set_acceleration(client.deref_mut(), params.acceleration) {
            warn!("Failed to set acceleration: {}", e);
            return Err(e);
        }

        debug!("Setting deceleration to {}", params.deceleration);
        if let Err(e) = standa.set_deceleration(client.deref_mut(), params.deceleration) {
            warn!("Failed to set deceleration: {}", e);
            return Err(e);
        }

        info!(
            "Successfully updated motor settings for id {}",
            self.rf256_id
        );
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        debug!("Attempting to stop axis {}", self.rf256_id);

        Standa::new()
            .stop(
                self.standa_client
                    .lock()
                    .map_err(|e| {
                        warn!("Failed to acquire lock on Standa client: {}", e);
                        "Failed to acquire lock".to_string()
                    })?
                    .deref_mut(),
            )
            .map_err(|e| {
                warn!("Failed to stop axis {}: {}", self.rf256_id, e);
                format!("Failed to stop axis {}: {}", self.rf256_id, e)
            })?;

        self.moving.store(false, Ordering::SeqCst);

        if let Some(handle) = self.move_thread.take() {
            debug!("Joining move thread for id {}", self.rf256_id);
            match handle.join() {
                Ok(_) => debug!("Successfully joined move thread"),
                Err(e) => {
                    warn!("Failed to join move thread: {:?}", e);
                    return Err("Failed to join move thread".to_string());
                }
            }
        } else {
            debug!("No move thread to join for id {}", self.rf256_id);
        }

        info!("Successfully stopped axis {}", self.rf256_id);
        Ok(())
    }

    pub fn move_to_position(
        &mut self,
        target: f32,
        params: Option<MovementParams>,
    ) -> Result<(), String> {
        debug!(
            "Attempting to move axis {} to position {}",
            self.rf256_id, target
        );
        if self.moving.load(Ordering::SeqCst) {
            warn!(
                "Attempted to move id {} which is already in motion",
                self.rf256_id
            );
            return Err("Axis is already in motion".to_string());
        }

        let params = params.unwrap_or_else(MovementParams::default);

        info!("Moving id {} to position {}", self.rf256_id, target);
        debug!("Updating motor settings before movement");
        self.update_motor_settings(params).map_err(|e| {
            warn!("Failed to update motor settings: {}", e);
            format!("Failed to update motor settings: {}", e)
        })?;

        debug!("Setting moving flag to true");
        self.moving.store(true, Ordering::SeqCst);

        debug!("Creating MoveThread for axis {}", self.rf256_id);
        let mut move_thread = MoveThread::new(
            Arc::clone(&self.rf256_client),
            Arc::clone(&self.standa_client),
            self.rf256_id,
            target,
            params.position_window,
            params.time_limit,
            Arc::clone(&self.moving),
        );

        debug!("Spawning thread for axis {} movement", self.rf256_id);
        let handle = std::thread::spawn({
            let rf256_id = self.rf256_id;
            move || {
                debug!("Move thread started for axis {}", rf256_id);
                move_thread.run();
                debug!("Move thread completed for axis {}", rf256_id);
            }
        });

        debug!("Storing thread handle");
        self.move_thread = Some(handle);

        info!(
            "Successfully initiated movement of axis {} to position {}",
            self.rf256_id, target
        );
        Ok(())
    }

    pub fn get_axis_state(&self) -> io::Result<AxisState> {
        debug!("Getting axis state");

        // Clone the necessary Arc<Mutex<>> references
        let rf256_client = Arc::clone(&self.rf256_client);
        let trid_client = Arc::clone(&self.trid_client);
        let standa_client = Arc::clone(&self.standa_client);
        let rf256_id = self.rf256_id;
        let trid_id = self.trid_id;

        // Spawn threads with cloned resources instead of capturing self
        let state_get_handle = thread::spawn(move || {
            debug!("Thread: reading state");
            let mut client = match standa_client.lock() {
                Ok(client) => client,
                Err(e) => return Err(format!("Failed to acquire lock on Standa client: {}", e)),
            };
            Standa::new()
                .get_state(client.deref_mut())
                .map_err(|e| e.to_string())
        });

        let position_get_handle = thread::spawn(move || {
            debug!("Thread: reading position");
            let mut attempts = 0;
            let retries = 3;
            loop {
                let mut client = match rf256_client.lock() {
                    Ok(client) => client,
                    Err(e) => return Err(format!("Failed to acquire lock on RF256 client: {}", e)),
                };

                let result = Rf256::new(rf256_id).read_id(client.deref_mut());
                if let Err(e) = result {
                    if attempts < retries {
                        attempts += 1;
                        continue;
                    }
                    return Err(e.to_string());
                }

                match Rf256::new(rf256_id).read_data(client.deref_mut()) {
                    Ok(position) => return Ok(position),
                    Err(_e) if attempts < retries => {
                        attempts += 1;
                        continue;
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        });

        let temperature_get_handle = thread::spawn(move || {
            debug!("Thread: reading temperature");
            let mut client = match trid_client.lock() {
                Ok(client) => client,
                Err(e) => return Err(format!("Failed to acquire lock on TRID client: {}", e)),
            };
            let trid = Trid::new(1);
            trid.read_data(client.deref_mut(), trid_id as u16)
                .map_err(|e| e.to_string())
        });

        // Get simple values before joining threads
        let is_moving = Ok(self.is_moving());

        // Join threads and collect results
        let state = state_get_handle.join().expect("State thread panicked");
        let position = position_get_handle
            .join()
            .expect("Position thread panicked");
        let temperature = temperature_get_handle
            .join()
            .expect("Temperature thread panicked");

        Ok(AxisState {
            position,
            state,
            is_moving,
            temperature,
        })
    }
}

impl Drop for SingleAxis {
    fn drop(&mut self) {
        if self.moving.load(Ordering::SeqCst) {
            let _ = self.stop();
        }
    }
}
