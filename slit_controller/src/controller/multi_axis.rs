use std::io::{self};

use serde::{Deserialize, Serialize};
use standa::command::state::StateParams;
use tracing::debug;

use utilities::motor_controller::MotorHolder as _;

use crate::controller::single_axis::SingleAxis;
use crate::models::AxisState;

use super::single_axis::MovementParams;

pub struct MultiAxis {
    axes: [SingleAxis; 4],
}

impl MultiAxis {
    pub fn new(axes: [SingleAxis; 4]) -> Self {
        Self { axes }
    }

    pub fn is_moving(&mut self, index: usize) -> bool {
        self.axes[index].is_moving()
    }

    pub async fn move_to_position(
        &mut self,
        index: usize,
        position: f32,
        params: MovementParams,
    ) -> Result<(), String> {
        debug!("Moving axis {} to position {}", index, position);

        // if position < 6.5 || position > 14.0 {
        //     error!("Move target is out of bounds: {position}");
        //     return Err("Position is out of bounds, (6.5, 14)".to_string());
        // }

        let result = self.axes[index].move_to(position, params).await;

        result
    }

    pub async fn stop(&mut self, index: usize) -> Result<(), String> {
        self.axes[index].stop().await.map_err(|e| e.to_string())
    }

    pub async fn position(&mut self, index: usize) -> Result<f32, String> {
        self.axes[index].get_position().await
    }

    pub async fn temperature(&mut self, index: usize) -> Result<f32, String> {
        self.axes[index]
            .temperature()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn state(&mut self, index: usize) -> Result<StateParams, String> {
        self.axes[index].get_state().await
    }

    pub async fn get_axis_state(&self, index: usize) -> io::Result<AxisState> {
        self.axes[index].get_axis_state().await
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AxisConfig {
    pub standa_ip: String,
    pub standa_port: u16,
    pub rf256_id: u8,
    pub trid_id: u8,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MultiAxisConfig {
    pub rf256_ip: String,
    pub rf256_port: u16,

    pub trid_ip: String,
    pub trid_port: u16,

    pub upper_axis: AxisConfig,

    pub lower_axis: AxisConfig,

    pub left_axis: AxisConfig,

    pub right_axis: AxisConfig,
}

impl Default for MultiAxisConfig {
    fn default() -> Self {
        Self {
            rf256_ip: "192.168.0.51".to_string(),
            rf256_port: 60003,

            trid_ip: "192.168.0.51".to_string(),
            trid_port: 60002,

            upper_axis: AxisConfig {
                standa_ip: "192.168.0.204".to_string(),
                standa_port: 2000,
                rf256_id: 6,
                trid_id: 0,
            },
            lower_axis: AxisConfig {
                standa_ip: "192.168.0.204".to_string(),
                standa_port: 3000,
                rf256_id: 15,
                trid_id: 1,
            },
            left_axis: AxisConfig {
                standa_ip: "192.168.0.205".to_string(),
                standa_port: 2000,
                rf256_id: 5,
                trid_id: 2,
            },
            right_axis: AxisConfig {
                standa_ip: "192.168.0.205".to_string(),
                standa_port: 3000,
                rf256_id: 4,
                trid_id: 3,
            },
        }
    }
}
