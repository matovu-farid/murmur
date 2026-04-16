# Murmur CLI Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate Murmur from a Tauri + React desktop app to a single Rust CLI binary that runs as a background daemon.

**Architecture:** Strip the Tauri/React layers. Move backend modules to a new project root. Add CLI (clap), daemon process management (PID files, launchd), interactive config wizard (dialoguer), and macOS notifications. Backend module code stays unchanged.

**Tech Stack:** Rust, clap, dialoguer, indicatif, nix, mac-notification-sys, tracing, plus existing dependencies (cpal, whisper-rs, reqwest, rusqlite, core-graphics).

**Spec:** `docs/superpowers/specs/2026-04-16-murmur-cli-design.md`

---

## File Structure

```
murmur/  (project root - was wipr/)
├── Cargo.toml                  # New, Tauri-free
├── src/                        # New location for all Rust source
│   ├── main.rs                 # NEW: CLI entry point
│   ├── lib.rs                  # NEW: re-exports for tests
│   ├── cli/
│   │   ├── mod.rs              # NEW
│   │   ├── commands.rs         # NEW: subcommand handlers
│   │   └── wizard.rs           # NEW: interactive config wizard
│   ├── daemon/
│   │   ├── mod.rs              # NEW
│   │   ├── process.rs          # NEW: PID file + fork
│   │   ├── pipeline.rs         # NEW: dictation pipeline (extracted from old lib.rs)
│   │   └── launchd.rs          # NEW: launchd plist install/uninstall
│   ├── notify.rs               # NEW: macOS notifications + logging setup
│   ├── audio/                  # MOVED from src-tauri/src/audio/ (unchanged)
│   ├── transcription/          # MOVED from src-tauri/src/transcription/ (unchanged)
│   ├── ai/                     # MOVED from src-tauri/src/ai/ (Tauri stripped)
│   ├── input/                  # MOVED from src-tauri/src/input/ (unchanged)
│   ├── config/                 # MOVED from src-tauri/src/config/ (unchanged)
│   └── history/                # MOVED from src-tauri/src/history/ (unchanged)
├── Homebrew/
│   └── murmur.rb               # MODIFIED: cask → formula
├── docs/                       # UNCHANGED
└── README.md
```

**Files to delete:**
- `src/` (the React frontend)
- `src-tauri/` directory after migration
- `package.json`, `package-lock.json`, `node_modules/`
- `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `index.html`, `dist/`

---

### Task 1: Create New Cargo.toml at Project Root

**Files:**
- Create: `Cargo.toml` (project root)

- [ ] **Step 1: Verify current state**

```bash
cd /Users/faridmatovu/projects/wipr
ls Cargo.toml 2>/dev/null && echo "EXISTS" || echo "MISSING"
```
Expected: MISSING (Cargo.toml is currently inside src-tauri/)

- [ ] **Step 2: Create new root Cargo.toml**

Create `/Users/faridmatovu/projects/wipr/Cargo.toml`:

```toml
[package]
name = "murmur"
version = "0.1.0"
edition = "2021"
description = "macOS voice-to-text dictation tool"
authors = [""]
license = "MIT"

[[bin]]
name = "murmur"
path = "src/main.rs"

[lib]
name = "murmur"
path = "src/lib.rs"

[dependencies]
# Audio
cpal = "0.15"
hound = "3.5"

# Transcription
whisper-rs = "0.16"

# HTTP
reqwest = { version = "0.12", features = ["json", "multipart", "stream"] }

# Storage
rusqlite = { version = "0.33", features = ["bundled"] }

# macOS system
core-graphics = "0.25"
core-foundation = "0.10"
mac-notification-sys = "0.6"

# Async
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
indicatif = "0.17"
colored = "2"

# Process management
nix = { version = "0.29", features = ["signal", "process"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Utilities
dirs = "6"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
regex-lite = "0.1"

[profile.release]
opt-level = 3
lto = true
strip = true
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat: add new Cargo.toml at project root for CLI binary"
```

---

### Task 2: Move Backend Modules to New src/

**Files:**
- Move: `src-tauri/src/audio/` → `src/audio/`
- Move: `src-tauri/src/transcription/` → `src/transcription/`
- Move: `src-tauri/src/ai/` → `src/ai/`
- Move: `src-tauri/src/input/` → `src/input/`
- Move: `src-tauri/src/config/` → `src/config/`
- Move: `src-tauri/src/history/` → `src/history/`

- [ ] **Step 1: Delete old React src/ directory**

```bash
cd /Users/faridmatovu/projects/wipr
git rm -rf src/
```
Expected: All React files removed.

- [ ] **Step 2: Move backend modules with git mv**

```bash
cd /Users/faridmatovu/projects/wipr
mkdir -p src
git mv src-tauri/src/audio src/audio
git mv src-tauri/src/transcription src/transcription
git mv src-tauri/src/ai src/ai
git mv src-tauri/src/input src/input
git mv src-tauri/src/config src/config
git mv src-tauri/src/history src/history
```

- [ ] **Step 3: Verify the move preserved all files**

```bash
ls src/audio src/transcription src/ai src/input src/config src/history
```
Expected: Each directory contains its module files (mod.rs, etc.)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: move backend modules to project root src/"
```

---

### Task 3: Create lib.rs and Strip Tauri-Specific Code

**Files:**
- Create: `src/lib.rs`
- Modify: `src/ai/context.rs` (remove if any Tauri imports)
- Modify: `src/audio/mod.rs`, `src/transcription/mod.rs`, etc. — verify no Tauri imports

- [ ] **Step 1: Create `src/lib.rs`**

```rust
pub mod audio;
pub mod transcription;
pub mod ai;
pub mod input;
pub mod config;
pub mod history;
pub mod cli;
pub mod daemon;
pub mod notify;
```

- [ ] **Step 2: Search for Tauri imports in moved files**

```bash
cd /Users/faridmatovu/projects/wipr
grep -rn "use tauri\|tauri::" src/audio src/transcription src/ai src/input src/config src/history 2>&1 || echo "No Tauri imports found"
```
Expected: "No Tauri imports found" (backend modules were already pure Rust).

- [ ] **Step 3: Verify the modules compile (placeholder check)**

The lib.rs references `cli`, `daemon`, and `notify` modules that don't exist yet. To verify the moved modules compile, temporarily comment out those lines:

```bash
sed -i '' 's/^pub mod cli;/\/\/ pub mod cli;/' src/lib.rs
sed -i '' 's/^pub mod daemon;/\/\/ pub mod daemon;/' src/lib.rs
sed -i '' 's/^pub mod notify;/\/\/ pub mod notify;/' src/lib.rs
cargo check --lib 2>&1 | tail -5
```
Expected: Compiles or shows only warnings about missing main.rs (that's fine).

- [ ] **Step 4: Restore the lib.rs**

```bash
sed -i '' 's/^\/\/ pub mod cli;/pub mod cli;/' src/lib.rs
sed -i '' 's/^\/\/ pub mod daemon;/pub mod daemon;/' src/lib.rs
sed -i '' 's/^\/\/ pub mod notify;/pub mod notify;/' src/lib.rs
```

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "feat: add lib.rs declaring all module roots"
```

---

### Task 4: Create Notification & Logging Module

**Files:**
- Create: `src/notify.rs`

- [ ] **Step 1: Write `src/notify.rs`**

```rust
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const APP_NAME: &str = "Murmur";

/// Initialize tracing with file rotation and optional stderr output (foreground mode).
/// Returns the worker guard that must be kept alive for log writes to flush.
pub fn init_logging(log_dir: &PathBuf, foreground: bool) -> tracing_appender::non_blocking::WorkerGuard {
    std::fs::create_dir_all(log_dir).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "murmur.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false);

    if foreground {
        let stderr_layer = fmt::layer().with_writer(std::io::stderr);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stderr_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    }

    guard
}

/// Send a macOS notification with the given subtitle and message.
fn send_notification(subtitle: &str, message: &str) {
    if let Err(e) = mac_notification_sys::send_notification(
        APP_NAME,
        Some(subtitle),
        message,
        None,
    ) {
        tracing::warn!("Failed to send notification: {}", e);
    }
}

pub fn notify_info(message: &str) {
    tracing::info!("notify_info: {}", message);
    send_notification("", message);
}

pub fn notify_error(message: &str) {
    tracing::error!("notify_error: {}", message);
    send_notification("Error", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_creates_log_dir() {
        let tmp = std::env::temp_dir().join(format!("murmur_test_log_{}", std::process::id()));
        let _guard = init_logging(&tmp, false);
        assert!(tmp.exists());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd /Users/faridmatovu/projects/wipr
cargo check --lib 2>&1 | grep -E "error" | head -5
```
Expected: No errors (or only errors about missing cli/daemon modules — fix in next tasks).

- [ ] **Step 3: Run notify tests**

```bash
cargo test notify
```
Expected: 1 test passes.

- [ ] **Step 4: Commit**

```bash
git add src/notify.rs
git commit -m "feat: add notification and structured logging module"
```

---

### Task 5: Create CLI Module Skeleton with clap

**Files:**
- Create: `src/cli/mod.rs`
- Create: `src/cli/commands.rs`

- [ ] **Step 1: Create `src/cli/mod.rs`**

```rust
pub mod commands;
pub mod wizard;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "murmur", version, about = "macOS voice-to-text dictation daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the dictation daemon
    Start {
        /// Run in foreground (don't fork)
        #[arg(long)]
        foreground: bool,
    },
    /// Stop the running daemon
    Stop,
    /// Show daemon status
    Status,
    /// Configure Murmur (interactive wizard if no subcommand)
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Manage transcription history
    History {
        #[command(subcommand)]
        action: Option<HistoryAction>,
    },
    /// Install launchd service for auto-start on login
    Install,
    /// Remove launchd service
    Uninstall,
    /// Download a Whisper model
    Download {
        /// Model name (e.g., medium.en, base.en, tiny.en)
        model: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// List all settings
    List,
    /// Get a setting value
    Get {
        /// Dotted key path (e.g., transcription.mode)
        key: String,
    },
    /// Set a setting value
    Set {
        /// Dotted key path
        key: String,
        /// New value
        value: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum HistoryAction {
    /// Search transcriptions
    Search {
        /// Search query
        query: String,
    },
    /// Clear all history
    Clear,
    /// Export history as JSON to stdout
    Export,
}
```

- [ ] **Step 2: Create `src/cli/commands.rs` with stub handlers**

```rust
use super::{Command, ConfigAction, HistoryAction};
use anyhow::Result;

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

fn start(_foreground: bool) -> Result<()> {
    println!("start: not yet implemented");
    Ok(())
}

fn stop() -> Result<()> {
    println!("stop: not yet implemented");
    Ok(())
}

fn status() -> Result<()> {
    println!("status: not yet implemented");
    Ok(())
}

fn config(_action: Option<ConfigAction>) -> Result<()> {
    println!("config: not yet implemented");
    Ok(())
}

fn history(_action: Option<HistoryAction>) -> Result<()> {
    println!("history: not yet implemented");
    Ok(())
}

fn install() -> Result<()> {
    println!("install: not yet implemented");
    Ok(())
}

fn uninstall() -> Result<()> {
    println!("uninstall: not yet implemented");
    Ok(())
}

fn download(_model: &str) -> Result<()> {
    println!("download: not yet implemented");
    Ok(())
}
```

- [ ] **Step 3: Add `anyhow` to Cargo.toml**

Add to `[dependencies]` in `Cargo.toml`:
```toml
anyhow = "1"
```

- [ ] **Step 4: Create stub `src/cli/wizard.rs`**

```rust
// Wizard implementation in Task 9
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/cli/
git commit -m "feat: add CLI module with clap subcommands and stub handlers"
```

---

### Task 6: Create main.rs Entry Point

**Files:**
- Create: `src/main.rs`

- [ ] **Step 1: Write `src/main.rs`**

```rust
use clap::Parser;
use murmur::cli::{Cli, commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => {
            if let Err(e) = commands::dispatch(cmd) {
                eprintln!("Error: {:#}", e);
                std::process::exit(1);
            }
        }
        None => {
            // No subcommand: print help
            use clap::CommandFactory;
            let _ = Cli::command().print_help();
            println!();
        }
    }
}
```

- [ ] **Step 2: Verify it builds**

```bash
cd /Users/faridmatovu/projects/wipr
cargo build 2>&1 | tail -3
```
Expected: Build succeeds.

- [ ] **Step 3: Test the binary runs**

```bash
./target/debug/murmur --help 2>&1 | head -20
```
Expected: Help text showing all subcommands.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add main.rs CLI entry point"
```

---

### Task 7: Implement Daemon Process Management

**Files:**
- Create: `src/daemon/mod.rs`
- Create: `src/daemon/process.rs`

- [ ] **Step 1: Create `src/daemon/mod.rs`**

```rust
pub mod process;
pub mod pipeline;
pub mod launchd;
```

- [ ] **Step 2: Write `src/daemon/process.rs`**

```rust
use crate::config::settings::config_dir;
use anyhow::{anyhow, Context, Result};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs;
use std::path::PathBuf;

fn pid_file_path() -> PathBuf {
    config_dir().join("murmur.pid")
}

/// Write the current process's PID to the PID file.
pub fn write_pid_file() -> Result<()> {
    let path = pid_file_path();
    fs::create_dir_all(path.parent().unwrap()).ok();
    let pid = std::process::id();
    fs::write(&path, pid.to_string()).context("Failed to write PID file")?;
    Ok(())
}

/// Read the PID from the PID file. Returns None if no file or invalid contents.
pub fn read_pid_file() -> Option<i32> {
    let path = pid_file_path();
    let content = fs::read_to_string(&path).ok()?;
    content.trim().parse().ok()
}

/// Remove the PID file.
pub fn remove_pid_file() -> Result<()> {
    let path = pid_file_path();
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove PID file")?;
    }
    Ok(())
}

/// Check if a process with the given PID is alive.
pub fn is_alive(pid: i32) -> bool {
    kill(Pid::from_raw(pid), None).is_ok()
}

/// Send SIGTERM to the daemon process.
pub fn stop_daemon() -> Result<()> {
    let pid = read_pid_file().ok_or_else(|| anyhow!("Daemon is not running (no PID file)"))?;
    if !is_alive(pid) {
        remove_pid_file()?;
        return Err(anyhow!("Daemon is not running (stale PID file removed)"));
    }
    kill(Pid::from_raw(pid), Signal::SIGTERM)
        .with_context(|| format!("Failed to send SIGTERM to PID {}", pid))?;
    remove_pid_file()?;
    Ok(())
}

/// Spawn the current binary with `start --foreground` to run as a background process.
pub fn spawn_background() -> Result<u32> {
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let child = std::process::Command::new(&exe)
        .args(["start", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn background process")?;
    Ok(child.id())
}

/// Get daemon status: running PID and uptime in seconds.
pub fn daemon_status() -> Option<(i32, u64)> {
    let pid = read_pid_file()?;
    if !is_alive(pid) {
        return None;
    }
    let uptime = pid_file_path()
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((pid, uptime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_alive_for_current_process() {
        assert!(is_alive(std::process::id() as i32));
    }

    #[test]
    fn test_is_alive_for_invalid_pid() {
        // PID 999999 is unlikely to exist
        assert!(!is_alive(999999));
    }

    #[test]
    fn test_read_pid_file_missing() {
        // Use a unique path; this just verifies the None branch
        let original = pid_file_path();
        let backup = original.with_extension("pid.bak");
        if original.exists() {
            fs::rename(&original, &backup).ok();
        }
        assert_eq!(read_pid_file(), None);
        if backup.exists() {
            fs::rename(&backup, &original).ok();
        }
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr
cargo test daemon::process
```
Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/daemon/
git commit -m "feat: add daemon process management with PID files and SIGTERM"
```

---

### Task 8: Implement Dictation Pipeline

**Files:**
- Create: `src/daemon/pipeline.rs`

- [ ] **Step 1: Write `src/daemon/pipeline.rs`**

```rust
use crate::audio;
use crate::ai;
use crate::config::settings::{config_dir, models_dir, AppConfig, TranscriptionMode};
use crate::history::HistoryStore;
use crate::input;
use crate::notify;
use crate::transcription;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Run the dictation pipeline. This blocks the calling thread.
pub async fn run(config: AppConfig) -> Result<()> {
    notify::notify_info("Murmur is listening");
    tracing::info!("Dictation pipeline starting");

    let history = Arc::new(Mutex::new(
        HistoryStore::new(config_dir().join("history.db").to_str().unwrap())?,
    ));

    let (hotkey_tx, hotkey_rx) = std::sync::mpsc::channel::<input::hotkey::HotkeyEvent>();
    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<(Vec<f32>, u32, f32)>();

    // Recording thread — owns Recorder and cpal::Stream (both !Send)
    std::thread::spawn(move || {
        let recorder = audio::Recorder::new();
        let mut active_stream: Option<cpal::Stream> = None;

        while let Ok(event) = hotkey_rx.recv() {
            match event {
                input::hotkey::HotkeyEvent::RecordStart => {
                    tracing::info!("Recording started");
                    match recorder.start() {
                        Ok(stream) => { active_stream = Some(stream); }
                        Err(e) => {
                            tracing::error!("Failed to start recording: {}", e);
                            notify::notify_error(&format!("Recording failed: {}", e));
                        }
                    }
                }
                input::hotkey::HotkeyEvent::RecordStop => {
                    let samples = recorder.stop();
                    active_stream = None;

                    let sample_rate = recorder.sample_rate();
                    let duration = audio::Recorder::duration_secs(&samples, sample_rate);

                    if duration < audio::Recorder::MIN_DURATION {
                        tracing::debug!("Recording too short ({:.2}s), discarding", duration);
                        continue;
                    }

                    tracing::info!("Recording stopped: {:.2}s, {} samples", duration, samples.len());
                    audio_tx.send((samples, sample_rate, duration)).ok();
                }
            }
        }
    });

    // Processing thread — runs async tasks
    let history_proc = history.clone();
    let config_proc = Arc::new(config);
    std::thread::spawn(move || {
        while let Ok((samples, sample_rate, duration)) = audio_rx.recv() {
            let cfg = config_proc.clone();
            let hist = history_proc.clone();
            tokio::runtime::Handle::current().spawn(async move {
                process_recording(cfg, hist, samples, sample_rate, duration).await;
            });
        }
    });

    // Hotkey monitor — runs CFRunLoop on its own thread
    let _monitor = input::hotkey::start_fn_key_monitor(Box::new(move |event| {
        hotkey_tx.send(event).ok();
    }));

    // Block forever (until SIGTERM)
    let (term_tx, term_rx) = std::sync::mpsc::channel::<()>();
    ctrlc::set_handler(move || {
        tracing::info!("Received shutdown signal");
        let _ = term_tx.send(());
    }).ok();
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
        tracing::debug!("Preprocessing produced empty audio");
        return;
    }

    let raw_text = match config.transcription.mode {
        TranscriptionMode::Api => {
            tracing::info!("Transcribing via OpenAI API");
            transcription::whisper_api::transcribe_api(
                &processed, sample_rate, &config.transcription.api_key,
            ).await
        }
        TranscriptionMode::Local => {
            tracing::info!("Transcribing via local Whisper");
            let model_path = models_dir().join(format!("ggml-{}.bin", config.transcription.model));
            let mp = model_path.to_string_lossy().to_string();
            let audio = processed.clone();
            tokio::task::spawn_blocking(move || {
                transcription::whisper_local::transcribe_local(&audio, &mp)
            }).await.unwrap_or_else(|e| {
                Err(transcription::TranscribeError::ModelError(e.to_string()))
            })
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
        tracing::debug!("Transcription returned empty text");
        return;
    }

    let cmd_result = ai::commands::process_commands(&raw_text);
    if cmd_result.should_stop {
        tracing::info!("Voice command 'stop listening' detected");
        return;
    }
    let text_after_commands = cmd_result.text.clone();
    if text_after_commands.is_empty() {
        return;
    }

    let final_text = if config.ai_cleanup.enabled && !config.transcription.api_key.is_empty() {
        let context = if config.general.context_aware {
            let ctx = ai::context::read_cursor_context(200);
            if ctx.is_empty() { None } else { Some(ctx) }
        } else { None };
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
        ).await {
            Ok(cleaned) => cleaned,
            Err(e) => {
                tracing::warn!("AI cleanup failed, using raw text: {}", e);
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

    if let Err(e) = history.lock().unwrap().insert(&raw_text, &final_text, duration) {
        tracing::warn!("Failed to log to history: {}", e);
    }

    tracing::info!("Inserted {} chars: '{}'", final_text.len(), final_text);
}
```

- [ ] **Step 2: Add ctrlc dependency**

Add to `Cargo.toml` `[dependencies]`:
```toml
ctrlc = "3"
```

- [ ] **Step 3: Verify it compiles**

```bash
cd /Users/faridmatovu/projects/wipr
cargo check 2>&1 | grep "error" | head -5
```
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/daemon/pipeline.rs
git commit -m "feat: extract dictation pipeline from old Tauri lib.rs"
```

---

### Task 9: Implement Launchd Integration

**Files:**
- Create: `src/daemon/launchd.rs`

- [ ] **Step 1: Write `src/daemon/launchd.rs`**

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const PLIST_LABEL: &str = "com.murmur.app";

fn plist_path() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", PLIST_LABEL))
}

fn generate_plist(exe_path: &str, log_dir: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>start</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_dir}/launchd.out.log</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/launchd.err.log</string>
</dict>
</plist>
"#,
        label = PLIST_LABEL,
        exe = exe_path,
        log_dir = log_dir,
    )
}

pub fn install() -> Result<()> {
    let exe = std::env::current_exe()
        .context("Failed to get current executable path")?
        .to_string_lossy()
        .to_string();

    let log_dir = crate::config::settings::config_dir()
        .to_string_lossy()
        .to_string();

    let plist = generate_plist(&exe, &log_dir);
    let plist_file = plist_path();
    fs::create_dir_all(plist_file.parent().unwrap())?;
    fs::write(&plist_file, plist).context("Failed to write plist")?;

    let output = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_file)
        .output()
        .context("Failed to run launchctl load")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("launchctl load failed: {}", err);
    }

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let plist_file = plist_path();

    if plist_file.exists() {
        let output = Command::new("launchctl")
            .args(["unload", "-w"])
            .arg(&plist_file)
            .output()
            .context("Failed to run launchctl unload")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("launchctl unload returned: {}", err);
        }

        fs::remove_file(&plist_file).context("Failed to remove plist")?;
    }

    Ok(())
}

pub fn is_installed() -> bool {
    plist_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_plist_contains_required_keys() {
        let plist = generate_plist("/usr/local/bin/murmur", "/tmp/murmur");
        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains("com.murmur.app"));
        assert!(plist.contains("/usr/local/bin/murmur"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert!(plist.contains("<string>start</string>"));
        assert!(plist.contains("<string>--foreground</string>"));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr
cargo test launchd
```
Expected: 1 test passes.

- [ ] **Step 3: Commit**

```bash
git add src/daemon/launchd.rs
git commit -m "feat: add launchd plist install/uninstall for auto-start on login"
```

---

### Task 10: Implement Interactive Config Wizard

**Files:**
- Modify: `src/cli/wizard.rs`

- [ ] **Step 1: Write `src/cli/wizard.rs`**

```rust
use crate::config::settings::{
    save_config, AppConfig, HotkeyMode, TranscriptionMode,
};
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

    // Transcription mode
    let modes = ["Local (offline, private)", "API (OpenAI)"];
    let mode_idx = Select::with_theme(&theme)
        .with_prompt("Transcription mode")
        .default(if config.transcription.mode == TranscriptionMode::Local { 0 } else { 1 })
        .items(&modes)
        .interact()?;
    config.transcription.mode = if mode_idx == 0 {
        TranscriptionMode::Local
    } else {
        TranscriptionMode::Api
    };

    // Model selection (only relevant for local)
    if config.transcription.mode == TranscriptionMode::Local {
        let model_labels: Vec<&str> = MODELS.iter().map(|(_, label)| *label).collect();
        let default_idx = MODELS.iter().position(|(name, _)| name == &config.transcription.model.as_str())
            .unwrap_or(3);
        let model_idx = Select::with_theme(&theme)
            .with_prompt("Whisper model")
            .default(default_idx)
            .items(&model_labels)
            .interact()?;
        config.transcription.model = MODELS[model_idx].0.to_string();
    }

    // API key (always offered, used for cleanup even in local mode)
    let api_key: String = Password::with_theme(&theme)
        .with_prompt("OpenAI API key (for AI cleanup, optional - press Enter to skip)")
        .allow_empty_password(true)
        .interact()?;
    if !api_key.is_empty() {
        config.transcription.api_key = api_key;
    }

    // AI cleanup
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

    // Context-aware
    config.general.context_aware = Confirm::with_theme(&theme)
        .with_prompt("Enable context-aware mode (read text around cursor)?")
        .default(config.general.context_aware)
        .interact()?;

    // Voice commands
    config.voice_commands.enabled = Confirm::with_theme(&theme)
        .with_prompt("Enable voice commands (period, new line, etc.)?")
        .default(config.voice_commands.enabled)
        .interact()?;

    // Hotkey mode
    let hotkey_modes = ["Hold to record (default)", "Press to toggle"];
    let hk_idx = Select::with_theme(&theme)
        .with_prompt("Hotkey mode")
        .default(if config.hotkey.mode == HotkeyMode::Hold { 0 } else { 1 })
        .items(&hotkey_modes)
        .interact()?;
    config.hotkey.mode = if hk_idx == 0 { HotkeyMode::Hold } else { HotkeyMode::Toggle };

    // Auto-start
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
```

- [ ] **Step 2: Verify it compiles**

```bash
cd /Users/faridmatovu/projects/wipr
cargo check 2>&1 | grep "error" | head -5
```
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/cli/wizard.rs
git commit -m "feat: add interactive config wizard with dialoguer"
```

---

### Task 11: Wire Up CLI Command Handlers

**Files:**
- Modify: `src/cli/commands.rs`

- [ ] **Step 1: Replace `src/cli/commands.rs` with full implementations**

```rust
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
    if process::read_pid_file().is_some() {
        if let Some(pid) = process::read_pid_file() {
            if process::is_alive(pid) {
                return Err(anyhow!("Murmur is already running (PID {})", pid));
            }
            process::remove_pid_file().ok();
        }
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
        None => {
            println!("{} Murmur is not running", "○".red());
        }
    }
    Ok(())
}

fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
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
        current = current.get_mut(*part)
            .ok_or_else(|| anyhow!("Unknown key path: {}", key))?;
    }
    let last = parts.last().unwrap();

    // Try to parse as JSON first (for booleans, numbers); fall back to string
    let parsed: Value = serde_json::from_str(new_value)
        .unwrap_or_else(|_| Value::String(new_value.to_string()));

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
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
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
```

- [ ] **Step 2: Verify build**

```bash
cd /Users/faridmatovu/projects/wipr
cargo build 2>&1 | tail -3
```
Expected: Build succeeds.

- [ ] **Step 3: Test help output**

```bash
./target/debug/murmur --help
./target/debug/murmur config --help
./target/debug/murmur history --help
```
Expected: All show proper help with subcommands.

- [ ] **Step 4: Test status command (no daemon running)**

```bash
./target/debug/murmur status
```
Expected: "○ Murmur is not running"

- [ ] **Step 5: Commit**

```bash
git add src/cli/commands.rs
git commit -m "feat: implement all CLI command handlers"
```

---

### Task 12: Delete Tauri-Related Files

**Files:**
- Delete: `src-tauri/`
- Delete: `package.json`, `package-lock.json`, `node_modules/`
- Delete: `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `index.html`, `dist/`
- Delete: `Homebrew/murmur.rb` (will be recreated for CLI)

- [ ] **Step 1: Verify all moves are committed**

```bash
cd /Users/faridmatovu/projects/wipr
git status
```
Expected: Working tree clean.

- [ ] **Step 2: Delete Tauri and frontend files**

```bash
git rm -rf src-tauri/
git rm -f package.json package-lock.json vite.config.ts vitest.config.ts tsconfig.json index.html
rm -rf node_modules dist
echo "node_modules/" > .gitignore.tmp
echo "dist/" >> .gitignore.tmp
echo "target/" >> .gitignore.tmp
echo ".DS_Store" >> .gitignore.tmp
mv .gitignore.tmp .gitignore
git add .gitignore
```

- [ ] **Step 3: Verify project still builds after deletions**

```bash
cargo build 2>&1 | tail -3
cargo test 2>&1 | grep "test result"
```
Expected: Build succeeds, all tests pass.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove Tauri, React, and frontend tooling"
```

---

### Task 13: Update Homebrew Formula

**Files:**
- Create: `Homebrew/murmur.rb` (replacing the old cask)

- [ ] **Step 1: Write `Homebrew/murmur.rb`**

```ruby
class Murmur < Formula
  desc "macOS voice-to-text dictation daemon"
  homepage "https://github.com/OWNER/murmur"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/OWNER/murmur/releases/download/v#{version}/murmur-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/OWNER/murmur/releases/download/v#{version}/murmur-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_SHA256"
    end
  end

  def install
    bin.install "murmur"
  end

  def caveats
    <<~EOS
      Murmur needs Input Monitoring permission to detect the fn key.
      Open System Settings → Privacy & Security → Input Monitoring,
      then enable Murmur after first run.

      To start the daemon:
        murmur start

      To install for auto-start on login:
        murmur install

      To configure interactively:
        murmur config
    EOS
  end

  test do
    assert_match "murmur", shell_output("#{bin}/murmur --version")
  end
end
```

- [ ] **Step 2: Commit**

```bash
git add Homebrew/murmur.rb
git commit -m "feat: update Homebrew to formula for CLI binary distribution"
```

---

### Task 14: Run Full Test Suite & Verify Build

**Files:**
- None (verification only)

- [ ] **Step 1: Run all tests**

```bash
cd /Users/faridmatovu/projects/wipr
cargo test 2>&1 | grep "test result"
```
Expected: All tests pass (should be 65+ tests from migrated backend modules + new daemon/notify/launchd tests).

- [ ] **Step 2: Run clippy**

```bash
cargo clippy 2>&1 | grep -E "error|warning\[" | head -10
```
Expected: No errors.

- [ ] **Step 3: Build release binary**

```bash
cargo build --release 2>&1 | tail -3
```
Expected: Builds successfully. Note: this will take a while (whisper-rs is slow to compile in release mode).

- [ ] **Step 4: Verify binary works**

```bash
./target/release/murmur --help
./target/release/murmur status
./target/release/murmur config list
```
Expected: All commands run without errors. `config list` shows the current config (will create defaults if first run).

- [ ] **Step 5: Check binary size**

```bash
ls -lh target/release/murmur
```
Expected: Reasonable size (~20-50MB depending on whisper-rs static linking).

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit --allow-empty -m "chore: verify CLI build and test suite passes"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Create new Cargo.toml at project root |
| 2 | Move backend modules to new src/ |
| 3 | Create lib.rs and verify Tauri-free |
| 4 | Notification & logging module |
| 5 | CLI module skeleton with clap |
| 6 | main.rs entry point |
| 7 | Daemon process management (PID/fork) |
| 8 | Dictation pipeline extraction |
| 9 | Launchd integration |
| 10 | Interactive config wizard |
| 11 | Wire up all CLI command handlers |
| 12 | Delete Tauri-related files |
| 13 | Update Homebrew formula |
| 14 | Run full test suite & verify |
