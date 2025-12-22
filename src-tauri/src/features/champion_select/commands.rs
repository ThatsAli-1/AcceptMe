use crate::core::models::{AppState, ChampionPreferences, RolePreferences, UserSettings};
use crate::core::settings::save_settings;

#[tauri::command]
pub async fn get_champion_preferences(state: tauri::State<'_, AppState>) -> Result<ChampionPreferences, String> {
    Ok(state.champion_preferences.lock().await.clone())
}

#[tauri::command]
pub async fn set_champion_preferences(state: tauri::State<'_, AppState>, prefs: ChampionPreferences) -> Result<(), String> {
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
pub async fn set_role_preferences(
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
pub async fn get_role_preferences(
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
