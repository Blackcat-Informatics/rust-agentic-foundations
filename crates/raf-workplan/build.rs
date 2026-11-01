// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Build the indexed native helper into the library's immutable embedded bytes.

use std::{env, error::Error, path::PathBuf, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/bin/exec_host.rs");
    println!("cargo:rerun-if-changed=src/exec/system/control.rs");
    if env::var("CARGO_CFG_TARGET_OS")? != "linux" {
        return Ok(());
    }
    if env::var("HOST")? != env::var("TARGET")? {
        return Err("the native exec helper requires matching host and target toolchains".into());
    }
    let destination =
        PathBuf::from(env::var_os("OUT_DIR").ok_or("missing build output directory")?)
            .join("raf-exec-host");
    let output = Command::new(env::var_os("RUSTC").ok_or("missing build compiler")?)
        .args([
            "--edition=2024",
            "--crate-name=raf_exec_host",
            "-Dwarnings",
            "-Copt-level=2",
            "-Cstrip=symbols",
        ])
        .arg("src/bin/exec_host.rs")
        .arg("-o")
        .arg(destination)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "native exec helper compilation failed: {}",
            output.stderr.escape_ascii()
        )
        .into());
    }
    Ok(())
}
