//! Login shell.

use std::env;
use std::path::Path;

pub fn shell() -> Option<String> {
    let path = env::var("SHELL").ok()?;
    Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
}
