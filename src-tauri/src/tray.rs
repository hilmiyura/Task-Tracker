use std::sync::atomic::Ordering;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow,
};

use crate::AppState;

const POPUP_WIDTH: f64 = 320.0;

/// Build the menu bar tray icon and wire up click-to-toggle behaviour.
pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Task Tracker", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    // Monochrome template icon (transparent bg) so macOS tints it for the menu bar.
    let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .icon_as_template(true) // adapts to light/dark menu bar
        .tooltip("Task Tracker — Idle")
        .menu(&menu)
        // Left click toggles the popup; right click opens the menu.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    toggle_popup(&window, rect.position);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Show the popup just under the tray icon (top-right), or hide it if visible.
fn toggle_popup(window: &WebviewWindow, tray_pos: tauri::Position) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }

    // The tray click position may come as logical or physical units; normalise
    // to physical so set_position lands in the right spot on Retina displays.
    let scale = window.scale_factor().unwrap_or(1.0);
    let phys = match tray_pos {
        tauri::Position::Physical(p) => tauri::PhysicalPosition::new(p.x as f64, p.y as f64),
        tauri::Position::Logical(p) => tauri::PhysicalPosition::new(p.x * scale, p.y * scale),
    };

    // Anchor the popup centred under the tray icon, a few px below the bar.
    let x = phys.x - (POPUP_WIDTH * scale) / 2.0;
    let y = phys.y + 8.0;
    let _ = window.set_position(tauri::PhysicalPosition { x, y });
    let _ = window.show();
    let _ = window.set_focus();
}

/// Called from the frontend whenever a task starts/stops so the menu bar icon
/// (tooltip) reflects idle vs active, and the scheduler can stay quiet.
#[tauri::command]
pub fn set_tray_state(app: AppHandle, active: bool) {
    let state = app.state::<AppState>();
    state.task_active.store(active, Ordering::Relaxed);

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(if active {
            "Task Tracker — Active"
        } else {
            "Task Tracker — Idle"
        }));
        // Show a small dot beside the icon while a task is running.
        let _ = tray.set_title(Some(if active { "●" } else { "" }));
    }
}
