use std::io::{self};

use serde::{Deserialize, Serialize};
use standa::command::state::StateParams;
use tracing::{debug, error};

use crate::command_executor::encoder::command_sender::Rf256CommandSender;
use crate::command_executor::temperature::command_sender::TridCommandSender;
use crate::controller::single_axis::SingleAxis;
use crate::models::AxisState;

use super::single_axis::MovementParams;

pub struct MultiAxis {
    axes: [SingleAxis; 4],

    rf256_cs: Rf256CommandSender,
    trid_cs: TridCommandSender,
}

impl MultiAxis {
    pub fn new(
        axes: [SingleAxis; 4],
        rf256_cs: Rf256CommandSender,
        trid_cs: TridCommandSender,
    ) -> Self {
        Self {
            rf256_cs,
            trid_cs,
            axes,
        }
    }

    async fn reconnect_rf256_client(&mut self) -> io::Result<()> {
        self.rf256_cs.reconnect().await
    }

    async fn reconnect_trid_client(&mut self) -> io::Result<()> {
        self.trid_cs.reconnect().await
    }

    async fn reconnect_standa_client(&mut self, index: usize) -> io::Result<()> {
        self.axes[index].reconnect_standa_client().await
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

        if position < 6.5 || position > 14.0 {
            error!("Move target is out of bounds: {position}");
            return Err("Position is out of bounds, (6.5, 14)".to_string());
        }

        let result = self.axes[index].move_to_position(position, params).await;

        result
    }

    pub async fn stop(&mut self, index: usize) -> Result<(), String> {
        self.axes[index].stop().await.map_err(|e| {
            error!("Failed to stop axis {}: {}", index, e);
            e.to_string()
        })
    }

    pub async fn position(&mut self, index: usize) -> io::Result<f32> {
        let result = self.axes[index].position_with_retries(5).await;

        if let Err(ref e) = result {
            if e.kind() == io::ErrorKind::BrokenPipe {
                self.reconnect_rf256_client().await?;
            }
        } else if let Ok(pos) = result {
            debug!("Got position {} for axis {}", pos, index);
        }
        result
    }

    pub async fn temperature(&mut self, index: usize) -> io::Result<f32> {
        let result = self.axes[index].temperature().await;

        if let Err(ref e) = result {
            if e.kind() == io::ErrorKind::BrokenPipe || e.kind() == io::ErrorKind::WouldBlock {
                self.reconnect_trid_client().await?;
            }
        } else {
            debug!("Successfully got temperature for axis {}", index);
        }
        result
    }

    pub async fn state(&mut self, index: usize) -> io::Result<StateParams> {
        let result = self.axes[index].state().await;

        if let Err(ref e) = result {
            if e.kind() == io::ErrorKind::BrokenPipe || e.kind() == io::ErrorKind::WouldBlock {
                self.reconnect_standa_client(index).await?;
            }
        } else {
            debug!("Successfully got state for axis {}", index);
        }
        result
    }

    pub async fn get_axis_state(&mut self, index: usize) -> io::Result<AxisState> {
        self.axes[index].get_axis_state().await
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MultiAxisConfig {
    pub rf256_ip: String,
    pub rf256_port: u16,

    pub trid_ip: String,
    pub trid_port: u16,

    pub upper_standa_ip: String,
    pub upper_standa_port: u16,
    pub upper_rf256_id: u8,
    pub upper_trid_id: u8,

    pub lower_standa_ip: String,
    pub lower_standa_port: u16,
    pub lower_rf256_id: u8,
    pub lower_trid_id: u8,

    pub left_standa_ip: String,
    pub left_standa_port: u16,
    pub left_rf256_id: u8,
    pub left_trid_id: u8,

    pub right_standa_ip: String,
    pub right_standa_port: u16,
    pub right_rf256_id: u8,
    pub right_trid_id: u8,
}

impl Default for MultiAxisConfig {
    fn default() -> Self {
        Self {
            rf256_ip: "192.168.0.51".to_string(),
            rf256_port: 60003,

            trid_ip: "192.168.0.51".to_string(),
            trid_port: 60002,

            upper_standa_ip: "192.168.0.204".to_string(),
            upper_standa_port: 2000,
            upper_rf256_id: 6,
            upper_trid_id: 0,

            lower_standa_ip: "192.168.0.204".to_string(),
            lower_standa_port: 3000,
            lower_rf256_id: 15,
            lower_trid_id: 1,

            left_standa_ip: "192.168.0.205".to_string(),
            left_standa_port: 2000,
            left_rf256_id: 5,
            left_trid_id: 2,

            right_standa_ip: "192.168.0.205".to_string(),
            right_standa_port: 3000,
            right_rf256_id: 4,
            right_trid_id: 3,
        }
    }
}
