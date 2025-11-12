use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct AttenuatorAxisConfig {
    pub lir_id: u8,
    pub lir_step: f32,

    pub em2rs_id: u8,
    pub em2rs_low_limit: u8,
    pub em2rs_high_limit: u8,

    pub steps_per_mm: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AttenuatorControllerConfig {
    pub axis: AttenuatorAxisConfig,
}

impl Default for AttenuatorControllerConfig {
    fn default() -> Self {
        Self {
            axis: AttenuatorAxisConfig {
                lir_id: 1,
                lir_step: 0.05,

                em2rs_id: 1,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
        }
    }
}
