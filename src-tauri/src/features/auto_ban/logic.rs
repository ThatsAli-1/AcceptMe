use crate::core::models::{AppState, RolePreferences};
use crate::core::lcu::get_or_read_client_info;
use crate::features::champion_select::session::{
    get_champion_select_session, get_my_cell_id
};
use std::time::Instant;

/// Execute auto-ban logic: ban the first available champion from the ban list
/// Returns (ban_succeeded, ban_attempted)
pub async fn execute_auto_ban(
    state: &AppState,
    role_prefs: &RolePreferences,
    last_ban_attempt: &mut Option<Instant>,
    failed_ban_ids: &mut Vec<i64>,
) -> (bool, bool) {
    // Check cooldown (1 second between attempts)
    let can_attempt_ban = match last_ban_attempt {
        Some(last) => last.elapsed() > std::time::Duration::from_secs(1),
        None => true,
    };
    eprintln!("[DEBUG] BAN: can_attempt={}, failed_ids={:?}", can_attempt_ban, failed_ban_ids);
    
    if !can_attempt_ban {
        return (false, false);
    }
    
    let ban_list = &role_prefs.auto_ban_champions;
    if ban_list.is_empty() {
        eprintln!("[DEBUG] BAN: No bans configured!");
        *state.status.lock().await = "Ban turn - No bans configured".to_string();
        return (false, false);
    }
    
    // Get fresh list of banned champions
    let banned_champs = get_banned_champions(state).await;
    eprintln!("[DEBUG] BAN: Already banned champs: {:?}", banned_champs);
    
    // Try each champion in ban priority order
    let mut attempted_any = false;
    for ban_id in ban_list {
        eprintln!("[DEBUG] BAN: Checking champion {}", ban_id);
        
        // Skip if already banned by anyone
        if banned_champs.contains(ban_id) {
            eprintln!("[DEBUG] BAN: {} already banned by someone, skip", ban_id);
            continue;
        }
        
        // Skip if we already failed this ban (API rejected it)
        if failed_ban_ids.contains(ban_id) {
            eprintln!("[DEBUG] BAN: {} in failed list, skip", ban_id);
            continue;
        }
        
        eprintln!("[DEBUG] BAN: >>> CALLING ban_champion for {}", ban_id);
        *state.status.lock().await = format!("Banning champion {}...", ban_id);
        *last_ban_attempt = Some(Instant::now());
        attempted_any = true;
        
        // Try to ban this champion
        let ban_result = ban_champion(state, *ban_id).await;
        eprintln!("[DEBUG] BAN: ban_champion({}) returned {}", ban_id, ban_result);
        
        if ban_result {
            *state.status.lock().await = "Banned champion!".to_string();
            eprintln!("[DEBUG] BAN: SUCCESS!");
            return (true, true);
        } else {
            // Ban failed - mark this champion and try next
            eprintln!("[DEBUG] BAN: FAILED for champion {}, adding to failed_ids", ban_id);
            failed_ban_ids.push(*ban_id);
            // Don't break - try the next champion in the list
            continue;
        }
    }
    
    // If we tried all champions and none worked
    if attempted_any {
        eprintln!("[DEBUG] BAN: Tried all available champions, none succeeded");
        *state.status.lock().await = "No champions available to ban".to_string();
    }
    
    (false, attempted_any)
}

// Get list of banned champion IDs
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

/// Check if we have a pending ban action
pub async fn has_pending_ban_action(state: &AppState) -> bool {
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

// Ban champion - sends the API request to ban a champion
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
                            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if action_type != "ban" {
                                continue;
                            }
                            
                            let actor_cell_id = action.get("actorCellId").and_then(|v| v.as_i64());
                            if actor_cell_id != my_cell_id {
                                continue;
                            }
                            
                            let completed = action.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
                            eprintln!("[DEBUG] ban_champion: Found our ban action, completed={}", completed);
                            if completed {
                                continue; 
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
                                
                                // Small delay
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
