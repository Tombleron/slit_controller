use std::io;

pub struct Config {}

pub fn load_config() -> io::Result<Config> {
    Ok(Config {})
}
