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
