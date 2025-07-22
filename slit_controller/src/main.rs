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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Ok(create_config) = std::env::var("CREATE_CONFIG") {
        if create_config == "1" || create_config.to_lowercase() == "true" {
            info!("Creating default configuration file...");
            slit_controller::config::save_default_config()?;
            info!("Default configuration saved. Exiting.");
            return Ok(());
        }
    }
    let config = slit_controller::config::load_config()?;

    logging::init();

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
    ) = create_controller(config.multi_axis_config);

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
