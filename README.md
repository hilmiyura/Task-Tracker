# Time Tracker — macOS Menu Bar App

A lightweight macOS **menu bar** app to log daily work activity (task description + start/end time) and automatically sync each finished entry to a **Google Sheet**. Built with **Tauri v2 + React + Tailwind**, designed to feel native (vibrancy popover, system fonts, light/dark aware).

> **Note — this app is intentionally personal/opinionated.** The column mapping and target sheet tab are set up for the author's own spreadsheet. It is **not** a one-size-fits-all product. That said, everything that's specific lives in one small Apps Script file and a couple of config values, so it's easy to fork and adapt. If that's you — welcome, see [Adapting it to your own sheet](#adapting-it-to-your-own-sheet).

---

## Features

- 🧊 Menu bar (tray) icon — no Dock icon (accessory app)
- 🪟 Native vibrancy popover that drops down from the tray icon
- ⏱️ Type a task → **Start** → live timer → **Stop**
- 📊 On Stop, the entry is POSTed to a Google Apps Script webhook that writes a row into your Sheet
- 🟢 Tray shows a dot + "Active" tooltip while a task is running
- 🔔 Reminder notification every 30 min during work hours (08:00–17:00) if no task is active
- 🌗 Automatic light / dark appearance
- ⚙️ In-app Settings to paste your webhook URL (stored locally)

---

## Why Apps Script instead of a Service Account?

The original plan used a Google Cloud Service Account. For a **personal** tool, a Google Apps Script Web App is dramatically simpler: no Cloud project, no JSON key file to guard, no API enablement. You paste a small script into your sheet, deploy it as a Web App, and the app POSTs to that URL. That's the approach used here.

---

## Tech Stack

| Layer        | Technology                          |
| ------------ | ----------------------------------- |
| Framework    | Tauri v2                            |
| UI           | React 18 + Tailwind CSS + Vite      |
| Backend      | Rust (Tauri core)                   |
| HTTP         | `reqwest` (Rust side, avoids CORS)  |
| Sheets sync  | Google Apps Script Web App webhook  |
| Build output | `.app` + `.dmg` (macOS, arm64)      |

---

## Project Structure

```
time-tracker/
├── src/                       # React frontend
│   ├── App.tsx                # Popup UI: timer, Start/Stop, Settings, sync status
│   ├── main.tsx               # React entry point
│   └── index.css              # Tailwind base + transparency for vibrancy
├── src-tauri/
│   ├── src/
│   │   ├── main.rs            # Binary entry → calls lib::run()
│   │   ├── lib.rs            # Setup, shared state, command registration
│   │   ├── tray.rs           # Menu bar icon + toggle popup positioning
│   │   ├── timer.rs          # Duration formatting helper (+ unit test)
│   │   ├── notification.rs    # 30-min reminder scheduler
│   │   └── sheets.rs         # Webhook config + sync_entry command
│   ├── capabilities/default.json
│   ├── tauri.conf.json       # Window (vibrancy), bundle config
│   └── Cargo.toml
├── google-apps-script/
│   └── Code.gs               # Paste this into your Sheet's Apps Script
└── README.md
```

---

## Prerequisites

- **macOS** (Apple Silicon / arm64 build by default)
- **Node.js** 18+
- **Rust** (stable) via [rustup](https://rustup.rs)
- **Xcode Command Line Tools** (`xcode-select --install`)

---

## Run & Build

```bash
# install JS deps
npm install

# dev (hot reload)
npm run tauri dev

# production build → .app + .dmg
npm run tauri build
```

Build outputs:

- App: `src-tauri/target/release/bundle/macos/Time Tracker.app`
- Installer: `src-tauri/target/release/bundle/dmg/Time Tracker_0.1.0_aarch64.dmg`

To install: open the `.dmg` and drag **Time Tracker.app** to **Applications**, or copy the `.app` directly.

> The app is **ad-hoc signed** (no Apple Developer certificate). On first launch macOS may warn it's from an unidentified developer — right-click the app → **Open**, or allow it in **System Settings → Privacy & Security**.

---

## Google Sheets Setup

### 1. Add the Apps Script

1. Open your target Google Sheet.
2. **Extensions → Apps Script**.
3. Delete the default code, paste the entire contents of [`google-apps-script/Code.gs`](google-apps-script/Code.gs), and **Save**.

### 2. Deploy as a Web App

1. **Deploy → New deployment**.
2. Select type ⚙️ → **Web app**.
3. Set:
   - **Execute as:** `Me`
   - **Who has access:** `Anyone`
4. **Deploy**, then **Authorize access** (you'll need to click *Advanced → Go to … (unsafe)* — this is normal for your own unverified script).
5. Copy the **Web app URL** (ends in `/exec`).

> When you change `Code.gs`, redeploy: **Manage deployments → Edit (✏️) → Version: New version → Deploy**. The URL stays the same.

### 3. Connect the app

1. Click the Time Tracker tray icon → **⚙︎** (top-right).
2. Paste the Web app URL → **Simpan** (Save).
3. **Start** then **Stop** a task. The entry shows ⏳ → ✓ (or ⚠ on error — hover for the message).

---

## How rows are written

The Apps Script writes **only** to the tab named in `SHEET_NAME`, using a single **anchor column** to find the next unused row, so static values and formulas in other columns are left untouched.

Current configuration (in `google-apps-script/Code.gs`):

| Field          | Column |
| -------------- | ------ |
| Date           | **B**  |
| Start time     | **C**  |
| End time       | **D**  |
| Task / activity | **I**  |

- **Target tab:** `Muhammad Hilmi Yura`
- **Anchor column:** `B` (a row is considered free when column B is empty)
- **Data starts at:** row `2` (row 1 is a header)

The app sends `{ date, task, start, end, duration }`; the script picks which columns to fill. `duration` is shown in-app but not written to the sheet in this configuration.

---

## Adapting it to your own sheet

Everything sheet-specific is in [`google-apps-script/Code.gs`](google-apps-script/Code.gs) — edit these constants and redeploy:

```js
var SHEET_NAME = "Muhammad Hilmi Yura"; // your tab name
var COL_DATE   = 2; // B
var COL_START  = 3; // C
var COL_END    = 4; // D
var COL_TASK   = 9; // I
var ANCHOR_COL = COL_DATE; // which column marks an "unused" row
var START_ROW  = 2;        // first data row (skip headers)
```

- **Column numbers are 1-based:** A=1, B=2, C=3, … I=9.
- **Anchor column** must be a column that is genuinely empty on unused rows (no formula that returns text/date — a formula-filled cell is treated as "occupied").
- Want `duration` written too? Add a `COL_DURATION` and a `setValue` line in `doPost`.

Other tweaks:

- **Vibrancy material** — `src-tauri/tauri.conf.json` → `windowEffects.effects` (e.g. `"menu"`, `"popover"`, `"sidebar"`, `"under-window"`).
- **Work hours / interval** — `src-tauri/src/notification.rs` (`WORK_START_HOUR`, `WORK_END_HOUR`, `INTERVAL`).
- **App identifier / name** — `src-tauri/tauri.conf.json` (`identifier`, `productName`).

---

## Notification Rules

| Condition                                            | Action            |
| ---------------------------------------------------- | ----------------- |
| Before 08:00 or after 17:00                          | No notification   |
| A task is currently active                           | No notification   |
| No active task & 30 min since the last notification  | Native macOS alert |

---

## Privacy

- The webhook URL is stored locally in the app config directory (`~/Library/Application Support/com.hilmiyura.timetracker/config.json`).
- Time entries are sent only to the Google Apps Script URL you configure — nowhere else.

---

## Roadmap / Out of Scope

Currently **out of scope**: multi-user/cloud sync, reporting/analytics, editing or deleting saved entries from the app, and OAuth login. Contributions/forks welcome.

---

## License

MIT — see [LICENSE](LICENSE). Provided as-is; adapt freely.
