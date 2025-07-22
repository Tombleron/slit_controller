use std::time::Duration;

pub mod encoder;
pub mod motor;
pub mod temperature;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
pub const READ_TIMEOUT: Duration = Duration::from_millis(100);
pub const WRITE_TIMEOUT: Duration = Duration::from_millis(100);
