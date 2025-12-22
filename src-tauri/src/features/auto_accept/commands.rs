use crate::core::models::AppState;

#[tauri::command]
pub async fn start_auto_accept(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = true;
    *state.status.lock().await = "Starting auto-accept...".to_string();
    Ok(())
}

#[tauri::command]
pub async fn stop_auto_accept(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.is_running.lock().await = false;
    *state.status.lock().await = "Stopped".to_string();
    Ok(())
}

#[tauri::command]
pub async fn set_accept_delay(state: tauri::State<'_, AppState>, delay_seconds: u64) -> Result<(), String> {
    *state.accept_delay_seconds.lock().await = delay_seconds;
    Ok(())
}

#[tauri::command]
pub async fn get_accept_delay(state: tauri::State<'_, AppState>) -> Result<u64, String> {
    Ok(*state.accept_delay_seconds.lock().await)
}

#[tauri::command]
pub async fn is_running(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.is_running.lock().await)
}

#[tauri::command]
pub async fn is_match_found(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(*state.match_found.lock().await)
}
