use crate::core::models::AppState;
use crate::core::lcu::check_league_connection_internal;
use crate::features::champion_select::session::{
    get_champion_select_session,
    is_in_champion_select,
    get_current_role,
    is_my_ban_turn,
    is_my_pick_turn,
};
use crate::features::auto_hover::execute_auto_hover;
use crate::features::auto_ban::{execute_auto_ban, has_pending_ban_action};
use crate::features::auto_lock::execute_auto_lock;
use std::time::Instant;

// Background task to monitor champion select and perform auto actions
pub async fn champion_select_loop(state: AppState) {
    let mut last_phase: Option<String> = None;
    let mut my_ban_completed: bool = false;
    let mut hover_attempted: bool = false; // Only hover once per pick phase, respect user changes
    let mut lock_attempted: bool = false; // Only lock once per pick phase, respect user changes
    
    // Cooldown tracking to prevent API spam
    let mut last_ban_attempt: Option<Instant> = None;
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
                    my_ban_completed = false;
                    hover_attempted = false;
                    lock_attempted = false;
                    last_ban_attempt = None;
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
                    // === AUTO HOVER (at champion select start) ===
                    // Hover ONCE when champion select first starts (before ban phase)
                    let auto_hover_enabled = *state.auto_hover_enabled.lock().await;
                    
                    // Hover immediately when champion select starts (only once)
                    if auto_hover_enabled && !hover_attempted {
                        if execute_auto_hover(&state, &role_prefs).await {
                            hover_attempted = true;
                        }
                    }

                    // === AUTO BAN ===
                    let auto_ban_enabled = *state.auto_ban_enabled.lock().await;
                    let has_pending_ban = has_pending_ban_action(&state).await;
                    eprintln!("[DEBUG] BAN CHECK: enabled={}, is_ban_turn={}, has_pending_ban={}, my_ban_completed={}, ban_list={:?}", 
                        auto_ban_enabled, is_ban_turn, has_pending_ban, my_ban_completed, role_prefs.auto_ban_champions);
                    
                    if auto_ban_enabled && !my_ban_completed && is_ban_turn && has_pending_ban {
                        let (ban_succeeded, _ban_attempted) = execute_auto_ban(
                            &state,
                            &role_prefs,
                            &mut last_ban_attempt,
                            &mut failed_ban_ids,
                        ).await;
                        
                        if ban_succeeded {
                            my_ban_completed = true;
                            eprintln!("[DEBUG] BAN: SUCCESS! my_ban_completed=true");
                        }
                    }


                    // === AUTO LOCK ===
                    // Lock when: it's our turn AND time is running low
                    let auto_select_enabled = *state.auto_select_enabled.lock().await;
                    eprintln!("[DEBUG] LOCK CHECK: enabled={}, lock_attempted={}, time_left_secs={}, is_pick_turn={}", 
                        auto_select_enabled, lock_attempted, time_left_secs, is_pick_turn);
                    
                    // Lock at 5 seconds or less to give more time for the action to complete
                    if auto_select_enabled && is_pick_turn && !lock_attempted && time_left_secs <= 5 {
                        *state.status.lock().await = format!("Locking in {}s...", time_left_secs);
                        
                        if execute_auto_lock(&state, &role_prefs).await {
                            lock_attempted = true;
                            *state.status.lock().await = "Locked in champion!".to_string();
                        } else {
                            // Mark as attempted even if failed to prevent spam
                            lock_attempted = true;
                        }
                    } else if auto_select_enabled && is_pick_turn && time_left_secs > 5 && !lock_attempted {
                        // Show countdown when waiting to lock
                        *state.status.lock().await = format!("Pick phase - Locking at 5s ({}s left)", time_left_secs);
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
            my_ban_completed = false;
            hover_attempted = false;
            lock_attempted = false;
            last_phase = None;
            last_ban_attempt = None;
            failed_ban_ids.clear();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
