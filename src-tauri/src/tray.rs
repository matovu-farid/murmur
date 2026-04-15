use tauri::{
    menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

/// Open or focus an existing window. If the window doesn't exist, create it.
fn open_or_focus_window(app: &tauri::AppHandle, label: &str, url: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    } else {
        let _win = tauri::WebviewWindowBuilder::new(app, label, tauri::WebviewUrl::App(url.into()))
            .title(match label {
                "settings" => "Wipr - Settings",
                "history" => "Wipr - History",
                _ => "Wipr",
            })
            .inner_size(800.0, 600.0)
            .build();
    }
}

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    // Read initial config for check-item states
    let config = crate::config::settings::load_config();

    let settings_i = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let history_i = MenuItemBuilder::with_id("history", "History").build(app)?;

    let local_mode_i = CheckMenuItemBuilder::new("Local Mode")
        .id("local_mode")
        .checked(config.transcription.mode == crate::config::settings::TranscriptionMode::Local)
        .build(app)?;

    let ai_cleanup_i = CheckMenuItemBuilder::new("AI Cleanup")
        .id("ai_cleanup")
        .checked(config.ai_cleanup.enabled)
        .build(app)?;

    let auto_start_i = CheckMenuItemBuilder::new("Auto-start")
        .id("auto_start")
        .checked(config.general.auto_start)
        .build(app)?;

    let quit_i = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&settings_i, &history_i])
        .separator()
        .items(&[&local_mode_i, &ai_cleanup_i, &auto_start_i])
        .separator()
        .items(&[&quit_i])
        .build()?;

    TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Wipr - Voice to Text")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "settings" => {
                open_or_focus_window(app, "settings", "/#/settings");
            }
            "history" => {
                open_or_focus_window(app, "history", "/#/history");
            }
            "quit" => {
                app.exit(0);
            }
            "local_mode" => {
                if let Ok(checked) = local_mode_i.is_checked() {
                    let _ = app.emit("tray-toggle", serde_json::json!({
                        "item": "local_mode",
                        "checked": checked,
                    }));
                }
            }
            "ai_cleanup" => {
                if let Ok(checked) = ai_cleanup_i.is_checked() {
                    let _ = app.emit("tray-toggle", serde_json::json!({
                        "item": "ai_cleanup",
                        "checked": checked,
                    }));
                }
            }
            "auto_start" => {
                if let Ok(checked) = auto_start_i.is_checked() {
                    let _ = app.emit("tray-toggle", serde_json::json!({
                        "item": "auto_start",
                        "checked": checked,
                    }));
                }
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
