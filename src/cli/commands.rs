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

fn start(_foreground: bool) -> Result<()> { println!("start: not yet implemented"); Ok(()) }
fn stop() -> Result<()> { println!("stop: not yet implemented"); Ok(()) }
fn status() -> Result<()> { println!("status: not yet implemented"); Ok(()) }
fn config(_action: Option<ConfigAction>) -> Result<()> { println!("config: not yet implemented"); Ok(()) }
fn history(_action: Option<HistoryAction>) -> Result<()> { println!("history: not yet implemented"); Ok(()) }
fn install() -> Result<()> { println!("install: not yet implemented"); Ok(()) }
fn uninstall() -> Result<()> { println!("uninstall: not yet implemented"); Ok(()) }
fn download(_model: &str) -> Result<()> { println!("download: not yet implemented"); Ok(()) }
