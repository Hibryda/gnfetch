//! Installed-package counts across common managers.
//!
//! Each manager is probed by running its "list" command and counting lines;
//! managers that aren't installed simply contribute nothing.

use crate::model::PackageCount;
use std::process::Command;

pub fn collect() -> Vec<PackageCount> {
    // (display name, command, args). `\n` is interpreted by dpkg-query itself.
    const PROBES: [(&str, &str, &[&str]); 4] = [
        ("dpkg", "dpkg-query", &["-f", "${binary:Package}\\n", "-W"]),
        ("rpm", "rpm", &["-qa"]),
        ("pacman", "pacman", &["-Qq"]),
        ("flatpak", "flatpak", &["list", "--app"]),
    ];

    PROBES
        .iter()
        .filter_map(|(name, cmd, args)| {
            let count = count_lines(cmd, args)?;
            (count > 0).then(|| PackageCount {
                manager: (*name).to_string(),
                count,
            })
        })
        .collect()
}

/// Run `cmd args` and count non-empty output lines, or `None` if it failed.
fn count_lines(cmd: &str, args: &[&str]) -> Option<usize> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Some(text.lines().filter(|l| !l.trim().is_empty()).count())
}
