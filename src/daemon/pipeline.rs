use crate::ai;
use crate::audio;
use crate::config::settings::{config_dir, models_dir, AppConfig, TranscriptionMode};
use crate::history::HistoryStore;
use crate::input;
use crate::notify;
use crate::transcription;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub async fn run(config: AppConfig) -> Result<()> {
    notify::notify_info("Murmur is listening");
    tracing::info!("Dictation pipeline starting");

    let history = Arc::new(Mutex::new(HistoryStore::new(
        config_dir().join("history.db").to_str().unwrap(),
    )?));

    let (hotkey_tx, hotkey_rx) = std::sync::mpsc::channel::<input::hotkey::HotkeyEvent>();
    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<(Vec<f32>, u32, f32)>();

    // Recording thread
    std::thread::spawn(move || {
        let recorder = audio::Recorder::new();
        let mut _active_stream: Option<cpal::Stream> = None;

        while let Ok(event) = hotkey_rx.recv() {
            match event {
                input::hotkey::HotkeyEvent::RecordStart => {
                    tracing::info!("Recording started");
                    match recorder.start() {
                        Ok(stream) => {
                            _active_stream = Some(stream);
                        }
                        Err(e) => {
                            tracing::error!("Failed to start recording: {}", e);
                            notify::notify_error(&format!("Recording failed: {}", e));
                        }
                    }
                }
                input::hotkey::HotkeyEvent::RecordStop => {
                    let samples = recorder.stop();
                    _active_stream = None;

                    let sample_rate = recorder.sample_rate();
                    let duration = audio::Recorder::duration_secs(&samples, sample_rate);

                    if duration < audio::Recorder::MIN_DURATION {
                        tracing::debug!("Recording too short ({:.2}s)", duration);
                        continue;
                    }

                    tracing::info!("Recording stopped: {:.2}s", duration);
                    audio_tx.send((samples, sample_rate, duration)).ok();
                }
            }
        }
    });

    // Processing thread
    let history_proc = history.clone();
    let config_proc = Arc::new(config);
    let runtime_handle = tokio::runtime::Handle::current();
    std::thread::spawn(move || {
        while let Ok((samples, sample_rate, duration)) = audio_rx.recv() {
            let cfg = config_proc.clone();
            let hist = history_proc.clone();
            runtime_handle.spawn(async move {
                process_recording(cfg, hist, samples, sample_rate, duration).await;
            });
        }
    });

    // Hotkey monitor (runs CFRunLoop on its own thread internally)
    let _monitor = input::hotkey::start_fn_key_monitor(Box::new(move |event| {
        hotkey_tx.send(event).ok();
    }));

    // Block until SIGTERM
    let (term_tx, term_rx) = std::sync::mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        tracing::info!("Received shutdown signal");
        let _ = term_tx.send(());
    })
    .ok();
    let _ = term_rx.recv();

    notify::notify_info("Murmur stopped");
    Ok(())
}

async fn process_recording(
    config: Arc<AppConfig>,
    history: Arc<Mutex<HistoryStore>>,
    samples: Vec<f32>,
    sample_rate: u32,
    duration: f32,
) {
    let processed = audio::preprocessing::preprocess(&samples, sample_rate);
    if processed.is_empty() {
        return;
    }

    let raw_text = match config.transcription.mode {
        TranscriptionMode::Api => {
            transcription::whisper_api::transcribe_api(
                &processed,
                sample_rate,
                &config.transcription.api_key,
            )
            .await
        }
        TranscriptionMode::Local => {
            let model_path = models_dir().join(format!("ggml-{}.bin", config.transcription.model));
            let mp = model_path.to_string_lossy().to_string();
            let audio = processed.clone();
            tokio::task::spawn_blocking(move || {
                transcription::whisper_local::transcribe_local(&audio, &mp)
            })
            .await
            .unwrap_or_else(|e| Err(transcription::TranscribeError::ModelError(e.to_string())))
        }
    };

    let raw_text = match raw_text {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Transcription failed: {}", e);
            notify::notify_error(&format!("Transcription failed: {}", e));
            return;
        }
    };

    if raw_text.trim().is_empty() {
        return;
    }

    let cmd_result = ai::commands::process_commands(&raw_text);
    if cmd_result.should_stop {
        return;
    }
    let text_after_commands = cmd_result.text.clone();
    if text_after_commands.is_empty() {
        return;
    }

    let final_text = if config.ai_cleanup.enabled && !config.transcription.api_key.is_empty() {
        let context = if config.general.context_aware {
            let ctx = ai::context::read_cursor_context(200);
            if ctx.is_empty() {
                None
            } else {
                Some(ctx)
            }
        } else {
            None
        };
        let instructions = if config.ai_cleanup.custom_instructions.is_empty() {
            None
        } else {
            Some(config.ai_cleanup.custom_instructions.as_str())
        };
        match ai::cleanup::cleanup_text(
            &text_after_commands,
            context.as_deref(),
            instructions,
            &config.transcription.api_key,
        )
        .await
        {
            Ok(cleaned) => cleaned,
            Err(e) => {
                tracing::warn!("AI cleanup failed: {}", e);
                text_after_commands.clone()
            }
        }
    } else {
        text_after_commands.clone()
    };

    if let Err(e) = input::insertion::insert_at_cursor(&final_text) {
        tracing::error!("Insert failed: {}", e);
        notify::notify_error(&format!("Insert failed: {}", e));
    }

    if let Err(e) = history
        .lock()
        .unwrap()
        .insert(&raw_text, &final_text, duration)
    {
        tracing::warn!("History insert failed: {}", e);
    }

    tracing::info!("Inserted {} chars", final_text.len());
}
