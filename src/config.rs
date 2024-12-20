use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    rpc_url: String,
    io_thread: usize,
    request_timeout: u64,
    test_duration: u64,
    accounts: Vec<String>,
    rollup_id: String,
}

impl Config {
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub fn io_thread(&self) -> usize {
        self.io_thread
    }

    pub fn request_timeout(&self) -> u64 {
        self.request_timeout
    }

    pub fn test_duration(&self) -> u64 {
        self.test_duration
    }

    pub fn accounts(&self) -> &Vec<String> {
        &self.accounts
    }

    pub fn rollup_id(&self) -> &str {
        &self.rollup_id
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_string = fs::read_to_string(path).map_err(ConfigError::Open)?;
        let config: Self = toml::from_str(&config_string).map_err(ConfigError::Parse)?;

        Ok(config)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Open(std::io::Error),
    Parse(toml::de::Error),
}
