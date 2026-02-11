use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub accent_r: u8,
    pub accent_b: u8,
    pub accent_g: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            accent_r: 40,
            accent_b: 40,
            accent_g: 40,
        }
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn Error>> {
    let file_path = Path::new("settings.json");

    if !file_path.exists() {
        let default_settings = AppConfig::default();
        let json_string = serde_json::to_string_pretty(&default_settings)?;
        fs::write(file_path, json_string)?;
        return Ok(default_settings);
    }

    let json_content = fs::read_to_string(file_path)?;
    let settings: AppConfig = serde_json::from_str(&json_content)?;

    Ok(settings)
}

// Renamed settings_write -> save_config
pub fn save_config(settings: &AppConfig) -> Result<(), Box<dyn Error>> {
    let file_path = Path::new("settings.json");
    let json_string = serde_json::to_string_pretty(settings)?;
    fs::write(file_path, json_string)?;
    Ok(())
}