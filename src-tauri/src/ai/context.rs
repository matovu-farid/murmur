/// Read text surrounding the cursor from the active application.
/// Falls back to empty string if accessibility API can't read the field.
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

pub fn open_accessibility_settings() {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .ok();
}
