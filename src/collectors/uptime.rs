//! System uptime.

use std::time::Duration;
use sysinfo::System;

pub fn collect() -> Option<Duration> {
    let secs = System::uptime();
    if secs == 0 {
        None
    } else {
        Some(Duration::from_secs(secs))
    }
}
