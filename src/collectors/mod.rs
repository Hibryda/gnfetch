//! System-information collectors.
//!
//! Each submodule owns one domain and returns typed data (or `None`/empty when
//! a field can't be determined). [`collect_all`] orchestrates them into a single
//! [`SystemInfo`]. Nothing in this layer touches rendering.

mod cpu;
mod de_wm;
mod disk;
mod gpu;
mod host;
mod kernel;
mod memory;
mod os;
mod packages;
mod shell;
mod uptime;

use crate::model::SystemInfo;
use sysinfo::System;

/// Gather everything gnfetch knows about the host into one [`SystemInfo`].
pub fn collect_all() -> SystemInfo {
    // `System::new_all()` + `refresh_all()` populates CPU/memory in one pass;
    // disks, GPUs and packages are gathered through their own dedicated APIs.
    let mut sys = System::new_all();
    sys.refresh_all();

    let (memory, swap) = memory::collect(&sys);

    SystemInfo {
        user: host::user(),
        hostname: host::hostname(),
        distro: os::distro(),
        distro_id: os::distro_id(),
        kernel: kernel::version(),
        uptime: uptime::collect(),
        cpu: cpu::collect(&sys),
        gpus: gpu::collect(),
        memory,
        swap,
        disks: disk::collect(),
        packages: packages::collect(),
        shell: shell::shell(),
        de: de_wm::desktop(),
        wm: de_wm::wm(),
    }
}
