use std::fs;
use std::path::PathBuf;
use crate::core::models::UserSettings;

// Get the path to the settings file
pub fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AcceptMe");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

// Load settings from file
pub fn load_settings() -> UserSettings {
    let path = get_settings_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }
    }
    UserSettings::default()
}

// Save settings to file
pub fn save_settings(settings: &UserSettings) {
    let path = get_settings_path();
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        fs::write(&path, json).ok();
    }
}
