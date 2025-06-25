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

            let axis_state = match multi_axis.get_axis_state(axis) {
                Ok(state) => state,
                Err(e) => AxisState {
                    position: Err(e.to_string()),
                    temperature: Err(e.to_string()),
                    state: Err(e.to_string()),
                    is_moving: Err(e.to_string()),
                    velocity: Err(e.to_string()),
                    acceleration: Err(e.to_string()),
                    deceleration: Err(e.to_string()),
                    position_window: Err(e.to_string()),
                    time_limit: Err(e.to_string()),
                },
            };

            let mut shared_state = shared_state.lock().await;
            shared_state.axes[axis] = Some(axis_state);
            dbg!(&shared_state.axes[axis]);
        }
    }
}
