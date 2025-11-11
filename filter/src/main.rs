pub mod command_executor;
pub mod config;
pub mod controllers;
pub mod logging;

use crate::{
    config::{create_default_config, init_config},
    controllers::filter::create_controller,
};

use motarem::{
    controller_manager::{ControllerManager, config::ManagerConfig},
    motor_controller::MotorController,
    socket_server::{SocketServer, config::SocketServerConfig},
};
use std::{path::PathBuf, sync::Arc, time::Duration};

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

    let controller = create_controller(&config);

    let manager_config = ManagerConfig {
        default_ttl: Duration::from_secs(1),
        cache_capacity: 1000,
    };

    let manager = Arc::new(ControllerManager::new(manager_config));

    manager
        .register_controller(controller.name().to_string(), Arc::new(controller))
        .await?;

    let socket_config = SocketServerConfig {
        socket_path: "/tmp/filter_controller.sock".to_string(),
        max_connections: 50,
        buffer_size: 8192,
    };

    let mut socket_server = SocketServer::new(socket_config, manager.clone());
    socket_server.start().await?;

    loop {}

    // Ok(())
}
