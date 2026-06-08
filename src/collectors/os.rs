//! Distribution / operating-system identification.

use sysinfo::System;

/// Human-readable OS name, e.g. "Debian GNU/Linux 13 (trixie)".
pub fn distro() -> Option<String> {
    System::long_os_version()
        .or_else(System::name)
        .filter(|s| !s.is_empty())
}

/// Machine-readable distribution id used to select a logo (e.g. "debian").
pub fn distro_id() -> Option<String> {
    let id = System::distribution_id();
    if id.is_empty() { None } else { Some(id) }
}
