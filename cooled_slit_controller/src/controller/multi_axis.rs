use crate::{
    command_executor::{
        motor::command_sender::Em2rsCommandSender, sensors::command_sender::SensorsCommandSender,
    },
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

    m7015_cs: SensorsCommandSender,
    em2rs_cs: Em2rsCommandSender,
}

impl MultiAxis {
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
            axis.position().await.map_err(|e| e.to_string())
        } else {
            Err("Incorrect axis index".to_string())
        }
    }

    pub async fn temperature(&mut self, index: usize) -> Result<f32, String> {
        if let Some(axis) = self.axes.by_index_mut(index) {
            unimplemented!()
        } else {
            Err("Incorrect axis index".to_string())
        }
    }

    pub async fn state(&mut self, index: usize) -> Result<AxisState, String> {
        if let Some(axis) = self.axes.by_index(index) {
            let (state, position, temperature) =
                tokio::join!(axis.state(), axis.position(), axis.temperature());

            let is_moving = self
                .is_moving(index)
                .ok_or("Incorrect axis index".to_string());

            Ok(AxisState {
                position: position.map_err(|e| e.to_string()),
                state: state.map_err(|e| e.to_string()),
                temperature: temperature.map_err(|e| e.to_string()),
                is_moving,
            })
        } else {
            Err("Incorrect axis index".to_string())
        }
    }
}
