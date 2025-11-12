use serde::{Deserialize, Serialize};

use crate::controllers::{
    attenuator::config::AttenuatorControllerConfig, collimator::config::CollimatorControllerConfig,
    water_input::config::WaterInputControllerConfig,
};

use super::cooled_slit::config::CooledSlitControllerConfig;

#[derive(Deserialize, Debug, Serialize)]
pub struct XafsConfig {
    pub sensors_ip: String,
    pub sensors_port: u16,

    pub em2rs_ip: String,
    pub em2rs_port: u16,

    pub slit_controller: CooledSlitControllerConfig,
    pub attenuator: AttenuatorControllerConfig,
    pub collimator: CollimatorControllerConfig,
    pub water_input: WaterInputControllerConfig,
}

impl Default for XafsConfig {
    fn default() -> Self {
        Self {
            sensors_ip: "127.0.0.1".to_string(),
            sensors_port: 50051,

            em2rs_ip: "127.0.0.1".to_string(),
            em2rs_port: 50052,

            slit_controller: CooledSlitControllerConfig::default(),
            attenuator: AttenuatorControllerConfig::default(),
            collimator: CollimatorControllerConfig::default(),
            water_input: WaterInputControllerConfig::default(),
        }
    }
}
