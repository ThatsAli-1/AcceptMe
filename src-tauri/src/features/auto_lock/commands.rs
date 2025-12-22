use crate::core::models::{AppState, UserSettings};
use crate::core::settings::save_settings;

#[tauri::command]
pub async fn set_auto_select(state: tauri::State<'_, AppState>, enabled: bool) -> Result<(), String> {
    *state.auto_select_enabled.lock().await = enabled;
    
    // Save to file
    let settings = UserSettings {
        champion_preferences: state.champion_preferences.lock().await.clone(),
        auto_hover_enabled: *state.auto_hover_enabled.lock().await,
        auto_select_enabled: enabled,
        auto_ban_enabled: *state.auto_ban_enabled.lock().await,
        accept_delay_seconds: *state.accept_delay_seconds.lock().await,
    };
    save_settings(&settings);
    
    Ok(())
}

#[tauri::command]
pub async fn get_auto_select(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.auto_select_enabled.lock().await)
}
