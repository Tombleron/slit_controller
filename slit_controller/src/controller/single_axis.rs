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
        encoder::command_sender::EncoderCommandSender, motor::command_sender::StandaCommandSender,
        temperature::command_sender::TridCommandSender,
    },
    controller::move_thread::MoveThread,
    models::AxisState,
};
use standa::command::state::StateParams;
use tokio::task::JoinHandle;
use tracing::{debug, warn};
use utilities::motor_controller::{Motor as _, MotorController};

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

    rf256_cs: EncoderCommandSender,
    trid_cs: TridCommandSender,
    standa_cs: StandaCommandSender,

    move_thread: Option<JoinHandle<Result<(), String>>>,
    moving: Arc<AtomicBool>,
}

impl SingleAxis {
    pub fn new(
        axis: u8,
        rf256_cs: EncoderCommandSender,
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

    pub fn get_rf256_cs(&self) -> EncoderCommandSender {
        self.rf256_cs.clone()
    }

    pub fn get_trid_cs(&self) -> TridCommandSender {
        self.trid_cs.clone()
    }

    pub async fn temperature(&self) -> io::Result<f32> {
        let result = self.trid_cs.read_temperature(self.axis).await;

        match &result {
            Ok(temperature) => debug!("Successfully read temperature: {}", temperature),
            Err(e) => warn!("Failed to read temperature: {}", e),
        };

        result
    }

    pub async fn get_axis_state(&self) -> io::Result<AxisState> {
        let (state, position, temperature) = tokio::join!(
            self.standa_cs.get_state(),
            self.get_position(),
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
        self.set_moving(false);
    }
}

impl MotorController for SingleAxis {
    type MovementParameters = MovementParams;
    type MotorState = StateParams;

    async fn stop(&mut self) -> Result<(), String> {
        self.moving.store(false, Ordering::SeqCst);

        self.standa_cs
            .stop()
            .await
            .map_err(|e| format!("Failed to stop axis {}: {}", self.axis, e))?;

        if let Some(handle) = self.move_thread.take() {
            match handle.await {
                Ok(_) => {}
                Err(_) => {
                    return Err("Failed to join move thread".to_string());
                }
            }
        }

        Ok(())
    }

    async fn update_parameters(
        &mut self,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        self.standa_cs
            .set_velocity(parameters.velocity)
            .await
            .map_err(|e| e.to_string())?;

        self.standa_cs
            .set_acceleration(parameters.acceleration)
            .await
            .map_err(|e| e.to_string())?;

        self.standa_cs
            .set_deceleration(parameters.deceleration)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_state(&self) -> Result<Self::MotorState, String> {
        self.standa_cs.get_state().await.map_err(|e| e.to_string())
    }
    async fn get_position(&self) -> Result<f32, String> {
        self.rf256_cs
            .get_position(self.axis)
            .await
            .map_err(|e| e.to_string())
    }

    fn is_moving(&self) -> bool {
        self.moving.load(Ordering::Relaxed)
    }

    fn set_moving(&mut self, is_moving: bool) {
        self.moving.store(is_moving, Ordering::Relaxed);
    }

    fn init_motion(
        &mut self,
        target_position: f32,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        let rf256_axis = self.axis;
        let rf256_cs = self.rf256_cs.clone();
        let standa_cs = self.standa_cs.clone();
        let moving = Arc::clone(&self.moving);
        let position_window = parameters.position_window;
        let time_limit = parameters.time_limit;

        let handle = tokio::spawn(async move {
            let mut move_thread = MoveThread::new(
                rf256_cs,
                standa_cs,
                rf256_axis,
                target_position,
                position_window,
                time_limit,
                moving,
            );

            move_thread.run().await
        });

        self.move_thread = Some(handle);

        Ok(())
    }
}
