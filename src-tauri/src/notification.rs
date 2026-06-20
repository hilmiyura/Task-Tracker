//! Reminder scheduler.
//!
//! Rules from the brief:
//!   - Only between 08:00 and 17:00 local time.
//!   - Never while a task is active.
//!   - At most one notification every 30 minutes.

use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use chrono::{Local, Timelike};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;

const WORK_START_HOUR: u32 = 8;
const WORK_END_HOUR: u32 = 17;
const INTERVAL: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Spawn a background thread that fires reminders on the 30-minute cadence.
pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(INTERVAL);

        let hour = Local::now().hour();
        let within_work_hours = hour >= WORK_START_HOUR && hour < WORK_END_HOUR;

        let task_active = app
            .state::<AppState>()
            .task_active
            .load(Ordering::Relaxed);

        if within_work_hours && !task_active {
            let _ = app
                .notification()
                .builder()
                .title("Task Tracker")
                .body("Belum ada task aktif. Mau mulai mencatat sekarang?")
                .show();
        }
    });
}
