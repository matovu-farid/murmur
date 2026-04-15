mod audio;
mod config;
mod transcription;
mod ai;
mod input;
mod history;
mod tray;

use config::settings::{AppConfig, load_config, save_config, config_dir, models_dir, TranscriptionMode};
use history::{HistoryStore, TranscriptionEntry};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub history: Mutex<HistoryStore>,
}

// ---------------------------------------------------------------------------
// Config commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_config(state: State<Arc<AppState>>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(state: State<Arc<AppState>>, config: AppConfig) -> Result<(), String> {
    save_config(&config)?;
    *state.config.lock().unwrap() = config;
    Ok(())
}

// ---------------------------------------------------------------------------
// History commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_history(state: State<Arc<AppState>>) -> Result<Vec<TranscriptionEntry>, String> {
    state.history.lock().unwrap().get_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn search_history(state: State<Arc<AppState>>, query: String) -> Result<Vec<TranscriptionEntry>, String> {
    state.history.lock().unwrap().search(&query).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history_entry(state: State<Arc<AppState>>, id: i64) -> Result<(), String> {
    state.history.lock().unwrap().delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: State<Arc<AppState>>) -> Result<(), String> {
    state.history.lock().unwrap().clear().map_err(|e| e.to_string())
}

#[tauri::command]
fn export_history(state: State<Arc<AppState>>) -> Result<String, String> {
    state.history.lock().unwrap().export_json().map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Accessibility commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn check_accessibility_permission() -> bool {
    ai::context::has_accessibility_permission()
}

#[tauri::command]
fn open_accessibility_settings_cmd() {
    ai::context::open_accessibility_settings();
}

#[tauri::command]
fn open_accessibility_pane_cmd() {
    ai::context::open_accessibility_pane();
}

#[tauri::command]
fn check_microphone_permission() -> bool {
    // macOS handles microphone permissions automatically via system dialog
    // when the app first tries to access the microphone through cpal
    true
}

// ---------------------------------------------------------------------------
// Model download command
// ---------------------------------------------------------------------------

#[tauri::command]
async fn download_model(app: tauri::AppHandle, model_name: String) -> Result<String, String> {
    let dir = models_dir();
    let app_clone = app.clone();
    transcription::whisper_local::download_model(&model_name, &dir, move |downloaded, total| {
        let _ = app_clone.emit(
            "model-download-progress",
            serde_json::json!({
                "downloaded": downloaded,
                "total": total,
            }),
        );
    })
    .await
    .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Dictation pipeline
// ---------------------------------------------------------------------------

fn start_dictation_pipeline(app_handle: tauri::AppHandle, state: Arc<AppState>) {
    // Channel for hotkey events → recording thread
    let (hotkey_tx, hotkey_rx) = std::sync::mpsc::channel::<input::hotkey::HotkeyEvent>();
    // Channel for recorded samples → processing thread
    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<(Vec<f32>, u32, f32)>();

    // Recording thread — owns the Recorder and cpal::Stream (both !Send)
    let app_rec = app_handle.clone();
    std::thread::spawn(move || {
        let recorder = audio::Recorder::new();
        let mut active_stream: Option<cpal::Stream> = None;

        while let Ok(event) = hotkey_rx.recv() {
            match event {
                input::hotkey::HotkeyEvent::RecordStart => {
                    let _ = app_rec.emit("dictation-state", "recording");
                    match recorder.start() {
                        Ok(stream) => { active_stream = Some(stream); }
                        Err(e) => {
                            eprintln!("Failed to start recording: {}", e);
                            let _ = app_rec.emit("dictation-error", e);
                        }
                    }
                }
                input::hotkey::HotkeyEvent::RecordStop => {
                    let samples = recorder.stop();
                    active_stream = None;

                    let sample_rate = recorder.sample_rate();
                    let duration = audio::Recorder::duration_secs(&samples, sample_rate);

                    if duration < audio::Recorder::MIN_DURATION {
                        let _ = app_rec.emit("dictation-state", "idle");
                        continue;
                    }

                    let _ = app_rec.emit("dictation-state", "processing");
                    audio_tx.send((samples, sample_rate, duration)).ok();
                }
            }
        }
    });

    // Processing thread — picks up recorded audio and spawns async tasks
    let app_proc = app_handle.clone();
    let st_proc = state.clone();
    std::thread::spawn(move || {
        while let Ok((samples, sample_rate, duration)) = audio_rx.recv() {
            let app2 = app_proc.clone();
            let st2 = st_proc.clone();
            tauri::async_runtime::spawn(async move {
                process_recording(app2, st2, samples, sample_rate, duration).await;
            });
        }
    });

    // Hotkey monitor thread — sends events to the recording thread via channel
    std::thread::spawn(move || {
        let _monitor = input::hotkey::start_fn_key_monitor(Box::new(move |event| {
            hotkey_tx.send(event).ok();
        }));
        loop { std::thread::park(); }
    });
}

// ---------------------------------------------------------------------------
// Recording processor (runs on async runtime)
// ---------------------------------------------------------------------------

async fn process_recording(
    app: tauri::AppHandle,
    state: Arc<AppState>,
    samples: Vec<f32>,
    sample_rate: u32,
    duration: f32,
) {
    // 1. Preprocess
    let processed = audio::preprocessing::preprocess(&samples, sample_rate);
    if processed.is_empty() {
        let _ = app.emit("dictation-state", "idle");
        return;
    }

    // 2. Read config snapshot
    let cfg = state.config.lock().unwrap().clone();

    // 3. Transcribe
    let _ = app.emit("dictation-state", "transcribing");
    let raw_text = match cfg.transcription.mode {
        TranscriptionMode::Api => {
            transcription::whisper_api::transcribe_api(
                &processed,
                sample_rate,
                &cfg.transcription.api_key,
            )
            .await
        }
        TranscriptionMode::Local => {
            let model_path =
                models_dir().join(format!("ggml-{}.bin", cfg.transcription.model));
            let mp = model_path.to_string_lossy().to_string();
            let audio = processed.clone();
            tokio::task::spawn_blocking(move || {
                transcription::whisper_local::transcribe_local(&audio, &mp)
            })
            .await
            .unwrap_or_else(|e| {
                Err(transcription::TranscribeError::ModelError(e.to_string()))
            })
        }
    };

    let raw_text = match raw_text {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Transcription error: {}", e);
            let _ = app.emit("dictation-error", e.to_string());
            let _ = app.emit("dictation-state", "idle");
            return;
        }
    };

    if raw_text.trim().is_empty() {
        let _ = app.emit("dictation-state", "idle");
        return;
    }

    // 4. Process voice commands
    let cmd_result = ai::commands::process_commands(&raw_text);
    if cmd_result.should_stop {
        let _ = app.emit("dictation-state", "idle");
        return;
    }

    let text_after_commands = cmd_result.text.clone();
    if text_after_commands.is_empty() {
        let _ = app.emit("dictation-state", "idle");
        return;
    }

    // 5. AI cleanup (if enabled)
    let final_text = if cfg.ai_cleanup.enabled && !cfg.transcription.api_key.is_empty() {
        let _ = app.emit("dictation-state", "cleaning");

        let context = if cfg.general.context_aware {
            let ctx = ai::context::read_cursor_context(200);
            if ctx.is_empty() { None } else { Some(ctx) }
        } else {
            None
        };

        let instructions = if cfg.ai_cleanup.custom_instructions.is_empty() {
            None
        } else {
            Some(cfg.ai_cleanup.custom_instructions.as_str())
        };

        match ai::cleanup::cleanup_text(
            &text_after_commands,
            context.as_deref(),
            instructions,
            &cfg.transcription.api_key,
        )
        .await
        {
            Ok(cleaned) => cleaned,
            Err(e) => {
                eprintln!("Cleanup error: {}", e);
                text_after_commands.clone()
            }
        }
    } else {
        text_after_commands.clone()
    };

    // 6. Insert at cursor
    let _ = app.emit("dictation-state", "inserting");
    if let Err(e) = input::insertion::insert_at_cursor(&final_text) {
        eprintln!("Insert error: {}", e);
        let _ = app.emit("dictation-error", e);
    }

    // 7. Log to history
    if let Err(e) = state.history.lock().unwrap().insert(&raw_text, &final_text, duration) {
        eprintln!("History insert error: {}", e);
    }

    let _ = app.emit("dictation-state", "idle");
    let _ = app.emit(
        "dictation-result",
        serde_json::json!({ "raw": raw_text, "final": final_text, "duration": duration }),
    );
}

// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

pub fn run() {
    let config = load_config();

    let db_path = config_dir().join("history.db");
    let history = HistoryStore::new(
        db_path.to_str().expect("Invalid history DB path"),
    )
    .expect("Failed to open history database");

    let state = Arc::new(AppState {
        config: Mutex::new(config),
        history: Mutex::new(history),
    });

    let state_for_pipeline = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            get_history,
            search_history,
            delete_history_entry,
            clear_history,
            export_history,
            check_accessibility_permission,
            open_accessibility_settings_cmd,
            open_accessibility_pane_cmd,
            check_microphone_permission,
            download_model,
        ])
        .setup(move |app| {
            tray::setup_tray(app)?;

            // Create the overlay window (transparent, always on top)
            let _overlay = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("/#/".into()),
            )
            .title("Murmur Overlay")
            .decorations(false)
            .always_on_top(true)
            .inner_size(400.0, 100.0)
            .skip_taskbar(true)
            .build()?;

            // Check if first run -> show onboarding
            {
                let cfg = load_config();
                if cfg.general.first_run {
                    let _onboarding = tauri::WebviewWindowBuilder::new(
                        app,
                        "onboarding",
                        tauri::WebviewUrl::App("/#/onboarding".into()),
                    )
                    .title("Welcome to Murmur")
                    .inner_size(500.0, 600.0)
                    .center()
                    .build()?;

                    let mut updated = cfg;
                    updated.general.first_run = false;
                    save_config(&updated).ok();
                }
            }

            let app_handle = app.handle().clone();
            start_dictation_pipeline(app_handle, state_for_pipeline);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
