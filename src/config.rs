use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct PrometheusConfig {
    pub mode: String,
    pub push_url: Option<String>,
    pub listen_address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub api_url: String,
    pub api_secret: String,
    pub groups_to_monitor: Vec<String>,
    pub interval_seconds: u64,
    pub test_url: String,
    pub test_timeout_seconds: u64,
    pub prometheus: PrometheusConfig,
    pub reporter: Option<String>,
}

impl Config {
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}