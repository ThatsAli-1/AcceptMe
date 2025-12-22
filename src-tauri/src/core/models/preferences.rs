use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePreferences {
    pub preferred_champions: Vec<i64>, // Champion IDs in order of preference
    pub auto_ban_champions: Vec<i64>,   // Champion IDs to auto-ban in order
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionPreferences {
    pub top: RolePreferences,
    pub jungle: RolePreferences,
    pub mid: RolePreferences,
    pub adc: RolePreferences,
    pub support: RolePreferences,
}

impl Default for RolePreferences {
    fn default() -> Self {
        Self {
            preferred_champions: Vec::new(),
            auto_ban_champions: Vec::new(),
        }
    }
}

impl Default for ChampionPreferences {
    fn default() -> Self {
        Self {
            top: RolePreferences::default(),
            jungle: RolePreferences::default(),
            mid: RolePreferences::default(),
            adc: RolePreferences::default(),
            support: RolePreferences::default(),
        }
    }
}

// User settings that persist across app restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub champion_preferences: ChampionPreferences,
    pub auto_hover_enabled: bool,
    pub auto_select_enabled: bool,
    pub auto_ban_enabled: bool,
    pub accept_delay_seconds: u64,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            champion_preferences: ChampionPreferences::default(),
            auto_hover_enabled: true,
            auto_select_enabled: true,
            auto_ban_enabled: true,
            accept_delay_seconds: 0,
        }
    }
}
