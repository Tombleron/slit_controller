use motarem::axis::{
    Axis, limit_switches::LimitSwitches, movement_parameters::MovementParams, state::AxisState,
    state_info::AxisStateInfo,
};
use utilities::motor_controller::MotorHolder;

use crate::command_executor::sensors::command_sender::SensorsCommandSender;

pub struct CollimatorAxis {
    pub name: String,
    axis: usize,

    sensors_cs: SensorsCommandSender,
}

impl CollimatorAxis {
    pub fn new(name: String, axis: usize, sensors_cs: SensorsCommandSender) -> Self {
        Self {
            name,
            axis,
            sensors_cs,
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
impl Axis for CollimatorAxis {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(
        &self,
        _position: f64,
        _parameters: Option<MovementParams>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_state(&self) -> anyhow::Result<AxisStateInfo> {
        Ok(AxisStateInfo {
            state: AxisState::On,
            message: None,
            limit_switches: LimitSwitches::None,
        })
    }

    async fn get_attribute(&self, name: &str) -> anyhow::Result<f64> {
        match name {
            "position" => self
                .get_temperature()
                .await
                .map(|pos| pos as f64)
                .map_err(|err| anyhow::Error::msg(format!("Failed to get temperature: {}", err))),
            "temperature" => self
                .get_temperature()
                .await
                .map(|temp| temp as f64)
                .map_err(|err| anyhow::Error::msg(format!("Failed to get temperature: {}", err))),
            _ => Err(anyhow::Error::msg(format!("Unknown attribute: {}", name))),
        }
    }

    async fn get_available_params(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec!["temperature".to_string()])
    }

    async fn get_supported_movement_params(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }
}

pub struct MotorParameters {}
pub struct StateParams {}

impl MotorHolder for CollimatorAxis {
    type MovementParameters = MotorParameters;
    type MotorState = StateParams;

    async fn stop(&self) -> Result<(), String> {
        Ok(())
    }

    async fn update_parameters(
        &self,
        _parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn get_state(&self) -> Result<Self::MotorState, String> {
        Ok(StateParams {})
    }

    async fn get_position(&self) -> Result<f32, String> {
        self.sensors_cs
            .get_position(self.axis as u8)
            .await
            .map_err(|e| format!("Failed to get position: {}", e))
    }

    fn is_moving(&self) -> bool {
        false
    }

    fn set_moving(&self, _is_moving: bool) {}

    async fn init_motion(
        &self,
        _target: f32,
        _parameters: &Self::MovementParameters,
    ) -> Result<(), String> {
        Ok(())
    }
}
