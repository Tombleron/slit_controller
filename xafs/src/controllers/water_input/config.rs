use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct WaterInputConfig {
    pub trid_axis: u16,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct WaterInputControllerConfig {
    pub trid_id: u8,
    pub axis: WaterInputConfig,
}

impl Default for WaterInputControllerConfig {
    fn default() -> Self {
        Self {
            trid_id: 1,
            axis: WaterInputConfig { trid_axis: 1 },
        }
    }
}
