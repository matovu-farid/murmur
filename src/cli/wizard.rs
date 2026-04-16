use crate::config::settings::{save_config, HotkeyMode, TranscriptionMode};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

const MODELS: &[(&str, &str)] = &[
    ("tiny.en", "Tiny (40MB, fastest, lower quality)"),
    ("base.en", "Base (140MB, fast)"),
    ("small.en", "Small (460MB, balanced)"),
    ("medium.en", "Medium (1.5GB, recommended)"),
];

pub fn run() -> Result<()> {
    let theme = ColorfulTheme::default();
    let mut config = crate::config::settings::load_config();

    println!("\n  Murmur Configuration\n");

    let modes = ["Local (offline, private)", "API (OpenAI)"];
    let mode_idx = Select::with_theme(&theme)
        .with_prompt("Transcription mode")
        .default(if config.transcription.mode == TranscriptionMode::Local { 0 } else { 1 })
        .items(&modes)
        .interact()?;
    config.transcription.mode = if mode_idx == 0 { TranscriptionMode::Local } else { TranscriptionMode::Api };

    if config.transcription.mode == TranscriptionMode::Local {
        let model_labels: Vec<&str> = MODELS.iter().map(|(_, l)| *l).collect();
        let default_idx = MODELS.iter().position(|(name, _)| *name == config.transcription.model.as_str()).unwrap_or(3);
        let model_idx = Select::with_theme(&theme)
            .with_prompt("Whisper model")
            .default(default_idx)
            .items(&model_labels)
            .interact()?;
        config.transcription.model = MODELS[model_idx].0.to_string();
    }

    let api_key: String = Password::with_theme(&theme)
        .with_prompt("OpenAI API key (for AI cleanup, optional - press Enter to skip)")
        .allow_empty_password(true)
        .interact()?;
    if !api_key.is_empty() {
        config.transcription.api_key = api_key;
    }

    config.ai_cleanup.enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable AI cleanup?")
        .default(config.ai_cleanup.enabled)
        .interact()?;

    if config.ai_cleanup.enabled {
        let custom: String = Input::with_theme(&theme)
            .with_prompt("Custom cleanup instructions (optional)")
            .default(config.ai_cleanup.custom_instructions.clone())
            .allow_empty(true)
            .interact()?;
        config.ai_cleanup.custom_instructions = custom;
    }

    config.general.context_aware = Confirm::with_theme(&theme)
        .with_prompt("Enable context-aware mode (read text around cursor)?")
        .default(config.general.context_aware)
        .interact()?;

    config.voice_commands.enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable voice commands (period, new line, etc.)?")
        .default(config.voice_commands.enabled)
        .interact()?;

    let hotkey_modes = ["Hold to record (default)", "Press to toggle"];
    let hk_idx = Select::with_theme(&theme)
        .with_prompt("Hotkey mode")
        .default(if config.hotkey.mode == HotkeyMode::Hold { 0 } else { 1 })
        .items(&hotkey_modes)
        .interact()?;
    config.hotkey.mode = if hk_idx == 0 { HotkeyMode::Hold } else { HotkeyMode::Toggle };

    config.general.auto_start = Confirm::with_theme(&theme)
        .with_prompt("Auto-start on login?")
        .default(config.general.auto_start)
        .interact()?;

    config.general.first_run = false;

    save_config(&config).map_err(anyhow::Error::msg)?;
    println!("\n  Saved to ~/.config/murmur/config.json\n");

    if config.general.auto_start && !crate::daemon::launchd::is_installed() {
        println!("  Installing launchd service...");
        crate::daemon::launchd::install()?;
        println!("  ✓ Murmur will start on login");
    }

    Ok(())
}
