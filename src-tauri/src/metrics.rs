//! System metrics (CPU, memory, disk, network) read via `sysinfo`.
//!
//! State holds a live `System`/`Networks`/`Disks` plus the timestamp of the last
//! sample so network throughput can be derived as bytes-per-second. The frontend
//! polls `get_metrics` every couple of seconds while the System view is open.

use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use sysinfo::{Disks, Networks, System};
use tauri::State;

pub struct MetricsState {
    inner: Mutex<Inner>,
}

struct Inner {
    system: System,
    networks: Networks,
    disks: Disks,
    last: Instant,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    cpu: f32,        // overall CPU usage, 0–100
    mem_used: u64,   // bytes
    mem_total: u64,  // bytes
    disk_used: u64,  // bytes (primary volume)
    disk_total: u64, // bytes
    net_down: f64,   // bytes/second
    net_up: f64,     // bytes/second
}

impl MetricsState {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();
        system.refresh_memory();
        MetricsState {
            inner: Mutex::new(Inner {
                system,
                networks: Networks::new_with_refreshed_list(),
                disks: Disks::new_with_refreshed_list(),
                last: Instant::now(),
            }),
        }
    }
}

/// Sample current system metrics. Network rates are computed from the bytes
/// transferred since the previous call divided by the elapsed time.
#[tauri::command]
pub fn get_metrics(state: State<MetricsState>) -> Metrics {
    let mut inner = state.inner.lock().unwrap();
    let elapsed = inner.last.elapsed().as_secs_f64().max(0.001);

    // CPU + memory
    inner.system.refresh_cpu_usage();
    inner.system.refresh_memory();
    let cpu = inner.system.global_cpu_usage();
    let mem_used = inner.system.used_memory();
    let mem_total = inner.system.total_memory();

    // Network — received()/transmitted() report bytes since the last refresh.
    inner.networks.refresh();
    let mut down = 0u64;
    let mut up = 0u64;
    for (_name, data) in inner.networks.iter() {
        down += data.received();
        up += data.transmitted();
    }
    let net_down = down as f64 / elapsed;
    let net_up = up as f64 / elapsed;

    // Disk — capacity of the primary ("/") volume, falling back to the largest.
    inner.disks.refresh();
    let mut disk_total = 0u64;
    let mut disk_avail = 0u64;
    for d in inner.disks.iter() {
        if d.mount_point() == Path::new("/") {
            disk_total = d.total_space();
            disk_avail = d.available_space();
        }
    }
    if disk_total == 0 {
        for d in inner.disks.iter() {
            if d.total_space() > disk_total {
                disk_total = d.total_space();
                disk_avail = d.available_space();
            }
        }
    }
    let disk_used = disk_total.saturating_sub(disk_avail);

    inner.last = Instant::now();

    Metrics {
        cpu,
        mem_used,
        mem_total,
        disk_used,
        disk_total,
        net_down,
        net_up,
    }
}
