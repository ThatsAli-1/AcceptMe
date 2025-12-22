use crate::core::models::{AppState, RolePreferences};
use crate::core::lcu::get_or_read_client_info;
use crate::features::champion_select::session::{
    get_champion_select_session, get_my_cell_id, is_champion_available
};

/// Execute auto-hover logic: hover the first available preferred champion
/// Returns true if hover was attempted (successfully or not), false if no preferences
pub async fn execute_auto_hover(
    state: &AppState, 
    role_prefs: &RolePreferences
) -> bool {
    eprintln!("[DEBUG] HOVER: Champion select started, hovering preferred champion now");
    
    // Find and hover the best available champion
    for champ_id in &role_prefs.preferred_champions {
        if !is_champion_available(state, *champ_id).await {
            eprintln!("[DEBUG] Champion {} not available", champ_id);
            continue;
        }
        
        eprintln!("[DEBUG] Calling hover_champion for {}", champ_id);
        if hover_champion(state, *champ_id).await {
            eprintln!("[DEBUG] Hover SUCCESS - will not hover again this session");
            *state.status.lock().await = "Hovered preferred champion".to_string();
            return true;
        }
    }
    
    // Even if we couldn't find an available champion, mark as attempted
    eprintln!("[DEBUG] No champion could be hovered, marking as attempted anyway");
    true
}

// Hover champion - sends the API request to hover a champion
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
                            
                            // RELAXED: Check completed == false
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
