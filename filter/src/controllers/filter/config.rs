use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct FilterControllerConfig {
    pub encoder_ip: String,
    pub encoder_port: u16,

    pub em2rs_ip: String,
    pub em2rs_port: u16,

    pub lir_id: u8,
    pub lir_step: f32,

    pub em2rs_id: u8,
    pub em2rs_low_limit: u8,
    pub em2rs_high_limit: u8,
    pub steps_per_mm: i32,
}

impl Default for FilterControllerConfig {
    fn default() -> Self {
        Self {
            encoder_ip: "127.0.0.1".to_string(),
            encoder_port: 5000,

            em2rs_ip: "127.0.0.1".to_string(),
            em2rs_port: 5001,

            lir_id: 1,
            lir_step: 0.1,

            em2rs_id: 2,
            em2rs_low_limit: 0,
            em2rs_high_limit: 100,
            steps_per_mm: 100,
        }
    }
}
