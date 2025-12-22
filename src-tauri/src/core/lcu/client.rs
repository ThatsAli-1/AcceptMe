use crate::core::models::{AppState, LeagueClientInfo};

// Get League client info; cached, only hits disk when missing
pub async fn get_or_read_client_info(state: &AppState) -> Option<LeagueClientInfo> {
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
pub async fn check_league_connection_internal(state: &AppState) -> bool {
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
