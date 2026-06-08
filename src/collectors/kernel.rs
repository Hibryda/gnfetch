//! Kernel version.

use sysinfo::System;

pub fn version() -> Option<String> {
    System::kernel_version().filter(|s| !s.is_empty())
}
