use std::sync::Arc;

use crate::{
    controller::multi_axis::MultiAxis,
    models::{AxisState, SharedState},
};
use anyhow::Result;
use tokio::sync::Mutex;

static INTERVAL_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

pub async fn run_state_monitor(
    shared_state: Arc<Mutex<SharedState>>,
    multi_axis_controller: Arc<Mutex<MultiAxis>>,
) -> Result<()> {
    let mut interval = tokio::time::interval(INTERVAL_DURATION);

    loop {
        interval.tick().await;

        for axis in 0..4 {
            let mut multi_axis_controller = multi_axis_controller.lock().await;

            let axis_state = match multi_axis_controller.state(axis).await {
                Ok(state) => state,
                Err(e) => AxisState {
                    position: Err(e.to_string()),
                    temperature: Err(e.to_string()),
                    state: Err(e.to_string()),
                    is_moving: Err(e.to_string()),
                },
            };

            let shared_state = shared_state.lock().await;
            if let Some(shared_state) = shared_state.cslit.get_axis_state(axis) {
                let mut shared_state = shared_state.lock().await;
                *shared_state = axis_state;
            }
        }
    }
}
