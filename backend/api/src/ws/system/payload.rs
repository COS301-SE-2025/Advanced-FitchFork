use serde::Serialize;

/* =========================
SHARED TYPES
========================= */

#[derive(Debug, Clone, Serialize)]
pub struct LoadAverages {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeManagerGeneral {
    pub running: usize,
    pub waiting: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeManagerAdmin {
    pub running: usize,
    pub waiting: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuInfo {
    pub cores: usize,
    pub avg_usage: f32,
    pub per_core: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskSummary {
    pub name: String,
    pub total: u64,
    pub available: u64,
    pub file_system: String,
    pub mount_point: String,
}

/* =========================
GENERAL PAYLOAD
========================= */

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealthGeneralPayload {
    /// RFC3339 timestamp
    pub ts: String,
    pub load: LoadAverages,
    pub code_manager: CodeManagerGeneral,
    // If you later want uptime here too, just add:
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub uptime_seconds: Option<u64>,
}

/* =========================
ADMIN PAYLOAD
========================= */

#[derive(Debug, Clone, Serialize)]
pub struct SystemHealthAdminPayload {
    /// RFC3339 timestamp
    pub ts: String,
    pub env: String,
    pub host: String,
    pub uptime_seconds: u64,
    pub load: LoadAverages,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskSummary>,
    pub code_manager: CodeManagerAdmin,
}
