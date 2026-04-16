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
    }
}
