use std::time::Duration;

use motarem::axis::movement_parameters::MovementParams;

#[derive(Debug)]
pub struct MotorParameters {
    pub acceleration: u16,
    pub deceleration: u16,
    pub velocity: u16,
    pub position_window: f32,
    pub time_limit: Duration,
}

impl Default for MotorParameters {
    fn default() -> Self {
        Self {
            acceleration: 1000,
            deceleration: 1000,
            velocity: 1000,
            position_window: 0.001,
            time_limit: Duration::from_secs(60),
        }
    }
}

impl From<MovementParams> for MotorParameters {
    fn from(value: MovementParams) -> Self {
        let mut params = Self::default();

        if let Some(acceleration) = value.acceleration {
            params.acceleration = acceleration as u16;
        }
        if let Some(deceleration) = value.deceleration {
            params.deceleration = deceleration as u16;
        }
        if let Some(velocity) = value.velocity {
            params.velocity = velocity as u16;
        }
        if let Some(position_window) = value.custom.get("position_window") {
            params.position_window = *position_window as f32;
        }
        if let Some(time_limit) = value.custom.get("time_limit") {
            params.time_limit = Duration::from_secs_f64(*time_limit);
        }

        params
    }
}
