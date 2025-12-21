// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeagueClientInfo {
    port: u16,
    password: String,
    protocol: String,
}

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

// Get the path to the settings file
fn get_settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("AcceptMe");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

// Load settings from file
fn load_settings() -> UserSettings {
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
fn save_settings(settings: &UserSettings) {
    let path = get_settings_path();
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        fs::write(&path, json).ok();
    }
}

#[derive(Debug, Clone)]
struct AppState {
    is_running: Arc<Mutex<bool>>,
    is_connected: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    match_found: Arc<Mutex<bool>>,
    client_info: Arc<Mutex<Option<LeagueClientInfo>>>,
    accept_delay_seconds: Arc<Mutex<u64>>,
    champion_preferences: Arc<Mutex<ChampionPreferences>>,
    auto_hover_enabled: Arc<Mutex<bool>>,
    auto_select_enabled: Arc<Mutex<bool>>,
    auto_ban_enabled: Arc<Mutex<bool>>,
    http_client: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("failed to build http client");

        // Load saved settings from file
        let saved_settings = load_settings();

        Self {
            is_running: Arc::new(Mutex::new(true)),
            is_connected: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Initializing...".to_string())),
            match_found: Arc::new(Mutex::new(false)),
            client_info: Arc::new(Mutex::new(None)),
            accept_delay_seconds: Arc::new(Mutex::new(saved_settings.accept_delay_seconds)),
            champion_preferences: Arc::new(Mutex::new(saved_settings.champion_preferences)),
            auto_hover_enabled: Arc::new(Mutex::new(saved_settings.auto_hover_enabled)),
            auto_select_enabled: Arc::new(Mutex::new(saved_settings.auto_select_enabled)),
            auto_ban_enabled: Arc::new(Mutex::new(saved_settings.auto_ban_enabled)),
            http_client,
        }
    }
}

// Get League client info; cached, only hits disk when missing
async fn get_or_read_client_info(state: &AppState) -> Option<LeagueClientInfo> {
    if let Some(info) = state.client_info.lock().await.clone() {
        return Some(info);
    }

    // Try common League of Legends installation paths
    let possible_paths = vec![
        // Most common: Installation directory (C:\Riot Games\League of Legends\)
        r"C:\Riot Games\League of Legends\lockfile".to_string(),
        // Alternative installation locations
        format!(
            r"{}\Riot Games\League of Legends\lockfile",
            std::env::var("PROGRAMFILES").unwrap_or_default()
        ),
        format!(
            r"{}\Riot Games\League of Legends\lockfile",
            std::env::var("PROGRAMFILES(X86)").unwrap_or_default()
        ),
        // Sometimes in AppData (less common)
        format!(
            r"{}\Riot Games\League of Legends\lockfile",
            std::env::var("LOCALAPPDATA").unwrap_or_default()
        ),
        // User-specific installation
        format!(
            r"{}\Riot Games\League of Legends\lockfile",
            std::env::var("USERPROFILE").unwrap_or_default()
        ),
    ];

    for path_str in possible_paths {
        let path = std::path::PathBuf::from(&path_str);
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                // Lockfile format: name:pid:port:password:protocol
                let parts: Vec<&str> = contents.split(':').collect();
                if parts.len() >= 5 {
                    if let Ok(port) = parts[2].parse::<u16>() {
                        let info = LeagueClientInfo {
                            port,
                            password: parts[3].to_string(),
                            protocol: parts[4].trim().to_string(),
                        };
                        *state.client_info.lock().await = Some(info.clone());
                        return Some(info);
                    }
                }
            }
        }
    }
    None
}

// Check if League client is running (internal helper)
async fn check_league_connection_internal(state: &AppState) -> bool {
    if let Some(client_info) = get_or_read_client_info(state).await {
        let url = format!(
            "{}://127.0.0.1:{}/lol-summoner/v1/current-summoner",
            client_info.protocol, client_info.port
        );

        let response = state
            .http_client
            .get(&url)
            .basic_auth("riot", Some(&client_info.password))
            .send()
            .await;

        if response.is_ok() {
            *state.is_connected.lock().await = true;
            return true;
        }
        // Connection failed with cached info; clear so we re-read lockfile next iteration
        *state.client_info.lock().await = None;
    }

    *state.is_connected.lock().await = false;
    false
}

// Accept match
async fn accept_match(state: &AppState) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        let url = format!(
            "{}://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept",
            info.protocol, info.port
        );

        let response = state
            .http_client
            .post(&url)
            .basic_auth("riot", Some(&info.password))
            .send()
            .await;

        return response.is_ok();
    }
    false
}

// Check for match found
async fn check_match_found(state: &AppState) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        let url = format!(
            "{}://127.0.0.1:{}/lol-matchmaking/v1/ready-check",
            info.protocol, info.port
        );

        if let Ok(response) = state
            .http_client
            .get(&url)
            .basic_auth("riot", Some(&info.password))
            .send()
            .await
        {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(state_str) = json.get("state").and_then(|v| v.as_str()) {
                    return state_str == "InProgress";
                }
            }
        }
    }
    false
}

// Get current champion select session
async fn get_champion_select_session(state: &AppState) -> Option<serde_json::Value> {
    if let Some(info) = get_or_read_client_info(state).await {
        let url = format!(
            "{}://127.0.0.1:{}/lol-champ-select/v1/session",
            info.protocol, info.port
        );

        if let Ok(response) = state
            .http_client
            .get(&url)
            .basic_auth("riot", Some(&info.password))
            .send()
            .await
        {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                return Some(json);
            }
        }
    }
    None
}

// Get current summoner's cell ID in champion select
async fn get_my_cell_id(state: &AppState) -> Option<i64> {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(local_player_cell_id) = session.get("localPlayerCellId").and_then(|v| v.as_i64()) {
            return Some(local_player_cell_id);
        }
    }
    None
}

// Get current role/position - directly from myTeam
async fn get_current_role(state: &AppState) -> Option<String> {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(my_cell_id) = get_my_cell_id(state).await {
            // Get position directly from myTeam
            if let Some(my_team) = session.get("myTeam").and_then(|v| v.as_array()) {
                for player in my_team {
                    if let Some(cell_id) = player.get("cellId").and_then(|v| v.as_i64()) {
                                if cell_id == my_cell_id {
                            if let Some(position) = player.get("assignedPosition").and_then(|v| v.as_str()) {
                                if !position.is_empty() {
                                                return Some(position.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// Hover champion
// RELAXED: Check completed == false instead of isInProgress == true
async fn hover_champion(state: &AppState, champion_id: i64) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                let my_cell_id = get_my_cell_id(state).await;
                
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            // Check if this is a pick action
                            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if action_type != "pick" {
                                continue;
                            }
                            
                            // Check if it's our action
                            let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                            if actor_cell_id != my_cell_id {
                                continue;
                            }
                            
                            // RELAXED: Check completed == false instead of isInProgress == true
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            if completed {
                                continue; // Already completed, skip
                            }
                            
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                            let patch_url = format!(
                                                "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                info.protocol, info.port, id
                                            );
                                            
                                            let payload = serde_json::json!({
                                    "championId": champion_id
                                            });

                                            let response = state
                                                .http_client
                                                .patch(&patch_url)
                                                .basic_auth("riot", Some(&info.password))
                                                .json(&payload)
                                                .send()
                                                .await;

                                            return response.is_ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
    false
}

// Just lock in the currently selected champion (without changing it)
// RELAXED: Don't require isInProgress - just check completed == false
async fn lock_current_champion(state: &AppState) -> bool {
    eprintln!("[DEBUG] lock_current_champion() called");
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                let my_cell_id = get_my_cell_id(state).await;
                
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if action_type != "pick" {
                                continue;
                            }
                            
                            let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                            if actor_cell_id != my_cell_id {
                                continue;
                            }
                            
                            // RELAXED: Check completed == false instead of isInProgress == true
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            if completed {
                                continue; // Already locked, skip
                            }
                            
                            // Check if a champion is selected (championId > 0)
                            let current_champ = action.get("championId").and_then(|v| v.as_i64()).unwrap_or(0);
                            eprintln!("[DEBUG] lock_current_champion: Found our pick action, completed=false, championId={}", current_champ);
                            
                            if current_champ <= 0 {
                                eprintln!("[DEBUG] lock_current_champion: No champion selected yet");
                                return false; // No champion selected, can't lock
                            }
                            
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                let patch_url = format!(
                                    "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                    info.protocol, info.port, id
                                );
                                
                                eprintln!("[DEBUG] lock_current_champion: Sending lock request for action {}", id);
                                
                                // Just lock - don't change the champion
                                let complete_payload = serde_json::json!({
                                    "completed": true
                                });

                                let result = state
                                    .http_client
                                    .patch(&patch_url)
                                    .basic_auth("riot", Some(&info.password))
                                    .json(&complete_payload)
                                    .send()
                                    .await;

                                if let Ok(response) = result {
                                    let success = response.status().is_success();
                                    eprintln!("[DEBUG] lock_current_champion: API response success={}", success);
                                    return success;
                                }
                                eprintln!("[DEBUG] lock_current_champion: API request failed");
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("[DEBUG] lock_current_champion: No valid pick action found");
    false
}

// Select champion (lock in) - two step: first hover, then lock
async fn select_champion(state: &AppState, champion_id: i64) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                let my_cell_id = get_my_cell_id(state).await;
                
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            // Check if this is a pick action
                            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if action_type != "pick" {
                                continue;
                            }
                            
                            // Check if it's our action
                            let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                            if actor_cell_id != my_cell_id {
                                continue;
                            }
                            
                            // Check if action is in progress (our turn)
                            let in_progress = action.get("isInProgress").and_then(|v| v.as_bool()).unwrap_or(false);
                            if !in_progress {
                                continue;
                            }
                            
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                                    let patch_url = format!(
                                                        "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                        info.protocol, info.port, id
                                                    );
                                                    
                                // Step 1: Hover the champion first
                                let hover_payload = serde_json::json!({
                                    "championId": champion_id
                                                    });

                                let hover_result = state
                                                        .http_client
                                                        .patch(&patch_url)
                                                        .basic_auth("riot", Some(&info.password))
                                    .json(&hover_payload)
                                                        .send()
                                                        .await;

                                if hover_result.is_err() {
                                    return false;
                                }
                                
                                // Small delay to ensure the hover is registered
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                
                                // Step 2: Lock in the champion
                                let complete_payload = serde_json::json!({
                                    "completed": true
                                });

                                let complete_result = state
                                    .http_client
                                    .patch(&patch_url)
                                    .basic_auth("riot", Some(&info.password))
                                    .json(&complete_payload)
                                    .send()
                                    .await;

                                if let Ok(response) = complete_result {
                                    return response.status().is_success();
                                }
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// Ban champion - two step: first select, then complete
// RELAXED: Check completed == false instead of isInProgress == true
async fn ban_champion(state: &AppState, champion_id: i64) -> bool {
    eprintln!("[DEBUG] ban_champion() called for champion {}", champion_id);
    
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                let my_cell_id = get_my_cell_id(state).await;
                eprintln!("[DEBUG] ban_champion: my_cell_id={:?}", my_cell_id);
                
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            // Check if this is a ban action
                            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if action_type != "ban" {
                                continue;
                            }
                            
                            // Check if it's our action
                            let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                            if actor_cell_id != my_cell_id {
                                continue;
                            }
                            
                            // RELAXED: Check completed == false instead of isInProgress == true
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            eprintln!("[DEBUG] ban_champion: Found our ban action, completed={}", completed);
                            if completed {
                                continue; // Already banned, skip
                            }
                            
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                eprintln!("[DEBUG] ban_champion: Action ID={}, sending ban for champ {}", id, champion_id);
                                                    let patch_url = format!(
                                                        "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                        info.protocol, info.port, id
                                                    );
                                                    
                                // Step 1: Set the champion to ban (hover it)
                                let hover_payload = serde_json::json!({
                                    "championId": champion_id
                                                    });

                                let hover_result = state
                                                        .http_client
                                                        .patch(&patch_url)
                                                        .basic_auth("riot", Some(&info.password))
                                    .json(&hover_payload)
                                                        .send()
                                                        .await;

                                if hover_result.is_err() {
                                    return false;
                                }
                                
                                // Small delay to ensure the hover is registered
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                
                                // Step 2: Complete the ban action
                                let complete_payload = serde_json::json!({
                                    "completed": true
                                });

                                let complete_result = state
                                    .http_client
                                    .patch(&patch_url)
                                    .basic_auth("riot", Some(&info.password))
                                    .json(&complete_payload)
                                    .send()
                                    .await;

                                if let Ok(response) = complete_result {
                                    return response.status().is_success();
                                                }
                                return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
    false
}

// Check if in champion select
async fn is_in_champion_select(state: &AppState) -> bool {
    if let Some(session) = get_champion_select_session(state).await {
        // If we got a valid session response, we're in champion select
        // Check for localPlayerCellId which is always present in champ select
        if session.get("localPlayerCellId").is_some() {
            return true;
        }
    }
    false
}

// Get list of banned champion IDs from the current session
async fn get_banned_champions(state: &AppState) -> Vec<i64> {
    let mut banned = Vec::new();
    if let Some(session) = get_champion_select_session(state).await {
        // Check bans array
        if let Some(bans) = session.get("bans").and_then(|v| v.as_object()) {
            // My team bans
            if let Some(my_team) = bans.get("myTeamBans").and_then(|v| v.as_array()) {
                for ban in my_team {
                    if let Some(id) = ban.as_i64() {
                        if id > 0 {
                            banned.push(id);
                        }
                    }
                }
            }
            // Their team bans
            if let Some(their_team) = bans.get("theirTeamBans").and_then(|v| v.as_array()) {
                for ban in their_team {
                    if let Some(id) = ban.as_i64() {
                        if id > 0 {
                            banned.push(id);
                        }
                    }
                }
            }
        }
        
        // Also check actions for completed bans
        if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
            for action_array in actions {
                if let Some(action_array) = action_array.as_array() {
                    for action in action_array {
                        if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                            if action_type == "ban" {
                                if let Some(completed) = action.get("completed").and_then(|v| v.as_bool()) {
                                    if completed {
                                        if let Some(champ_id) = action.get("championId").and_then(|v| v.as_i64()) {
                                            if champ_id > 0 && !banned.contains(&champ_id) {
                                                banned.push(champ_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    banned
}

// Get list of picked champion IDs from the current session (by other players)
async fn get_picked_champions(state: &AppState) -> Vec<i64> {
    let mut picked = Vec::new();
    let my_cell_id = get_my_cell_id(state).await;
    
    if let Some(session) = get_champion_select_session(state).await {
        // Check myTeam picks
        if let Some(my_team) = session.get("myTeam").and_then(|v| v.as_array()) {
            for player in my_team {
                if let Some(cell_id) = player.get("cellId").and_then(|v| v.as_i64()) {
                    // Skip our own pick
                    if my_cell_id == Some(cell_id) {
                        continue;
                    }
                    if let Some(champ_id) = player.get("championId").and_then(|v| v.as_i64()) {
                        if champ_id > 0 && !picked.contains(&champ_id) {
                            picked.push(champ_id);
                        }
                    }
                }
            }
        }
        
        // Check theirTeam picks  
        if let Some(their_team) = session.get("theirTeam").and_then(|v| v.as_array()) {
            for player in their_team {
                if let Some(champ_id) = player.get("championId").and_then(|v| v.as_i64()) {
                    if champ_id > 0 && !picked.contains(&champ_id) {
                        picked.push(champ_id);
                    }
                }
            }
        }
    }
    picked
}

// Check if champion is available (not banned and not picked)
async fn is_champion_available(state: &AppState, champion_id: i64) -> bool {
    let banned = get_banned_champions(state).await;
    let picked = get_picked_champions(state).await;
    !banned.contains(&champion_id) && !picked.contains(&champion_id)
}

// Check if it's currently our turn to pick
async fn is_my_pick_turn(state: &AppState) -> bool {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
            let my_cell_id = get_my_cell_id(state).await;
            for action_array in actions {
                if let Some(action_array) = action_array.as_array() {
                    for action in action_array {
                        if let Some(in_progress) = action.get("isInProgress").and_then(|v| v.as_bool()) {
                            if in_progress {
                                if let Some(actor_cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                    if my_cell_id == Some(actor_cell_id) {
                                        if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                                            return action_type == "pick";
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// Check if it's currently our turn to ban (strict - uses isInProgress)
async fn is_my_ban_turn(state: &AppState) -> bool {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
            let my_cell_id = get_my_cell_id(state).await;
            for action_array in actions {
                if let Some(action_array) = action_array.as_array() {
                    for action in action_array {
                        if let Some(in_progress) = action.get("isInProgress").and_then(|v| v.as_bool()) {
                            if in_progress {
                                if let Some(actor_cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                    if my_cell_id == Some(actor_cell_id) {
                                        if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                                            return action_type == "ban";
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

// RELAXED: Check if we have a pending ban action (completed == false, not requiring isInProgress)
async fn has_pending_ban_action(state: &AppState) -> bool {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
            let my_cell_id = get_my_cell_id(state).await;
            for action_array in actions {
                if let Some(action_array) = action_array.as_array() {
                    for action in action_array {
                        let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if action_type != "ban" {
                            continue;
                        }
                        
                        let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                        if actor_cell_id != my_cell_id {
                            continue;
                        }
                        
                        // RELAXED: Check completed == false instead of isInProgress == true
                        let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                        if !completed {
                            return true; // We have a pending ban!
                        }
                    }
                }
            }
        }
    }
    false
}

// Background task to monitor champion select and perform auto actions
async fn champion_select_loop(state: AppState) {
    let mut last_phase: Option<String> = None;
    let mut hovered_champion: Option<i64> = None;
    let mut locked_champion: Option<i64> = None;
    let mut my_ban_completed: bool = false;
    let mut hover_attempted: bool = false; // Only hover once per pick phase, respect user changes
    let mut lock_attempted: bool = false; // Only lock once per pick phase, respect user changes
    
    // Cooldown tracking to prevent API spam
    let mut last_ban_attempt: Option<Instant> = None;
    let mut last_lock_attempt: Option<Instant> = None;
    let mut failed_ban_ids: Vec<i64> = Vec::new(); // Track failed bans to skip them

    loop {
        let is_running = *state.is_running.lock().await;
        if !is_running {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            continue;
        }

        let connected = check_league_connection_internal(&state).await;
        if !connected {
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            continue;
        }

        if is_in_champion_select(&state).await {
            if let Some(session) = get_champion_select_session(&state).await {
                // Get phase and remaining time from timer
                let timer = session.get("timer").and_then(|v| v.as_object());
                let phase = timer
                    .and_then(|t| t.get("phase"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN");
                
                // Get remaining time in milliseconds (adjustedTimeLeftInPhase is in ms)
                let time_left_ms = timer
                    .and_then(|t| t.get("adjustedTimeLeftInPhase"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(30000); // Default to 30 seconds if not found
                let time_left_secs = time_left_ms / 1000;
                
                // Debug timer
                eprintln!("[DEBUG] Timer: phase={}, time_left_ms={}, time_left_secs={}", phase, time_left_ms, time_left_secs);
                eprintln!("[DEBUG] hover_attempted={}, lock_attempted={}, my_ban_completed={}", hover_attempted, lock_attempted, my_ban_completed);
                
                // Reset state when entering new champion select (not just phase change)
                if last_phase.is_none() {
                            hovered_champion = None;
                    locked_champion = None;
                    my_ban_completed = false;
                    hover_attempted = false;
                    lock_attempted = false;
                    last_ban_attempt = None;
                    last_lock_attempt = None;
                    failed_ban_ids.clear();
                }
                            last_phase = Some(phase.to_string());

                        // Get current role
                        let current_role = get_current_role(&state).await.unwrap_or_else(|| "".to_string());
                        let role_key = match current_role.as_str() {
                    "TOP" | "top" => "top",
                    "JUNGLE" | "jungle" => "jungle",
                    "MIDDLE" | "middle" | "MID" | "mid" => "mid",
                    "BOTTOM" | "bottom" | "ADC" | "adc" => "adc",
                    "UTILITY" | "utility" | "SUPPORT" | "support" => "support",
                            _ => "",
                        };

                // Check turn status
                let is_ban_turn = is_my_ban_turn(&state).await;
                let is_pick_turn = is_my_pick_turn(&state).await;
                
                eprintln!("[DEBUG] TURN STATUS: is_ban_turn={}, is_pick_turn={}, role={}", is_ban_turn, is_pick_turn, role_key);

                // Reset my_ban_completed when not our ban turn anymore
                // This handles ARAM/Clash/custom formats with multiple ban phases
                if !is_ban_turn {
                    my_ban_completed = false;
                    failed_ban_ids.clear(); // Also clear failed bans for next ban phase
                }

                // Update status with detailed info
                if is_ban_turn {
                    *state.status.lock().await = format!("Ban phase - Your turn! ({})", if role_key.is_empty() { "no role" } else { role_key });
                } else if is_pick_turn {
                    *state.status.lock().await = format!("Pick phase - Your turn! ({})", if role_key.is_empty() { "no role" } else { role_key });
                } else {
                    *state.status.lock().await = format!("Champ Select - Waiting... ({})", phase);
                }

                // Get preferences - try role-specific first, then try all roles for fallback
                let prefs = state.champion_preferences.lock().await.clone();
                let role_prefs = if !role_key.is_empty() {
                    match role_key {
                        "top" => Some(prefs.top.clone()),
                        "jungle" => Some(prefs.jungle.clone()),
                        "mid" => Some(prefs.mid.clone()),
                        "adc" => Some(prefs.adc.clone()),
                        "support" => Some(prefs.support.clone()),
                        _ => None,
                    }
                } else {
                    // No role detected - try to find any role with preferences
                    // This is a fallback for blind pick or when role detection fails
                    if !prefs.mid.auto_ban_champions.is_empty() || !prefs.mid.preferred_champions.is_empty() {
                        Some(prefs.mid.clone())
                    } else if !prefs.top.auto_ban_champions.is_empty() || !prefs.top.preferred_champions.is_empty() {
                        Some(prefs.top.clone())
                    } else if !prefs.jungle.auto_ban_champions.is_empty() || !prefs.jungle.preferred_champions.is_empty() {
                        Some(prefs.jungle.clone())
                    } else if !prefs.adc.auto_ban_champions.is_empty() || !prefs.adc.preferred_champions.is_empty() {
                        Some(prefs.adc.clone())
                    } else if !prefs.support.auto_ban_champions.is_empty() || !prefs.support.preferred_champions.is_empty() {
                        Some(prefs.support.clone())
                    } else {
                        None
                    }
                };
                drop(prefs); // Release the lock before async operations

                if let Some(role_prefs) = role_prefs {
                    // === AUTO BAN ===
                    // RELAXED: Use has_pending_ban_action instead of is_ban_turn
                    // This checks completed == false instead of isInProgress == true
                    let auto_ban_enabled = *state.auto_ban_enabled.lock().await;
                    let has_pending_ban = has_pending_ban_action(&state).await;
                    eprintln!("[DEBUG] BAN CHECK: enabled={}, has_pending_ban={}, my_ban_completed={}, ban_list={:?}", 
                        auto_ban_enabled, has_pending_ban, my_ban_completed, role_prefs.auto_ban_champions);
                    
                    if auto_ban_enabled && !my_ban_completed && has_pending_ban {
                        // Check cooldown (1 second between attempts)
                        let can_attempt_ban = match last_ban_attempt {
                            Some(last) => last.elapsed() > std::time::Duration::from_secs(1),
                            None => true,
                        };
                        eprintln!("[DEBUG] BAN: can_attempt={}, failed_ids={:?}", can_attempt_ban, failed_ban_ids);
                        
                        if can_attempt_ban {
                            let ban_list = &role_prefs.auto_ban_champions;
                            if ban_list.is_empty() {
                                eprintln!("[DEBUG] BAN: No bans configured!");
                                *state.status.lock().await = "Ban turn - No bans configured".to_string();
                            } else {
                                let banned_champs = get_banned_champions(&state).await;
                                eprintln!("[DEBUG] BAN: Already banned champs: {:?}", banned_champs);
                                
                                // Try each champion in ban priority order
                                for ban_id in ban_list {
                                    eprintln!("[DEBUG] BAN: Checking champion {}", ban_id);
                                    
                                    // Skip if already banned
                                    if banned_champs.contains(ban_id) {
                                        eprintln!("[DEBUG] BAN: {} already banned, skip", ban_id);
                                        continue;
                                    }
                                    
                                    // Skip if we already failed this ban (API rejected it)
                                    if failed_ban_ids.contains(ban_id) {
                                        eprintln!("[DEBUG] BAN: {} in failed list, skip", ban_id);
                                        continue;
                                    }
                                    
                                    eprintln!("[DEBUG] BAN: >>> CALLING ban_champion for {}", ban_id);
                                    *state.status.lock().await = format!("Banning...");
                                    last_ban_attempt = Some(Instant::now());
                                    
                                    // Try to ban this champion
                                    let ban_result = ban_champion(&state, *ban_id).await;
                                    eprintln!("[DEBUG] BAN: ban_champion returned {}", ban_result);
                                    
                                    if ban_result {
                                        *state.status.lock().await = "Banned champion!".to_string();
                                        my_ban_completed = true;
                                        eprintln!("[DEBUG] BAN: SUCCESS! my_ban_completed=true");
                                            break;
                                    } else {
                                        // Ban failed - mark this champion and try next
                                        eprintln!("[DEBUG] BAN: FAILED, adding to failed_ids");
                                        failed_ban_ids.push(*ban_id);
                                        continue;
                                    }
                                        }
                                    }
                                }
                            }

                    // === AUTO HOVER ===
                    // Only hover ONCE per pick phase - respect user's manual changes
                    let auto_hover_enabled = *state.auto_hover_enabled.lock().await;
                    eprintln!("[DEBUG] HOVER CHECK: enabled={}, is_pick_turn={}, hover_attempted={}", auto_hover_enabled, is_pick_turn, hover_attempted);
                    
                    if auto_hover_enabled && is_pick_turn && !hover_attempted {
                        eprintln!("[DEBUG] Attempting to hover - first time this session");
                        // Find and hover the best available champion (only once)
                                for champ_id in &role_prefs.preferred_champions {
                            if !is_champion_available(&state, *champ_id).await {
                                eprintln!("[DEBUG] Champion {} not available", champ_id);
                                continue;
                            }
                            
                            eprintln!("[DEBUG] Calling hover_champion for {}", champ_id);
                                        if hover_champion(&state, *champ_id).await {
                                            hovered_champion = Some(*champ_id);
                                hover_attempted = true; // Don't hover again, respect user changes
                                eprintln!("[DEBUG] Hover SUCCESS - hover_attempted now TRUE");
                                *state.status.lock().await = "Hovering champion".to_string();
                                break;
                                        }
                                    }
                        // Even if we couldn't find an available champion, mark as attempted
                        if !hover_attempted {
                            eprintln!("[DEBUG] No champion could be hovered, marking as attempted anyway");
                            hover_attempted = true;
                        }
                    }

                    // === AUTO LOCK ===
                    // Lock when 3 seconds or less remain - respects user's champion choice
                    // RELAXED: Don't require is_pick_turn - trust the timer!
                    let auto_select_enabled = *state.auto_select_enabled.lock().await;
                    eprintln!("[DEBUG] LOCK CHECK: enabled={}, lock_attempted={}, time_left_secs={}", 
                        auto_select_enabled, lock_attempted, time_left_secs);
                    
                    // Trust the timer, not isInProgress
                    if auto_select_enabled && !lock_attempted && time_left_secs <= 3 {
                        eprintln!("[DEBUG] Time <= 3s, attempting to lock current champion");
                        *state.status.lock().await = format!("Locking in {}s...", time_left_secs);
                        
                        // Lock whatever champion the user currently has selected
                        // This respects any manual changes they made
                        if lock_current_champion(&state).await {
                            lock_attempted = true;
                            *state.status.lock().await = "Locked in champion!".to_string();
                            eprintln!("[DEBUG] Lock SUCCESS");
                        } else {
                            eprintln!("[DEBUG] Lock failed - trying hover+lock fallback");
                            // If no champion is selected, try to hover+lock our first preference
                            for champ_id in &role_prefs.preferred_champions {
                                if !is_champion_available(&state, *champ_id).await {
                                    continue;
                                }
                                
                                if hover_champion(&state, *champ_id).await {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                    
                                    if lock_current_champion(&state).await {
                                        lock_attempted = true;
                                        *state.status.lock().await = "Locked in champion!".to_string();
                                        eprintln!("[DEBUG] Fallback lock SUCCESS");
                                    }
                                            break;
                                        }
                                    }
                                }
                    } else if auto_select_enabled && time_left_secs > 3 && !lock_attempted {
                        // Show countdown when waiting to lock
                        *state.status.lock().await = format!("Pick phase - Locking at 3s ({}s left)", time_left_secs);
                    }
                } else {
                    // No preferences found
                    if is_ban_turn || is_pick_turn {
                        *state.status.lock().await = "Your turn - No champions configured".to_string();
                    }
                }
            }
        } else {
            // Reset state when not in champion select
            hovered_champion = None;
            locked_champion = None;
            my_ban_completed = false;
            hover_attempted = false;
            lock_attempted = false;
            last_phase = None;
            last_ban_attempt = None;
            last_lock_attempt = None;
            failed_ban_ids.clear();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}

// Background task to monitor and auto-accept
async fn auto_accept_loop(state: AppState) {
    loop {
        let is_running = *state.is_running.lock().await;
        if !is_running {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            continue;
        }

        // Check connection
        let connected = check_league_connection_internal(&state).await;
        if !connected {
            *state.status.lock().await = "Waiting for League client...".to_string();
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            continue;
        }

        // Check if in champion select - if so, don't overwrite status
        let in_champ_select = is_in_champion_select(&state).await;

        // Check for match
        let match_found = check_match_found(&state).await;
        *state.match_found.lock().await = match_found;

        if match_found {
            let delay = *state.accept_delay_seconds.lock().await;
            if delay > 0 {
                *state.status.lock().await = format!("Match found! Waiting {}s before accepting...", delay);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
            } else {
                *state.status.lock().await = "Match found! Auto-accepting...".to_string();
            }
            
            if accept_match(&state).await {
                *state.status.lock().await = "Match accepted!".to_string();
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                *state.match_found.lock().await = false;
            }
        } else if !in_champ_select {
            // Only update to "Waiting for match" if not in champion select
            *state.status.lock().await = "Waiting for match...".to_string();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}

#[tauri::command]
async fn check_league_connection(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(check_league_connection_internal(&state).await)
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.status.lock().await.clone())
}

#[tauri::command]
async fn is_running(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_running.lock().await)
}

#[tauri::command]
async fn is_match_found(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.match_found.lock().await)
}

#[tauri::command]
async fn start_auto_accept(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = true;
    *state.status.lock().await = "Starting auto-accept...".to_string();
    Ok(())
}

#[tauri::command]
async fn stop_auto_accept(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = false;
    *state.status.lock().await = "Stopped".to_string();
    Ok(())
}

#[tauri::command]
async fn set_accept_delay(state: tauri::State<'_, AppState>, delay_seconds: u64) -> Result<(), String> {
    *state.accept_delay_seconds.lock().await = delay_seconds;
    Ok(())
}

#[tauri::command]
async fn get_accept_delay(state: tauri::State<'_, AppState>) -> Result<u64, String> {
    Ok(*state.accept_delay_seconds.lock().await)
}

#[tauri::command]
async fn get_champion_preferences(state: tauri::State<'_, AppState>) -> Result<ChampionPreferences, String> {
    Ok(state.champion_preferences.lock().await.clone())
}

#[tauri::command]
async fn set_champion_preferences(state: tauri::State<'_, AppState>, prefs: ChampionPreferences) -> Result<(), String> {
    *state.champion_preferences.lock().await = prefs.clone();
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: prefs,
        auto_hover_enabled: *state.auto_hover_enabled.lock().await,
        auto_select_enabled: *state.auto_select_enabled.lock().await,
        auto_ban_enabled: *state.auto_ban_enabled.lock().await,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
async fn set_role_preferences(
    state: tauri::State<'_, AppState>,
    role: String,
    prefs: RolePreferences,
) -> Result<(), String> {
    let mut champ_prefs = state.champion_preferences.lock().await;
    match role.as_str() {
        "top" => champ_prefs.top = prefs,
        "jungle" => champ_prefs.jungle = prefs,
        "mid" => champ_prefs.mid = prefs,
        "adc" => champ_prefs.adc = prefs,
        "support" => champ_prefs.support = prefs,
        _ => return Err("Invalid role".to_string()),
    }
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: champ_prefs.clone(),
        auto_hover_enabled: *state.auto_hover_enabled.lock().await,
        auto_select_enabled: *state.auto_select_enabled.lock().await,
        auto_ban_enabled: *state.auto_ban_enabled.lock().await,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
async fn get_role_preferences(
    state: tauri::State<'_, AppState>,
    role: String,
) -> Result<RolePreferences, String> {
    let prefs = state.champion_preferences.lock().await;
    match role.as_str() {
        "top" => Ok(prefs.top.clone()),
        "jungle" => Ok(prefs.jungle.clone()),
        "mid" => Ok(prefs.mid.clone()),
        "adc" => Ok(prefs.adc.clone()),
        "support" => Ok(prefs.support.clone()),
        _ => Err("Invalid role".to_string()),
    }
}

#[tauri::command]
async fn set_auto_hover(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_hover_enabled.lock().await = enabled;
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: state.champion_preferences.lock().await.clone(),
        auto_hover_enabled: enabled,
        auto_select_enabled: *state.auto_select_enabled.lock().await,
        auto_ban_enabled: *state.auto_ban_enabled.lock().await,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
async fn get_auto_hover(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_hover_enabled.lock().await)
}

#[tauri::command]
async fn set_auto_select(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_select_enabled.lock().await = enabled;
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: state.champion_preferences.lock().await.clone(),
        auto_hover_enabled: *state.auto_hover_enabled.lock().await,
        auto_select_enabled: enabled,
        auto_ban_enabled: *state.auto_ban_enabled.lock().await,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
async fn get_auto_select(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_select_enabled.lock().await)
}

#[tauri::command]
async fn set_auto_ban(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_ban_enabled.lock().await = enabled;
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: state.champion_preferences.lock().await.clone(),
        auto_hover_enabled: *state.auto_hover_enabled.lock().await,
        auto_select_enabled: *state.auto_select_enabled.lock().await,
        auto_ban_enabled: enabled,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
async fn get_auto_ban(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_ban_enabled.lock().await)
}

// Static champion data - works without League Client connection
// Complete list of all 172 champions as of December 2025
fn get_all_champions_static() -> Vec<serde_json::Value> {
    let champions = vec![
        (266, "Aatrox", "Aatrox"),
        (103, "Ahri", "Ahri"),
        (84, "Akali", "Akali"),
        (166, "Akshan", "Akshan"),
        (12, "Alistar", "Alistar"),
        (893, "Ambessa", "Ambessa"),
        (32, "Amumu", "Amumu"),
        (34, "Anivia", "Anivia"),
        (1, "Annie", "Annie"),
        (523, "Aphelios", "Aphelios"),
        (22, "Ashe", "Ashe"),
        (136, "Aurelion Sol", "AurelionSol"),
        (799, "Aurora", "Aurora"),
        (268, "Azir", "Azir"),
        (432, "Bard", "Bard"),
        (200, "Bel'Veth", "Belveth"),
        (53, "Blitzcrank", "Blitzcrank"),
        (63, "Brand", "Brand"),
        (201, "Braum", "Braum"),
        (233, "Briar", "Briar"),
        (51, "Caitlyn", "Caitlyn"),
        (164, "Camille", "Camille"),
        (69, "Cassiopeia", "Cassiopeia"),
        (31, "Cho'Gath", "Chogath"),
        (42, "Corki", "Corki"),
        (122, "Darius", "Darius"),
        (131, "Diana", "Diana"),
        (119, "Draven", "Draven"),
        (36, "Dr. Mundo", "DrMundo"),
        (245, "Ekko", "Ekko"),
        (60, "Elise", "Elise"),
        (28, "Evelynn", "Evelynn"),
        (81, "Ezreal", "Ezreal"),
        (9, "Fiddlesticks", "Fiddlesticks"),
        (114, "Fiora", "Fiora"),
        (105, "Fizz", "Fizz"),
        (3, "Galio", "Galio"),
        (41, "Gangplank", "Gangplank"),
        (86, "Garen", "Garen"),
        (150, "Gnar", "Gnar"),
        (79, "Gragas", "Gragas"),
        (104, "Graves", "Graves"),
        (887, "Gwen", "Gwen"),
        (120, "Hecarim", "Hecarim"),
        (74, "Heimerdinger", "Heimerdinger"),
        (910, "Hwei", "Hwei"),
        (420, "Illaoi", "Illaoi"),
        (39, "Irelia", "Irelia"),
        (427, "Ivern", "Ivern"),
        (40, "Janna", "Janna"),
        (59, "Jarvan IV", "JarvanIV"),
        (24, "Jax", "Jax"),
        (126, "Jayce", "Jayce"),
        (202, "Jhin", "Jhin"),
        (222, "Jinx", "Jinx"),
        (145, "Kai'Sa", "Kaisa"),
        (429, "Kalista", "Kalista"),
        (43, "Karma", "Karma"),
        (30, "Karthus", "Karthus"),
        (38, "Kassadin", "Kassadin"),
        (55, "Katarina", "Katarina"),
        (10, "Kayle", "Kayle"),
        (141, "Kayn", "Kayn"),
        (85, "Kennen", "Kennen"),
        (121, "Kha'Zix", "Khazix"),
        (203, "Kindred", "Kindred"),
        (240, "Kled", "Kled"),
        (96, "Kog'Maw", "KogMaw"),
        (897, "K'Sante", "KSante"),
        (7, "LeBlanc", "Leblanc"),
        (64, "Lee Sin", "LeeSin"),
        (89, "Leona", "Leona"),
        (876, "Lillia", "Lillia"),
        (127, "Lissandra", "Lissandra"),
        (117, "Lulu", "Lulu"),
        (99, "Lux", "Lux"),
        (54, "Malphite", "Malphite"),
        (90, "Malzahar", "Malzahar"),
        (57, "Maokai", "Maokai"),
        (11, "Master Yi", "MasterYi"),
        (942, "Mel", "Mel"),
        (902, "Milio", "Milio"),
        (21, "Miss Fortune", "MissFortune"),
        (62, "Wukong", "MonkeyKing"),
        (82, "Mordekaiser", "Mordekaiser"),
        (25, "Morgana", "Morgana"),
        (950, "Naafiri", "Naafiri"),
        (267, "Nami", "Nami"),
        (75, "Nasus", "Nasus"),
        (111, "Nautilus", "Nautilus"),
        (518, "Neeko", "Neeko"),
        (76, "Nidalee", "Nidalee"),
        (895, "Nilah", "Nilah"),
        (56, "Nocturne", "Nocturne"),
        (20, "Nunu & Willump", "Nunu"),
        (2, "Olaf", "Olaf"),
        (61, "Orianna", "Orianna"),
        (516, "Ornn", "Ornn"),
        (80, "Pantheon", "Pantheon"),
        (78, "Poppy", "Poppy"),
        (555, "Pyke", "Pyke"),
        (246, "Qiyana", "Qiyana"),
        (133, "Quinn", "Quinn"),
        (497, "Rakan", "Rakan"),
        (33, "Rammus", "Rammus"),
        (421, "Rek'Sai", "RekSai"),
        (526, "Rell", "Rell"),
        (888, "Renata Glasc", "Renata"),
        (58, "Renekton", "Renekton"),
        (107, "Rengar", "Rengar"),
        (92, "Riven", "Riven"),
        (68, "Rumble", "Rumble"),
        (13, "Ryze", "Ryze"),
        (360, "Samira", "Samira"),
        (113, "Sejuani", "Sejuani"),
        (235, "Senna", "Senna"),
        (147, "Seraphine", "Seraphine"),
        (875, "Sett", "Sett"),
        (35, "Shaco", "Shaco"),
        (98, "Shen", "Shen"),
        (102, "Shyvana", "Shyvana"),
        (27, "Singed", "Singed"),
        (14, "Sion", "Sion"),
        (15, "Sivir", "Sivir"),
        (72, "Skarner", "Skarner"),
        (901, "Smolder", "Smolder"),
        (37, "Sona", "Sona"),
        (16, "Soraka", "Soraka"),
        (50, "Swain", "Swain"),
        (517, "Sylas", "Sylas"),
        (134, "Syndra", "Syndra"),
        (223, "Tahm Kench", "TahmKench"),
        (163, "Taliyah", "Taliyah"),
        (91, "Talon", "Talon"),
        (44, "Taric", "Taric"),
        (17, "Teemo", "Teemo"),
        (412, "Thresh", "Thresh"),
        (18, "Tristana", "Tristana"),
        (48, "Trundle", "Trundle"),
        (23, "Tryndamere", "Tryndamere"),
        (4, "Twisted Fate", "TwistedFate"),
        (29, "Twitch", "Twitch"),
        (77, "Udyr", "Udyr"),
        (6, "Urgot", "Urgot"),
        (110, "Varus", "Varus"),
        (67, "Vayne", "Vayne"),
        (45, "Veigar", "Veigar"),
        (161, "Vel'Koz", "Velkoz"),
        (711, "Vex", "Vex"),
        (254, "Vi", "Vi"),
        (234, "Viego", "Viego"),
        (112, "Viktor", "Viktor"),
        (8, "Vladimir", "Vladimir"),
        (106, "Volibear", "Volibear"),
        (19, "Warwick", "Warwick"),
        (498, "Xayah", "Xayah"),
        (101, "Xerath", "Xerath"),
        (5, "Xin Zhao", "XinZhao"),
        (157, "Yasuo", "Yasuo"),
        (777, "Yone", "Yone"),
        (83, "Yorick", "Yorick"),
        (350, "Yuumi", "Yuumi"),
        (154, "Zac", "Zac"),
        (168, "Zaahen", "Zaahen"),
        (238, "Zed", "Zed"),
        (221, "Zeri", "Zeri"),
        (115, "Ziggs", "Ziggs"),
        (26, "Zilean", "Zilean"),
        (142, "Zoe", "Zoe"),
        (143, "Zyra", "Zyra"),
    ];

    champions
        .into_iter()
        .map(|(id, name, alias)| {
            serde_json::json!({
                "id": id,
                "name": name,
                "alias": alias
            })
        })
        .collect()
}

#[tauri::command]
async fn get_champions(_state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    // Return static champion list - works without League Client
    Ok(get_all_champions_static())
}

fn main() {
    let app_state = AppState::default();
    let state_clone = app_state.clone();
    let champ_select_state = app_state.clone();

    // Start background tasks
    tauri::async_runtime::spawn(async move {
        auto_accept_loop(state_clone).await;
    });

    tauri::async_runtime::spawn(async move {
        champion_select_loop(champ_select_state).await;
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_league_connection,
            get_status,
            is_running,
            is_match_found,
            start_auto_accept,
            stop_auto_accept,
            set_accept_delay,
            get_accept_delay,
            get_champion_preferences,
            set_champion_preferences,
            set_role_preferences,
            get_role_preferences,
            set_auto_hover,
            get_auto_hover,
            set_auto_select,
            get_auto_select,
            set_auto_ban,
            get_auto_ban,
            get_champions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

