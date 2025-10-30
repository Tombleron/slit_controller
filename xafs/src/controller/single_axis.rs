use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use em2rs::StateParams;
use tokio::task::JoinHandle;
use utilities::motor_controller::{Motor as _, MotorController};

use crate::{
    command_executor::{
        motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
    },
    controller::move_thread::MoveThread,
    models::AxisState,
};

#[derive(Debug)]
pub struct MoveArgs {
    pub acceleration: u16,
    pub deceleration: u16,
    pub velocity: u16,
    pub position_window: f32,
    pub time_limit: Duration,
}

impl Default for MoveArgs {
    fn default() -> Self {
        Self {
            acceleration: 100,
            deceleration: 100,
            velocity: 100,
            position_window: 0.001,
            time_limit: Duration::from_secs(60),
        }
    }
}

pub struct SingleAxis {
    axis: usize,

    sensors_cs: SensorsCommandSender,
    motor_cs: Em2rsCommandSender,

    move_thread: Option<JoinHandle<Result<(), String>>>,
    moving: Arc<AtomicBool>,

    steps_per_mm: u32,
}

impl SingleAxis {
    pub fn new(
        axis: usize,
        steps_per_mm: u32,
        sensors_cs: SensorsCommandSender,
        motor_cs: Em2rsCommandSender,
    ) -> Self {
        Self {
            axis,
            sensors_cs,
            motor_cs,
            move_thread: None,
            moving: Arc::new(AtomicBool::new(false)),
            steps_per_mm,
        }
    }

    // pub async fn get_temperature(&self) -> Result<f32, String> {
    //     self.sensors_cs
    //         .get_temperature(self.axis as u8)
    //         .await
    //         .map_err(|e| format!("Failed to get temperature: {}", e))
    // }

    pub async fn get_axis_state(&self) -> AxisState {
        let (
            state,
            position,
            // temperature
        ) = tokio::join!(
            self.get_state(),
            self.get_position(),
            // self.get_temperature()
        );

        let is_moving = Ok(self.moving.load(Ordering::Relaxed));

        AxisState {
            state,
            position,
            // temperature,
            is_moving,
        }
    }
}

impl MotorController for SingleAxis {
    type MovementParameters = MoveArgs;
    type MotorState = StateParams;

    async fn stop(&mut self) -> Result<(), String> {
        if self.is_moving() {
            self.moving.store(false, Ordering::SeqCst);

            self.motor_cs
                .stop(self.axis)
                .await
                .map_err(|e| format!("Failed to stop motor: {}", e))?;

            if let Some(handle) = self.move_thread.take() {
                handle
                    .await
                    .map_err(|e| format!("Failed to stop move thread: {}", e))??;
            }
        }
        Ok(())
    }

    async fn update_parameters(
        &mut self,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        self.motor_cs
            .set_acceleration(self.axis, parameters.acceleration)
            .await
            .map_err(|e| format!("Failed to set acceleration: {}", e))?;
        self.motor_cs
            .set_deceleration(self.axis, parameters.deceleration)
            .await
            .map_err(|e| format!("Failed to set deceleration: {}", e))?;
        self.motor_cs
            .set_velocity(self.axis, parameters.velocity as u16)
            .await
            .map_err(|e| format!("Failed to set velocity: {}", e))?;
        dbg!(parameters);
        Ok(())
    }

    async fn get_state(&self) -> Result<Self::MotorState, String> {
        self.motor_cs
            .get_state(self.axis)
            .await
            .map_err(|e| format!("Failed to get state: {}", e))
    }

    async fn get_position(&self) -> Result<f32, String> {
        self.sensors_cs
            .get_position(self.axis as u8)
            .await
            .map_err(|e| format!("Failed to get position: {}", e))
    }

    fn is_moving(&self) -> bool {
        self.moving.load(Ordering::Relaxed)
    }

    fn set_moving(&mut self, is_moving: bool) {
        self.moving.store(is_moving, Ordering::Relaxed);
    }

    fn init_motion(
        &mut self,
        target: f32,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        let mut move_thread = MoveThread::new(
            self.axis,
            self.sensors_cs.clone(),
            self.motor_cs.clone(),
            target,
            parameters.position_window,
            parameters.time_limit,
            self.moving.clone(),
            self.steps_per_mm,
        );

        let handle = tokio::spawn(async move { move_thread.run().await });

        self.move_thread = Some(handle);

        Ok(())
    }
}
