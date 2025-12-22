use crate::core::models::AppState;
use crate::core::lcu::{check_league_connection_internal, get_or_read_client_info};
use crate::features::champion_select::session::is_in_champion_select;

// Background task to monitor and auto-accept
pub async fn auto_accept_loop(state: AppState) {
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
