use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration — persisted as JSON at:
/// `%APPDATA%\SnapAndSearch\config.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Global hotkey string, e.g. "Ctrl+Shift+S"
    pub hotkey: String,

    /// Folder where screenshots are saved.
    /// Defaults to `.\screenshots` next to the exe.
    pub screenshots_dir: String,

    /// If true, the service writes itself to the Windows startup registry key.
    pub launch_on_startup: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+S".to_string(),
            screenshots_dir: default_screenshots_dir(),
            launch_on_startup: false,
        }
    }
}

impl Config {
    /// Load config from disk. Creates the default config file if it does not exist.
    pub fn load() -> Self {
        let path = config_path();

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(text) => match serde_json::from_str(&text) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("Config parse error (using defaults): {}", e),
                },
                Err(e) => eprintln!("Config read error (using defaults): {}", e),
            }
        }

        // Write defaults so the user can edit the file
        let default = Config::default();
        default.save();
        default
    }

    /// Persist the current config to disk.
    pub fn save(&self) {
        let path = config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&path, text) {
                    eprintln!("Failed to save config: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize config: {}", e),
        }
    }

    /// Return the screenshots directory as a PathBuf, creating it if needed.
    pub fn screenshots_path(&self) -> PathBuf {
        let p = PathBuf::from(&self.screenshots_dir);
        let _ = std::fs::create_dir_all(&p);
        p
    }
}

/// Returns `%APPDATA%\SnapAndSearch\config.json`
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("SnapAndSearch")
        .join("config.json")
}

/// Returns a sensible default screenshots folder next to the running exe.
fn default_screenshots_dir() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("screenshots")))
        .unwrap_or_else(|| PathBuf::from("screenshots"))
        .display()
        .to_string()
}
