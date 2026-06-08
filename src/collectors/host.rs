//! User and hostname.

use std::env;
use sysinfo::System;

pub fn user() -> Option<String> {
    env::var("USER").ok().or_else(|| env::var("LOGNAME").ok())
}

pub fn hostname() -> Option<String> {
    System::host_name()
}
