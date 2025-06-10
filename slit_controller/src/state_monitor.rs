use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::controller::multi_axis::MultiAxis;
use crate::models::{AxisState, SharedState};

pub async fn run_state_monitor(
    shared_state: Arc<Mutex<SharedState>>,
    multi_axis_controller: Arc<Mutex<MultiAxis>>,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        interval.tick().await;

        for axis in 0..4 {
            let mut multi_axis = multi_axis_controller.lock().await;

            let state = multi_axis.state(axis).map_err(|e| e.to_string());

            let position = multi_axis.position(axis).map_err(|e| e.to_string());
            let velocity = multi_axis.get_velocity(axis).map_err(|e| e.to_string());
            let acceleration = multi_axis.get_acceleration(axis).map_err(|e| e.to_string());
            let deceleration = multi_axis.get_deceleration(axis).map_err(|e| e.to_string());
            let position_window = multi_axis
                .get_position_window(axis)
                .map_err(|e| e.to_string());
            let is_moving = Ok(multi_axis.is_moving(axis));

            let axis_state = AxisState {
                position,
                state,
                is_moving,
                velocity,
                acceleration,
                deceleration,
                position_window,
            };

            let mut shared_state = shared_state.lock().await;
            shared_state.axes[axis] = Some(axis_state);
        }
    }
}
