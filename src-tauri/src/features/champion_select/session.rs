use crate::core::models::AppState;
use crate::core::lcu::get_or_read_client_info;

// Get current champion select session
pub async fn get_champion_select_session(state: &AppState) -> Option<serde_json::Value> {
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

// Check if in champion select
pub async fn is_in_champion_select(state: &AppState) -> bool {
    if let Some(session) = get_champion_select_session(state).await {
        if session.get("localPlayerCellId").is_some() {
            return true;
        }
    }
    false
}

// Get current summoner's cell ID in champion select
pub async fn get_my_cell_id(state: &AppState) -> Option<i64> {
    if let Some(session) = get_champion_select_session(state).await {
        if let Some(local_player_cell_id) = session.get("localPlayerCellId").and_then(|v| v.as_i64()) {
            return Some(local_player_cell_id);
        }
    }
    None
}

// Get current role/position - directly from myTeam
pub async fn get_current_role(state: &AppState) -> Option<String> {
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

// Check if it's currently our turn to pick
pub async fn is_my_pick_turn(state: &AppState) -> bool {
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

// Check if it's currently our turn to ban
pub async fn is_my_ban_turn(state: &AppState) -> bool {
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

// Get list of banned champion IDs
pub async fn get_banned_champions(state: &AppState) -> Vec<i64> {
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

// Get list of picked champion IDs
pub async fn get_picked_champions(state: &AppState) -> Vec<i64> {
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

// Check if champion is available
pub async fn is_champion_available(state: &AppState, champion_id: i64) -> bool {
    let banned = get_banned_champions(state).await;
    let picked = get_picked_champions(state).await;
    !banned.contains(&champion_id) && !picked.contains(&champion_id)
}
