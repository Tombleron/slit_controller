use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct CooledSlitAxisConfig {
    pub lir_id: u8,
    pub lir_step: f32,

    pub em2rs_id: u8,
    pub em2rs_low_limit: u8,
    pub em2rs_high_limit: u8,
    pub steps_per_mm: u32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CooledSlitControllerConfig {
    pub sensors_ip: String,
    pub sensors_port: u16,

    pub em2rs_ip: String,
    pub em2rs_port: u16,

    pub icpcon_id: u8,

    pub upper_axis: CooledSlitAxisConfig,
    pub lower_axis: CooledSlitAxisConfig,
    pub left_axis: CooledSlitAxisConfig,
    pub right_axis: CooledSlitAxisConfig,
}

impl Default for CooledSlitControllerConfig {
    fn default() -> Self {
        Self {
            sensors_ip: "127.0.0.1".to_string(),
            sensors_port: 50051,

            em2rs_ip: "127.0.0.1".to_string(),
            em2rs_port: 50052,

            icpcon_id: 1,

            upper_axis: CooledSlitAxisConfig {
                lir_id: 1,
                lir_step: 0.05,

                em2rs_id: 1,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            lower_axis: CooledSlitAxisConfig {
                lir_id: 2,
                lir_step: 0.05,

                em2rs_id: 2,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            left_axis: CooledSlitAxisConfig {
                lir_id: 3,
                lir_step: 0.05,

                em2rs_id: 3,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            right_axis: CooledSlitAxisConfig {
                lir_id: 4,
                lir_step: 0.05,

                em2rs_id: 4,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
        }
    }
}
