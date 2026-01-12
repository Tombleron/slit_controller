use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use motarem::axis::{
    limit_switches::LimitSwitches, movement_parameters::MovementParams, state::AxisState,
    state_info::AxisStateInfo, Axis,
};
use standa::command::state::StateParams;
use tokio::{sync::Mutex, task::JoinHandle};
use utilities::motor_controller::{Motor as _, MotorHolder};

use crate::{
    command_executor::{
        encoder::command_sender::EncoderCommandSender, motor::command_sender::StandaCommandSender,
        temperature::command_sender::TridCommandSender,
    },
    controllers::slit_controller::{motor::SlitMotor, params::MotorParameters},
};

pub struct SlitAxis {
    pub name: String,
    axis: u8,

    rf256_cs: EncoderCommandSender,
    trid_cs: TridCommandSender,
    standa_cs: StandaCommandSender,

    move_thread: Arc<Mutex<Option<JoinHandle<Result<(), String>>>>>,
    is_moving: Arc<AtomicBool>,

    steps_per_mm: i32,
}

impl SlitAxis {
    pub fn new(
        name: String,
        axis: u8,
        rf256_cs: EncoderCommandSender,
        trid_cs: TridCommandSender,
        standa_cs: StandaCommandSender,
        steps_per_mm: i32,
    ) -> Self {
        Self {
            name,
            axis,
            rf256_cs,
            trid_cs,
            standa_cs,
            move_thread: Arc::new(Mutex::new(None)),
            is_moving: Arc::new(AtomicBool::new(false)),
            steps_per_mm,
        }
    }

    pub async fn get_temperature(&self) -> Result<f32, String> {
        self.trid_cs
            .read_temperature(self.axis)
            .await
            .map_err(|e| format!("Failed to read temperature: {}", e))
    }
}

#[async_trait::async_trait]
impl Axis for SlitAxis {
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

        let limit_switches = match (motor_state.left_switch(), motor_state.right_switch()) {
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
            "position_window".to_string(),
            "time_limit".to_string(),
        ])
    }
}

impl MotorHolder for SlitAxis {
    type MovementParameters = MotorParameters;
    type MotorState = StateParams;

    async fn stop(&self) -> Result<(), String> {
        if self.is_moving() {
            self.is_moving.store(false, Ordering::Relaxed);

            self.standa_cs
                .stop()
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
        self.standa_cs
            .set_acceleration(parameters.acceleration)
            .await
            .map_err(|e| format!("Failed to set acceleration: {}", e))?;
        self.standa_cs
            .set_deceleration(parameters.deceleration)
            .await
            .map_err(|e| format!("Failed to set deceleration: {}", e))?;
        self.standa_cs
            .set_velocity(parameters.velocity)
            .await
            .map_err(|e| format!("Failed to set velocity: {}", e))?;
        Ok(())
    }

    async fn get_state(&self) -> Result<Self::MotorState, String> {
        match self
            .standa_cs
            .get_state()
            .await
            .map_err(|e| format!("Failed to get state: {}", e))
        {
            Err(e) => {
                self.standa_cs
                    .reconnect()
                    .await
                    .map_err(|e| format!("Failed to reconnect: {}", e))?;
                self.standa_cs
                    .get_state()
                    .await
                    .map_err(|e| format!("Failed to get state: {}", e))
            }
            Ok(s) => Ok(s),
        }
    }

    async fn get_position(&self) -> Result<f32, String> {
        self.rf256_cs
            .get_position(self.axis)
            .await
            .map_err(|e| format!("Failed to get position: {}", e))
    }

    async fn init_motion(
        &self,
        target: f32,
        parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        let mut move_thread = SlitMotor::new(
            self.rf256_cs.clone(),
            self.axis,
            self.standa_cs.clone(),
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

    fn is_moving(&self) -> bool {
        self.is_moving.load(Ordering::Relaxed)
    }

    fn set_moving(&self, is_moving: bool) {
        self.is_moving.store(is_moving, Ordering::Relaxed);
    }
}
