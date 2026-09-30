use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub save_dir: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let save_dir = std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("OneDrive")
            .join("Pictures")
            .join("Screen Captures")
            .to_string_lossy()
            .to_string();

        Self {
            hotkey: "Ctrl+Shift+S".to_string(),
            save_dir,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("settings.json")
}

pub fn load_settings() -> AppSettings {
    let path = get_config_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&data) {
            return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) {
    let path = get_config_path();
    if let Ok(data) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, data);
    }
}
