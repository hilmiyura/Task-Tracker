use std::sync::atomic::Ordering;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewWindow,
};

use crate::AppState;

const POPUP_WIDTH: f64 = 300.0;

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
            let app = tray.app_handle();

            // Remember the icon position from any event that carries it, so the
            // scheduler can anchor a force-opened popup under the icon.
            let pos = match &event {
                TrayIconEvent::Click { rect, .. }
                | TrayIconEvent::DoubleClick { rect, .. }
                | TrayIconEvent::Enter { rect, .. }
                | TrayIconEvent::Move { rect, .. } => Some(rect.position.clone()),
                _ => None,
            };
            if let Some(p) = pos {
                *app.state::<AppState>().tray_pos.lock().unwrap() = Some(p);
            }

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                if let Some(window) = app.get_webview_window("main") {
                    toggle_popup(&window, rect.position);
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Position the popup centred just below the given tray-icon position.
fn place_popup(window: &WebviewWindow, tray_pos: tauri::Position) {
    // The tray position may come as logical or physical units; normalise to
    // physical so set_position lands in the right spot on Retina displays.
    let scale = window.scale_factor().unwrap_or(1.0);
    let phys = match tray_pos {
        tauri::Position::Physical(p) => tauri::PhysicalPosition::new(p.x as f64, p.y as f64),
        tauri::Position::Logical(p) => tauri::PhysicalPosition::new(p.x * scale, p.y * scale),
    };
    let x = phys.x - (POPUP_WIDTH * scale) / 2.0;
    let y = phys.y + 8.0;
    let _ = window.set_position(tauri::PhysicalPosition { x, y });
}

/// Toggle the popup on tray click: hide if visible, else show under the icon.
fn toggle_popup(window: &WebviewWindow, tray_pos: tauri::Position) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    place_popup(window, tray_pos);
    let _ = window.show();
    let _ = window.set_focus();
}

/// Force the popup open under the tray icon. Used by the scheduler's idle
/// reminder so the user is nudged to fill a task.
pub fn show_popup(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let stored = app.state::<AppState>().tray_pos.lock().unwrap().clone();
    if let Some(pos) = stored {
        // Anchor under the tray icon, exactly like a click would.
        place_popup(&window, pos);
    } else if let Ok(Some(monitor)) = window.current_monitor() {
        // Fallback (never saw the tray yet): top-right under the menu bar.
        let scale = window.scale_factor().unwrap_or(1.0);
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let win_w = window
            .outer_size()
            .map(|s| s.width as f64)
            .unwrap_or(POPUP_WIDTH * scale);
        let x = m_pos.x as f64 + m_size.width as f64 - win_w - 10.0 * scale;
        let y = m_pos.y as f64 + 32.0 * scale;
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }

    let _ = window.show();
    let _ = window.set_focus();
}

/// Called from the frontend whenever a task starts/stops so the menu bar icon
/// (tooltip) reflects idle vs active, and the scheduler can stay quiet.
#[tauri::command]
pub fn set_tray_state(app: AppHandle, active: bool) {
    let state = app.state::<AppState>();
    state.task_active.store(active, Ordering::Relaxed);

    // Track when the task started so the scheduler can warn after 3 hours,
    // and reset the "already warned" flag for each new task run.
    if active {
        *state.task_started_at.lock().unwrap() = Some(std::time::Instant::now());
        state.long_task_notified.store(false, Ordering::Relaxed);
    } else {
        *state.task_started_at.lock().unwrap() = None;
    }

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
