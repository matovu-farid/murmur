# Murmur CLI — macOS Voice-to-Text Dictation Daemon

## Overview

Murmur is a macOS voice-to-text dictation tool that runs as a background daemon. It captures voice input via the fn key, transcribes it using Whisper (local or API), enhances the text with GPT-4o-mini, and inserts the result at the cursor. Distributed as a single Rust CLI binary — no GUI, no Electron, no Tauri.

This is a migration from the previous Tauri + React desktop app. The core backend modules are preserved as-is; the UI layer is replaced by a CLI and macOS notifications.

## Architecture

Single Rust binary. Backend modules (audio, transcription, AI, input, config, history) are unchanged. New layers added on top:
- **CLI** — Argument parsing via `clap`, subcommand handlers, interactive config wizard
- **Daemon** — Process management (PID files, fork to background), launchd integration
- **Notify** — macOS notification center + structured logging

## Project Structure

```
murmur/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, clap CLI setup
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs       # Subcommand handlers (start, stop, status, config, history, install, uninstall, download)
│   │   └── wizard.rs         # Interactive config wizard
│   ├── daemon/
│   │   ├── mod.rs
│   │   ├── process.rs        # PID file management, fork to background
│   │   ├── pipeline.rs       # Core dictation pipeline
│   │   └── launchd.rs        # launchd plist generation, install/uninstall
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── capture.rs        # cpal recording
│   │   └── preprocessing.rs  # Noise reduction, silence trim, gain normalization
│   ├── transcription/
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   ├── whisper_api.rs
│   │   └── whisper_local.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   ├── cleanup.rs        # GPT-4o-mini text cleanup
│   │   ├── context.rs        # macOS Accessibility API context reading
│   │   └── commands.rs       # Voice command detection
│   ├── input/
│   │   ├── mod.rs
│   │   ├── hotkey.rs         # CGEventTap fn key monitor
│   │   └── insertion.rs      # Clipboard + Cmd+V insertion
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── history/
│   │   ├── mod.rs
│   │   └── store.rs          # SQLite history
│   └── notify.rs             # macOS notifications + logging
├── Homebrew/
│   └── murmur.rb             # Updated for binary distribution
└── README.md
```

## CLI Interface

```
murmur                          # Show help
murmur start                    # Start daemon in background
murmur start --foreground       # Run in foreground (for debugging)
murmur stop                     # Stop running daemon
murmur status                   # Show daemon status (running/stopped, PID, uptime)

murmur config                   # Interactive config wizard
murmur config list              # Print all settings
murmur config get <key>         # Get a setting (e.g., transcription.mode)
murmur config set <key> <value> # Set a setting

murmur history                  # List recent transcriptions (default last 20)
murmur history --limit N        # Show last N entries
murmur history search <query>   # Search transcriptions
murmur history clear            # Clear all history (with confirmation)
murmur history export           # Export to JSON on stdout

murmur install                  # Install launchd service (auto-start on login)
murmur uninstall                # Remove launchd service

murmur download <model>         # Download a Whisper model
```

Implemented with `clap` derive macros.

## Daemon & Pipeline

### Process Management (`daemon/process.rs`)

- PID file: `~/.config/murmur/murmur.pid`
- `start`: Forks to background by spawning self with `--foreground` flag, writes PID, exits parent
- `stop`: Reads PID, sends `SIGTERM` via `nix::sys::signal::kill`, removes PID file
- `status`: Checks if PID is alive via `kill(pid, 0)`, reports uptime from PID file mtime
- Foreground mode: runs pipeline directly (also used by launchd)

### Pipeline (`daemon/pipeline.rs`)

Same channel-based architecture as the current Tauri version:

1. **Hotkey thread** — `start_fn_key_monitor()` runs CFRunLoop, sends `HotkeyEvent` via mpsc channel
2. **Recording thread** — Owns `Recorder` and `cpal::Stream` (both `!Send`). Receives hotkey events, manages stream lifecycle, sends recorded samples to processing thread
3. **Processing thread** — Receives audio samples, runs tokio runtime for async work:
   - Preprocess audio (noise reduction, silence trim, gain normalization)
   - Transcribe (local whisper-rs via `spawn_blocking`, or OpenAI API)
   - Process voice commands (punctuation, formatting, editing commands)
   - AI cleanup via GPT-4o-mini (if enabled, with optional context from accessibility API)
   - Insert at cursor via clipboard + Cmd+V
   - Log to SQLite history

Each stage emits structured logs via `tracing` and triggers notifications on errors.

### Launchd (`daemon/launchd.rs`)

- `install`: Generates `~/Library/LaunchAgents/com.murmur.app.plist` pointing to the murmur binary with `start --foreground`. Runs `launchctl load`.
- `uninstall`: Runs `launchctl unload`, removes plist.
- Plist settings:
  - `RunAtLoad: true` — auto-start on login
  - `KeepAlive: true` — auto-restart on crash
  - `StandardOutPath` and `StandardErrorPath` point to log files

## Notifications & Logging

### Notifications (`notify.rs`)

Uses `mac-notification-sys` for native macOS banners:

- `notify_error(msg)` — Errors (transcription failure, missing model, permission denied)
- `notify_info(msg)` — Status changes (daemon started/stopped, first install confirmation)

**When notifications fire:**
- Daemon start: "Murmur is listening"
- Daemon stop: "Murmur stopped"
- Transcription error
- Missing accessibility permission
- Missing model file
- First successful dictation after install (one-time confirmation)

**When notifications do NOT fire (too noisy):**
- Every successful transcription
- Every hotkey press
- Routine pipeline events

### Logging

- `tracing` + `tracing-appender` for structured logging
- Log file: `~/.config/murmur/murmur.log`
- Daily rotation, keeps 7 days
- Log levels: ERROR, WARN, INFO, DEBUG
- Foreground mode also outputs to stderr for live debugging
- Each pipeline stage logs entry/exit with relevant context (audio duration, transcript length, errors with stack)

## Config Wizard & Commands

### Wizard (`cli/wizard.rs`)

Triggered by `murmur config` with no arguments. Uses `dialoguer` for interactive prompts:

```
$ murmur config
? Transcription mode: › Local (offline) / API (OpenAI)
? Whisper model: › tiny.en / base.en / small.en / medium.en (recommended)
? OpenAI API key (for AI cleanup, optional): › <hidden input>
? Enable AI cleanup? › Yes
? Enable context-aware mode (read text around cursor)? › Yes
? Enable voice commands? › Yes
? Hotkey mode: › Hold to record / Press to toggle
? Auto-start on login? › Yes
✓ Saved to ~/.config/murmur/config.json
```

If the chosen model isn't downloaded yet, the wizard offers to download it inline with an `indicatif` progress bar. If "Auto-start on login" is yes, runs `murmur install` automatically.

### Direct Config Commands

- `config get <key>` — Dotted path access (e.g., `transcription.mode`). Prints raw value.
- `config set <key> <value>` — Validates type against schema, updates JSON file. Errors clearly on invalid keys/values.
- `config list` — Pretty-prints the full config with current values, grouped by section.

### History Commands

- `history` (no args) — Last 20 entries in a table: timestamp, duration, cleaned text (truncated to terminal width)
- `history --limit N` — Last N entries
- `history search <query>` — Filtered list
- `history export` — JSON array on stdout
- `history clear` — Confirmation prompt, then deletes all

## Dependencies

```toml
[dependencies]
# Core (preserved from current implementation)
cpal = "0.15"
hound = "3.5"
whisper-rs = "0.16"
reqwest = { version = "0.12", features = ["json", "multipart", "stream"] }
rusqlite = { version = "0.33", features = ["bundled"] }
core-graphics = "0.25"
core-foundation = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
dirs = "6"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
regex-lite = "0.1"

# CLI
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
indicatif = "0.17"
colored = "2"

# Daemon
nix = { version = "0.29", features = ["signal", "process"] }
mac-notification-sys = "0.6"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"
```

**Removed entirely:**
- `tauri`, `tauri-build`, `tauri-plugin-opener`, `tauri-plugin-autostart`, `tauri-plugin-global-shortcut`
- All frontend tooling (no `package.json`, no `vite`, no React, no TypeScript, no Tailwind)

## Distribution

### Cargo
- `cargo install murmur` — Build from source
- Published to crates.io

### Homebrew
- `brew install murmur` (formula, not cask — it's a CLI now)
- Formula points to GitHub Releases prebuilt binary

### GitHub Releases
- Prebuilt binaries for `aarch64-apple-darwin` (Apple Silicon) and `x86_64-apple-darwin` (Intel)
- CI workflow builds and uploads on tag push

## Migration Path

This is a full replacement of the current Tauri + React app:

1. Create new project structure at `murmur/` (or restructure in place)
2. Delete: `src/` (React), `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`, `src-tauri/icons/`, `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `node_modules/`, `dist/`
3. Move backend modules from `src-tauri/src/` to new `src/` (preserving git history via `git mv`)
4. Strip Tauri-specific code from modules:
   - Remove `#[tauri::command]` attributes
   - Remove `tauri::Emitter` calls (replaced by `tracing` + notifications)
   - Remove `tauri::AppHandle` and `tauri::State` from function signatures
5. Rewrite `lib.rs` (or move logic to `daemon/pipeline.rs`)
6. Add new files: `cli/`, `daemon/`, `notify.rs`
7. Update `Cargo.toml` with new dependencies, remove Tauri ones
8. Tests for backend modules carry over unchanged
9. Add tests for new CLI and daemon code

## Config File Format

Unchanged from current implementation:

```json
{
  "transcription": { "mode": "local", "model": "medium.en", "api_key": "" },
  "ai_cleanup": { "enabled": true, "custom_instructions": "" },
  "hotkey": { "key": "fn", "mode": "hold" },
  "voice_commands": { "enabled": true },
  "general": { "auto_start": true, "context_aware": true, "first_run": true }
}
```

The `first_run` flag now triggers the config wizard on first `murmur start` if config doesn't exist.
