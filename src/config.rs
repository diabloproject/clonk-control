use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiButton {
    pub name: String,
    pub api_name: String,
    pub requires_input: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub buttons: Vec<ApiButton>,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }
}
