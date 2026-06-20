//! Google Sheets integration via a Google Apps Script Web App webhook.
//!
//! The user deploys a small Apps Script bound to their spreadsheet that accepts
//! a POST with a JSON entry and appends it as a row. This module persists the
//! webhook URL in the app config dir and POSTs finished entries to it.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Default)]
struct Config {
    webhook_url: String,
}

/// One finished time entry — shape sent to the webhook (and to the Sheet row).
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub date: String,
    pub task: String,
    pub start: String,
    pub end: String,
    pub duration: String,
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("config.json"))
}

fn load_config(app: &AppHandle) -> Config {
    config_path(app)
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Return the currently saved webhook URL (empty string if unset).
#[tauri::command]
pub fn get_webhook_url(app: AppHandle) -> String {
    load_config(&app).webhook_url
}

/// Persist the webhook URL to the app config dir.
#[tauri::command]
pub fn set_webhook_url(app: AppHandle, url: String) -> Result<(), String> {
    let path = config_path(&app)?;
    let cfg = Config {
        webhook_url: url.trim().to_string(),
    };
    let json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

/// POST a finished entry to the configured Apps Script webhook.
#[tauri::command]
pub async fn sync_entry(app: AppHandle, entry: Entry) -> Result<(), String> {
    let url = load_config(&app).webhook_url;
    if url.is_empty() {
        return Err("Webhook URL belum diatur. Buka Settings (⚙︎).".into());
    }

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(&url)
        .json(&entry)
        .send()
        .await
        .map_err(|e| format!("Gagal terhubung: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Webhook menolak (HTTP {})", resp.status()));
    }
    Ok(())
}
