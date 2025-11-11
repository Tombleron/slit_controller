use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use em2rs::StateParams;
use motarem::axis::{
    Axis, limit_switches::LimitSwitches, movement_parameters::MovementParams, state::AxisState,
    state_info::AxisStateInfo,
};
use tokio::{sync::Mutex, task::JoinHandle};
use utilities::motor_controller::{Motor as _, MotorHolder};

use super::params::MotorParameters;
use crate::{
    command_executor::{
        motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
    },
    controllers::cooled_slit::motor::CooledSlitMotor,
};

pub struct CooledSlitAxis {
    pub name: String,
    axis: usize,

    sensors_cs: SensorsCommandSender,
    motor_cs: Em2rsCommandSender,

    move_thread: Arc<Mutex<Option<JoinHandle<Result<(), String>>>>>,
    is_moving: Arc<AtomicBool>,

    steps_per_mm: u32,
}

impl CooledSlitAxis {
    pub fn new(
        name: String,
        axis: usize,
        sensors_cs: SensorsCommandSender,
        motor_cs: Em2rsCommandSender,
        steps_per_mm: u32,
    ) -> Self {
        Self {
            name,
            axis,
            sensors_cs,
            motor_cs,
            move_thread: Arc::new(Mutex::new(None)),
            is_moving: Arc::new(AtomicBool::new(false)),
            steps_per_mm,
        }
    }

    pub async fn get_temperature(&self) -> Result<f32, String> {
        self.sensors_cs
            .get_temperature(self.axis as u8)
            .await
            .map_err(|e| format!("Failed to get temperature: {}", e))
    }
}

#[async_trait::async_trait]
impl Axis for CooledSlitAxis {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self, position: f64, parameters: Option<MovementParams>) -> anyhow::Result<()> {
        let motor_params = parameters.unwrap_or_default().into();

        self.move_to(position as f32, motor_params)
            .await
            .map_err(|e| anyhow::Error::msg(format!("Failed to move motor: {}", e)))?;

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        MotorHolder::stop(self)
            .await
            .map_err(|e| anyhow::Error::msg(format!("Failed to stop motor: {}", e)))
    }

    async fn get_state(&self) -> anyhow::Result<AxisStateInfo> {
        let motor_state = MotorHolder::get_state(self)
            .await
            .map_err(|e| anyhow::Error::msg(format!("Failed to get motor state: {}", e)))?;

        let is_moving = self.is_moving.load(Ordering::Relaxed);

        let state = if is_moving {
            AxisState::Moving
        } else {
            AxisState::On
        };

        let limit_switches = match (
            motor_state.low_limit_triggered(),
            motor_state.high_limit_triggered(),
        ) {
            (true, true) => LimitSwitches::Both,
            (true, false) => LimitSwitches::Lower,
            (false, true) => LimitSwitches::Upper,
            (false, false) => LimitSwitches::None,
        };

        let message = match (motor_state.is_moving(), is_moving) {
            (true, false) => Some("Motor is moving, but axis is not".to_string()),
            _ => None,
        };

        Ok(AxisStateInfo {
            state,
            message,
            limit_switches,
        })
    }
    async fn get_attribute(&self, name: &str) -> anyhow::Result<f64> {
        match name {
            "position" => MotorHolder::get_position(self)
                .await
                .map(|pos| pos as f64)
                .map_err(|err| anyhow::Error::msg(format!("Failed to get position: {}", err))),
            "temperature" => self
                .get_temperature()
                .await
                .map(|temp| temp as f64)
                .map_err(|err| anyhow::Error::msg(format!("Failed to get temperature: {}", err))),
            _ => Err(anyhow::Error::msg(format!("Unknown attribute: {}", name))),
        }
    }

    async fn get_available_params(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec!["position".to_string(), "temperature".to_string()])
    }

    async fn get_supported_movement_params(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            "velocity".to_string(),
            "acceleration".to_string(),
            "deceleration".to_string(),
        ])
    }
}

impl MotorHolder for CooledSlitAxis {
    type MovementParameters = MotorParameters;
    type MotorState = StateParams;

    async fn stop(&self) -> Result<(), String> {
        if self.is_moving() {
            self.is_moving.store(false, Ordering::Relaxed);

            self.motor_cs
                .stop(self.axis)
                .await
                .map_err(|e| format!("Failed to stop motor: {}", e))?;

            let mut move_thread = self.move_thread.lock().await;
            if let Some(handle) = move_thread.take() {
                let _ = handle.await.map_err(|_| "Failed to join move thread")?;
            }
        }

        Ok(())
    }

    async fn update_parameters(&self, parameters: &Self::MovementParameters) -> Result<(), String> {
        self.motor_cs
            .set_acceleration(self.axis, parameters.acceleration)
            .await
            .map_err(|e| format!("Failed to set acceleration: {}", e))?;
        self.motor_cs
            .set_deceleration(self.axis, parameters.deceleration)
            .await
            .map_err(|e| format!("Failed to set deceleration: {}", e))?;
        self.motor_cs
            .set_velocity(self.axis, parameters.velocity)
            .await
            .map_err(|e| format!("Failed to set velocity: {}", e))?;
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
        self.is_moving.load(Ordering::Relaxed)
    }

    fn set_moving(&self, is_moving: bool) {
        self.is_moving.store(is_moving, Ordering::Relaxed);
    }

    async fn init_motion(
        &self,
        target: f32,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        let mut move_thread = CooledSlitMotor::new(
            self.axis,
            self.sensors_cs.clone(),
            self.motor_cs.clone(),
            target,
            parameters.position_window,
            parameters.time_limit,
            self.is_moving.clone(),
            self.steps_per_mm,
        );

        let handle = tokio::spawn(async move { move_thread.run().await });

        let mut move_thread = self.move_thread.lock().await;
        *move_thread = Some(handle);

        Ok(())
    }
}
