// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Scan a registry, or check the built-in sample when invoked without arguments.

#[path = "registry_scan/driver.rs"]
mod driver;

fn main() -> std::process::ExitCode {
    driver::run()
}
