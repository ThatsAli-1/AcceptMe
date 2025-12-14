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

#[derive(Debug, Clone)]
struct AppState {
    is_running: Arc<Mutex<bool>>,
    is_connected: Arc<Mutex<bool>>,
    status: Arc<Mutex<String>>,
    match_found: Arc<Mutex<bool>>,
    client_info: Arc<Mutex<Option<LeagueClientInfo>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            is_connected: Arc::new(Mutex::new(false)),
            status: Arc::new(Mutex::new("Initializing...".to_string())),
            match_found: Arc::new(Mutex::new(false)),
            client_info: Arc::new(Mutex::new(None)),
        }
    }
}

// Get League client info from lockfile
fn get_league_client_info() -> Option<LeagueClientInfo> {
    use std::path::PathBuf;
    
    // Try common League of Legends installation paths
    // The lockfile is typically in the game installation directory, not AppData
    let possible_paths = vec![
        // Most common: Installation directory (C:\Riot Games\League of Legends\)
        r"C:\Riot Games\League of Legends\lockfile".to_string(),
        // Alternative installation locations
        format!(r"{}\Riot Games\League of Legends\lockfile", 
            std::env::var("PROGRAMFILES").unwrap_or_default()),
        format!(r"{}\Riot Games\League of Legends\lockfile", 
            std::env::var("PROGRAMFILES(X86)").unwrap_or_default()),
        // Sometimes in AppData (less common)
        format!(r"{}\Riot Games\League of Legends\lockfile", 
            std::env::var("LOCALAPPDATA").unwrap_or_default()),
        // User-specific installation
        format!(r"{}\Riot Games\League of Legends\lockfile", 
            std::env::var("USERPROFILE").unwrap_or_default()),
    ];

    for path_str in possible_paths {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                // Lockfile format: name:pid:port:password:protocol
                let parts: Vec<&str> = contents.split(':').collect();
                if parts.len() >= 5 {
                    if let Ok(port) = parts[2].parse::<u16>() {
                        return Some(LeagueClientInfo {
                            port,
                            password: parts[3].to_string(),
                            protocol: parts[4].trim().to_string(),
                        });
                    }
                }
            }
        }
    }
    None
}

// Check if League client is running (internal helper)
async fn check_league_connection_internal(state: &AppState) -> bool {
    if let Some(client_info) = get_league_client_info() {
        if let Ok(client) = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
        {
            let url = format!("{}://127.0.0.1:{}/lol-summoner/v1/current-summoner", 
                client_info.protocol, client_info.port);
            
            let response = client
                .get(&url)
                .basic_auth("riot", Some(&client_info.password))
                .send()
                .await;
            
            if response.is_ok() {
                *state.client_info.lock().await = Some(client_info);
                *state.is_connected.lock().await = true;
                return true;
            }
        }
    }
    
    *state.is_connected.lock().await = false;
    *state.client_info.lock().await = None;
    false
}

// Accept match
async fn accept_match(state: &AppState) -> bool {
    let client_info = state.client_info.lock().await.clone();
    if let Some(info) = client_info {
        if let Ok(client) = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
        {
            let url = format!("{}://127.0.0.1:{}/lol-matchmaking/v1/ready-check/accept", 
                info.protocol, info.port);
            
            let response = client
                .post(&url)
                .basic_auth("riot", Some(&info.password))
                .send()
                .await;
            
            return response.is_ok();
        }
    }
    false
}

// Check for match found
async fn check_match_found(state: &AppState) -> bool {
    let client_info = state.client_info.lock().await.clone();
    if let Some(info) = client_info {
        if let Ok(client) = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
        {
            let url = format!("{}://127.0.0.1:{}/lol-matchmaking/v1/ready-check", 
                info.protocol, info.port);
            
            if let Ok(response) = client
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
    }
    false
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
            *state.status.lock().await = "Match found! Auto-accepting...".to_string();
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

fn main() {
    let app_state = AppState::default();
    let state_clone = app_state.clone();

    // Start background task
    tauri::async_runtime::spawn(async move {
        auto_accept_loop(state_clone).await;
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_league_connection,
            get_status,
            is_running,
            is_match_found,
            start_auto_accept,
            stop_auto_accept
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

