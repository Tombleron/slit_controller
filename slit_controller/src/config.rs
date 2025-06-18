use std::io;

use serde::{Deserialize, Serialize};

use crate::controller::multi_axis::MultiAxisConfig;

#[derive(Default, Deserialize, Serialize, Debug)]
pub struct Config {
    multi_axis_config: MultiAxisConfig,
}

pub fn load_config() -> io::Result<Config> {
    let config_path = match std::env::var("CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => "default_config.toml".to_string(),
    };

    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "Failed to read config file '{}': {}\nUsing default one",
                config_path, e
            );
            return Ok(Config::default());
        }
    };

    let config: Config = match toml::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to parse config file '{}': {}", config_path, e);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse config file",
            ));
        }
    };

    Ok(config)
}

pub fn save_default_config() -> io::Result<()> {
    let default_config = Config::default();
    let config_path = match std::env::var("CONFIG_PATH") {
        Ok(path) => path,
        Err(_) => "default_config.toml".to_string(),
    };

    let toml_content = toml::to_string(&default_config).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize config: {}", e),
        )
    })?;

    std::fs::write(config_path, toml_content).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to write config file: {}", e),
        )
    })?;

    Ok(())
}
