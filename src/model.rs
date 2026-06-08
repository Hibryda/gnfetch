//! The data model shared between collection and rendering.
//!
//! [`SystemInfo`] is the single aggregate the collectors populate and the
//! renderers read from. It is the contract between the two layers: collectors
//! never render, renderers never collect. Every optional field encodes
//! "not available on this system / not detected" as `None` so the pipeline
//! degrades gracefully instead of panicking.

use std::time::Duration;

/// Aggregate of everything gnfetch knows about the host.
#[derive(Debug, Default, Clone)]
pub struct SystemInfo {
    pub user: Option<String>,
    pub hostname: Option<String>,
    /// Human-readable distribution / OS name (e.g. "Debian GNU/Linux 13").
    pub distro: Option<String>,
    /// Machine id used to pick a logo (e.g. "debian", "arch").
    pub distro_id: Option<String>,
    pub kernel: Option<String>,
    pub uptime: Option<Duration>,
    pub cpu: Option<CpuInfo>,
    pub gpus: Vec<String>,
    pub memory: Option<UsageInfo>,
    pub swap: Option<UsageInfo>,
    pub disks: Vec<DiskInfo>,
    pub packages: Vec<PackageCount>,
    pub shell: Option<String>,
    /// Desktop environment (e.g. "GNOME", "KDE").
    pub de: Option<String>,
    /// Window manager / compositor, when detectable.
    pub wm: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub brand: String,
    /// Logical core (thread) count.
    pub cores: usize,
    pub freq_mhz: u64,
}

/// A used/total pair in bytes (memory, swap).
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount: String,
    pub used: u64,
    pub total: u64,
    pub fs: String,
}

#[derive(Debug, Clone)]
pub struct PackageCount {
    pub manager: String,
    pub count: usize,
}
