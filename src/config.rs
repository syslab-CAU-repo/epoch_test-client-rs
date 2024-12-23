use std::{fs, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};

pub struct Config {
    inner: Arc<ConfigInner>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigInner {
    rpc_url: String,
    io_thread: usize,
    request_timeout: u64,
    test_duration: u64,
    accounts: Vec<String>,
    rollup_id: String,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Config {
    pub fn rpc_url(&self) -> &str {
        &self.inner.rpc_url
    }

    pub fn io_thread(&self) -> usize {
        self.inner.io_thread
    }

    pub fn request_timeout(&self) -> u64 {
        self.inner.request_timeout
    }

    pub fn test_duration(&self) -> u64 {
        self.inner.test_duration
    }

    pub fn accounts(&self) -> &Vec<String> {
        &self.inner.accounts
    }

    pub fn rollup_id(&self) -> &str {
        &self.inner.rollup_id
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_string = fs::read_to_string(path).map_err(ConfigError::Open)?;
        let config_inner: ConfigInner =
            toml::from_str(&config_string).map_err(ConfigError::Parse)?;

        Ok(Self {
            inner: config_inner.into(),
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Open(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ConfigError {}
