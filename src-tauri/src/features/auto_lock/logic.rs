use crate::core::models::{AppState, RolePreferences};
use crate::core::lcu::get_or_read_client_info;
use crate::features::champion_select::session::{
    get_champion_select_session, get_my_cell_id, is_champion_available
};

/// Execute auto-lock logic: lock in the current or preferred champion
/// Returns true if lock was successful
pub async fn execute_auto_lock(
    state: &AppState,
    role_prefs: &RolePreferences,
) -> bool {
    eprintln!("[DEBUG] Time <= 5s AND it's our turn, attempting to lock");
    
    // Try to lock whatever champion is currently selected
    if lock_current_champion(state).await {
        eprintln!("[DEBUG] Lock SUCCESS");
        return true;
    } else {
        eprintln!("[DEBUG] Lock failed - trying hover+lock fallback");
        // If no champion is selected, try to hover+lock our first preference
        for champ_id in &role_prefs.preferred_champions {
            if !is_champion_available(state, *champ_id).await {
                continue;
            }
            
            eprintln!("[DEBUG] Fallback: Hovering champion {}", champ_id);
            if hover_champion(state, *champ_id).await {
                tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                
                if lock_current_champion(state).await {
                    eprintln!("[DEBUG] Fallback lock SUCCESS");
                    return true;
                }
            }
        }
        eprintln!("[DEBUG] Fallback lock FAILED - no valid champions");
        false
    }
}

// Just lock in the currently selected champion
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
                            
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            if completed {
                                continue; 
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

// Hover champion (used in fallback logic)
async fn hover_champion(state: &AppState, champion_id: i64) -> bool {
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
                            
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            if completed {
                                continue;
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
