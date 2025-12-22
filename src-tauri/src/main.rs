// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod features;
mod data;

use core::models::AppState;
use core::settings::load_settings;
use core::lcu::check_league_connection_internal;
use features::auto_accept::{auto_accept_loop, start_auto_accept, stop_auto_accept, set_accept_delay, get_accept_delay, is_running, is_match_found};
use features::auto_hover::{set_auto_hover, get_auto_hover};
use features::auto_ban::{set_auto_ban, get_auto_ban};
use features::auto_lock::{set_auto_select, get_auto_select};
use features::champion_select::{champion_select_loop, get_champion_preferences, set_champion_preferences, set_role_preferences, get_role_preferences};

#[tauri::command]
async fn check_league_connection(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(check_league_connection_internal(&state).await)
}

#[tauri::command]
async fn get_status(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.status.lock().await.clone())
}

#[tauri::command]
async fn get_champions(_state: tauri::State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    // Return static champion list - works without League Client
    Ok(data::champions::get_all_champions_static())
}

fn main() {
    // Load settings
    let settings = load_settings();
    let app_state = AppState::new(settings);
    let state_clone = app_state.clone();
    let champ_select_state = app_state.clone();

    // Start background tasks
    tauri::async_runtime::spawn(async move {
        auto_accept_loop(state_clone).await;
    });

    tauri::async_runtime::spawn(async move {
        champion_select_loop(champ_select_state).await;
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_league_connection,
            get_status,
            is_running,
            is_match_found,
            start_auto_accept,
            stop_auto_accept,
            set_accept_delay,
            get_accept_delay,
            get_champion_preferences,
            set_champion_preferences,
            set_role_preferences,
            get_role_preferences,
            set_auto_hover,
            get_auto_hover,
            set_auto_select,
            get_auto_select,
            set_auto_ban,
            get_auto_ban,
            get_champions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
