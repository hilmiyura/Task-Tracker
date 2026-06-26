mod metrics;
mod notification;
mod sheets;
mod timer;
mod tray;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use tauri::Manager;
use tauri_plugin_notification::{NotificationExt, PermissionState};

/// Shared app state, read by the notification scheduler.
/// - `task_active`: whether a task is currently running (silences idle reminders).
/// - `task_started_at`: when the active task started (for the >3h reminder).
/// - `long_task_notified`: ensures the >3h reminder fires once per task run.
pub struct AppState {
    pub task_active: AtomicBool,
    pub task_started_at: Mutex<Option<Instant>>,
    pub long_task_notified: AtomicBool,
    /// Last seen tray-icon position, so the scheduler can anchor the popup
    /// under the icon when force-opening it.
    pub tray_pos: Mutex<Option<tauri::Position>>,
    /// On Leave: when true, all reminders are paused (vacation/holiday).
    pub on_leave: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            task_active: AtomicBool::new(false),
            task_started_at: Mutex::new(None),
            long_task_notified: AtomicBool::new(false),
            tray_pos: Mutex::new(None),
            on_leave: AtomicBool::new(false),
        })
        .manage(metrics::MetricsState::new())
        .invoke_handler(tauri::generate_handler![
            tray::set_tray_state,
            sheets::get_webhook_url,
            sheets::set_webhook_url,
            sheets::get_sheet_name,
            sheets::set_sheet_name,
            sheets::get_on_leave,
            sheets::set_on_leave,
            sheets::sync_entry,
            metrics::get_metrics
        ])
        .setup(|app| {
            // Hide the Dock icon — this is a menu bar (accessory) app.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Request macOS notification permission up front. Without this the
            // scheduler's reminders (idle 30 min / task >3h) are silently
            // dropped by the OS.
            let notifier = app.notification();
            if notifier.permission_state().unwrap_or(PermissionState::Denied)
                != PermissionState::Granted
            {
                let _ = notifier.request_permission();
            }

            // Seed the On Leave flag from the persisted config.
            let on_leave = sheets::load_on_leave(app.handle());
            app.state::<AppState>().on_leave.store(on_leave, Ordering::Relaxed);

            tray::create_tray(app.handle())?;
            notification::start_scheduler(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Task Tracker")
        .run(|_app, event| {
            // Keep running in the background when the popup window closes.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
