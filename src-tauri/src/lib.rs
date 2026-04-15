mod audio;
mod config;

use config::settings::{AppConfig, load_config, save_config};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[tauri::command]
fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    save_config(&config)?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

pub fn run() {
    let config = load_config();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            config: Mutex::new(config),
        })
        .invoke_handler(tauri::generate_handler![get_config, update_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
