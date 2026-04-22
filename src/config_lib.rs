use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub accent_r: u8,
    pub accent_b: u8,
    pub accent_g: u8,
    pub main_r: u8,
    pub main_b: u8,
    pub main_g: u8,
    pub text_r: u8,
    pub text_b: u8,
    pub text_g: u8,
    pub scheme: String,
    pub last_browsed: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            accent_r: 40,
            accent_b: 40,
            accent_g: 40,
            main_r: 30,
            main_b: 30,
            main_g: 30,
            text_r: 255,
            text_b: 255,
            text_g: 255,
            scheme: "rust".to_string(),
            last_browsed: "~/".to_string(),
        }
    }
}

fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(user_dirs) = UserDirs::new() {
        let home_dir = user_dirs.home_dir();
        
        let config_dir = home_dir.join(".config").join("gravity");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        return Ok(config_dir.join("settings.json"));
    }

    Err("Could not determine home directory".into())
}

pub fn load_config() -> Result<AppConfig, Box<dyn Error>> {
    let file_path = get_config_path()?;

    if !file_path.exists() {
        let default_settings = AppConfig::default();
        save_config(&default_settings)?;
        return Ok(default_settings);
    }

    let json_content = fs::read_to_string(&file_path)?;

    let settings: AppConfig = match serde_json::from_str(&json_content) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Config error: {}. Resetting.", e);
            let default = AppConfig::default();
            save_config(&default)?;
            default
        }
    };

    Ok(settings)
}

pub fn save_config(settings: &AppConfig) -> Result<(), Box<dyn Error>> {
    let file_path = get_config_path()?;
    let json_string = serde_json::to_string_pretty(settings)?;
    fs::write(file_path, json_string)?;
    Ok(())
}