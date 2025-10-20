use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use slit_controller::{
    communication::communication::run_communication_layer,
    config::{create_default_config, init_config},
    controller::{
        controller_service::{create_controller, run_controller},
        state_monitor::run_state_monitor,
    },
    logging,
    models::SharedState,
};
use tracing::info;

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

    info!("Starting slit controller...");
    let (command_tx, command_rx) = mpsc::channel(100);

    let state = Arc::new(Mutex::new(SharedState {
        axes: [None, None, None, None],
    }));

    let (
        mut rf256_command_executor,
        mut trid_command_executor,
        mut upper_standa_command_executor,
        mut lower_standa_command_executor,
        mut right_standa_command_executor,
        mut left_standa_command_executor,
        multi_axis_controller,
    ) = create_controller(config);

    let multi_axis = Arc::new(Mutex::new(multi_axis_controller));
    let multi_axis_clone = Arc::clone(&multi_axis);
    let controller_handle =
        tokio::spawn(async move { run_controller(command_rx, multi_axis_clone).await });

    let state_clone = state.clone();
    let multi_axis_clone = Arc::clone(&multi_axis);
    let state_monitor_handle =
        tokio::spawn(async move { run_state_monitor(state_clone, multi_axis_clone).await });

    let rf256_handle = tokio::task::spawn_blocking(move || rf256_command_executor.run());
    let trid_handle = tokio::task::spawn_blocking(move || trid_command_executor.run());
    let upper_standa_handle =
        tokio::task::spawn_blocking(move || upper_standa_command_executor.run());
    let lower_standa_handle =
        tokio::task::spawn_blocking(move || lower_standa_command_executor.run());
    let right_standa_handle =
        tokio::task::spawn_blocking(move || right_standa_command_executor.run());
    let left_standa_handle =
        tokio::task::spawn_blocking(move || left_standa_command_executor.run());

    run_communication_layer(command_tx, state).await?;
    controller_handle.await??;
    state_monitor_handle.await??;

    rf256_handle.await??;
    trid_handle.await??;
    upper_standa_handle.await??;
    lower_standa_handle.await??;
    right_standa_handle.await??;
    left_standa_handle.await??;

    Ok(())
}
