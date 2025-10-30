use serde::{Deserialize, Serialize};
use utilities::motor_controller::MotorController as _;

use crate::{
    controller::single_axis::{MoveArgs, SingleAxis},
    models::AxisState,
};

pub struct Axes {
    pub y_up: SingleAxis,
    pub y_down: SingleAxis,
    pub x_left: SingleAxis,
    pub x_right: SingleAxis,
}

impl Axes {
    pub fn new(
        y_up: SingleAxis,
        y_down: SingleAxis,
        x_left: SingleAxis,
        x_right: SingleAxis,
    ) -> Self {
        Self {
            y_up,
            y_down,
            x_left,
            x_right,
        }
    }

    pub fn by_index(&self, index: usize) -> Option<&SingleAxis> {
        match index {
            0 => Some(&self.y_up),
            1 => Some(&self.y_down),
            2 => Some(&self.x_left),
            3 => Some(&self.x_right),
            _ => None,
        }
    }

    pub fn by_index_mut(&mut self, index: usize) -> Option<&mut SingleAxis> {
        match index {
            0 => Some(&mut self.y_up),
            1 => Some(&mut self.y_down),
            2 => Some(&mut self.x_left),
            3 => Some(&mut self.x_right),
            _ => None,
        }
    }
}

pub struct MultiAxis {
    axes: Axes,
}

impl MultiAxis {
    pub fn new(axes: Axes) -> Self {
        Self { axes }
    }

    pub fn is_moving(&self, index: usize) -> Option<bool> {
        if let Some(axis) = self.axes.by_index(index) {
            Some(axis.is_moving())
        } else {
            None
        }
    }

    pub async fn move_to_position(
        &mut self,
        index: usize,
        position: f32,
        args: MoveArgs,
    ) -> Result<(), String> {
        if let Some(axis) = self.axes.by_index_mut(index) {
            axis.move_to(position, args)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Incorrect axis index".to_string())
        }
    }

    pub async fn stop(&mut self, index: usize) -> Result<(), String> {
        if let Some(axis) = self.axes.by_index_mut(index) {
            axis.stop().await.map_err(|e| e.to_string())
        } else {
            Err("Incorrect axis index".to_string())
        }
    }

    pub async fn position(&mut self, index: usize) -> Result<f32, String> {
        if let Some(axis) = self.axes.by_index_mut(index) {
            axis.get_position().await.map_err(|e| e.to_string())
        } else {
            Err("Incorrect axis index".to_string())
        }
    }

    // pub async fn temperature(&mut self, index: usize) -> Result<f32, String> {
    //     if let Some(axis) = self.axes.by_index_mut(index) {
    //         axis.get_temperature().await.map_err(|e| e.to_string())
    //     } else {
    //         Err("Incorrect axis index".to_string())
    //     }
    // }

    pub async fn state(&mut self, index: usize) -> Result<AxisState, String> {
        if let Some(axis) = self.axes.by_index(index) {
            let state = axis.get_axis_state().await;

            Ok(state)
        } else {
            Err("Incorrect axis index".to_string())
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AxisConfig {
    pub lir_id: u8,
    pub lir_step: f32,

    pub em2rs_id: u8,
    pub em2rs_low_limit: u8,
    pub em2rs_high_limit: u8,
    pub steps_per_mm: u32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MultiAxisConfig {
    pub sensors_ip: String,
    pub sensors_port: u16,

    pub em2rs_ip: String,
    pub em2rs_port: u16,

    pub icpcon_id: u8,

    pub upper_axis: AxisConfig,
    pub lower_axis: AxisConfig,
    pub left_axis: AxisConfig,
    pub right_axis: AxisConfig,
}

impl Default for MultiAxisConfig {
    fn default() -> Self {
        Self {
            sensors_ip: "127.0.0.1".to_string(),
            sensors_port: 50051,

            em2rs_ip: "127.0.0.1".to_string(),
            em2rs_port: 50052,

            icpcon_id: 1,

            upper_axis: AxisConfig {
                lir_id: 1,
                lir_step: 0.05,

                em2rs_id: 1,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            lower_axis: AxisConfig {
                lir_id: 2,
                lir_step: 0.05,

                em2rs_id: 2,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            left_axis: AxisConfig {
                lir_id: 3,
                lir_step: 0.05,

                em2rs_id: 3,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            right_axis: AxisConfig {
                lir_id: 4,
                lir_step: 0.05,

                em2rs_id: 4,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
        }
    }
}
