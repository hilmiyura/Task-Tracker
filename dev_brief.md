# Developer Brief — Time Tracker macOS Menu Bar App

**Dibuat:** 2026-06-20  
**Author:** Muhammad Hilmi Yura  
**Status:** In Planning

---

## Tujuan

Membangun aplikasi macOS ringan berbentuk menu bar icon untuk mencatat aktivitas pekerjaan harian beserta waktu mulai dan selesai, lalu menyinkronkannya secara otomatis ke Google Sheets sebagai catatan produktivitas pribadi.

---

## Goals

- [ ] Aplikasi berjalan sebagai menu bar icon di macOS
- [ ] Popup muncul saat icon diklik (floating, pojok kanan atas)
- [ ] User dapat mencatat deskripsi pekerjaan, lalu klik Start dan Stop
- [ ] Live timer tampil selama task berjalan
- [ ] Data otomatis dikirim ke Google Sheets saat Stop
- [ ] Notifikasi muncul setiap 30 menit jika tidak ada task aktif di jam kerja (08:00–17:00)

---

## Tech Stack

| Layer | Teknologi |
|-------|-----------|
| Framework | Tauri v2 |
| UI | React + Tailwind CSS |
| Backend | Rust (built-in Tauri) |
| Integrasi | Google Sheets API v4 (Service Account) |
| Build output | `.app` (macOS) |

---

## Data Output (Google Sheets)

| Date | Task | Start | End | Duration |
|------|------|-------|-----|----------|
| 2026-06-20 | Weekly meeting client | 08:00 | 09:00 | 1h 0m |

---

## Scope

**In scope:**
- Menu bar icon dengan dua state (idle / active)
- Popup floating dengan input task, timer, tombol Start & Stop
- Notifikasi native macOS (jam kerja, interval 30 menit)
- Sinkronisasi ke Google Sheets via Service Account

**Out of scope:**
- Multi-user / cloud sync
- Reporting / analytics
- Edit atau hapus entri yang sudah tersimpan
- OAuth login (tidak diperlukan, pakai Service Account)

---

## Struktur File

```
time-tracker-app/
├── src-tauri/src/
│   ├── main.rs           # Entry point & Tauri commands
│   ├── tray.rs           # Menu bar icon & toggle window
│   ├── timer.rs          # State start/stop & kalkulasi durasi
│   ├── notification.rs   # Scheduler notifikasi 30 menit
│   └── sheets.rs         # Google Sheets API integration
├── src/
│   ├── App.tsx           # UI utama (idle & active state)
│   ├── main.tsx          # React entry point
│   └── index.css         # Tailwind base styles
├── .env                  # GOOGLE_SERVICE_ACCOUNT_JSON, GOOGLE_SHEET_ID
└── README.md
```

---

## Setup yang Dibutuhkan

1. Install Rust & Tauri CLI
2. Buat Google Cloud Project → Enable Sheets API → Buat Service Account → Download JSON key
3. Share Google Sheet ke email Service Account (editor access)
4. Isi `.env` dengan path JSON key dan Sheet ID

---

## Notifikasi Rules

| Kondisi | Aksi |
|---------|------|
| Jam < 08:00 atau > 17:00 | Tidak ada notifikasi |
| Task sedang aktif | Tidak ada notifikasi |
| Tidak ada task aktif & sudah 30 menit sejak notif terakhir | Tampilkan notifikasi native macOS |
