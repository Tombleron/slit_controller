use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

use slit_controller::{
    communication::run_communication_layer,
    controller_service::{create_controller, run_controller},
    logging,
    models::SharedState,
    state_monitor::run_state_monitor,
};
use tracing::info;

/// Programs works as three separate tasks:
/// 1. Communication layer - handles unix domain socket connections, parses commands and sends them to controller
/// 2. Controller - owns MultiAxis, executes actions like move, stop, etc.
/// 3. State monitor - polls controller for positions and states of axes, sends them to communication layer
///
/// +-----------------------------+
/// | 1. Communication Thread     |  <-- Async, handles client commands
/// | 2. Controller Thread        |  <-- Owns MultiAxis, executes actions
/// | 3. State Monitor Thread     |  <-- Polls controller for positions/states
/// +-----------------------------+
///           Shared via channels

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if we should create a default config file
    if let Ok(create_config) = std::env::var("CREATE_CONFIG") {
        if create_config == "1" || create_config.to_lowercase() == "true" {
            info!("Creating default configuration file...");
            slit_controller::config::save_default_config()?;
            info!("Default configuration saved. Exiting.");
            return Ok(());
        }
    }
    let _config = slit_controller::config::load_config()?;

    logging::init();

    info!("Starting slit controller...");
    let (command_tx, command_rx) = mpsc::channel(100);

    let state = Arc::new(Mutex::new(SharedState {
        axes: [None, None, None, None],
    }));

    let multi_axis_controller = create_controller();

    let multi_axis = multi_axis_controller.clone();
    let controller_handle =
        tokio::spawn(async move { run_controller(command_rx, multi_axis).await });

    let state_clone = state.clone();
    let state_monitor_handle =
        tokio::spawn(async move { run_state_monitor(state_clone, multi_axis_controller).await });

    run_communication_layer(command_tx, state).await?;
    controller_handle.await??;
    state_monitor_handle.await??;

    Ok(())
}
