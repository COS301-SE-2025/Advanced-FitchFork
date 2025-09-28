use std::time::Duration;

use serde::Serialize;
use sysinfo::{CpuRefreshKind, Disks, System};

#[derive(Debug, Serialize, Clone)]
pub struct DiskSummary {
    pub name: String,
    pub total: u64,
    pub available: u64,
    pub file_system: String,
    pub mount_point: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemMetrics {
    pub load_one: f64,
    pub load_five: f64,
    pub load_fifteen: f64,
    pub cpu_cores: usize,
    pub cpu_avg_usage: f32,
    pub cpu_per_core: Vec<f32>,
    pub mem_total: u64,
    pub mem_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub disks: Vec<DiskSummary>,
    pub uptime_seconds: u64,
}

/// De-duplicate disks across all OSes by (name, total, fs).
/// If multiple entries match, prefer mount_point "/", otherwise prefer the shortest path.
fn dedupe_disks(disks: Vec<DiskSummary>) -> Vec<DiskSummary> {
    use std::collections::HashMap;

    fn mount_score(mp: &str) -> (u8, usize) {
        let root_rank = if mp == "/" { 0 } else { 1 };
        (root_rank, mp.len())
    }

    let mut pick: HashMap<(String, u64, String), DiskSummary> = HashMap::new();
    for d in disks {
        let key = (d.name.clone(), d.total, d.file_system.clone());
        match pick.get(&key) {
            None => {
                pick.insert(key, d);
            }
            Some(prev) => {
                let cur_mp = d.mount_point.as_str();
                let prev_mp = prev.mount_point.as_str();
                let prefer_cur = mount_score(cur_mp) < mount_score(prev_mp);
                if prefer_cur {
                    pick.insert(key, d);
                }
            }
        }
    }
    pick.into_values().collect()
}

/// Samples current system metrics using sysinfo in a portable way compatible with v0.30.
pub fn sample_system_metrics() -> SystemMetrics {
    let mut sys = System::new();

    // CPU & memory refresh (these are fine)
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    std::thread::sleep(Duration::from_millis(200));
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpus = sys.cpus();
    let per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let cpu_count = per_core.len();
    let cpu_avg_usage: f32 = if cpu_count > 0 {
        per_core.iter().sum::<f32>() / cpu_count as f32
    } else {
        0.0
    };

    let disks = Disks::new_with_refreshed_list();
    let collected: Vec<DiskSummary> = disks
        .list()
        .iter()
        .map(|d| DiskSummary {
            name: d.name().to_string_lossy().to_string(),
            total: d.total_space(),
            available: d.available_space(),
            file_system: d.file_system().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
        })
        .collect();

    let disk_summaries = dedupe_disks(collected);

    // ✔ sysinfo (this version): uptime is a static/associated fn
    let uptime_seconds = System::uptime();

    SystemMetrics {
        load_one: System::load_average().one,
        load_five: System::load_average().five,
        load_fifteen: System::load_average().fifteen,
        cpu_cores: cpu_count,
        cpu_avg_usage,
        cpu_per_core: per_core,
        mem_total: sys.total_memory(),
        mem_used: sys.used_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        disks: disk_summaries,
        uptime_seconds,
    }
}
