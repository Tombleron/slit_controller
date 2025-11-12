use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct CooledSlitAxisConfig {
    pub lir_id: u8,
    pub lir_step: f32,

    pub knife_trid_axis: u16,
    pub water_trid_axis: u16,

    pub em2rs_id: u8,
    pub em2rs_low_limit: u8,
    pub em2rs_high_limit: u8,

    pub steps_per_mm: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CooledSlitControllerConfig {
    pub knife_trid_id: u8,
    pub water_trid_id: u8,

    pub upper_axis: CooledSlitAxisConfig,
    pub lower_axis: CooledSlitAxisConfig,
    pub left_axis: CooledSlitAxisConfig,
    pub right_axis: CooledSlitAxisConfig,
}

impl Default for CooledSlitControllerConfig {
    fn default() -> Self {
        Self {
            knife_trid_id: 1,
            water_trid_id: 2,

            upper_axis: CooledSlitAxisConfig {
                lir_id: 1,
                lir_step: 0.05,

                knife_trid_axis: 1,
                water_trid_axis: 2,

                em2rs_id: 1,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            lower_axis: CooledSlitAxisConfig {
                lir_id: 2,
                lir_step: 0.05,

                knife_trid_axis: 3,
                water_trid_axis: 4,

                em2rs_id: 2,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            left_axis: CooledSlitAxisConfig {
                lir_id: 3,
                lir_step: 0.05,

                knife_trid_axis: 5,
                water_trid_axis: 6,

                em2rs_id: 3,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
            right_axis: CooledSlitAxisConfig {
                lir_id: 4,
                lir_step: 0.05,

                knife_trid_axis: 7,
                water_trid_axis: 8,

                em2rs_id: 4,
                em2rs_low_limit: 0,
                em2rs_high_limit: 100,
                steps_per_mm: 100,
            },
        }
    }
}
