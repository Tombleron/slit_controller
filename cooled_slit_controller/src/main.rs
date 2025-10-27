use tokio::sync::{Mutex, mpsc};

use crate::{
    communication::communication::run_communication_layer,
    config::{create_default_config, init_config},
    controller::{
        controller_service::{create_controller, run_controller},
        state_monitor::run_state_monitor,
    },
    models::SharedState,
};
use std::{path::PathBuf, sync::Arc};
mod command_executor;
pub mod communication;
pub mod config;
mod controller;
pub mod logging;
pub mod models;

fn should_create_config() -> bool {
    std::env::var("CREATE_CONFIG")
        .map(|val| val == "1" || val.to_lowercase() == "true")
        .unwrap_or(false)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logging::init();

    if should_create_config() {
        create_default_config(None::<PathBuf>)?;
    }

    let (_config_manager, config) = init_config().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("Run with CREATE_CONFIG=1 to create a default configuration file.");
        e
    })?;

    let (command_tx, command_rx) = mpsc::channel(100);

    let state = Arc::new(Mutex::new(SharedState {
        axes: [None, None, None, None],
    }));

    let (mut sensors_command_executor, mut em2rs_command_executor, multi_axis_controller) =
        create_controller(config);

    let multi_axis = Arc::new(Mutex::new(multi_axis_controller));
    let multi_axis_clone = Arc::clone(&multi_axis);
    let controller_handle =
        tokio::spawn(async move { run_controller(command_rx, multi_axis_clone).await });

    let state_clone = state.clone();
    let multi_axis_clone = Arc::clone(&multi_axis);
    let state_monitor_handle =
        tokio::spawn(async move { run_state_monitor(state_clone, multi_axis_clone).await });

    let sensors_handle = tokio::task::spawn_blocking(move || sensors_command_executor.run());
    let em2rs_handle = tokio::task::spawn_blocking(move || em2rs_command_executor.run());

    run_communication_layer(command_tx, state).await?;
    controller_handle.await??;
    state_monitor_handle.await??;

    sensors_handle.await??;
    em2rs_handle.await??;

    Ok(())
}
