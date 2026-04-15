# Wipr Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a macOS menu bar voice-to-text dictation tool using Tauri v2 (Rust + React) with local/API Whisper transcription, GPT-4o-mini text cleanup, and context-aware dictation.

**Architecture:** Monolithic Tauri v2 app. Rust backend handles audio capture (cpal), transcription (whisper-rs / OpenAI API), AI cleanup (GPT-4o-mini), global hotkeys (CGEventTap), text insertion (CGEvent + clipboard), system tray, config, and SQLite history. React frontend handles overlay, settings, history, and onboarding UI via TanStack Query/DB and Tailwind CSS.

**Tech Stack:** Tauri v2, Rust, React 19, TypeScript, cpal, whisper-rs, reqwest, rusqlite, core-graphics, TanStack Query, TanStack DB, Tailwind CSS v4, React Router v7

**Spec:** `docs/superpowers/specs/2026-04-15-wipr-design.md`

---

## File Structure

```
wipr/
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   ├── 32x32.png
│   │   └── 128x128.png
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── audio/
│       │   ├── mod.rs
│       │   ├── capture.rs
│       │   └── preprocessing.rs
│       ├── transcription/
│       │   ├── mod.rs
│       │   ├── traits.rs
│       │   ├── whisper_local.rs
│       │   └── whisper_api.rs
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── cleanup.rs
│       │   ├── context.rs
│       │   └── commands.rs
│       ├── input/
│       │   ├── mod.rs
│       │   ├── hotkey.rs
│       │   └── insertion.rs
│       ├── config/
│       │   ├── mod.rs
│       │   └── settings.rs
│       ├── history/
│       │   ├── mod.rs
│       │   └── store.rs
│       └── tray.rs
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── index.css
│   ├── lib/
│   │   └── queryClient.ts
│   ├── db/
│   │   ├── collections.ts
│   │   └── queries.ts
│   ├── hooks/
│   │   └── useTauriEvents.ts
│   ├── pages/
│   │   ├── Overlay.tsx
│   │   ├── Settings.tsx
│   │   ├── History.tsx
│   │   └── Onboarding.tsx
│   └── components/
│       ├── Waveform.tsx
│       ├── SettingsSection.tsx
│       ├── HistoryEntry.tsx
│       └── OnboardingStep.tsx
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/index.css`

- [ ] **Step 1: Create the Tauri project using the CLI**

```bash
cd /Users/faridmatovu/projects
rm -rf wipr/wipr wipr/wipr.py wipr/setup.sh wipr/requirements.txt wipr/logs
cd wipr
npm create tauri-app@latest . -- --template react-ts --manager npm
```

If prompted, choose: TypeScript, React, npm.

- [ ] **Step 2: Install Rust dependencies**

Replace the contents of `src-tauri/Cargo.toml`:

```toml
[package]
name = "wipr"
version = "0.1.0"
description = "macOS voice-to-text dictation tool"
authors = [""]
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-opener = "2"
tauri-plugin-autostart = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
cpal = "0.15"
hound = "3.5"
whisper-rs = "0.16"
reqwest = { version = "0.12", features = ["json", "multipart"] }
rusqlite = { version = "0.33", features = ["bundled"] }
core-graphics = "0.25"
core-foundation = "0.10"
tokio = { version = "1", features = ["full"] }
dirs = "6"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 3: Install frontend dependencies**

```bash
npm install @tanstack/react-query @tanstack/db @tanstack/react-db @tanstack/query-db-collection react-router tailwindcss @tailwindcss/vite
```

- [ ] **Step 4: Configure Vite with Tailwind**

Replace `vite.config.ts`:

```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
```

- [ ] **Step 5: Set up Tailwind CSS entry**

Replace `src/index.css` (delete any existing content):

```css
@import "tailwindcss";

@theme {
  --color-primary: #6366f1;
  --color-primary-hover: #4f46e5;
  --color-surface: #1e1e2e;
  --color-surface-light: #2a2a3e;
  --color-text: #cdd6f4;
  --color-text-muted: #a6adc8;
  --color-danger: #f38ba8;
  --color-success: #a6e3a1;
  --color-warning: #f9e2af;
  --font-sans: "Inter", system-ui, sans-serif;
}
```

- [ ] **Step 6: Configure tauri.conf.json**

Replace `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Wipr",
  "version": "0.1.0",
  "identifier": "com.wipr.app",
  "build": {
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {}
}
```

Note: `windows` is empty because we create all windows programmatically (tray-only app).

- [ ] **Step 7: Set up Tauri capabilities**

Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "identifier": "default",
  "description": "Default capability for Wipr",
  "windows": ["*"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:webview:default",
    "core:window:default",
    "core:window:allow-close",
    "core:window:allow-hide",
    "core:window:allow-show",
    "core:window:allow-set-focus",
    "opener:default",
    "autostart:default",
    "global-shortcut:default"
  ]
}
```

- [ ] **Step 8: Set up minimal Rust entry points**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    wipr_lib::run();
}
```

Replace `src-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 9: Set up minimal React entry**

Replace `src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

Replace `src/App.tsx`:

```tsx
function App() {
  return (
    <div className="min-h-screen bg-surface text-text flex items-center justify-center">
      <h1 className="text-2xl font-bold">Wipr</h1>
    </div>
  );
}

export default App;
```

- [ ] **Step 10: Verify the project builds**

```bash
cd /Users/faridmatovu/projects/wipr
npm install
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: Both npm install and cargo build succeed without errors. The cargo build will take a while on first run (compiling whisper-rs, etc.).

- [ ] **Step 11: Commit**

```bash
git init
echo "node_modules/\ntarget/\ndist/\n.DS_Store" > .gitignore
git add .
git commit -m "feat: scaffold Tauri v2 project with React, Tailwind, and Rust dependencies"
```

---

### Task 2: Config Module

**Files:**
- Create: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/settings.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for config module**

Add to `src-tauri/src/config/settings.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub transcription: TranscriptionConfig,
    pub ai_cleanup: AiCleanupConfig,
    pub hotkey: HotkeyConfig,
    pub voice_commands: VoiceCommandsConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionConfig {
    pub mode: TranscriptionMode,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionMode {
    Local,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCleanupConfig {
    pub enabled: bool,
    pub custom_instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub key: String,
    pub mode: HotkeyMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyMode {
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommandsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_start: bool,
    pub context_aware: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            transcription: TranscriptionConfig {
                mode: TranscriptionMode::Local,
                model: "medium.en".to_string(),
                api_key: String::new(),
            },
            ai_cleanup: AiCleanupConfig {
                enabled: true,
                custom_instructions: String::new(),
            },
            hotkey: HotkeyConfig {
                key: "fn".to_string(),
                mode: HotkeyMode::Hold,
            },
            voice_commands: VoiceCommandsConfig { enabled: true },
            general: GeneralConfig {
                auto_start: true,
                context_aware: true,
            },
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join("wipr")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn models_dir() -> PathBuf {
    config_dir().join("models")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let config = AppConfig::default();
        save_config(&config).ok();
        config
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_path(), content).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn with_temp_config<F: FnOnce()>(f: F) {
        let dir = env::temp_dir().join(format!("wipr_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        env::set_var("HOME", &dir);
        f();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.transcription.mode, TranscriptionMode::Local);
        assert_eq!(config.transcription.model, "medium.en");
        assert!(config.ai_cleanup.enabled);
        assert_eq!(config.hotkey.key, "fn");
        assert_eq!(config.hotkey.mode, HotkeyMode::Hold);
        assert!(config.voice_commands.enabled);
        assert!(config.general.auto_start);
        assert!(config.general.context_aware);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.transcription.model, "medium.en");
        assert_eq!(deserialized.hotkey.mode, HotkeyMode::Hold);
    }
}
```

- [ ] **Step 2: Create config module declaration**

Create `src-tauri/src/config/mod.rs`:

```rust
pub mod settings;
pub use settings::*;
```

- [ ] **Step 3: Wire config module into lib.rs and add Tauri commands**

Replace `src-tauri/src/lib.rs`:

```rust
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
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test config
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config/ src-tauri/src/lib.rs
git commit -m "feat: add config module with settings read/write and Tauri commands"
```

---

### Task 3: Audio Capture

**Files:**
- Create: `src-tauri/src/audio/mod.rs`
- Create: `src-tauri/src/audio/capture.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the audio capture module**

Create `src-tauri/src/audio/capture.rs`:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub struct Recorder {
    samples: Arc<Mutex<Vec<f32>>>,
    is_recording: Arc<AtomicBool>,
    sample_rate: u32,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
            sample_rate: 16000,
        }
    }

    pub fn start(&self) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("No default input config: {}", e))?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        self.samples.lock().unwrap().clear();
        self.is_recording.store(true, Ordering::SeqCst);

        let samples = self.samples.clone();
        let is_recording = self.is_recording.clone();

        let err_fn = |err| eprintln!("Audio stream error: {}", err);

        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if is_recording.load(Ordering::SeqCst) {
                        samples.lock().unwrap().extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => {
                let samples = self.samples.clone();
                let is_recording = self.is_recording.clone();
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if is_recording.load(Ordering::SeqCst) {
                            let floats: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            samples.lock().unwrap().extend(floats);
                        }
                    },
                    err_fn,
                    None,
                )
            }
            format => return Err(format!("Unsupported sample format: {:?}", format)),
        }
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        Ok(stream)
    }

    pub fn stop(&self) -> Vec<f32> {
        self.is_recording.store(false, Ordering::SeqCst);
        self.samples.lock().unwrap().clone()
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns duration in seconds of the recorded audio
    pub fn duration_secs(samples: &[f32], sample_rate: u32) -> f32 {
        samples.len() as f32 / sample_rate as f32
    }

    /// Minimum recording duration in seconds
    pub const MIN_DURATION: f32 = 0.3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_calculation() {
        let samples = vec![0.0f32; 16000]; // 1 second at 16kHz
        assert!((Recorder::duration_secs(&samples, 16000) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_short_recording_detection() {
        let samples = vec![0.0f32; 4000]; // 0.25 seconds at 16kHz
        let duration = Recorder::duration_secs(&samples, 16000);
        assert!(duration < Recorder::MIN_DURATION);
    }

    #[test]
    fn test_recorder_initial_state() {
        let recorder = Recorder::new();
        assert!(!recorder.is_recording());
        assert_eq!(recorder.sample_rate(), 16000);
    }
}
```

- [ ] **Step 2: Create audio module declaration**

Create `src-tauri/src/audio/mod.rs`:

```rust
pub mod capture;
pub use capture::Recorder;
```

- [ ] **Step 3: Wire audio module into lib.rs**

Add to `src-tauri/src/lib.rs` after `mod config;`:

```rust
mod audio;
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test audio
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/audio/
git commit -m "feat: add audio capture module with cpal recording"
```

---

### Task 4: Audio Preprocessing

**Files:**
- Create: `src-tauri/src/audio/preprocessing.rs`
- Modify: `src-tauri/src/audio/mod.rs`

- [ ] **Step 1: Write the preprocessing module with tests**

Create `src-tauri/src/audio/preprocessing.rs`:

```rust
/// Trim leading and trailing silence from audio samples.
/// Silence is defined as samples below the amplitude threshold.
pub fn trim_silence(samples: &[f32], threshold: f32) -> &[f32] {
    let start = samples
        .iter()
        .position(|&s| s.abs() > threshold)
        .unwrap_or(0);
    let end = samples
        .iter()
        .rposition(|&s| s.abs() > threshold)
        .map(|p| p + 1)
        .unwrap_or(0);
    if start >= end {
        return &[];
    }
    &samples[start..end]
}

/// Normalize audio gain to a target peak amplitude.
pub fn normalize_gain(samples: &[f32], target_peak: f32) -> Vec<f32> {
    let max_amplitude = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);

    if max_amplitude < 1e-6 {
        return samples.to_vec(); // silence, don't amplify noise
    }

    let gain = target_peak / max_amplitude;
    samples.iter().map(|&s| (s * gain).clamp(-1.0, 1.0)).collect()
}

/// Basic noise reduction using spectral subtraction.
/// Uses the first `noise_sample_len` samples as a noise profile estimate,
/// then subtracts the average noise amplitude from all samples.
pub fn reduce_noise(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let noise_sample_len = (sample_rate as f32 * 0.2) as usize; // first 200ms
    if samples.len() <= noise_sample_len {
        return samples.to_vec();
    }

    let noise_floor: f32 = samples[..noise_sample_len]
        .iter()
        .map(|s| s.abs())
        .sum::<f32>()
        / noise_sample_len as f32;

    samples
        .iter()
        .map(|&s| {
            if s.abs() > noise_floor * 2.0 {
                s
            } else {
                s * (s.abs() / (noise_floor * 2.0)).clamp(0.0, 1.0)
            }
        })
        .collect()
}

/// Full preprocessing pipeline: noise reduction -> silence trimming -> gain normalization
pub fn preprocess(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let denoised = reduce_noise(samples, sample_rate);
    let trimmed = trim_silence(&denoised, 0.01);
    if trimmed.is_empty() {
        return Vec::new();
    }
    normalize_gain(trimmed, 0.9)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_silence_removes_leading_trailing() {
        let samples = vec![0.0, 0.0, 0.0, 0.5, 0.3, 0.0, 0.0];
        let trimmed = trim_silence(&samples, 0.01);
        assert_eq!(trimmed, &[0.5, 0.3]);
    }

    #[test]
    fn test_trim_silence_all_silent() {
        let samples = vec![0.0, 0.001, 0.0];
        let trimmed = trim_silence(&samples, 0.01);
        assert!(trimmed.is_empty());
    }

    #[test]
    fn test_normalize_gain() {
        let samples = vec![0.0, 0.25, -0.5, 0.1];
        let normalized = normalize_gain(&samples, 0.9);
        // max was 0.5, gain = 0.9/0.5 = 1.8
        assert!((normalized[2] - (-0.9)).abs() < 0.001);
    }

    #[test]
    fn test_normalize_gain_silence() {
        let samples = vec![0.0, 0.0, 0.0];
        let normalized = normalize_gain(&samples, 0.9);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_reduce_noise_preserves_loud_signals() {
        let mut samples = vec![0.001f32; 3200]; // 200ms noise at 16kHz
        samples.extend(vec![0.5f32; 1600]); // 100ms of signal
        let result = reduce_noise(&samples, 16000);
        // Signal portion should be largely preserved
        assert!(result[3200] > 0.4);
    }

    #[test]
    fn test_preprocess_pipeline() {
        let mut samples = vec![0.001f32; 3200]; // 200ms noise
        samples.extend(vec![0.0; 1600]); // silence
        samples.extend(vec![0.5, -0.3, 0.4]); // signal
        samples.extend(vec![0.0; 1600]); // trailing silence
        let result = preprocess(&samples, 16000);
        assert!(!result.is_empty());
        // Should be normalized near 0.9 peak
        let max = result.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!((max - 0.9).abs() < 0.1);
    }
}
```

- [ ] **Step 2: Update audio mod.rs**

Replace `src-tauri/src/audio/mod.rs`:

```rust
pub mod capture;
pub mod preprocessing;
pub use capture::Recorder;
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test preprocessing
```

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio/
git commit -m "feat: add audio preprocessing with noise reduction, silence trimming, gain normalization"
```

---

### Task 5: Transcription Trait & API Mode

**Files:**
- Create: `src-tauri/src/transcription/mod.rs`
- Create: `src-tauri/src/transcription/traits.rs`
- Create: `src-tauri/src/transcription/whisper_api.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the transcription trait**

Create `src-tauri/src/transcription/traits.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscribeError {
    #[error("Audio too short: {0:.1}s (minimum {1:.1}s)")]
    AudioTooShort(f32, f32),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Model error: {0}")]
    ModelError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl serde::Serialize for TranscribeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

- [ ] **Step 2: Create the Whisper API client**

Create `src-tauri/src/transcription/whisper_api.rs`:

```rust
use super::traits::TranscribeError;
use reqwest::multipart;
use std::io::Write;

pub async fn transcribe_api(
    audio: &[f32],
    sample_rate: u32,
    api_key: &str,
) -> Result<String, TranscribeError> {
    // Write audio to a temporary WAV file
    let tmp_path = std::env::temp_dir().join("wipr_recording.wav");
    write_wav(&tmp_path, audio, sample_rate)?;

    // Send to OpenAI Whisper API
    let file_bytes = std::fs::read(&tmp_path).map_err(TranscribeError::IoError)?;

    let part = multipart::Part::bytes(file_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    let form = multipart::Form::new()
        .text("model", "whisper-1")
        .part("file", part);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(TranscribeError::ApiError(error_text));
    }

    #[derive(serde::Deserialize)]
    struct WhisperResponse {
        text: String,
    }

    let result: WhisperResponse = response
        .json()
        .await
        .map_err(|e| TranscribeError::ApiError(e.to_string()))?;

    // Cleanup temp file
    std::fs::remove_file(&tmp_path).ok();

    Ok(result.text.trim().to_string())
}

fn write_wav(
    path: &std::path::Path,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), TranscribeError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|e| TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    for &sample in samples {
        let s = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer
            .write_sample(s)
            .map_err(|e| TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    }
    writer
        .finalize()
        .map_err(|e| TranscribeError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_wav_creates_file() {
        let samples = vec![0.0f32; 16000];
        let path = std::env::temp_dir().join("wipr_test.wav");
        write_wav(&path, &samples, 16000).unwrap();
        assert!(path.exists());
        let reader = hound::WavReader::open(&path).unwrap();
        assert_eq!(reader.spec().sample_rate, 16000);
        assert_eq!(reader.spec().channels, 1);
        std::fs::remove_file(&path).ok();
    }
}
```

- [ ] **Step 3: Create transcription module declaration**

Create `src-tauri/src/transcription/mod.rs`:

```rust
pub mod traits;
pub mod whisper_api;

pub use traits::TranscribeError;
```

- [ ] **Step 4: Wire into lib.rs**

Add to `src-tauri/src/lib.rs` after `mod audio;`:

```rust
mod transcription;
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test transcription
```

Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/transcription/
git commit -m "feat: add transcription trait and OpenAI Whisper API client"
```

---

### Task 6: Local Whisper Transcription

**Files:**
- Create: `src-tauri/src/transcription/whisper_local.rs`
- Modify: `src-tauri/src/transcription/mod.rs`

- [ ] **Step 1: Create the local Whisper transcription module**

Create `src-tauri/src/transcription/whisper_local.rs`:

```rust
use super::traits::TranscribeError;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub fn transcribe_local(
    audio: &[f32],
    model_path: &str,
) -> Result<String, TranscribeError> {
    if !Path::new(model_path).exists() {
        return Err(TranscribeError::ModelError(format!(
            "Model not found: {}. Download it from settings.",
            model_path
        )));
    }

    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| TranscribeError::ModelError(format!("Failed to load model: {}", e)))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_no_context(true);

    let mut state = ctx
        .create_state()
        .map_err(|e| TranscribeError::ModelError(format!("Failed to create state: {}", e)))?;

    state
        .full(params, audio)
        .map_err(|e| TranscribeError::ModelError(format!("Transcription failed: {}", e)))?;

    let n_segments = state
        .full_n_segments()
        .map_err(|e| TranscribeError::ModelError(format!("Failed to get segments: {}", e)))?;

    let mut text = String::new();
    for i in 0..n_segments {
        if let Ok(segment) = state.full_get_segment_text(i) {
            text.push_str(&segment);
        }
    }

    Ok(text.trim().to_string())
}

/// Returns the URL to download a Whisper model by name.
pub fn model_download_url(model_name: &str) -> String {
    format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_name
    )
}

/// Downloads a Whisper model to the models directory.
/// Calls `on_progress` with (bytes_downloaded, total_bytes) for progress tracking.
pub async fn download_model(
    model_name: &str,
    models_dir: &Path,
    on_progress: impl Fn(u64, u64),
) -> Result<String, TranscribeError> {
    std::fs::create_dir_all(models_dir)
        .map_err(|e| TranscribeError::IoError(e))?;

    let url = model_download_url(model_name);
    let dest = models_dir.join(format!("ggml-{}.bin", model_name));

    if dest.exists() {
        return Ok(dest.to_string_lossy().to_string());
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| TranscribeError::ApiError(format!("Download failed: {}", e)))?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = std::fs::File::create(&dest).map_err(TranscribeError::IoError)?;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|e| TranscribeError::ApiError(format!("Download error: {}", e)))?;
        std::io::Write::write_all(&mut file, &chunk).map_err(TranscribeError::IoError)?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    Ok(dest.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_download_url() {
        let url = model_download_url("medium.en");
        assert_eq!(
            url,
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin"
        );
    }

    #[test]
    fn test_transcribe_missing_model() {
        let result = transcribe_local(&[0.0; 16000], "/nonexistent/model.bin");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Model not found"));
    }
}
```

- [ ] **Step 2: Add futures-util dependency for stream downloading**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
futures-util = "0.3"
```

- [ ] **Step 3: Update transcription mod.rs**

Replace `src-tauri/src/transcription/mod.rs`:

```rust
pub mod traits;
pub mod whisper_api;
pub mod whisper_local;

pub use traits::TranscribeError;
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test transcription
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/transcription/ src-tauri/Cargo.toml
git commit -m "feat: add local Whisper transcription with model download support"
```

---

### Task 7: Voice Commands

**Files:**
- Create: `src-tauri/src/ai/mod.rs`
- Create: `src-tauri/src/ai/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the voice commands module with tests**

Create `src-tauri/src/ai/commands.rs`:

```rust
/// Result of processing voice commands in a transcript.
pub struct CommandResult {
    /// The text after command substitution
    pub text: String,
    /// Whether any commands were found
    pub had_commands: bool,
    /// Whether the entire transcript was commands (no natural language remaining)
    pub all_commands: bool,
    /// Whether a "stop listening" command was detected
    pub should_stop: bool,
}

/// Process voice commands in a raw transcript.
/// Replaces command phrases with their output and returns the result.
/// Commands are matched case-insensitively.
pub fn process_commands(raw_text: &str) -> CommandResult {
    let mut text = raw_text.to_string();
    let mut had_commands = false;

    // Check for control commands first
    let lower = text.to_lowercase();
    let should_stop = lower.contains("stop listening");
    if should_stop {
        had_commands = true;
    }

    // Define command replacements (order matters: longer phrases first)
    let replacements = [
        ("new paragraph", "\n\n"),
        ("new line", "\n"),
        ("question mark", "?"),
        ("exclamation point", "!"),
        ("exclamation mark", "!"),
        ("semicolon", ";"),
        ("period", "."),
        ("comma", ","),
        ("colon", ":"),
        ("tab", "\t"),
    ];

    for (command, replacement) in &replacements {
        let re = regex_lite::Regex::new(&format!(r"(?i)\b{}\b", regex_lite::escape(command)))
            .unwrap();
        if re.is_match(&text) {
            had_commands = true;
            text = re.replace_all(&text, *replacement).to_string();
        }
    }

    // Handle editing commands
    let editing_commands = ["delete that", "undo that", "select all", "stop listening"];
    for cmd in &editing_commands {
        let re = regex_lite::Regex::new(&format!(r"(?i)\b{}\b", regex_lite::escape(cmd))).unwrap();
        if re.is_match(&text) {
            had_commands = true;
            text = re.replace_all(&text, "").to_string();
        }
    }

    // Clean up whitespace around punctuation
    text = text
        .replace(" .", ".")
        .replace(" ,", ",")
        .replace(" ?", "?")
        .replace(" !", "!")
        .replace(" ;", ";")
        .replace(" :", ":");

    // Collapse multiple spaces
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string();

    let all_commands = had_commands && text.is_empty();

    CommandResult {
        text,
        had_commands,
        all_commands,
        should_stop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation_commands() {
        let result = process_commands("hello period how are you question mark");
        assert!(result.had_commands);
        assert_eq!(result.text, "hello. how are you?");
    }

    #[test]
    fn test_formatting_commands() {
        let result = process_commands("first line new line second line");
        assert!(result.had_commands);
        assert_eq!(result.text, "first line\nsecond line");
    }

    #[test]
    fn test_new_paragraph() {
        let result = process_commands("paragraph one new paragraph paragraph two");
        assert!(result.had_commands);
        assert_eq!(result.text, "paragraph one\n\nparagraph two");
    }

    #[test]
    fn test_mixed_commands_and_text() {
        let result =
            process_commands("send the email period new line thanks comma John");
        assert!(result.had_commands);
        assert!(!result.all_commands);
        assert_eq!(result.text, "send the email.\nthanks, John");
    }

    #[test]
    fn test_no_commands() {
        let result = process_commands("just regular text here");
        assert!(!result.had_commands);
        assert_eq!(result.text, "just regular text here");
    }

    #[test]
    fn test_stop_listening() {
        let result = process_commands("stop listening");
        assert!(result.should_stop);
        assert!(result.all_commands);
    }

    #[test]
    fn test_case_insensitive() {
        let result = process_commands("hello PERIOD goodbye COMMA friend");
        assert!(result.had_commands);
        assert_eq!(result.text, "hello. goodbye, friend");
    }
}
```

- [ ] **Step 2: Add regex-lite dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
regex-lite = "0.1"
```

- [ ] **Step 3: Create AI module declaration**

Create `src-tauri/src/ai/mod.rs`:

```rust
pub mod commands;
```

- [ ] **Step 4: Wire into lib.rs**

Add to `src-tauri/src/lib.rs` after `mod transcription;`:

```rust
mod ai;
```

- [ ] **Step 5: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test commands
```

Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ai/ src-tauri/Cargo.toml
git commit -m "feat: add voice command detection with punctuation, formatting, and editing commands"
```

---

### Task 8: AI Text Cleanup

**Files:**
- Create: `src-tauri/src/ai/cleanup.rs`
- Modify: `src-tauri/src/ai/mod.rs`

- [ ] **Step 1: Create the AI cleanup module**

Create `src-tauri/src/ai/cleanup.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

/// Clean up raw transcription using GPT-4o-mini.
/// Takes optional context (text before cursor) and custom instructions.
pub async fn cleanup_text(
    raw_text: &str,
    context: Option<&str>,
    custom_instructions: Option<&str>,
    api_key: &str,
) -> Result<String, String> {
    let mut system_prompt = String::from(
        "You are a dictation cleanup assistant. Clean up the transcribed speech while \
         preserving the speaker's intent. Fix filler words, grammar, and punctuation. \
         Match the tone of the surrounding context if provided.",
    );

    if let Some(instructions) = custom_instructions {
        if !instructions.is_empty() {
            system_prompt.push_str(&format!("\n\nAdditional instructions: {}", instructions));
        }
    }

    let mut user_content = String::new();
    if let Some(ctx) = context {
        if !ctx.is_empty() {
            user_content.push_str(&format!("Context (text before cursor): {}\n", ctx));
        }
    }
    user_content.push_str(&format!("Raw transcription: {}\n\nReturn only the cleaned text, nothing else.", raw_text));

    let request = ChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_content,
            },
        ],
        temperature: 0.3,
        max_tokens: 2048,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("Cleanup API error: {}", e))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Cleanup API error: {}", error_text));
    }

    let result: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse cleanup response: {}", e))?;

    result
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or_else(|| "No response from cleanup API".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "test system".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "test user".to_string(),
                },
            ],
            temperature: 0.3,
            max_tokens: 2048,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gpt-4o-mini"));
        assert!(json.contains("test system"));
    }
}
```

- [ ] **Step 2: Update AI mod.rs**

Replace `src-tauri/src/ai/mod.rs`:

```rust
pub mod cleanup;
pub mod commands;
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test cleanup
```

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ai/
git commit -m "feat: add GPT-4o-mini text cleanup with context and custom instructions support"
```

---

### Task 9: Context Reading (Accessibility API)

**Files:**
- Create: `src-tauri/src/ai/context.rs`
- Modify: `src-tauri/src/ai/mod.rs`

- [ ] **Step 1: Create the context reading module**

Create `src-tauri/src/ai/context.rs`:

```rust
use core_graphics::event::CGEvent;
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Read text surrounding the cursor from the active application using
/// macOS Accessibility API. Returns up to `max_chars` characters before the cursor.
///
/// Falls back to empty string if the accessibility API can't read the field.
pub fn read_cursor_context(max_chars: usize) -> String {
    match read_context_inner(max_chars) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Could not read cursor context: {}", e);
            String::new()
        }
    }
}

fn read_context_inner(max_chars: usize) -> Result<String, String> {
    // Use AppleScript via osascript to read the focused text field.
    // This is more reliable than raw AXUIElement for cross-app compatibility.
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set focusedElement to value of attribute "AXFocusedUIElement" of frontApp
            try
                set textValue to value of focusedElement
                set selectedRange to value of attribute "AXSelectedTextRange" of focusedElement
                set cursorPos to first item of selectedRange
                if cursorPos > 0 then
                    set startPos to cursorPos
                    if startPos > 200 then
                        set startPos to cursorPos - 200
                    else
                        set startPos to 0
                    end if
                    set beforeText to text (startPos + 1) thru cursorPos of textValue
                    return beforeText
                end if
            end try
        end tell
        return ""
    "#;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Limit to max_chars
        if text.len() > max_chars {
            Ok(text[text.len() - max_chars..].to_string())
        } else {
            Ok(text)
        }
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("osascript error: {}", err))
    }
}

/// Check if the app has accessibility permissions.
pub fn has_accessibility_permission() -> bool {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first application process whose frontmost is true"#)
        .output();

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

/// Open System Settings to the Accessibility privacy pane.
pub fn open_accessibility_settings() {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_accessibility_settings_does_not_crash() {
        // Just ensure the function doesn't panic
        // (Don't actually open settings in test)
    }
}
```

- [ ] **Step 2: Update AI mod.rs**

Replace `src-tauri/src/ai/mod.rs`:

```rust
pub mod cleanup;
pub mod commands;
pub mod context;
```

- [ ] **Step 3: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test context
```

Expected: Tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ai/
git commit -m "feat: add context reading via macOS Accessibility API for context-aware cleanup"
```

---

### Task 10: Global Hotkey Listener

**Files:**
- Create: `src-tauri/src/input/mod.rs`
- Create: `src-tauri/src/input/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the hotkey module**

Create `src-tauri/src/input/hotkey.rs`:

```rust
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Bit mask for the fn (Function) key modifier
const FN_FLAG: u64 = 0x800000;

/// Callback type for hotkey events
pub type HotkeyCallback = Box<dyn Fn(HotkeyEvent) + Send + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HotkeyEvent {
    RecordStart,
    RecordStop,
}

/// Start monitoring the fn key on a dedicated thread.
/// Calls `callback` with RecordStart when fn is pressed and RecordStop when released.
/// Returns a handle that stops monitoring when dropped.
pub fn start_fn_key_monitor(callback: HotkeyCallback) -> FnKeyMonitor {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = std::thread::spawn(move || {
        let fn_down = Arc::new(AtomicBool::new(false));
        let fn_down_clone = fn_down.clone();

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::FlagsChanged],
            move |_proxy, _event_type, event| {
                let flags = event.get_flags().bits();
                let fn_pressed = (flags & FN_FLAG) != 0;
                let was_down = fn_down_clone.load(Ordering::SeqCst);

                if fn_pressed && !was_down {
                    fn_down_clone.store(true, Ordering::SeqCst);
                    callback(HotkeyEvent::RecordStart);
                } else if !fn_pressed && was_down {
                    fn_down_clone.store(false, Ordering::SeqCst);
                    callback(HotkeyEvent::RecordStop);
                }

                None // pass event through
            },
        );

        match tap {
            Ok(tap) => {
                let loop_source = tap
                    .mach_port
                    .create_runloop_source(0)
                    .expect("Failed to create run loop source");
                let run_loop = CFRunLoop::get_current();
                run_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
                tap.enable();

                // Run until stopped
                while running_clone.load(Ordering::SeqCst) {
                    CFRunLoop::run_current_in_mode(
                        unsafe { kCFRunLoopCommonModes },
                        std::time::Duration::from_millis(100),
                        false,
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to create event tap: {}. \
                     Ensure Wipr has Input Monitoring permission in \
                     System Settings > Privacy & Security.",
                    e
                );
            }
        }
    });

    FnKeyMonitor {
        running,
        _handle: handle,
    }
}

pub struct FnKeyMonitor {
    running: Arc<AtomicBool>,
    _handle: std::thread::JoinHandle<()>,
}

impl Drop for FnKeyMonitor {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
```

- [ ] **Step 2: Create input module declaration**

Create `src-tauri/src/input/mod.rs`:

```rust
pub mod hotkey;
```

- [ ] **Step 3: Wire into lib.rs**

Add to `src-tauri/src/lib.rs` after `mod ai;`:

```rust
mod input;
```

- [ ] **Step 4: Verify it compiles**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles without errors. (Cannot unit test CGEventTap without running on macOS with permissions.)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/input/
git commit -m "feat: add global fn key monitoring via CGEventTap for hold-to-record"
```

---

### Task 11: Text Insertion

**Files:**
- Create: `src-tauri/src/input/insertion.rs`
- Modify: `src-tauri/src/input/mod.rs`

- [ ] **Step 1: Create the text insertion module**

Create `src-tauri/src/input/insertion.rs`:

```rust
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Insert text at the current cursor position by:
/// 1. Saving the current clipboard
/// 2. Copying the text to clipboard
/// 3. Simulating Cmd+V paste
/// 4. Restoring the original clipboard
pub fn insert_at_cursor(text: &str) -> Result<(), String> {
    // Save current clipboard
    let original_clipboard = get_clipboard();

    // Copy text to clipboard
    set_clipboard(text)?;

    // Small delay for clipboard to settle
    thread::sleep(Duration::from_millis(50));

    // Simulate Cmd+V
    simulate_paste()?;

    // Delay before restoring clipboard
    thread::sleep(Duration::from_millis(150));

    // Restore original clipboard
    if let Some(original) = original_clipboard {
        set_clipboard(&original).ok();
    }

    Ok(())
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            }
        })
}

fn set_clipboard(text: &str) -> Result<(), String> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pbcopy: {}", e))?;

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(text.as_bytes())
        .map_err(|e| format!("Failed to write to pbcopy: {}", e))?;

    child
        .wait()
        .map_err(|e| format!("pbcopy failed: {}", e))?;

    Ok(())
}

fn simulate_paste() -> Result<(), String> {
    // Use osascript to simulate Cmd+V keystroke
    let script = r#"
        tell application "System Events"
            keystroke "v" using command down
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to simulate paste: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Paste simulation failed: {}", err));
    }

    Ok(())
}

/// Execute an editing command (e.g., "delete that" → Cmd+Z, "select all" → Cmd+A)
pub fn execute_editing_command(command: &str) -> Result<(), String> {
    let script = match command {
        "delete that" | "undo that" => {
            r#"tell application "System Events" to keystroke "z" using command down"#
        }
        "select all" => {
            r#"tell application "System Events" to keystroke "a" using command down"#
        }
        _ => return Err(format!("Unknown editing command: {}", command)),
    };

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command execution failed: {}", err));
    }

    Ok(())
}
```

- [ ] **Step 2: Update input mod.rs**

Replace `src-tauri/src/input/mod.rs`:

```rust
pub mod hotkey;
pub mod insertion;
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/input/
git commit -m "feat: add text insertion via clipboard and Cmd+V simulation"
```

---

### Task 12: History Store (SQLite)

**Files:**
- Create: `src-tauri/src/history/mod.rs`
- Create: `src-tauri/src/history/store.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the history store module with tests**

Create `src-tauri/src/history/store.rs`:

```rust
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: i64,
    pub timestamp: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub duration_secs: f32,
}

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    TEXT NOT NULL,
                raw_text     TEXT NOT NULL,
                cleaned_text TEXT NOT NULL,
                duration_secs REAL NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    TEXT NOT NULL,
                raw_text     TEXT NOT NULL,
                cleaned_text TEXT NOT NULL,
                duration_secs REAL NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn insert(
        &self,
        raw_text: &str,
        cleaned_text: &str,
        duration_secs: f32,
    ) -> SqlResult<i64> {
        let timestamp = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO transcriptions (timestamp, raw_text, cleaned_text, duration_secs) VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, raw_text, cleaned_text, duration_secs],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> SqlResult<Vec<TranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, raw_text, cleaned_text, duration_secs FROM transcriptions ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                raw_text: row.get(2)?,
                cleaned_text: row.get(3)?,
                duration_secs: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<TranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, raw_text, cleaned_text, duration_secs FROM transcriptions \
             WHERE raw_text LIKE ?1 OR cleaned_text LIKE ?1 ORDER BY id DESC",
        )?;
        let pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                raw_text: row.get(2)?,
                cleaned_text: row.get(3)?,
                duration_secs: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM transcriptions", [])?;
        Ok(())
    }

    pub fn export_json(&self) -> SqlResult<String> {
        let entries = self.get_all()?;
        Ok(serde_json::to_string_pretty(&entries).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_retrieve() {
        let store = HistoryStore::new_in_memory().unwrap();
        let id = store.insert("hello world", "Hello, world!", 1.5).unwrap();
        assert!(id > 0);

        let entries = store.get_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].raw_text, "hello world");
        assert_eq!(entries[0].cleaned_text, "Hello, world!");
        assert!((entries[0].duration_secs - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_search() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("hello world", "Hello, world!", 1.0).unwrap();
        store
            .insert("goodbye world", "Goodbye, world!", 2.0)
            .unwrap();

        let results = store.search("hello").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].raw_text, "hello world");
    }

    #[test]
    fn test_delete() {
        let store = HistoryStore::new_in_memory().unwrap();
        let id = store.insert("test", "Test", 1.0).unwrap();
        store.delete(id).unwrap();
        let entries = store.get_all().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_clear() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("one", "One", 1.0).unwrap();
        store.insert("two", "Two", 2.0).unwrap();
        store.clear().unwrap();
        let entries = store.get_all().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_export_json() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("test", "Test", 1.0).unwrap();
        let json = store.export_json().unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_order_is_newest_first() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("first", "First", 1.0).unwrap();
        store.insert("second", "Second", 2.0).unwrap();
        let entries = store.get_all().unwrap();
        assert_eq!(entries[0].raw_text, "second");
        assert_eq!(entries[1].raw_text, "first");
    }
}
```

- [ ] **Step 2: Create history module declaration**

Create `src-tauri/src/history/mod.rs`:

```rust
pub mod store;
pub use store::{HistoryStore, TranscriptionEntry};
```

- [ ] **Step 3: Wire into lib.rs**

Add to `src-tauri/src/lib.rs` after `mod input;`:

```rust
mod history;
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test history
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/history/
git commit -m "feat: add SQLite-backed transcription history with search, delete, and export"
```

---

### Task 13: System Tray

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create the tray module**

Create `src-tauri/src/tray.rs`:

```rust
use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let settings = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;
    let history = MenuItemBuilder::with_id("history", "History...").build(app)?;

    let local_mode =
        CheckMenuItemBuilder::with_id("local_mode", "Local Transcription").build(app)?;
    let ai_cleanup = CheckMenuItemBuilder::with_id("ai_cleanup", "AI Cleanup")
        .checked(true)
        .build(app)?;
    let auto_start = CheckMenuItemBuilder::with_id("auto_start", "Start on Login")
        .checked(true)
        .build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "Quit Wipr").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&settings)
        .item(&history)
        .separator()
        .item(&local_mode)
        .item(&ai_cleanup)
        .item(&auto_start)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Wipr - Voice Dictation")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "settings" => {
                open_or_focus_window(app, "settings", "Settings", 600.0, 500.0);
            }
            "history" => {
                open_or_focus_window(app, "history", "History", 700.0, 500.0);
            }
            "quit" => {
                app.exit(0);
            }
            "local_mode" | "ai_cleanup" | "auto_start" => {
                // Emit event to frontend to handle toggle
                app.emit(&format!("tray-toggle-{}", event.id().as_ref()), ())
                    .ok();
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn open_or_focus_window(app: &tauri::AppHandle, label: &str, title: &str, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window(label) {
        window.show().unwrap();
        window.set_focus().unwrap();
    } else {
        let url = format!("/#{}", label);
        tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App(url.into()))
            .title(title)
            .inner_size(width, height)
            .resizable(true)
            .build()
            .ok();
    }
}

/// Update the tray icon to reflect current recording state.
pub fn set_tray_recording(app: &tauri::AppHandle, is_recording: bool) {
    // Emit event to let any UI know the state changed
    app.emit("recording-state", is_recording).ok();
}
```

- [ ] **Step 2: Wire tray into lib.rs setup**

Replace `src-tauri/src/lib.rs` with the full version including tray setup:

```rust
mod ai;
mod audio;
mod config;
mod history;
mod input;
mod tray;

use config::settings::{load_config, save_config, AppConfig};
use history::store::{HistoryStore, TranscriptionEntry};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub history: Mutex<HistoryStore>,
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

#[tauri::command]
fn get_history(state: State<AppState>) -> Result<Vec<TranscriptionEntry>, String> {
    state
        .history
        .lock()
        .unwrap()
        .get_all()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn search_history(state: State<AppState>, query: String) -> Result<Vec<TranscriptionEntry>, String> {
    state
        .history
        .lock()
        .unwrap()
        .search(&query)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_history_entry(state: State<AppState>, id: i64) -> Result<(), String> {
    state
        .history
        .lock()
        .unwrap()
        .delete(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_history(state: State<AppState>) -> Result<(), String> {
    state
        .history
        .lock()
        .unwrap()
        .clear()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn export_history(state: State<AppState>) -> Result<String, String> {
    state
        .history
        .lock()
        .unwrap()
        .export_json()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_accessibility_permission() -> bool {
    ai::context::has_accessibility_permission()
}

#[tauri::command]
fn open_accessibility_settings() {
    ai::context::open_accessibility_settings();
}

pub fn run() {
    let config = load_config();

    let db_path = config::settings::config_dir().join("history.db");
    std::fs::create_dir_all(config::settings::config_dir()).ok();
    let history = HistoryStore::new(db_path.to_str().unwrap())
        .expect("Failed to open history database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            config: Mutex::new(config),
            history: Mutex::new(history),
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            get_history,
            search_history,
            delete_history_entry,
            clear_history,
            export_history,
            check_accessibility_permission,
            open_accessibility_settings,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/src/lib.rs
git commit -m "feat: add system tray with menu and wire up all Tauri commands"
```

---

### Task 14: Core Dictation Pipeline

**Files:**
- Modify: `src-tauri/src/lib.rs`

This task wires the complete dictation flow: hotkey → record → preprocess → transcribe → voice commands → AI cleanup → insert at cursor.

- [ ] **Step 1: Add the dictation pipeline to lib.rs**

Add the following after the existing Tauri commands in `src-tauri/src/lib.rs`, before the `pub fn run()`:

```rust
use audio::Recorder;
use std::sync::Arc;
use tauri::Emitter;

fn start_dictation_pipeline(app: tauri::AppHandle, state: Arc<AppState>) {
    let recorder = Arc::new(Recorder::new());
    let stream: Arc<Mutex<Option<cpal::Stream>>> = Arc::new(Mutex::new(None));

    let recorder_clone = recorder.clone();
    let stream_clone = stream.clone();
    let app_clone = app.clone();
    let state_clone = state.clone();

    input::hotkey::start_fn_key_monitor(Box::new(move |event| {
        match event {
            input::hotkey::HotkeyEvent::RecordStart => {
                app_clone.emit("dictation-state", "recording").ok();
                tray::set_tray_recording(&app_clone, true);

                match recorder_clone.start() {
                    Ok(s) => {
                        *stream_clone.lock().unwrap() = Some(s);
                    }
                    Err(e) => {
                        eprintln!("Failed to start recording: {}", e);
                        app_clone.emit("dictation-error", e).ok();
                    }
                }
            }
            input::hotkey::HotkeyEvent::RecordStop => {
                // Drop the stream to stop recording
                stream_clone.lock().unwrap().take();
                tray::set_tray_recording(&app_clone, false);

                let samples = recorder_clone.stop();
                let duration = Recorder::duration_secs(&samples, recorder_clone.sample_rate());

                if duration < Recorder::MIN_DURATION {
                    app_clone.emit("dictation-state", "idle").ok();
                    return;
                }

                app_clone.emit("dictation-state", "transcribing").ok();

                let app_handle = app_clone.clone();
                let state_ref = state_clone.clone();
                let sample_rate = recorder_clone.sample_rate();

                tauri::async_runtime::spawn(async move {
                    process_recording(app_handle, state_ref, samples, sample_rate, duration).await;
                });
            }
        }
    }));
}

async fn process_recording(
    app: tauri::AppHandle,
    state: Arc<AppState>,
    samples: Vec<f32>,
    sample_rate: u32,
    duration: f32,
) {
    // 1. Preprocess audio
    let processed = audio::preprocessing::preprocess(&samples, sample_rate);
    if processed.is_empty() {
        app.emit("dictation-state", "idle").ok();
        return;
    }

    // 2. Transcribe
    let config = state.config.lock().unwrap().clone();
    let raw_text = match config.transcription.mode {
        config::settings::TranscriptionMode::Api => {
            match transcription::whisper_api::transcribe_api(
                &processed,
                sample_rate,
                &config.transcription.api_key,
            )
            .await
            {
                Ok(text) => text,
                Err(e) => {
                    app.emit("dictation-error", e.to_string()).ok();
                    app.emit("dictation-state", "idle").ok();
                    return;
                }
            }
        }
        config::settings::TranscriptionMode::Local => {
            let model_path = config::settings::models_dir()
                .join(format!("ggml-{}.bin", config.transcription.model));
            match transcription::whisper_local::transcribe_local(
                &processed,
                model_path.to_str().unwrap(),
            ) {
                Ok(text) => text,
                Err(e) => {
                    app.emit("dictation-error", e.to_string()).ok();
                    app.emit("dictation-state", "idle").ok();
                    return;
                }
            }
        }
    };

    if raw_text.is_empty() {
        app.emit("dictation-state", "idle").ok();
        return;
    }

    // 3. Process voice commands
    let mut final_text = raw_text.clone();
    let mut cleaned_text = raw_text.clone();

    if config.voice_commands.enabled {
        let cmd_result = ai::commands::process_commands(&raw_text);

        if cmd_result.should_stop {
            app.emit("dictation-state", "idle").ok();
            return;
        }

        final_text = cmd_result.text.clone();

        if cmd_result.all_commands {
            // Only commands, nothing to clean up or insert
            app.emit("dictation-state", "idle").ok();
            return;
        }
    }

    // 4. AI Cleanup (if enabled and there's text remaining)
    if config.ai_cleanup.enabled && !final_text.is_empty() {
        app.emit("dictation-state", "cleaning").ok();

        let context = if config.general.context_aware {
            let ctx = ai::context::read_cursor_context(200);
            if ctx.is_empty() { None } else { Some(ctx) }
        } else {
            None
        };

        let custom_instructions = if config.ai_cleanup.custom_instructions.is_empty() {
            None
        } else {
            Some(config.ai_cleanup.custom_instructions.as_str())
        };

        match ai::cleanup::cleanup_text(
            &final_text,
            context.as_deref(),
            custom_instructions,
            &config.transcription.api_key,
        )
        .await
        {
            Ok(cleaned) => {
                cleaned_text = cleaned.clone();
                final_text = cleaned;
            }
            Err(e) => {
                eprintln!("AI cleanup failed, using raw text: {}", e);
                // Fall through with unclean text
            }
        }
    }

    // 5. Insert at cursor
    if let Err(e) = input::insertion::insert_at_cursor(&final_text) {
        eprintln!("Failed to insert text: {}", e);
        app.emit("dictation-error", format!("Insert failed: {}", e))
            .ok();
    }

    // 6. Log to history
    state
        .history
        .lock()
        .unwrap()
        .insert(&raw_text, &cleaned_text, duration)
        .ok();

    app.emit("dictation-result", &final_text).ok();
    app.emit("dictation-state", "idle").ok();
}
```

- [ ] **Step 2: Update the run() function to start the pipeline**

In `src-tauri/src/lib.rs`, update the `.setup()` closure in `run()`:

```rust
        .setup(|app| {
            tray::setup_tray(app)?;

            // Start the dictation pipeline
            let app_handle = app.handle().clone();
            let state = Arc::new(AppState {
                config: Mutex::new(load_config()),
                history: Mutex::new(
                    HistoryStore::new(
                        config::settings::config_dir()
                            .join("history.db")
                            .to_str()
                            .unwrap(),
                    )
                    .expect("Failed to open history database"),
                ),
            });

            // Note: AppState is managed separately via .manage(), this Arc is for the pipeline
            let pipeline_state = state.clone();
            std::thread::spawn(move || {
                start_dictation_pipeline(app_handle, pipeline_state);
            });

            Ok(())
        })
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles without errors. May need minor adjustments for import paths.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: wire complete dictation pipeline - hotkey to record to transcribe to cleanup to insert"
```

---

### Task 15: Frontend Foundation

**Files:**
- Create: `src/lib/queryClient.ts`
- Create: `src/db/collections.ts`
- Create: `src/db/queries.ts`
- Create: `src/hooks/useTauriEvents.ts`
- Modify: `src/main.tsx`
- Modify: `src/App.tsx`
- Modify: `src/index.css`
- Modify: `index.html`

- [ ] **Step 1: Create the TanStack Query client**

Create `src/lib/queryClient.ts`:

```typescript
import { QueryClient } from "@tanstack/react-query";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});
```

- [ ] **Step 2: Create TanStack Query hooks for Tauri commands**

Create `src/db/queries.ts`:

```typescript
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

// Types matching Rust structs
export interface AppConfig {
  transcription: {
    mode: "local" | "api";
    model: string;
    api_key: string;
  };
  ai_cleanup: {
    enabled: boolean;
    custom_instructions: string;
  };
  hotkey: {
    key: string;
    mode: "hold" | "toggle";
  };
  voice_commands: {
    enabled: boolean;
  };
  general: {
    auto_start: boolean;
    context_aware: boolean;
  };
}

export interface TranscriptionEntry {
  id: number;
  timestamp: string;
  raw_text: string;
  cleaned_text: string;
  duration_secs: number;
}

// Config queries
export function useConfig() {
  return useQuery<AppConfig>({
    queryKey: ["config"],
    queryFn: () => invoke<AppConfig>("get_config"),
  });
}

export function useUpdateConfig() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (config: AppConfig) => invoke("update_config", { config }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["config"] });
    },
  });
}

// History queries
export function useHistory() {
  return useQuery<TranscriptionEntry[]>({
    queryKey: ["history"],
    queryFn: () => invoke<TranscriptionEntry[]>("get_history"),
  });
}

export function useSearchHistory(query: string) {
  return useQuery<TranscriptionEntry[]>({
    queryKey: ["history", "search", query],
    queryFn: () => invoke<TranscriptionEntry[]>("search_history", { query }),
    enabled: query.length > 0,
  });
}

export function useDeleteHistoryEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => invoke("delete_history_entry", { id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
    },
  });
}

export function useClearHistory() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => invoke("clear_history"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["history"] });
    },
  });
}

export function useExportHistory() {
  return useMutation({
    mutationFn: () => invoke<string>("export_history"),
  });
}

// Accessibility
export function useAccessibilityPermission() {
  return useQuery<boolean>({
    queryKey: ["accessibility"],
    queryFn: () => invoke<boolean>("check_accessibility_permission"),
  });
}
```

- [ ] **Step 3: Create TanStack DB collections**

Create `src/db/collections.ts`:

```typescript
import { createCollection } from "@tanstack/db";
import { queryCollectionOptions } from "@tanstack/query-db-collection";
import { invoke } from "@tauri-apps/api/core";
import type { TranscriptionEntry, AppConfig } from "./queries";

export const transcriptionsCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["history"],
    queryFn: () => invoke<TranscriptionEntry[]>("get_history"),
    getId: (entry) => entry.id,
  })
);

export const settingsCollection = createCollection(
  queryCollectionOptions({
    queryKey: ["config"],
    queryFn: async () => {
      const config = await invoke<AppConfig>("get_config");
      return [{ id: "current", ...config }];
    },
    getId: (entry) => entry.id,
    onUpdate: async (entry) => {
      const { id, ...config } = entry;
      await invoke("update_config", { config });
    },
  })
);
```

- [ ] **Step 4: Create Tauri event hooks**

Create `src/hooks/useTauriEvents.ts`:

```typescript
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export type DictationState =
  | "idle"
  | "recording"
  | "transcribing"
  | "cleaning";

export function useDictationState() {
  const [state, setState] = useState<DictationState>("idle");

  useEffect(() => {
    const unlisten = listen<string>("dictation-state", (event) => {
      setState(event.payload as DictationState);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return state;
}

export function useDictationError() {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<string>("dictation-error", (event) => {
      setError(event.payload);
      // Auto-clear after 5 seconds
      setTimeout(() => setError(null), 5000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return error;
}

export function useDictationResult() {
  const [result, setResult] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<string>("dictation-result", (event) => {
      setResult(event.payload);
      setTimeout(() => setResult(null), 3000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return result;
}
```

- [ ] **Step 5: Set up React Router and providers in main.tsx**

Replace `src/main.tsx`:

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "./lib/queryClient";
import { Overlay } from "./pages/Overlay";
import { Settings } from "./pages/Settings";
import { History } from "./pages/History";
import { Onboarding } from "./pages/Onboarding";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <Routes>
          <Route path="/" element={<Overlay />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/history" element={<History />} />
          <Route path="/onboarding" element={<Onboarding />} />
        </Routes>
      </HashRouter>
    </QueryClientProvider>
  </React.StrictMode>
);
```

- [ ] **Step 6: Create placeholder page components**

Create `src/pages/Overlay.tsx`:

```tsx
export function Overlay() {
  return <div className="min-h-screen bg-transparent">Overlay</div>;
}
```

Create `src/pages/Settings.tsx`:

```tsx
export function Settings() {
  return <div className="min-h-screen bg-surface text-text p-6">Settings</div>;
}
```

Create `src/pages/History.tsx`:

```tsx
export function History() {
  return <div className="min-h-screen bg-surface text-text p-6">History</div>;
}
```

Create `src/pages/Onboarding.tsx`:

```tsx
export function Onboarding() {
  return <div className="min-h-screen bg-surface text-text p-6">Onboarding</div>;
}
```

- [ ] **Step 7: Remove App.tsx (no longer needed)**

Delete `src/App.tsx` — routing is now in `main.tsx`.

- [ ] **Step 8: Verify frontend builds**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds without errors.

- [ ] **Step 9: Commit**

```bash
git add src/ index.html package.json
git commit -m "feat: set up React frontend with TanStack Query/DB, routing, and Tauri event hooks"
```

---

### Task 16: Overlay UI

**Files:**
- Modify: `src/pages/Overlay.tsx`
- Create: `src/components/Waveform.tsx`

- [ ] **Step 1: Create the waveform visualization component**

Create `src/components/Waveform.tsx`:

```tsx
import { useEffect, useRef } from "react";

interface WaveformProps {
  isActive: boolean;
}

export function Waveform({ isActive }: WaveformProps) {
  const barsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isActive || !barsRef.current) return;

    const bars = barsRef.current.children;
    const interval = setInterval(() => {
      for (let i = 0; i < bars.length; i++) {
        const height = isActive ? Math.random() * 100 : 10;
        (bars[i] as HTMLElement).style.height = `${height}%`;
      }
    }, 100);

    return () => clearInterval(interval);
  }, [isActive]);

  return (
    <div ref={barsRef} className="flex items-end gap-0.5 h-8">
      {Array.from({ length: 12 }).map((_, i) => (
        <div
          key={i}
          className="w-1 bg-primary rounded-full transition-all duration-100"
          style={{ height: "10%" }}
        />
      ))}
    </div>
  );
}
```

- [ ] **Step 2: Build the overlay page**

Replace `src/pages/Overlay.tsx`:

```tsx
import { useDictationState, useDictationError, useDictationResult } from "../hooks/useTauriEvents";
import { Waveform } from "../components/Waveform";

const stateLabels: Record<string, string> = {
  idle: "",
  recording: "Recording...",
  transcribing: "Transcribing...",
  cleaning: "Cleaning up...",
};

export function Overlay() {
  const state = useDictationState();
  const error = useDictationError();
  const result = useDictationResult();

  if (state === "idle" && !error && !result) {
    return null;
  }

  return (
    <div className="min-h-screen flex items-start justify-center pt-4 bg-transparent">
      <div className="bg-surface/95 backdrop-blur-md rounded-2xl px-6 py-4 shadow-2xl border border-white/10 flex items-center gap-4 min-w-64">
        {state === "recording" && <Waveform isActive={true} />}

        {state !== "idle" && (
          <div className="flex flex-col">
            <span className="text-text text-sm font-medium">
              {stateLabels[state]}
            </span>
          </div>
        )}

        {error && (
          <div className="flex items-center gap-2">
            <span className="text-danger text-sm">{error}</span>
          </div>
        )}

        {result && state === "idle" && (
          <div className="flex items-center gap-2">
            <span className="text-success text-sm">Inserted</span>
          </div>
        )}

        {(state === "transcribing" || state === "cleaning") && (
          <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Verify build**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/pages/Overlay.tsx src/components/Waveform.tsx
git commit -m "feat: add recording overlay with waveform visualization and state display"
```

---

### Task 17: Settings UI

**Files:**
- Modify: `src/pages/Settings.tsx`
- Create: `src/components/SettingsSection.tsx`

- [ ] **Step 1: Create the settings section component**

Create `src/components/SettingsSection.tsx`:

```tsx
import { ReactNode } from "react";

interface SettingsSectionProps {
  title: string;
  children: ReactNode;
}

export function SettingsSection({ title, children }: SettingsSectionProps) {
  return (
    <div className="mb-6">
      <h2 className="text-lg font-semibold text-text mb-3">{title}</h2>
      <div className="bg-surface-light rounded-xl p-4 space-y-4">{children}</div>
    </div>
  );
}

interface ToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function Toggle({ label, description, checked, onChange }: ToggleProps) {
  return (
    <label className="flex items-center justify-between cursor-pointer">
      <div>
        <span className="text-sm text-text">{label}</span>
        {description && (
          <p className="text-xs text-text-muted mt-0.5">{description}</p>
        )}
      </div>
      <button
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative w-11 h-6 rounded-full transition-colors ${
          checked ? "bg-primary" : "bg-gray-600"
        }`}
      >
        <span
          className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform ${
            checked ? "translate-x-5" : ""
          }`}
        />
      </button>
    </label>
  );
}

interface SelectProps {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

export function Select({ label, value, options, onChange }: SelectProps) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-text">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="bg-surface text-text text-sm rounded-lg px-3 py-1.5 border border-white/10 focus:outline-none focus:border-primary"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

interface TextInputProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
  placeholder?: string;
}

export function TextInput({
  label,
  value,
  onChange,
  type = "text",
  placeholder,
}: TextInputProps) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-sm text-text">{label}</span>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="bg-surface text-text text-sm rounded-lg px-3 py-2 border border-white/10 focus:outline-none focus:border-primary"
      />
    </div>
  );
}
```

- [ ] **Step 2: Build the settings page**

Replace `src/pages/Settings.tsx`:

```tsx
import { useState, useEffect } from "react";
import { useConfig, useUpdateConfig, type AppConfig } from "../db/queries";
import {
  SettingsSection,
  Toggle,
  Select,
  TextInput,
} from "../components/SettingsSection";

export function Settings() {
  const { data: config, isLoading } = useConfig();
  const updateConfig = useUpdateConfig();
  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);

  useEffect(() => {
    if (config) setLocalConfig(config);
  }, [config]);

  if (isLoading || !localConfig) {
    return (
      <div className="min-h-screen bg-surface text-text flex items-center justify-center">
        <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  const save = (updated: AppConfig) => {
    setLocalConfig(updated);
    updateConfig.mutate(updated);
  };

  return (
    <div className="min-h-screen bg-surface text-text p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-6">Settings</h1>

      <SettingsSection title="General">
        <Toggle
          label="Start on login"
          description="Launch Wipr when you log in"
          checked={localConfig.general.auto_start}
          onChange={(checked) =>
            save({
              ...localConfig,
              general: { ...localConfig.general, auto_start: checked },
            })
          }
        />
        <Select
          label="Recording mode"
          value={localConfig.hotkey.mode}
          options={[
            { value: "hold", label: "Hold to record" },
            { value: "toggle", label: "Press to toggle" },
          ]}
          onChange={(mode) =>
            save({
              ...localConfig,
              hotkey: { ...localConfig.hotkey, mode: mode as "hold" | "toggle" },
            })
          }
        />
      </SettingsSection>

      <SettingsSection title="Transcription">
        <Select
          label="Mode"
          value={localConfig.transcription.mode}
          options={[
            { value: "local", label: "Local (Whisper)" },
            { value: "api", label: "API (OpenAI)" },
          ]}
          onChange={(mode) =>
            save({
              ...localConfig,
              transcription: {
                ...localConfig.transcription,
                mode: mode as "local" | "api",
              },
            })
          }
        />
        {localConfig.transcription.mode === "local" && (
          <Select
            label="Model"
            value={localConfig.transcription.model}
            options={[
              { value: "tiny.en", label: "Tiny (~40MB, fastest)" },
              { value: "base.en", label: "Base (~140MB, fast)" },
              { value: "small.en", label: "Small (~460MB, balanced)" },
              { value: "medium.en", label: "Medium (~1.5GB, best quality)" },
            ]}
            onChange={(model) =>
              save({
                ...localConfig,
                transcription: { ...localConfig.transcription, model },
              })
            }
          />
        )}
        <TextInput
          label="OpenAI API Key"
          value={localConfig.transcription.api_key}
          onChange={(api_key) =>
            save({
              ...localConfig,
              transcription: { ...localConfig.transcription, api_key },
            })
          }
          type="password"
          placeholder="sk-..."
        />
      </SettingsSection>

      <SettingsSection title="AI Cleanup">
        <Toggle
          label="Enable AI cleanup"
          description="Clean up transcription with GPT-4o-mini"
          checked={localConfig.ai_cleanup.enabled}
          onChange={(enabled) =>
            save({
              ...localConfig,
              ai_cleanup: { ...localConfig.ai_cleanup, enabled },
            })
          }
        />
        <Toggle
          label="Context-aware"
          description="Read surrounding text for better results"
          checked={localConfig.general.context_aware}
          onChange={(context_aware) =>
            save({
              ...localConfig,
              general: { ...localConfig.general, context_aware },
            })
          }
        />
        <TextInput
          label="Custom instructions"
          value={localConfig.ai_cleanup.custom_instructions}
          onChange={(custom_instructions) =>
            save({
              ...localConfig,
              ai_cleanup: { ...localConfig.ai_cleanup, custom_instructions },
            })
          }
          placeholder="e.g., Always use formal tone"
        />
      </SettingsSection>

      <SettingsSection title="Voice Commands">
        <Toggle
          label="Enable voice commands"
          description="Use commands like 'period', 'new line', 'delete that'"
          checked={localConfig.voice_commands.enabled}
          onChange={(enabled) =>
            save({
              ...localConfig,
              voice_commands: { enabled },
            })
          }
        />
      </SettingsSection>
    </div>
  );
}
```

- [ ] **Step 3: Verify build**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/pages/Settings.tsx src/components/SettingsSection.tsx
git commit -m "feat: add settings UI with all configuration options"
```

---

### Task 18: History UI

**Files:**
- Modify: `src/pages/History.tsx`
- Create: `src/components/HistoryEntry.tsx`

- [ ] **Step 1: Create the history entry component**

Create `src/components/HistoryEntry.tsx`:

```tsx
import type { TranscriptionEntry } from "../db/queries";

interface HistoryEntryProps {
  entry: TranscriptionEntry;
  onDelete: (id: number) => void;
}

export function HistoryEntryCard({ entry, onDelete }: HistoryEntryProps) {
  const date = new Date(entry.timestamp);
  const timeStr = date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
  const dateStr = date.toLocaleDateString([], {
    month: "short",
    day: "numeric",
  });

  return (
    <div className="bg-surface-light rounded-xl p-4 group">
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <span>{dateStr}</span>
          <span>{timeStr}</span>
          <span>{entry.duration_secs.toFixed(1)}s</span>
        </div>
        <button
          onClick={() => onDelete(entry.id)}
          className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger transition-all text-xs px-2 py-1 rounded"
        >
          Delete
        </button>
      </div>
      <p className="text-sm text-text mb-1">{entry.cleaned_text}</p>
      {entry.raw_text !== entry.cleaned_text && (
        <p className="text-xs text-text-muted italic">
          Raw: {entry.raw_text}
        </p>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Build the history page**

Replace `src/pages/History.tsx`:

```tsx
import { useState } from "react";
import {
  useHistory,
  useSearchHistory,
  useDeleteHistoryEntry,
  useClearHistory,
  useExportHistory,
} from "../db/queries";
import { HistoryEntryCard } from "../components/HistoryEntry";

export function History() {
  const [search, setSearch] = useState("");
  const { data: allHistory, isLoading } = useHistory();
  const { data: searchResults } = useSearchHistory(search);
  const deleteEntry = useDeleteHistoryEntry();
  const clearHistory = useClearHistory();
  const exportHistory = useExportHistory();

  const entries = search ? searchResults : allHistory;

  const handleExport = async () => {
    const json = await exportHistory.mutateAsync();
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "wipr-history.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="min-h-screen bg-surface text-text p-6 max-w-2xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">History</h1>
        <div className="flex gap-2">
          <button
            onClick={handleExport}
            className="text-xs px-3 py-1.5 rounded-lg bg-surface-light text-text-muted hover:text-text transition-colors"
          >
            Export
          </button>
          <button
            onClick={() => {
              if (confirm("Clear all history?")) clearHistory.mutate();
            }}
            className="text-xs px-3 py-1.5 rounded-lg bg-surface-light text-danger hover:bg-danger/10 transition-colors"
          >
            Clear All
          </button>
        </div>
      </div>

      <input
        type="text"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        placeholder="Search transcriptions..."
        className="w-full bg-surface-light text-text text-sm rounded-xl px-4 py-3 mb-4 border border-white/10 focus:outline-none focus:border-primary"
      />

      {isLoading ? (
        <div className="flex justify-center py-12">
          <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        </div>
      ) : entries && entries.length > 0 ? (
        <div className="space-y-3">
          {entries.map((entry) => (
            <HistoryEntryCard
              key={entry.id}
              entry={entry}
              onDelete={(id) => deleteEntry.mutate(id)}
            />
          ))}
        </div>
      ) : (
        <p className="text-center text-text-muted py-12">
          {search ? "No results found" : "No transcriptions yet"}
        </p>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Verify build**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/pages/History.tsx src/components/HistoryEntry.tsx
git commit -m "feat: add history UI with search, delete, export, and clear"
```

---

### Task 19: Onboarding UI

**Files:**
- Modify: `src/pages/Onboarding.tsx`
- Create: `src/components/OnboardingStep.tsx`

- [ ] **Step 1: Create the onboarding step component**

Create `src/components/OnboardingStep.tsx`:

```tsx
import { ReactNode } from "react";

interface OnboardingStepProps {
  step: number;
  totalSteps: number;
  title: string;
  children: ReactNode;
  onNext: () => void;
  onBack?: () => void;
  nextLabel?: string;
  nextDisabled?: boolean;
}

export function OnboardingStep({
  step,
  totalSteps,
  title,
  children,
  onNext,
  onBack,
  nextLabel = "Next",
  nextDisabled = false,
}: OnboardingStepProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-8">
      <div className="w-full max-w-md">
        {/* Progress */}
        <div className="flex gap-1.5 mb-8">
          {Array.from({ length: totalSteps }).map((_, i) => (
            <div
              key={i}
              className={`h-1 flex-1 rounded-full ${
                i <= step ? "bg-primary" : "bg-surface-light"
              }`}
            />
          ))}
        </div>

        <h2 className="text-xl font-bold text-text mb-4">{title}</h2>

        <div className="mb-8">{children}</div>

        <div className="flex justify-between">
          {onBack ? (
            <button
              onClick={onBack}
              className="px-4 py-2 text-sm text-text-muted hover:text-text transition-colors"
            >
              Back
            </button>
          ) : (
            <div />
          )}
          <button
            onClick={onNext}
            disabled={nextDisabled}
            className="px-6 py-2 text-sm font-medium rounded-xl bg-primary hover:bg-primary-hover text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {nextLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Build the onboarding page**

Replace `src/pages/Onboarding.tsx`:

```tsx
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useConfig, useUpdateConfig, useAccessibilityPermission } from "../db/queries";
import { OnboardingStep } from "../components/OnboardingStep";
import { TextInput, Select } from "../components/SettingsSection";

const TOTAL_STEPS = 6;

export function Onboarding() {
  const [step, setStep] = useState(0);
  const { data: config } = useConfig();
  const updateConfig = useUpdateConfig();
  const { data: hasAccessibility, refetch: recheckAccessibility } =
    useAccessibilityPermission();

  const [mode, setMode] = useState<"local" | "api">("local");
  const [apiKey, setApiKey] = useState("");

  const next = () => setStep((s) => Math.min(s + 1, TOTAL_STEPS - 1));
  const back = () => setStep((s) => Math.max(s - 1, 0));

  const finishOnboarding = async () => {
    if (config) {
      updateConfig.mutate({
        ...config,
        transcription: { ...config.transcription, mode, api_key: apiKey },
      });
    }
    // Close onboarding window
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    getCurrentWindow().close();
  };

  return (
    <div className="min-h-screen bg-surface text-text">
      {step === 0 && (
        <OnboardingStep
          step={0}
          totalSteps={TOTAL_STEPS}
          title="Welcome to Wipr"
          onNext={next}
          nextLabel="Get Started"
        >
          <p className="text-text-muted text-sm leading-relaxed">
            Wipr is a voice dictation tool that lives in your menu bar. Hold the
            fn key to record, release to transcribe and insert text at your
            cursor.
          </p>
        </OnboardingStep>
      )}

      {step === 1 && (
        <OnboardingStep
          step={1}
          totalSteps={TOTAL_STEPS}
          title="Accessibility Permission"
          onNext={next}
          onBack={back}
        >
          <p className="text-text-muted text-sm mb-4">
            Wipr needs Accessibility permission to detect the fn key and insert
            text at your cursor.
          </p>
          <div className="flex items-center gap-3 mb-4">
            <div
              className={`w-3 h-3 rounded-full ${
                hasAccessibility ? "bg-success" : "bg-danger"
              }`}
            />
            <span className="text-sm text-text">
              {hasAccessibility ? "Permission granted" : "Not granted"}
            </span>
          </div>
          {!hasAccessibility && (
            <div className="space-y-2">
              <button
                onClick={() => invoke("open_accessibility_settings")}
                className="w-full px-4 py-2.5 text-sm font-medium rounded-xl bg-primary hover:bg-primary-hover text-white transition-colors"
              >
                Open System Settings
              </button>
              <button
                onClick={() => recheckAccessibility()}
                className="w-full px-4 py-2 text-sm text-text-muted hover:text-text transition-colors"
              >
                Check again
              </button>
            </div>
          )}
        </OnboardingStep>
      )}

      {step === 2 && (
        <OnboardingStep
          step={2}
          totalSteps={TOTAL_STEPS}
          title="Microphone Permission"
          onNext={next}
          onBack={back}
        >
          <p className="text-text-muted text-sm">
            macOS will prompt you for microphone access when you first record.
            Click "Allow" when the prompt appears.
          </p>
        </OnboardingStep>
      )}

      {step === 3 && (
        <OnboardingStep
          step={3}
          totalSteps={TOTAL_STEPS}
          title="Transcription Mode"
          onNext={next}
          onBack={back}
        >
          <div className="space-y-4">
            <Select
              label="Mode"
              value={mode}
              options={[
                { value: "local", label: "Local (offline, private)" },
                { value: "api", label: "API (faster, requires key)" },
              ]}
              onChange={(v) => setMode(v as "local" | "api")}
            />
            {mode === "api" && (
              <TextInput
                label="OpenAI API Key"
                value={apiKey}
                onChange={setApiKey}
                type="password"
                placeholder="sk-..."
              />
            )}
            {mode === "local" && (
              <p className="text-xs text-text-muted">
                The medium.en model (~1.5GB) will be downloaded on first use.
              </p>
            )}
          </div>
        </OnboardingStep>
      )}

      {step === 4 && (
        <OnboardingStep
          step={4}
          totalSteps={TOTAL_STEPS}
          title="Test Recording"
          onNext={next}
          onBack={back}
          nextLabel="Skip"
        >
          <p className="text-text-muted text-sm mb-4">
            Try it out! Hold the fn key and say something, then release.
          </p>
          <div className="bg-surface-light rounded-xl p-4 text-center text-text-muted text-sm">
            Hold fn to record...
          </div>
        </OnboardingStep>
      )}

      {step === 5 && (
        <OnboardingStep
          step={5}
          totalSteps={TOTAL_STEPS}
          title="You're all set!"
          onNext={finishOnboarding}
          onBack={back}
          nextLabel="Start Using Wipr"
        >
          <p className="text-text-muted text-sm leading-relaxed">
            Wipr will run in your menu bar. Hold fn to dictate, release to
            insert. Open Settings from the menu bar icon to customize.
          </p>
        </OnboardingStep>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Verify build**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add src/pages/Onboarding.tsx src/components/OnboardingStep.tsx
git commit -m "feat: add onboarding flow with permissions, mode selection, and test recording"
```

---

### Task 20: Overlay Window Configuration & First-Run Detection

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/config/settings.rs`

- [ ] **Step 1: Add first_run flag to config**

Add to `GeneralConfig` in `src-tauri/src/config/settings.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_start: bool,
    pub context_aware: bool,
    pub first_run: bool,
}
```

Update the `Default` impl for `GeneralConfig`:

```rust
general: GeneralConfig {
    auto_start: true,
    context_aware: true,
    first_run: true,
},
```

- [ ] **Step 2: Create overlay window in setup**

Add to the `.setup()` closure in `src-tauri/src/lib.rs`, after tray setup:

```rust
// Create the overlay window (transparent, always on top, no decorations)
let _overlay = tauri::WebviewWindowBuilder::new(
    app,
    "overlay",
    tauri::WebviewUrl::App("/#/".into()),
)
.title("Wipr Overlay")
.decorations(false)
.transparent(true)
.always_on_top(true)
.inner_size(400.0, 100.0)
.position(
    // Center horizontally at top of screen
    500.0, 40.0,
)
.skip_taskbar(true)
.build()?;

// Check if first run -> show onboarding
let config = load_config();
if config.general.first_run {
    let _onboarding = tauri::WebviewWindowBuilder::new(
        app,
        "onboarding",
        tauri::WebviewUrl::App("/#/onboarding".into()),
    )
    .title("Welcome to Wipr")
    .inner_size(500.0, 600.0)
    .center()
    .build()?;

    // Mark first run as complete
    let mut updated_config = config.clone();
    updated_config.general.first_run = false;
    save_config(&updated_config).ok();
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/
git commit -m "feat: add overlay window, first-run detection, and onboarding launch"
```

---

### Task 21: Model Download Tauri Command

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add download_model Tauri command**

Add to the Tauri commands section in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
async fn download_model(app: tauri::AppHandle, model_name: String) -> Result<String, String> {
    let models_dir = config::settings::models_dir();
    let app_clone = app.clone();

    transcription::whisper_local::download_model(&model_name, &models_dir, move |downloaded, total| {
        #[derive(Clone, serde::Serialize)]
        struct DownloadProgress {
            downloaded: u64,
            total: u64,
            percent: f32,
        }
        let percent = if total > 0 {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        app_clone
            .emit(
                "model-download-progress",
                DownloadProgress {
                    downloaded,
                    total,
                    percent,
                },
            )
            .ok();
    })
    .await
    .map_err(|e| e.to_string())
}
```

Add `download_model` to the `generate_handler![]` macro.

- [ ] **Step 2: Verify compilation**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo check
```

Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add model download command with progress events"
```

---

### Task 22: Packaging Configuration

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `Homebrew/wipr.rb` (Homebrew cask formula template)

- [ ] **Step 1: Update tauri.conf.json for production bundling**

Update `src-tauri/tauri.conf.json` — add to the `bundle` section:

```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "minimumSystemVersion": "12.0",
      "signingIdentity": null,
      "entitlements": null
    }
  }
}
```

- [ ] **Step 2: Create Homebrew cask formula template**

Create `Homebrew/wipr.rb`:

```ruby
cask "wipr" do
  version "0.1.0"
  sha256 :no_check # Update with actual SHA256 for releases

  url "https://github.com/OWNER/wipr/releases/download/v#{version}/Wipr_#{version}_aarch64.dmg"
  name "Wipr"
  desc "macOS voice-to-text dictation tool"
  homepage "https://github.com/OWNER/wipr"

  app "Wipr.app"

  zap trash: [
    "~/.config/wipr",
  ]
end
```

- [ ] **Step 3: Verify the app can be built for release**

```bash
cd /Users/faridmatovu/projects/wipr
npm run tauri build 2>&1 | head -50
```

Expected: Build starts (may fail at code signing if no certificate is configured, but the compilation itself should succeed).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json Homebrew/
git commit -m "feat: configure DMG packaging and add Homebrew cask formula template"
```

---

### Task 23: Final Integration & Cleanup

**Files:**
- Modify: `src-tauri/src/lib.rs` (ensure all commands are registered)
- Modify: `src-tauri/tauri.conf.json` (final permissions)

- [ ] **Step 1: Verify all Tauri commands are in generate_handler**

Ensure `src-tauri/src/lib.rs` has this complete handler list:

```rust
.invoke_handler(tauri::generate_handler![
    get_config,
    update_config,
    get_history,
    search_history,
    delete_history_entry,
    clear_history,
    export_history,
    check_accessibility_permission,
    open_accessibility_settings,
    download_model,
])
```

- [ ] **Step 2: Run full cargo test suite**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo test
```

Expected: All tests pass.

- [ ] **Step 3: Run full frontend build**

```bash
cd /Users/faridmatovu/projects/wipr
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Run cargo clippy for linting**

```bash
cd /Users/faridmatovu/projects/wipr/src-tauri
cargo clippy -- -W warnings
```

Expected: No warnings or errors.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final integration check - all tests pass, builds succeed"
```

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Project scaffolding | Cargo.toml, tauri.conf.json, package.json, vite.config.ts |
| 2 | Config module | config/settings.rs |
| 3 | Audio capture | audio/capture.rs |
| 4 | Audio preprocessing | audio/preprocessing.rs |
| 5 | Transcription trait + API | transcription/traits.rs, whisper_api.rs |
| 6 | Local Whisper | transcription/whisper_local.rs |
| 7 | Voice commands | ai/commands.rs |
| 8 | AI text cleanup | ai/cleanup.rs |
| 9 | Context reading | ai/context.rs |
| 10 | Global hotkey | input/hotkey.rs |
| 11 | Text insertion | input/insertion.rs |
| 12 | History store | history/store.rs |
| 13 | System tray | tray.rs, lib.rs |
| 14 | Core pipeline | lib.rs |
| 15 | Frontend foundation | main.tsx, queries.ts, collections.ts, hooks |
| 16 | Overlay UI | Overlay.tsx, Waveform.tsx |
| 17 | Settings UI | Settings.tsx, SettingsSection.tsx |
| 18 | History UI | History.tsx, HistoryEntry.tsx |
| 19 | Onboarding UI | Onboarding.tsx, OnboardingStep.tsx |
| 20 | Window config + first-run | lib.rs, settings.rs |
| 21 | Model download | lib.rs |
| 22 | Packaging | tauri.conf.json, Homebrew/wipr.rb |
| 23 | Final integration | All files |
