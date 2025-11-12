use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct CollimatorConfig {
    pub trid_axis: u16,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CollimatorControllerConfig {
    pub trid_id: u8,
    pub input_axis: CollimatorConfig,
    pub output_axis: CollimatorConfig,
}

impl Default for CollimatorControllerConfig {
    fn default() -> Self {
        Self {
            trid_id: 1,
            input_axis: CollimatorConfig { trid_axis: 1 },
            output_axis: CollimatorConfig { trid_axis: 2 },
        }
    }
}
