//! RAM and swap usage (bytes).

use crate::model::UsageInfo;
use sysinfo::System;

/// Returns `(memory, swap)`; either is `None` when the total is reported as 0.
pub fn collect(sys: &System) -> (Option<UsageInfo>, Option<UsageInfo>) {
    let memory = (sys.total_memory() > 0).then(|| UsageInfo {
        used: sys.used_memory(),
        total: sys.total_memory(),
    });

    let swap = (sys.total_swap() > 0).then(|| UsageInfo {
        used: sys.used_swap(),
        total: sys.total_swap(),
    });

    (memory, swap)
}
