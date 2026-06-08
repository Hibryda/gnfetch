//! Value formatting shared by the ANSI and graphical renderers.

use crate::model::CpuInfo;
use std::time::Duration;

/// Format a byte count as GiB (>= 1 GiB) or MiB.
pub fn fmt_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else {
        format!("{:.0} MiB", b / MIB)
    }
}

/// Integer percentage of `used` over `total`, guarding against a zero total.
pub fn percent(used: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        (used as f64 / total as f64 * 100.0).round() as u64
    }
}

/// Human-readable uptime, omitting empty leading units (e.g. "3h 5m").
pub fn fmt_uptime(d: Duration) -> String {
    let secs = d.as_secs();
    let (days, hours, mins) = (secs / 86_400, (secs % 86_400) / 3_600, (secs % 3_600) / 60);
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    parts.push(format!("{mins}m"));
    parts.join(" ")
}

/// "brand (cores) @ X.YZ GHz", dropping the frequency when it's unknown.
pub fn fmt_cpu(c: &CpuInfo) -> String {
    if c.freq_mhz > 0 {
        format!(
            "{} ({}) @ {:.2} GHz",
            c.brand,
            c.cores,
            c.freq_mhz as f64 / 1000.0
        )
    } else {
        format!("{} ({})", c.brand, c.cores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_scale_to_gib_and_mib() {
        assert_eq!(fmt_bytes(0), "0 MiB");
        assert_eq!(fmt_bytes(512 * 1024 * 1024), "512 MiB");
        assert_eq!(fmt_bytes(1024 * 1024 * 1024), "1.00 GiB");
        assert_eq!(fmt_bytes(3 * 1024 * 1024 * 1024 / 2), "1.50 GiB");
    }

    #[test]
    fn percent_rounds_and_guards_zero_total() {
        assert_eq!(percent(0, 0), 0);
        assert_eq!(percent(1, 0), 0);
        assert_eq!(percent(1, 2), 50);
        assert_eq!(percent(2, 3), 67); // 66.6.. rounds up
    }

    #[test]
    fn uptime_omits_empty_leading_units() {
        assert_eq!(fmt_uptime(Duration::from_secs(0)), "0m");
        assert_eq!(fmt_uptime(Duration::from_secs(90)), "1m");
        assert_eq!(fmt_uptime(Duration::from_secs(3 * 3600 + 5 * 60)), "3h 5m");
        assert_eq!(
            fmt_uptime(Duration::from_secs(2 * 86_400 + 4 * 60)),
            "2d 4m"
        );
    }

    #[test]
    fn cpu_drops_frequency_when_unknown() {
        let c = CpuInfo {
            brand: "Test CPU".to_string(),
            cores: 8,
            freq_mhz: 0,
        };
        assert_eq!(fmt_cpu(&c), "Test CPU (8)");
    }
}
