use crate::{
    config::{create_default_config, init_config},
    controllers::create_controllers,
};

use motarem::{
    controller_manager::{ControllerManager, config::ManagerConfig},
    motor_controller::MotorController,
    socket_server::{SocketServer, config::SocketServerConfig},
};
use std::{path::PathBuf, sync::Arc, time::Duration};

pub mod command_executor;
pub mod config;
pub mod controllers;
pub mod logging;

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

    let (
        collimator,
        cooled_slit,
        attenuator,
        water_input,
        mut em2rs_command_executor,
        mut sensors_command_executor,
    ) = create_controllers(&config);

    let manager_config = ManagerConfig {
        default_ttl: Duration::from_secs(1),
        cache_capacity: 1000,
    };

    let manager = Arc::new(ControllerManager::new(manager_config));

    manager
        .register_controller(collimator.name().to_string(), Arc::new(collimator))
        .await?;
    manager
        .register_controller(cooled_slit.name().to_string(), Arc::new(cooled_slit))
        .await?;
    manager
        .register_controller(attenuator.name().to_string(), Arc::new(attenuator))
        .await?;
    manager
        .register_controller(water_input.name().to_string(), Arc::new(water_input))
        .await?;

    let socket_config = SocketServerConfig {
        socket_path: "/tmp/xafs_controller.sock".to_string(),
        max_connections: 50,
        buffer_size: 8192,
    };

    let sensors_handle = tokio::task::spawn_blocking(move || sensors_command_executor.run());
    let em2rs_handle = tokio::task::spawn_blocking(move || em2rs_command_executor.run());

    let mut socket_server = SocketServer::new(socket_config, manager.clone());
    socket_server.start().await?;

    let _sensors_handle = sensors_handle.await?;
    let _em2rs_handle = em2rs_handle.await?;

    loop {}
}
