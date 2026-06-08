//! Turn a [`SystemInfo`] into an ordered list of items every layout arranges.
//!
//! The item order is configurable (`--fields` / config `fields`); [`DEFAULT_ORDER`]
//! is used when unset. Each key expands to zero or more [`Item`]s (e.g. `gpu` and
//! `disk` can produce several). Unknown keys are ignored.

use super::format::{fmt_bytes, fmt_cpu, fmt_uptime, percent};
use crate::model::SystemInfo;

/// Maximum number of disks shown (keeps a card visiting-card sized).
pub const MAX_DISKS: usize = 5;

/// Default field order when none is configured.
pub const DEFAULT_ORDER: &[&str] = &[
    "kernel", "uptime", "packages", "shell", "de", "wm", "cpu", "gpu", "blank", "memory", "swap",
    "disk",
];

/// Selectable field keys, for `--list-fields` and documentation.
pub const AVAILABLE: &[&str] = &[
    "os", "kernel", "uptime", "packages", "shell", "de", "wm", "cpu", "gpu", "memory", "swap",
    "disk", "blank",
];

/// A usage bar (memory, swap, a disk).
pub struct BarItem {
    pub key: String,
    pub used: u64,
    pub total: u64,
    /// Trailing note (e.g. filesystem); shown after the value where space allows.
    pub suffix: Option<String>,
}

impl BarItem {
    pub fn ratio(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.used as f64 / self.total as f64).clamp(0.0, 1.0) as f32
        }
    }

    /// "used / total (pct%)", optionally with the suffix appended.
    pub fn detail(&self, with_suffix: bool) -> String {
        let mut s = format!(
            "{} / {} ({}%)",
            fmt_bytes(self.used),
            fmt_bytes(self.total),
            percent(self.used, self.total)
        );
        if with_suffix && let Some(suf) = &self.suffix {
            s.push_str(&format!("  {suf}"));
        }
        s
    }
}

/// One laid-out body item.
pub enum Item {
    Kv {
        key: String,
        value: String,
    },
    Bar(BarItem),
    /// Vertical spacer.
    Gap,
}

/// Structured, layout-agnostic view of the system.
pub struct CardContent {
    pub title: String,
    pub subtitle: Option<String>,
    pub items: Vec<Item>,
}

impl CardContent {
    /// Value of the first key/value item whose key equals `label`.
    pub fn kv(&self, label: &str) -> Option<&str> {
        self.items.iter().find_map(|it| match it {
            Item::Kv { key, value } if key == label => Some(value.as_str()),
            _ => None,
        })
    }

    /// First bar item whose key starts with `prefix` (e.g. "Memory", "Disk").
    pub fn bar(&self, prefix: &str) -> Option<&BarItem> {
        self.items.iter().find_map(|it| match it {
            Item::Bar(b) if b.key.starts_with(prefix) => Some(b),
            _ => None,
        })
    }
}

/// Extract content using `order` (or [`DEFAULT_ORDER`] when `None`).
pub fn extract(info: &SystemInfo, order: Option<&[String]>) -> CardContent {
    let user = info.user.clone().unwrap_or_else(|| "user".to_string());
    let host = info
        .hostname
        .clone()
        .unwrap_or_else(|| "localhost".to_string());

    let mut items = Vec::new();
    match order {
        Some(keys) => keys.iter().for_each(|k| push_items(k, info, &mut items)),
        None => DEFAULT_ORDER
            .iter()
            .for_each(|k| push_items(k, info, &mut items)),
    }

    CardContent {
        title: format!("{user}@{host}"),
        subtitle: info.distro.clone(),
        items,
    }
}

fn kv(key: &str, value: String) -> Item {
    Item::Kv {
        key: key.to_string(),
        value,
    }
}

/// Append the item(s) for one field key (case-insensitive). No-op if unavailable.
fn push_items(key: &str, info: &SystemInfo, out: &mut Vec<Item>) {
    match key.trim().to_ascii_lowercase().as_str() {
        "os" | "distro" => {
            if let Some(distro) = &info.distro {
                out.push(kv("OS", distro.clone()));
            }
        }
        "kernel" => {
            if let Some(v) = &info.kernel {
                out.push(kv("Kernel", v.clone()));
            }
        }
        "uptime" => {
            if let Some(u) = info.uptime {
                out.push(kv("Uptime", fmt_uptime(u)));
            }
        }
        "packages" | "pkgs" => {
            if !info.packages.is_empty() {
                let pkgs = info
                    .packages
                    .iter()
                    .map(|p| format!("{} ({})", p.count, p.manager))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push(kv("Packages", pkgs));
            }
        }
        "shell" => {
            if let Some(v) = &info.shell {
                out.push(kv("Shell", v.clone()));
            }
        }
        "de" => {
            if let Some(v) = &info.de {
                out.push(kv("DE", v.clone()));
            }
        }
        "wm" => {
            if let Some(v) = &info.wm {
                out.push(kv("WM", v.clone()));
            }
        }
        "cpu" => {
            if let Some(c) = &info.cpu {
                out.push(kv("CPU", fmt_cpu(c)));
            }
        }
        "gpu" | "gpus" => {
            for g in &info.gpus {
                out.push(kv("GPU", g.clone()));
            }
        }
        "memory" | "mem" | "ram" => {
            if let Some(m) = &info.memory {
                out.push(Item::Bar(BarItem {
                    key: "Memory".to_string(),
                    used: m.used,
                    total: m.total,
                    suffix: None,
                }));
            }
        }
        "swap" => {
            if let Some(s) = &info.swap {
                out.push(Item::Bar(BarItem {
                    key: "Swap".to_string(),
                    used: s.used,
                    total: s.total,
                    suffix: None,
                }));
            }
        }
        "disk" | "disks" => {
            for d in info.disks.iter().take(MAX_DISKS) {
                out.push(Item::Bar(BarItem {
                    key: format!("Disk {}", short_mount(&d.mount)),
                    used: d.used,
                    total: d.total,
                    suffix: Some(d.fs.clone()),
                }));
            }
        }
        "blank" | "gap" | "-" | "" => out.push(Item::Gap),
        _ => {} // unknown key: ignore
    }
}

/// Shorten a long mount path, keeping the tail (e.g. "…de/models").
pub fn short_mount(mount: &str) -> String {
    const MAX: usize = 10;
    let count = mount.chars().count();
    if count <= MAX {
        return mount.to_string();
    }
    let tail: String = mount.chars().skip(count - (MAX - 1)).collect();
    format!("…{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiskInfo, UsageInfo};

    fn sample() -> SystemInfo {
        SystemInfo {
            kernel: Some("6.x".to_string()),
            memory: Some(UsageInfo { used: 1, total: 2 }),
            disks: (0..8)
                .map(|i| DiskInfo {
                    mount: format!("/m{i}"),
                    used: 1,
                    total: 2,
                    fs: "ext4".to_string(),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn short_mount_keeps_tail() {
        assert_eq!(short_mount("/home"), "/home");
        let s = short_mount("/home/hibryda/code/models");
        assert_eq!(s, "…de/models");
        assert_eq!(s.chars().count(), 10);
    }

    #[test]
    fn default_order_caps_disks() {
        let c = extract(&sample(), None);
        let disks = c
            .items
            .iter()
            .filter(|it| matches!(it, Item::Bar(b) if b.key.starts_with("Disk")))
            .count();
        assert_eq!(disks, MAX_DISKS);
    }

    #[test]
    fn custom_order_selects_and_orders() {
        let order = vec!["memory".to_string(), "kernel".to_string()];
        let c = extract(&sample(), Some(&order));
        // exactly two items, memory bar first, kernel kv second
        assert_eq!(c.items.len(), 2);
        assert!(matches!(&c.items[0], Item::Bar(b) if b.key == "Memory"));
        assert!(matches!(&c.items[1], Item::Kv { key, .. } if key == "Kernel"));
    }

    #[test]
    fn unknown_keys_ignored_unavailable_skipped() {
        let order = vec![
            "bogus".to_string(),
            "swap".to_string(),
            "kernel".to_string(),
        ];
        let c = extract(&sample(), Some(&order));
        // bogus ignored, swap unavailable (None), only kernel remains
        assert_eq!(c.items.len(), 1);
        assert_eq!(c.kv("Kernel"), Some("6.x"));
    }
}
