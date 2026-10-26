// Copyright (c) 2026 Johannes Overmann
// Distributed under the Boost Software License, Version 1.0.
// See LICENSE for details.

use std::process::Command;

/// Record a short compile date for the help footer.
fn main() {
    if let Ok(output) = Command::new("date").arg("+%Y-%m-%d").output() {
        if output.status.success() {
            let date = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=BUILD_DATE={}", date.trim());
        }
    }
}
