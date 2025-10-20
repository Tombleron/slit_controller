use crate::controller::multi_axis::MultiAxisConfig;
use anyhow::Context as _;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found at {path}")]
    FileNotFound { path: PathBuf },

    #[error("Failed to read configuration file: {source}")]
    ReadError { source: std::io::Error },

    #[error("Failed to parse configuration: {source}")]
    ParseError { source: toml::de::Error },

    #[error("Failed to serialize configuration: {source}")]
    SerializeError { source: toml::ser::Error },

    #[error("Failed to write configuration file: {source}")]
    WriteError { source: std::io::Error },

    #[error("Configuration validation failed: {message}")]
    ValidationError { message: String },
}

#[derive(Debug)]
pub struct ConfigOptions {
    pub config_path: PathBuf,
    pub create_if_missing: bool,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self {
            config_path: Self::default_config_path(),
            create_if_missing: true,
        }
    }
}

impl ConfigOptions {
    pub fn default_config_path() -> PathBuf {
        std::env::var("CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("default_config.toml"))
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            config_path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ConfigManager {
    options: ConfigOptions,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            options: ConfigOptions::default(),
        }
    }

    pub fn with_options(options: ConfigOptions) -> Self {
        Self { options }
    }

    pub fn load(&self) -> anyhow::Result<MultiAxisConfig> {
        let config_path = self.options.config_path.clone();

        if !config_path.exists() {
            if self.options.create_if_missing {
                let default_config = MultiAxisConfig::default();
                self.save(&default_config)
                    .context("Failed to save default config")?;
                return Ok(default_config);
            } else {
                return Err(ConfigError::FileNotFound {
                    path: config_path.clone(),
                }
                .into());
            }
        }

        let content =
            fs::read_to_string(config_path).map_err(|e| ConfigError::ReadError { source: e })?;

        let config: MultiAxisConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError { source: e })?;

        Ok(config)
    }

    pub fn save(&self, config: &MultiAxisConfig) -> anyhow::Result<()> {
        let config_path = &self.options.config_path;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::WriteError { source: e })?;
        }

        // Serialize and write config
        let content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializeError { source: e })?;

        fs::write(config_path, content).map_err(|e| ConfigError::WriteError { source: e })?;

        Ok(())
    }
}

pub fn init_config() -> anyhow::Result<(ConfigManager, MultiAxisConfig)> {
    let manager = ConfigManager::new();
    let config = manager.load()?;
    Ok((manager, config))
}

pub fn init_config_with_options(
    options: ConfigOptions,
) -> anyhow::Result<(ConfigManager, MultiAxisConfig)> {
    let manager = ConfigManager::with_options(options);
    let config = manager.load()?;
    Ok((manager, config))
}

pub fn create_default_config<P: AsRef<Path>>(path: Option<P>) -> anyhow::Result<()> {
    let config_path = path
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(ConfigOptions::default_config_path);

    let options = ConfigOptions {
        config_path,
        create_if_missing: true,
    };

    let manager = ConfigManager::with_options(options);
    let default_config = MultiAxisConfig::default();
    manager.save(&default_config)?;

    Ok(())
}

pub fn load_config() -> anyhow::Result<MultiAxisConfig> {
    let (_manager, config) = init_config()?;
    Ok(config)
}

pub fn save_default_config() -> anyhow::Result<()> {
    create_default_config(None::<PathBuf>)
}
