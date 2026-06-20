//! Reminder scheduler.
//!
//! Two reminders:
//!   1. Idle reminder — between 08:00–17:00 local, when no task is active,
//!      at most once every 30 minutes.
//!   2. Long-running task reminder — when a task has been running for more than
//!      3 hours, fired once per task run (in case it was left running).

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, Timelike};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;

const WORK_START_HOUR: u32 = 8;
const WORK_END_HOUR: u32 = 17;
const TICK: Duration = Duration::from_secs(60); // check every minute
const IDLE_REMINDER_EVERY: Duration = Duration::from_secs(30 * 60); // 30 minutes
const LONG_TASK_AFTER: Duration = Duration::from_secs(3 * 60 * 60); // 3 hours

/// Spawn the background scheduler thread.
pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || {
        // Start the idle clock now so the first idle reminder is ~30 min in.
        let mut last_idle_notify = Instant::now();

        loop {
            thread::sleep(TICK);

            let state = app.state::<AppState>();
            let active = state.task_active.load(Ordering::Relaxed);
            let hour = Local::now().hour();
            let within_work_hours = hour >= WORK_START_HOUR && hour < WORK_END_HOUR;

            // 1) Idle reminder during work hours.
            if within_work_hours && !active {
                if last_idle_notify.elapsed() >= IDLE_REMINDER_EVERY {
                    notify(&app, "Belum ada task aktif. Mau mulai mencatat sekarang?");
                    last_idle_notify = Instant::now();
                }
            }

            // 2) Long-running task reminder (>3h), once per task run.
            if active {
                let started = *state.task_started_at.lock().unwrap();
                if let Some(start) = started {
                    if start.elapsed() >= LONG_TASK_AFTER
                        && !state.long_task_notified.load(Ordering::Relaxed)
                    {
                        notify(
                            &app,
                            "Sebuah task sudah berjalan lebih dari 3 jam. Masih dikerjakan, atau lupa di-Stop?",
                        );
                        state.long_task_notified.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
    });
}

fn notify(app: &AppHandle, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Task Tracker")
        .body(body)
        .show();
}
