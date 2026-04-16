use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn insert_at_cursor(text: &str) -> Result<(), String> {
    let original_clipboard = get_clipboard();
    set_clipboard(text)?;
    thread::sleep(Duration::from_millis(50));
    simulate_paste()?;
    thread::sleep(Duration::from_millis(150));
    if let Some(original) = original_clipboard {
        set_clipboard(&original).ok();
    }
    Ok(())
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste").output().ok().and_then(|output| {
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
    child.wait().map_err(|e| format!("pbcopy failed: {}", e))?;
    Ok(())
}

fn simulate_paste() -> Result<(), String> {
    let script = r#"tell application "System Events" to keystroke "v" using command down"#;
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

pub fn execute_editing_command(command: &str) -> Result<(), String> {
    let script = match command {
        "delete that" | "undo that" => {
            r#"tell application "System Events" to keystroke "z" using command down"#
        }
        "select all" => r#"tell application "System Events" to keystroke "a" using command down"#,
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
