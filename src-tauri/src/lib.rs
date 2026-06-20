mod notification;
mod sheets;
mod timer;
mod tray;

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::Instant;

/// Shared app state, read by the notification scheduler.
/// - `task_active`: whether a task is currently running (silences idle reminders).
/// - `task_started_at`: when the active task started (for the >3h reminder).
/// - `long_task_notified`: ensures the >3h reminder fires once per task run.
pub struct AppState {
    pub task_active: AtomicBool,
    pub task_started_at: Mutex<Option<Instant>>,
    pub long_task_notified: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            task_active: AtomicBool::new(false),
            task_started_at: Mutex::new(None),
            long_task_notified: AtomicBool::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            tray::set_tray_state,
            sheets::get_webhook_url,
            sheets::set_webhook_url,
            sheets::sync_entry
        ])
        .setup(|app| {
            // Hide the Dock icon — this is a menu bar (accessory) app.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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
