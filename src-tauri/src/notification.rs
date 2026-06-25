//! Reminder scheduler.
//!
//! Reminders (only during work hours 08:00–18:00, and never while On Leave):
//!   1. Idle reminder — when no task is active, at most once every 30 minutes;
//!      also force-opens the popup so the user is nudged to fill a task.
//!   2. Long-running task reminder — when a task has been running for more than
//!      3 hours, fired once per task run.
//!
//! Plus: at the end of the work day (18:00) any running task is force-stopped
//! (the frontend handles the stop + Google Sheets sync).

use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, Timelike};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::AppState;

// Flip to true for quick local testing: short intervals, work-hours gate
// bypassed for reminders, and force-stop after a short run instead of at 18:00.
const TEST_MODE: bool = false;

const WORK_START_HOUR: u32 = 8;
const WORK_END_HOUR: u32 = 18;

const TICK: Duration = if TEST_MODE {
    Duration::from_secs(15)
} else {
    Duration::from_secs(60)
};
const IDLE_REMINDER_EVERY: Duration = if TEST_MODE {
    Duration::from_secs(60)
} else {
    Duration::from_secs(30 * 60)
};
const LONG_TASK_AFTER: Duration = if TEST_MODE {
    Duration::from_secs(90)
} else {
    Duration::from_secs(3 * 60 * 60)
};
// In test mode, force-stop a task after this long instead of waiting for 18:00.
const TEST_FORCE_STOP_AFTER: Duration = Duration::from_secs(120);

/// Spawn the background scheduler thread.
pub fn start_scheduler(app: AppHandle) {
    thread::spawn(move || {
        // Start the idle clock now so the first idle reminder is ~30 min in.
        let mut last_idle_notify = Instant::now();

        loop {
            thread::sleep(TICK);

            let state = app.state::<AppState>();
            let active = state.task_active.load(Ordering::Relaxed);
            let on_leave = state.on_leave.load(Ordering::Relaxed);
            let hour = Local::now().hour();
            let within_work_hours =
                TEST_MODE || (hour >= WORK_START_HOUR && hour < WORK_END_HOUR);

            // Reminders only fire during work hours and when not On Leave.
            let reminders_on = within_work_hours && !on_leave;

            // 1) Idle reminder + force-open the popup.
            if reminders_on && !active && last_idle_notify.elapsed() >= IDLE_REMINDER_EVERY {
                notify(&app, "No active task. Want to start tracking now?");
                crate::tray::show_popup(&app);
                last_idle_notify = Instant::now();
            }

            // 2) Long-running task reminder (>3h), once per task run.
            if reminders_on && active {
                let started = *state.task_started_at.lock().unwrap();
                if let Some(start) = started {
                    if start.elapsed() >= LONG_TASK_AFTER
                        && !state.long_task_notified.load(Ordering::Relaxed)
                    {
                        notify(
                            &app,
                            "A task has been running for over 3 hours. Still working, or forgot to Stop?",
                        );
                        state.long_task_notified.store(true, Ordering::Relaxed);
                    }
                }
            }

            // 3) Force-stop a running task at the end of the work day (18:00).
            //    The frontend performs the actual stop + Sheets sync.
            if active {
                let should_force_stop = if TEST_MODE {
                    state
                        .task_started_at
                        .lock()
                        .unwrap()
                        .map(|s| s.elapsed() >= TEST_FORCE_STOP_AFTER)
                        .unwrap_or(false)
                } else {
                    hour >= WORK_END_HOUR
                };
                if should_force_stop {
                    let _ = app.emit("force-stop", ());
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
