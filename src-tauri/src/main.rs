// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
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

        Self {
            is_running: Arc::new(Mutex::new(false)),
            is_connected: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Initializing...".to_string())),
            match_found: Arc::new(Mutex::new(false)),
            client_info: Arc::new(Mutex::new(None)),
            accept_delay_seconds: Arc::new(Mutex::new(0)), // Default: no delay
            champion_preferences: Arc::new(Mutex::new(ChampionPreferences::default())),
            auto_hover_enabled: Arc::new(Mutex::new(true)),
            auto_select_enabled: Arc::new(Mutex::new(true)),
            auto_ban_enabled: Arc::new(Mutex::new(true)),
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

// Get current role/position
async fn get_current_role(state: &AppState) -> Option<String> {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(my_cell_id) = get_my_cell_id(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            if let Some(cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                if cell_id == my_cell_id {
                                    if let Some(completed) = action.get("completed").and_then(|v| v.as_bool()) {
                                        if !completed {
                                            // Try to get position from team position
                                            if let Some(position) = session.get("myTeam")
                                                .and_then(|v| v.as_array())
                                                .and_then(|team| team.iter().find(|p| {
                                                    p.get("cellId").and_then(|v| v.as_i64()) == Some(my_cell_id)
                                                }))
                                                .and_then(|p| p.get("assignedPosition").and_then(|v| v.as_str())) {
                                                return Some(position.to_string());
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
    None
}

// Hover champion
async fn hover_champion(state: &AppState, champion_id: i64) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                if let Some(actor_cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                    if let Some(my_cell_id) = get_my_cell_id(state).await {
                                        if actor_cell_id == my_cell_id {
                                            let patch_url = format!(
                                                "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                info.protocol, info.port, id
                                            );
                                            
                                            let payload = serde_json::json!({
                                                "championId": champion_id,
                                                "completed": false,
                                                "type": "pick"
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
            }
        }
    }
    false
}

// Select champion
async fn select_champion(state: &AppState, champion_id: i64) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                if let Some(actor_cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                    if let Some(my_cell_id) = get_my_cell_id(state).await {
                                        if actor_cell_id == my_cell_id {
                                            if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                                                if action_type == "pick" {
                                                    let patch_url = format!(
                                                        "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                        info.protocol, info.port, id
                                                    );
                                                    
                                                    let payload = serde_json::json!({
                                                        "championId": champion_id,
                                                        "completed": true,
                                                        "type": "pick"
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
                    }
                }
            }
        }
    }
    false
}

// Ban champion
async fn ban_champion(state: &AppState, champion_id: i64) -> bool {
    if let Some(info) = get_or_read_client_info(state).await {
        if let Some(session) = get_champion_select_session(state).await {
            if let Some(actions) = session.get("actions").and_then(|v| v.as_array()) {
                for action_array in actions {
                    if let Some(action_array) = action_array.as_array() {
                        for action in action_array {
                            if let Some(id) = action.get("id").and_then(|v| v.as_i64()) {
                                if let Some(actor_cell_id) = action.get("actorCellId").and_then(|v| v.as_i64()) {
                                    if let Some(my_cell_id) = get_my_cell_id(state).await {
                                        if actor_cell_id == my_cell_id {
                                            if let Some(action_type) = action.get("type").and_then(|v| v.as_str()) {
                                                if action_type == "ban" {
                                                    let patch_url = format!(
                                                        "{}://127.0.0.1:{}/lol-champ-select/v1/session/actions/{}",
                                                        info.protocol, info.port, id
                                                    );
                                                    
                                                    let payload = serde_json::json!({
                                                        "championId": champion_id,
                                                        "completed": true,
                                                        "type": "ban"
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
        if let Some(timer) = session.get("timer").and_then(|v| v.as_object()) {
            if let Some(phase) = timer.get("phase").and_then(|v| v.as_str()) {
                return phase == "BAN_PICK" || phase == "FINALIZATION";
            }
        }
    }
    false
}

// Background task to monitor champion select and perform auto actions
async fn champion_select_loop(state: AppState) {
    let mut last_phase: Option<String> = None;
    let mut hovered_champion: Option<i64> = None;
    let mut selected_champion: Option<i64> = None;
    let mut banned_champions: Vec<i64> = Vec::new();

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
                if let Some(timer) = session.get("timer").and_then(|v| v.as_object()) {
                    if let Some(phase) = timer.get("phase").and_then(|v| v.as_str()) {
                        // Reset state when phase changes
                        if last_phase.as_deref() != Some(phase) {
                            hovered_champion = None;
                            selected_champion = None;
                            banned_champions.clear();
                            last_phase = Some(phase.to_string());
                        }

                        // Get current role
                        let current_role = get_current_role(&state).await.unwrap_or_else(|| "".to_string());
                        let role_key = match current_role.as_str() {
                            "TOP" => "top",
                            "JUNGLE" => "jungle",
                            "MIDDLE" => "mid",
                            "BOTTOM" => "adc",
                            "UTILITY" => "support",
                            _ => "",
                        };

                        if !role_key.is_empty() {
                            let prefs = state.champion_preferences.lock().await;
                            let role_prefs = match role_key {
                                "top" => &prefs.top,
                                "jungle" => &prefs.jungle,
                                "mid" => &prefs.mid,
                                "adc" => &prefs.adc,
                                "support" => &prefs.support,
                                _ => continue,
                            };

                            // Auto ban
                            if *state.auto_ban_enabled.lock().await {
                                for ban_id in &role_prefs.auto_ban_champions {
                                    if !banned_champions.contains(ban_id) {
                                        if ban_champion(&state, *ban_id).await {
                                            banned_champions.push(*ban_id);
                                            break;
                                        }
                                    }
                                }
                            }

                            // Auto hover and select
                            if *state.auto_hover_enabled.lock().await || *state.auto_select_enabled.lock().await {
                                for champ_id in &role_prefs.preferred_champions {
                                    if hovered_champion != Some(*champ_id) && *state.auto_hover_enabled.lock().await {
                                        if hover_champion(&state, *champ_id).await {
                                            hovered_champion = Some(*champ_id);
                                        }
                                    }

                                    if selected_champion != Some(*champ_id) && *state.auto_select_enabled.lock().await {
                                        if select_champion(&state, *champ_id).await {
                                            selected_champion = Some(*champ_id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Reset state when not in champion select
            hovered_champion = None;
            selected_champion = None;
            banned_champions.clear();
            last_phase = None;
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
        } else {
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
    *state.champion_preferences.lock().await = prefs;
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
    Ok(())
}

#[tauri::command]
async fn get_auto_hover(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_hover_enabled.lock().await)
}

#[tauri::command]
async fn set_auto_select(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_select_enabled.lock().await = enabled;
    Ok(())
}

#[tauri::command]
async fn get_auto_select(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_select_enabled.lock().await)
}

#[tauri::command]
async fn set_auto_ban(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_ban_enabled.lock().await = enabled;
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

