use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct SlitAxisConfig {
    pub rf256_id: u8,
    pub trid_id: u16,

    pub standa_ip: String,
    pub standa_port: u16,

    pub steps_per_mm: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct SlitControllerConfig {
    pub rf256_ip: String,
    pub rf256_port: u16,

    pub trid_ip: String,
    pub trid_port: u16,
    pub trid_device_id: u8,

    pub upper_axis: SlitAxisConfig,
    pub lower_axis: SlitAxisConfig,
    pub left_axis: SlitAxisConfig,
    pub right_axis: SlitAxisConfig,
}

impl Default for SlitControllerConfig {
    fn default() -> Self {
        Self {
            rf256_ip: String::from("192.168.1.1"),
            rf256_port: 502,

            trid_ip: String::from("192.168.1.2"),
            trid_port: 502,
            trid_device_id: 1,

            upper_axis: SlitAxisConfig {
                rf256_id: 1,
                trid_id: 1,
                standa_ip: String::from("192.168.1.3"),
                standa_port: 502,
                steps_per_mm: 800,
            },
            lower_axis: SlitAxisConfig {
                rf256_id: 2,
                trid_id: 2,
                standa_ip: String::from("192.168.1.4"),
                standa_port: 502,
                steps_per_mm: 800,
            },
            left_axis: SlitAxisConfig {
                rf256_id: 3,
                trid_id: 3,
                standa_ip: String::from("192.168.1.5"),
                standa_port: 502,
                steps_per_mm: 800,
            },
            right_axis: SlitAxisConfig {
                rf256_id: 4,
                trid_id: 4,
                standa_ip: String::from("192.168.1.6"),
                standa_port: 502,
                steps_per_mm: 800,
            },
        }
    }
}
