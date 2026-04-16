pub mod commands;
pub mod wizard;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "murmur",
    version,
    about = "macOS voice-to-text dictation daemon"
)]
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
