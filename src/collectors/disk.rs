//! Mounted filesystem usage.
//!
//! Pseudo and boot/virtual mounts are filtered out, and entries are deduplicated
//! by backing device so the output focuses on the volumes a user cares about
//! (root, home, external media) instead of every btrfs subvolume / bind mount.

use crate::model::DiskInfo;
use std::collections::HashMap;
use sysinfo::Disks;

pub fn collect() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();

    // One device can appear under many mount points (btrfs subvolumes, bind
    // mounts). Keep a single entry per device — the one with the shortest mount
    // path, which is the closest to its "primary" location.
    let mut by_device: HashMap<String, DiskInfo> = HashMap::new();

    for disk in &disks {
        let total = disk.total_space();
        if total == 0 {
            continue;
        }

        let mount = disk.mount_point().display().to_string();
        if is_pseudo(&mount) {
            continue;
        }

        let device = disk.name().to_string_lossy().into_owned();
        let available = disk.available_space();
        let info = DiskInfo {
            mount,
            used: total.saturating_sub(available),
            total,
            fs: disk.file_system().to_string_lossy().into_owned(),
        };

        match by_device.get(&device) {
            Some(existing) if existing.mount.len() <= info.mount.len() => {}
            _ => {
                by_device.insert(device, info);
            }
        }
    }

    let mut out: Vec<DiskInfo> = by_device.into_values().collect();
    out.sort_by(|a, b| a.mount.cmp(&b.mount));
    out
}

/// Mounts that are noise for a fetch tool (virtual fs, boot, snap/flatpak loops).
fn is_pseudo(mount: &str) -> bool {
    // Removable media on modern desktops mounts under /run/media/<user>/… — keep
    // it even though /run itself is filtered below.
    if mount.starts_with("/run/media") {
        return false;
    }
    const PREFIXES: [&str; 7] = [
        "/boot",
        "/snap",
        "/sys",
        "/proc",
        "/dev",
        "/run",
        "/var/lib/docker",
    ];
    PREFIXES.iter().any(|p| mount.starts_with(p))
}
