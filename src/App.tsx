import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type SyncStatus = "syncing" | "ok" | "error";

type Entry = {
  id: number;
  date: string;
  task: string;
  start: string;
  end: string;
  duration: string;
  status: SyncStatus;
  error?: string;
};

/** Format seconds as HH:MM:SS for the live timer. */
function formatClock(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  const s = totalSeconds % 60;
  return [h, m, s].map((n) => String(n).padStart(2, "0")).join(":");
}

/** Human duration like "1h 5m" used in the Sheets row. */
function formatDuration(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  return `${h}h ${m}m`;
}

function clockTime(d: Date): string {
  return d.toTimeString().slice(0, 5); // HH:MM
}

function dateStr(d: Date): string {
  return d.toISOString().slice(0, 10); // YYYY-MM-DD
}

export default function App() {
  const [task, setTask] = useState("");
  const [running, setRunning] = useState(false);
  const [elapsed, setElapsed] = useState(0); // seconds
  const [entries, setEntries] = useState<Entry[]>([]);
  const [showSettings, setShowSettings] = useState(false);
  const [webhookUrl, setWebhookUrl] = useState("");
  const [savedHint, setSavedHint] = useState("");
  const startedAt = useRef<Date | null>(null);

  // load the saved webhook URL once
  useEffect(() => {
    invoke<string>("get_webhook_url")
      .then((u) => setWebhookUrl(u))
      .catch(() => {});
  }, []);

  // tick the live timer every second while running
  useEffect(() => {
    if (!running) return;
    const id = setInterval(() => {
      if (startedAt.current) {
        setElapsed(Math.floor((Date.now() - startedAt.current.getTime()) / 1000));
      }
    }, 1000);
    return () => clearInterval(id);
  }, [running]);

  function syncTray(active: boolean) {
    invoke("set_tray_state", { active }).catch(() => {});
  }

  function patchEntry(id: number, patch: Partial<Entry>) {
    setEntries((prev) => prev.map((e) => (e.id === id ? { ...e, ...patch } : e)));
  }

  function handleStart() {
    if (!task.trim()) return;
    startedAt.current = new Date();
    setElapsed(0);
    setRunning(true);
    syncTray(true);
  }

  function handleStop() {
    if (!startedAt.current) return;
    const end = new Date();
    const totalSeconds = Math.floor((end.getTime() - startedAt.current.getTime()) / 1000);
    const entry: Entry = {
      id: Date.now(),
      date: dateStr(startedAt.current),
      task: task.trim(),
      start: clockTime(startedAt.current),
      end: clockTime(end),
      duration: formatDuration(totalSeconds),
      status: "syncing",
    };
    setEntries((prev) => [entry, ...prev].slice(0, 6));
    setRunning(false);
    setElapsed(0);
    setTask("");
    startedAt.current = null;
    syncTray(false);

    // Push to Google Sheets via the webhook (handled in Rust).
    const { id, status, error, ...payload } = entry;
    void status;
    void error;
    invoke("sync_entry", { entry: payload })
      .then(() => patchEntry(id, { status: "ok" }))
      .catch((err: string) => patchEntry(id, { status: "error", error: String(err) }));
  }

  function saveWebhook() {
    invoke("set_webhook_url", { url: webhookUrl })
      .then(() => {
        setSavedHint("Tersimpan ✓");
        setTimeout(() => setSavedHint(""), 1500);
      })
      .catch((err: string) => setSavedHint(String(err)));
  }

  return (
    <div className="flex h-full w-full flex-col text-black/85 dark:text-white/90">
      {/* Title bar */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-3.5 pt-3 pb-2"
      >
        <span className="text-[13px] font-semibold tracking-tight">Task Tracker</span>
        <div className="flex items-center gap-2">
          {!showSettings && (
            <span
              className={`flex items-center gap-1.5 text-[11px] font-medium ${
                running ? "text-green-600 dark:text-green-400" : "text-black/40 dark:text-white/40"
              }`}
            >
              <span
                className={`h-1.5 w-1.5 rounded-full ${
                  running ? "animate-pulse bg-green-500" : "bg-black/25 dark:bg-white/30"
                }`}
              />
              {running ? "Active" : "Idle"}
            </span>
          )}
          <button
            onClick={() => setShowSettings((s) => !s)}
            title="Settings"
            className="rounded p-0.5 text-black/40 transition hover:text-black/80 dark:text-white/40 dark:hover:text-white/90"
          >
            {showSettings ? (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
                <path d="M18 6 6 18M6 6l12 12" />
              </svg>
            ) : (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
            )}
          </button>
        </div>
      </div>

      <div className="mx-3.5 border-t border-black/10 dark:border-white/10" />

      {showSettings ? (
        /* Settings */
        <div className="flex flex-1 flex-col gap-2 px-3.5 py-3">
          <label className="text-[11px] font-medium text-black/55 dark:text-white/55">
            Google Sheets — Webhook URL
          </label>
          <textarea
            value={webhookUrl}
            onChange={(e) => setWebhookUrl(e.target.value)}
            placeholder="https://script.google.com/macros/s/…/exec"
            rows={4}
            className="w-full resize-none rounded-[7px] border border-black/10 bg-black/[0.04] px-2.5 py-1.5 text-[11px] leading-snug text-black/85 placeholder-black/35 outline-none transition focus:border-blue-500/60 focus:ring-2 focus:ring-blue-500/20 dark:border-white/10 dark:bg-white/[0.06] dark:text-white/90 dark:placeholder-white/30"
          />
          <div className="flex items-center gap-2">
            <button
              onClick={saveWebhook}
              className="rounded-[7px] bg-blue-500 px-3 py-1.5 text-[12px] font-medium text-white shadow-sm transition hover:bg-blue-600 active:scale-[0.99]"
            >
              Simpan
            </button>
            {savedHint && (
              <span className="text-[11px] text-black/50 dark:text-white/50">{savedHint}</span>
            )}
          </div>
          <p className="mt-1 text-[10px] leading-relaxed text-black/40 dark:text-white/40">
            Tempel URL Web App dari Apps Script (Deploy → Web app). Data dikirim
            otomatis saat menekan Stop.
          </p>
        </div>
      ) : (
        /* Main */
        <div className="flex flex-1 flex-col gap-3 px-3.5 py-3">
          <input
            type="text"
            value={task}
            onChange={(e) => setTask(e.target.value)}
            disabled={running}
            placeholder="Apa yang sedang kamu kerjakan?"
            className="w-full rounded-[7px] border border-black/10 bg-black/[0.04] px-2.5 py-1.5 text-[13px] text-black/85 placeholder-black/35 outline-none transition focus:border-blue-500/60 focus:ring-2 focus:ring-blue-500/20 disabled:opacity-50 dark:border-white/10 dark:bg-white/[0.06] dark:text-white/90 dark:placeholder-white/30"
            onKeyDown={(e) => {
              if (e.key === "Enter" && !running) handleStart();
            }}
          />

          <div className="flex flex-col items-center justify-center py-1">
            <div
              className={`text-[26px] font-semibold tracking-tight tabular-nums ${
                running ? "text-black/90 dark:text-white" : "text-black/25 dark:text-white/25"
              }`}
            >
              {formatClock(elapsed)}
            </div>
            {running && (
              <div className="mt-0.5 max-w-full truncate text-[11px] text-black/45 dark:text-white/45">
                {task}
              </div>
            )}
          </div>

          {!running ? (
            <button
              onClick={handleStart}
              disabled={!task.trim()}
              className="w-full rounded-[7px] bg-blue-500 py-1.5 text-[13px] font-medium text-white shadow-sm transition hover:bg-blue-600 active:scale-[0.99] disabled:cursor-not-allowed disabled:bg-black/10 disabled:text-black/30 disabled:shadow-none dark:disabled:bg-white/10 dark:disabled:text-white/30"
            >
              Start
            </button>
          ) : (
            <button
              onClick={handleStop}
              className="w-full rounded-[7px] bg-red-500 py-1.5 text-[13px] font-medium text-white shadow-sm transition hover:bg-red-600 active:scale-[0.99]"
            >
              Stop
            </button>
          )}

          {entries.length > 0 && (
            <div className="mt-0.5">
              <div className="mb-1 px-0.5 text-[10px] font-medium uppercase tracking-wider text-black/35 dark:text-white/35">
                Tercatat
              </div>
              <ul className="flex flex-col">
                {entries.map((e) => (
                  <li
                    key={e.id}
                    className="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 text-[12px] hover:bg-black/[0.04] dark:hover:bg-white/[0.06]"
                  >
                    <span
                      title={
                        e.status === "error"
                          ? e.error
                          : e.status === "ok"
                          ? "Tersimpan ke Sheets"
                          : "Mengirim…"
                      }
                      className="shrink-0"
                    >
                      {e.status === "syncing" && <span className="text-black/35 dark:text-white/35">⏳</span>}
                      {e.status === "ok" && <span className="text-green-600 dark:text-green-400">✓</span>}
                      {e.status === "error" && <span className="text-red-500">⚠</span>}
                    </span>
                    <span className="min-w-0 flex-1 truncate text-black/75 dark:text-white/80">
                      {e.task}
                    </span>
                    <span className="shrink-0 text-black/35 dark:text-white/35">
                      {e.start}–{e.end}
                    </span>
                    <span className="shrink-0 font-medium tabular-nums text-black/55 dark:text-white/55">
                      {e.duration}
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
