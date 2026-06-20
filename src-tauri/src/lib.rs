mod notification;
mod sheets;
mod timer;
mod tray;

use std::sync::atomic::AtomicBool;

/// Shared app state. `task_active` is read by the notification scheduler so it
/// stays quiet while the user is tracking a task.
pub struct AppState {
    pub task_active: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            task_active: AtomicBool::new(false),
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
        .expect("error while building Time Tracker")
        .run(|_app, event| {
            // Keep running in the background when the popup window closes.
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
