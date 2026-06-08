//! CPU brand, core count and frequency.

use crate::model::CpuInfo;
use sysinfo::System;

pub fn collect(sys: &System) -> Option<CpuInfo> {
    let cpus = sys.cpus();
    let first = cpus.first()?;

    let brand = {
        let b = first.brand().trim();
        if b.is_empty() {
            first.name().to_string()
        } else {
            b.to_string()
        }
    };

    Some(CpuInfo {
        brand,
        cores: cpus.len(),
        freq_mhz: first.frequency(),
    })
}
