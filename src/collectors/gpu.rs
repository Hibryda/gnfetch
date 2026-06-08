//! GPU detection via `lspci`.
//!
//! Shelling out to `lspci` mirrors what neofetch does; it returns an empty list
//! when the tool is missing or no display controller is found. A future, more
//! portable path could read `/sys/class/drm` or query the graphics API directly.

use std::process::Command;

pub fn collect() -> Vec<String> {
    let Ok(output) = Command::new("lspci").output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter(|line| {
            line.contains("VGA compatible controller")
                || line.contains("3D controller")
                || line.contains("Display controller")
        })
        .filter_map(|line| line.split_once(": ").map(|(_, name)| clean(name)))
        .collect()
}

/// Strip the trailing PCI revision suffix, e.g. "... (rev a1)".
fn clean(name: &str) -> String {
    name.split(" (rev")
        .next()
        .unwrap_or(name)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn strips_revision_suffix_and_trims() {
        assert_eq!(
            clean("NVIDIA Corporation AD107M [GeForce RTX 4060] (rev a1)"),
            "NVIDIA Corporation AD107M [GeForce RTX 4060]"
        );
        assert_eq!(clean("Intel UHD Graphics  "), "Intel UHD Graphics");
    }
}
