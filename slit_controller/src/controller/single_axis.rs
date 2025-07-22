use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    command_executor::{
        encoder::command_sender::Rf256CommandSender, motor::command_sender::StandaCommandSender,
        temperature::command_sender::TridCommandSender,
    },
    controller::move_thread::MoveThread,
    models::AxisState,
};
use standa::command::state::StateParams;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

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
            velocity: 400,
            position_window: 0.0005,
            time_limit: Duration::from_secs(60),
        }
    }
}

pub struct SingleAxis {
    axis: u8,

    rf256_cs: Rf256CommandSender,
    trid_cs: TridCommandSender,
    standa_cs: StandaCommandSender,

    move_thread: Option<JoinHandle<()>>,
    moving: Arc<AtomicBool>,
}

impl SingleAxis {
    pub fn new(
        axis: u8,
        rf256_cs: Rf256CommandSender,
        trid_cs: TridCommandSender,
        standa_cs: StandaCommandSender,
    ) -> Self {
        Self {
            axis,

            rf256_cs,
            trid_cs,
            standa_cs,

            move_thread: None,
            moving: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_rf256_cs(&self) -> Rf256CommandSender {
        self.rf256_cs.clone()
    }

    pub fn get_trid_cs(&self) -> TridCommandSender {
        self.trid_cs.clone()
    }

    pub async fn reconnect_standa_client(&self) -> io::Result<()> {
        self.standa_cs.reconnect().await
    }

    pub async fn temperature(&self) -> io::Result<f32> {
        let result = self.trid_cs.read_temperature(self.axis).await;

        match &result {
            Ok(temperature) => debug!("Successfully read temperature: {}", temperature),
            Err(e) => warn!("Failed to read temperature: {}", e),
        };

        result
    }

    pub async fn position_with_retries(&self, retries: u8) -> io::Result<f32> {
        self.rf256_cs
            .read_position_with_retries(self.axis, retries)
            .await
            .map_err(|e| {
                warn!("Failed to read position with retries: {}", e);
                io::Error::new(io::ErrorKind::Other, e)
            })
    }

    pub async fn position(&self) -> io::Result<f32> {
        self.rf256_cs.read_position(self.axis).await
    }

    pub fn is_moving(&self) -> bool {
        let moving = self.moving.load(Ordering::SeqCst);
        debug!("Axis {} is moving: {}", self.axis, moving);
        moving
    }

    pub async fn state(&self) -> io::Result<StateParams> {
        self.standa_cs.get_state().await
    }

    pub async fn update_motor_settings(&self, params: MovementParams) -> io::Result<()> {
        if let Err(e) = self.standa_cs.set_velocity(params.velocity).await {
            warn!("Failed to set velocity: {}", e);
            return Err(e);
        }

        if let Err(e) = self.standa_cs.set_acceleration(params.acceleration).await {
            warn!("Failed to set acceleration: {}", e);
            return Err(e);
        }

        if let Err(e) = self.standa_cs.set_deceleration(params.deceleration).await {
            warn!("Failed to set deceleration: {}", e);
            return Err(e);
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        debug!("Attempting to stop axis {}", self.axis);

        self.standa_cs
            .stop()
            .await
            .map_err(|e| format!("Failed to stop axis {}: {}", self.axis, e))?;

        self.moving.store(false, Ordering::SeqCst);

        if let Some(handle) = self.move_thread.take() {
            debug!("Joining move thread for id {}", self.axis);
            match handle.await {
                Ok(_) => debug!("Successfully joined move thread"),
                Err(e) => {
                    warn!("Failed to join move thread: {:?}", e);
                    return Err("Failed to join move thread".to_string());
                }
            }
        } else {
            debug!("No move thread to join for id {}", self.axis);
        }

        info!("Successfully stopped axis {}", self.axis);
        Ok(())
    }

    pub async fn move_to_position(
        &mut self,
        target: f32,
        params: MovementParams,
    ) -> Result<(), String> {
        debug!(
            "Attempting to move axis {} to position {}",
            self.axis, target
        );
        if self.moving.load(Ordering::SeqCst) {
            warn!(
                "Attempted to move id {} which is already in motion",
                self.axis
            );
            return Err("Axis is already in motion".to_string());
        }

        info!("Moving id {} to position {}", self.axis, target);
        self.update_motor_settings(params)
            .await
            .map_err(|e| format!("Failed to update motor settings: {}", e))?;

        debug!("Setting moving flag to true");
        self.moving.store(true, Ordering::SeqCst);

        let rf256_axis = self.axis;
        let rf256_cs = self.rf256_cs.clone();
        let standa_cs = self.standa_cs.clone();
        let moving = Arc::clone(&self.moving);

        debug!("Spawning thread for axis {} movement", self.axis);
        let handle = tokio::spawn(async move {
            let mut move_thread = MoveThread::new(
                rf256_cs,
                standa_cs,
                rf256_axis,
                target,
                params.position_window,
                params.time_limit,
                moving,
            );

            move_thread.run().await
        });

        debug!("Storing thread handle");
        self.move_thread = Some(handle);

        info!(
            "Successfully initiated movement of axis {} to position {}",
            self.axis, target
        );
        Ok(())
    }

    pub async fn get_axis_state(&self) -> io::Result<AxisState> {
        let (state, position, temperature) = tokio::join!(
            self.standa_cs.get_state(),
            self.position_with_retries(5),
            self.temperature()
        );

        let is_moving = Ok(self.is_moving());

        Ok(AxisState {
            position: position.map_err(|e| e.to_string()),
            state: state.map_err(|e| e.to_string()),
            is_moving,
            temperature: temperature.map_err(|e| e.to_string()),
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
