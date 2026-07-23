//! Menu bar (macOS) / system tray (Linux) for OnionGate.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

pub fn setup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Open OnionGate", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit OnionGate", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &sep, &quit])?;

    let mut builder = TrayIconBuilder::with_id("oniongate-tray")
        .menu(&menu)
        .tooltip("OnionGate")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "hide" => hide_main(app),
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    // Template icons adapt to light/dark menu bars on macOS.
    #[cfg(target_os = "macos")]
    {
        builder = builder.icon_as_template(true);
    }

    let _tray = builder.build(app)?;
    Ok(())
}

fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn hide_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}
