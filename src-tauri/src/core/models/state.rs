use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use super::preferences::{ChampionPreferences, UserSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueClientInfo {
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub is_running: Arc<Mutex<bool>>,
    pub is_connected: Arc<Mutex<bool>>,
    pub status: Arc<Mutex<String>>,
    pub match_found: Arc<Mutex<bool>>,
    pub client_info: Arc<Mutex<Option<LeagueClientInfo>>>,
    pub accept_delay_seconds: Arc<Mutex<u64>>,
    pub champion_preferences: Arc<Mutex<ChampionPreferences>>,
    pub auto_hover_enabled: Arc<Mutex<bool>>,
    pub auto_select_enabled: Arc<Mutex<bool>>,
    pub auto_ban_enabled: Arc<Mutex<bool>>,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(settings: UserSettings) -> Self {
        Self {
            is_running: Arc::new(Mutex::new(true)), // Auto-accept enabled by default
            is_connected: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Starting...".to_string())),
            match_found: Arc::new(Mutex::new(false)),
            client_info: Arc::new(Mutex::new(None)),
            accept_delay_seconds: Arc::new(Mutex::new(settings.accept_delay_seconds)),
            champion_preferences: Arc::new(Mutex::new(settings.champion_preferences)),
            auto_hover_enabled: Arc::new(Mutex::new(settings.auto_hover_enabled)),
            auto_select_enabled: Arc::new(Mutex::new(settings.auto_select_enabled)),
            auto_ban_enabled: Arc::new(Mutex::new(settings.auto_ban_enabled)),
            http_client: reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
        }
    }
}
