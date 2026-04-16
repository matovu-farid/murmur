use super::{Command, ConfigAction, HistoryAction};
use crate::config::settings::{config_dir, load_config, save_config};
use crate::daemon::{launchd, pipeline, process};
use crate::history::HistoryStore;
use crate::notify;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde_json::Value;

pub fn dispatch(command: Command) -> Result<()> {
    match command {
        Command::Start { foreground } => start(foreground),
        Command::Stop => stop(),
        Command::Status => status(),
        Command::Config { action } => config(action),
        Command::History { action } => history(action),
        Command::Install => install(),
        Command::Uninstall => uninstall(),
        Command::Download { model } => download(&model),
    }
}

fn start(foreground: bool) -> Result<()> {
    if let Some(pid) = process::read_pid_file() {
        if process::is_alive(pid) {
            return Err(anyhow!("Murmur is already running (PID {})", pid));
        }
        process::remove_pid_file().ok();
    }

    if foreground {
        let _guard = notify::init_logging(&config_dir(), true);
        process::write_pid_file()?;
        let cfg = load_config();
        let runtime = tokio::runtime::Runtime::new().context("Failed to create tokio runtime")?;
        runtime.block_on(pipeline::run(cfg))?;
        process::remove_pid_file().ok();
        Ok(())
    } else {
        let pid = process::spawn_background()?;
        println!("{} Murmur started (PID {})", "✓".green(), pid);
        Ok(())
    }
}

fn stop() -> Result<()> {
    process::stop_daemon()?;
    println!("{} Murmur stopped", "✓".green());
    Ok(())
}

fn status() -> Result<()> {
    match process::daemon_status() {
        Some((pid, uptime)) => {
            println!("{} Murmur is running", "●".green());
            println!("  PID:    {}", pid);
            println!("  Uptime: {}", format_uptime(uptime));
        }
        None => println!("{} Murmur is not running", "○".red()),
    }
    Ok(())
}

fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 { format!("{}h {}m {}s", h, m, s) }
    else if m > 0 { format!("{}m {}s", m, s) }
    else { format!("{}s", s) }
}

fn config(action: Option<ConfigAction>) -> Result<()> {
    match action {
        None => crate::cli::wizard::run(),
        Some(ConfigAction::List) => {
            let cfg = load_config();
            let json = serde_json::to_string_pretty(&cfg)?;
            println!("{}", json);
            Ok(())
        }
        Some(ConfigAction::Get { key }) => config_get(&key),
        Some(ConfigAction::Set { key, value }) => config_set(&key, &value),
    }
}

fn config_get(key: &str) -> Result<()> {
    let cfg = load_config();
    let json = serde_json::to_value(&cfg)?;
    let value = lookup_dotted(&json, key).ok_or_else(|| anyhow!("Unknown key: {}", key))?;
    match value {
        Value::String(s) => println!("{}", s),
        v => println!("{}", v),
    }
    Ok(())
}

fn config_set(key: &str, value: &str) -> Result<()> {
    let cfg = load_config();
    let mut json = serde_json::to_value(&cfg)?;
    set_dotted(&mut json, key, value)?;
    let new_cfg: crate::config::settings::AppConfig = serde_json::from_value(json)
        .context("Invalid value for that key")?;
    save_config(&new_cfg).map_err(anyhow::Error::msg)?;
    println!("{} Set {} = {}", "✓".green(), key, value);
    Ok(())
}

fn lookup_dotted<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn set_dotted(value: &mut Value, key: &str, new_value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = value;
    for part in &parts[..parts.len() - 1] {
        current = current.get_mut(*part).ok_or_else(|| anyhow!("Unknown key path: {}", key))?;
    }
    let last = parts.last().unwrap();
    let parsed: Value = serde_json::from_str(new_value).unwrap_or_else(|_| Value::String(new_value.to_string()));
    let target = current.get_mut(*last).ok_or_else(|| anyhow!("Unknown key: {}", key))?;
    *target = parsed;
    Ok(())
}

fn history(action: Option<HistoryAction>) -> Result<()> {
    let store = HistoryStore::new(config_dir().join("history.db").to_str().unwrap())?;
    match action {
        None => {
            let entries = store.get_all()?;
            print_history(&entries.into_iter().take(20).collect::<Vec<_>>());
            Ok(())
        }
        Some(HistoryAction::Search { query }) => {
            let entries = store.search(&query)?;
            print_history(&entries);
            Ok(())
        }
        Some(HistoryAction::Clear) => {
            use dialoguer::Confirm;
            let confirm = Confirm::new()
                .with_prompt("Delete ALL transcription history?")
                .default(false)
                .interact()?;
            if confirm {
                store.clear()?;
                println!("{} History cleared", "✓".green());
            } else {
                println!("Cancelled");
            }
            Ok(())
        }
        Some(HistoryAction::Export) => {
            let json = store.export_json()?;
            println!("{}", json);
            Ok(())
        }
    }
}

fn print_history(entries: &[crate::history::TranscriptionEntry]) {
    if entries.is_empty() {
        println!("No transcriptions yet");
        return;
    }
    for entry in entries {
        let ts = entry.timestamp.split('T').next().unwrap_or(&entry.timestamp);
        let time = entry.timestamp
            .split('T').nth(1).unwrap_or("")
            .split('.').next().unwrap_or("")
            .split('+').next().unwrap_or("");
        println!(
            "{} {} {} {}",
            ts.dimmed(),
            time.dimmed(),
            format!("({:.1}s)", entry.duration_secs).dimmed(),
            entry.cleaned_text,
        );
    }
}

fn install() -> Result<()> {
    launchd::install()?;
    println!("{} Murmur installed - will start on login", "✓".green());
    Ok(())
}

fn uninstall() -> Result<()> {
    launchd::uninstall()?;
    println!("{} Murmur launchd service removed", "✓".green());
    Ok(())
}

fn download(model: &str) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    let dir = crate::config::settings::models_dir();
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let pb_clone = pb.clone();
    let runtime = tokio::runtime::Runtime::new()?;
    let result = runtime.block_on(crate::transcription::whisper_local::download_model(
        model,
        &dir,
        move |downloaded, total| {
            if pb_clone.length().unwrap_or(0) == 0 && total > 0 {
                pb_clone.set_length(total);
            }
            pb_clone.set_position(downloaded);
        },
    ));

    pb.finish();

    match result {
        Ok(path) => {
            println!("{} Model downloaded to {}", "✓".green(), path);
            Ok(())
        }
        Err(e) => Err(anyhow!("Download failed: {}", e)),
    }
}
