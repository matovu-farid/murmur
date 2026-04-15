# Wipr — macOS Voice-to-Text Dictation Tool

## Overview

Wipr is a macOS menu bar dictation tool that captures voice input via a global hotkey, transcribes it using Whisper (local or API), enhances the text with GPT-4o-mini, and inserts it at the cursor position. Built as a Tauri app with a Rust backend and React frontend.

## Architecture

Monolithic Tauri application. Rust handles all system-level work (audio, hotkeys, transcription, AI, clipboard, tray). React handles UI (overlay, settings, history, onboarding). Communication via Tauri commands and events.

## Project Structure

```
wipr/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── lib.rs               # Module declarations
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs       # Audio recording via cpal
│   │   │   └── preprocessing.rs # Noise reduction, silence trimming, gain normalization
│   │   ├── transcription/
│   │   │   ├── mod.rs
│   │   │   ├── whisper_local.rs  # whisper-rs local transcription
│   │   │   └── whisper_api.rs    # OpenAI Whisper API client
│   │   ├── ai/
│   │   │   ├── mod.rs
│   │   │   ├── cleanup.rs       # GPT-4o-mini text cleanup/enhancement
│   │   │   ├── context.rs       # Read surrounding text via Accessibility API
│   │   │   └── commands.rs      # Voice command detection & execution
│   │   ├── input/
│   │   │   ├── mod.rs
│   │   │   ├── hotkey.rs        # Global hotkey listener (fn key + custom shortcuts)
│   │   │   └── insertion.rs     # Text insertion at cursor via clipboard + CGEvent
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   └── settings.rs      # Config read/write (~/.config/wipr/config.json)
│   │   ├── history/
│   │   │   ├── mod.rs
│   │   │   └── store.rs         # SQLite-backed transcription history
│   │   └── tray.rs              # System tray setup and menu
│   └── icons/
├── src/                          # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── Overlay.tsx           # Recording indicator overlay
│   │   ├── Settings.tsx          # Settings window
│   │   ├── History.tsx           # Transcription history viewer
│   │   └── Onboarding.tsx        # First-run accessibility permissions guide
│   ├── hooks/
│   │   └── useTauriEvents.ts     # Listen to Tauri events from Rust
│   ├── db/
│   │   ├── collections.ts        # TanStack DB collection definitions
│   │   └── queries.ts            # TanStack Query hooks for remote state
│   └── styles/
├── package.json
└── README.md
```

## Core Data Flow

1. User presses hotkey → Rust `hotkey.rs` detects it
2. `capture.rs` starts recording audio via `cpal` (mono, f32, 16kHz)
3. User releases hotkey → recording stops
4. `preprocessing.rs` applies silence trimming, gain normalization, noise reduction
5. Audio sent to `whisper_local.rs` or `whisper_api.rs` (based on config)
6. Raw transcript checked for voice commands in `commands.rs`
7. If not a command → `cleanup.rs` sends to GPT-4o-mini with optional context from `context.rs`
8. Cleaned text inserted at cursor via `insertion.rs`
9. Transcription logged to `store.rs` (SQLite)
10. Frontend overlay shows status throughout via Tauri events

## Audio System

### Recording (capture.rs)
- Uses `cpal` crate for audio capture
- Records mono, 16-bit PCM at 16kHz sample rate
- Streams audio into a ring buffer during recording
- Minimum recording threshold: 0.3 seconds (ignore accidental taps)

### Preprocessing (preprocessing.rs)
- Silence trimming: strip leading/trailing silence using amplitude threshold detection
- Gain normalization: normalize volume to consistent level
- Noise reduction: basic spectral subtraction using first ~200ms as noise profile estimate
- Output: cleaned f32 buffer ready for transcription

### Audio Format
- Internal format: f32 samples, mono, 16kHz
- For Whisper API: convert to WAV file in temp directory (via `hound` crate)
- For whisper-rs: pass f32 buffer directly

## Transcription Engine

Dual-mode transcription. Config setting determines which engine. Both implement a common trait:

```rust
trait Transcriber {
    async fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<String, TranscribeError>;
}
```

### Local Mode (whisper_local.rs)
- Uses `whisper-rs` (bindings to whisper.cpp)
- Default model: `medium.en` (~1.5GB)
- Models stored in `~/.config/wipr/models/`
- Downloaded on first launch with progress bar (not bundled in DMG)
- Alternative models available in settings: tiny.en, base.en, small.en
- Runs inference on a dedicated thread

### API Mode (whisper_api.rs)
- Sends WAV to OpenAI's `whisper-1` endpoint
- Requires API key stored in config
- Timeout: 30 seconds
- Shows error in overlay on failure (no silent failures)

## AI Text Cleanup & Voice Commands

### Text Cleanup (cleanup.rs)
- Raw transcription sent to GPT-4o-mini for enhancement
- Removes filler words (um, uh, like, you know)
- Fixes grammar and punctuation
- Proper capitalization and sentence structure
- Preserves speaker's intent and tone
- Optional — user can toggle off in settings

### Context-Aware Mode (context.rs)
- Reads ~200 characters before cursor via macOS Accessibility API (`AXUIElement`)
- Context sent alongside raw transcript to GPT-4o-mini
- Helps match tone (casual in chat, formal in email), continue sentences, match formatting
- Falls back gracefully if accessibility API can't read the field

### GPT-4o-mini Prompt

```
System: You are a dictation cleanup assistant. Clean up the
transcribed speech while preserving the speaker's intent.
Fix filler words, grammar, and punctuation. Match the tone
of the surrounding context if provided.

User:
Context (text before cursor): {context_text}
Raw transcription: {raw_text}

Return only the cleaned text, nothing else.
```

### Voice Commands (commands.rs)
- Detected via exact string matching before AI cleanup (for speed)
- `commands.rs` scans the raw transcript, replaces command phrases with their output (e.g., "period" → "."), and returns the processed text. Only the remaining natural language portions are sent to AI cleanup.
- Commands can appear mid-dictation: "send the email period new line thanks comma John" → `send the email.\nThanks, John`
- If the entire transcript is commands (no natural language left), skip AI cleanup entirely
- Supported commands (~15):
  - **Punctuation**: "period", "comma", "question mark", "exclamation point", "colon", "semicolon"
  - **Formatting**: "new line", "new paragraph", "tab"
  - **Editing**: "delete that", "undo that", "select all"
  - **Control**: "stop listening"

## Input System

### Global Hotkeys (hotkey.rs)
- Default: Function (fn) key for hold-to-record
- fn key requires raw event monitoring via `CGEventTap` (not available through Tauri's standard shortcut plugin)
- Users can remap to any key combo in settings (e.g., Cmd+Shift+Space, double-tap Ctrl)
- Two recording modes:
  - **Hold-to-record** (default): hold key to record, release to transcribe
  - **Toggle**: press once to start, press again to stop

### Text Insertion (insertion.rs)
- Save clipboard → copy text to clipboard → simulate Cmd+V via `CGEvent` → restore clipboard after 150ms delay
- Uses macOS `CGEvent` API directly from Rust (via `core-graphics` crate)
- Clipboard save/restore ensures user's clipboard isn't lost

## UI Components

### System Tray (tray.rs)
- Menu bar icon with state indicators:
  - Idle (default icon)
  - Recording (red dot / filled icon)
  - Transcribing (pulsing icon)
- Right-click menu: Settings, History, Transcription mode toggle, AI Cleanup toggle, Auto-start toggle, Quit

### Recording Overlay (Overlay.tsx)
- Small floating window near top-center of screen
- Shows state: "Recording...", "Transcribing...", "Cleaning up..."
- Audio waveform visualization during recording (simple amplitude bars)
- Disappears after text insertion
- Transparent, borderless Tauri window with `always_on_top`

### Settings Window (Settings.tsx)
- Sections:
  - **General**: Auto-start on login, hotkey configuration, recording mode (hold/toggle)
  - **Transcription**: Local vs API mode, model selection, OpenAI API key input
  - **AI Cleanup**: Enable/disable, custom instructions (e.g., "always use formal tone")
  - **Voice Commands**: Enable/disable, view supported commands
  - **History**: Clear history, export

### History Window (History.tsx)
- Searchable list of past transcriptions
- Each entry: timestamp, raw transcription, cleaned text, duration
- Stored in SQLite via `rusqlite` at `~/.config/wipr/history.db`

### Onboarding (Onboarding.tsx)
- Shown on first launch, step-by-step:
  1. Welcome screen
  2. Grant Accessibility permissions (with "Open System Settings" button)
  3. Grant Microphone permissions
  4. Choose transcription mode (local/API) and enter API key if needed
  5. Test recording — try a quick dictation
  6. Done — app minimizes to tray

## Frontend Data Layer

### TanStack DB (db/collections.ts)
- Reactive client-side collections for:
  - `transcriptions` — history entries synced from SQLite via Tauri commands
  - `settings` — reactive config state, synced to `~/.config/wipr/config.json` on change
- Provides reactive queries for UI components (auto-updates when data changes)

### TanStack Query (db/queries.ts)
- Manages remote/async state:
  - OpenAI API calls (transcription, cleanup) — loading/error states
  - Model download progress
  - Update checks (GitHub Releases)
- Handles caching, retry logic, and error boundaries

## Packaging & Distribution

### DMG Installer
- Tauri's built-in bundler produces `.dmg` and `.app`
- Code signing via Apple Developer certificate
- Notarization via `xcrun notarytool`
- Whisper model NOT bundled (downloaded on first launch)

### Homebrew
- Cask formula pointing to GitHub Releases
- `brew install --cask wipr`

### Auto-start
- Tauri's `autostart` plugin (wraps `launchd` / LoginItems)
- Enabled by default on first install
- Toggle in settings and tray menu

### Updates
- Tauri's built-in updater plugin
- Checks GitHub Releases on app launch
- Notifies user via tray — no auto-update without consent

## Dependencies

### Rust (Cargo.toml)
| Crate | Purpose |
|-------|---------|
| `tauri` | App framework |
| `tauri-plugin-global-shortcut` | Custom hotkeys |
| `tauri-plugin-autostart` | Login item |
| `tauri-plugin-updater` | Auto-updates |
| `tauri-plugin-shell` | System interactions |
| `cpal` | Audio capture |
| `whisper-rs` | Local Whisper inference |
| `reqwest` | HTTP client for OpenAI APIs |
| `serde` / `serde_json` | Config serialization |
| `rusqlite` | History database |
| `core-graphics` | CGEvent for key simulation |
| `accessibility` | AXUIElement for context reading |
| `hound` | WAV encoding |

### React (package.json)
| Package | Purpose |
|---------|---------|
| `react` / `react-dom` | UI framework |
| `@tauri-apps/api` | Tauri JS bindings |
| `@tauri-apps/plugin-*` | Plugin JS interfaces |
| `tailwindcss` | Styling |
| `@tanstack/react-query` | Remote/async state management |
| `@tanstack/db` | Reactive client-side collections |
| `react-router` | Navigation between settings/history/onboarding |

## Config File Format

Location: `~/.config/wipr/config.json`

```json
{
  "transcription": {
    "mode": "local",
    "model": "medium.en",
    "api_key": "sk-..."
  },
  "ai_cleanup": {
    "enabled": true,
    "custom_instructions": ""
  },
  "hotkey": {
    "key": "fn",
    "mode": "hold"
  },
  "voice_commands": {
    "enabled": true
  },
  "general": {
    "auto_start": true,
    "context_aware": true
  }
}
```
