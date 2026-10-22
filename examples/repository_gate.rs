// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Apply the ordinary workplan gate to this repository's selected read root.
//! The example succeeds only when the loaded repository has no policy finding.
//! It does not write a debt record or repair documents while checking them.

use raf_workplan::Gate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let findings = Gate::workspace(rust_agentic_foundations::gate_root()).try_check()?;
    if findings.is_empty() {
        return Ok(());
    }
    let report = findings
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    Err(format!(
        "the repository has {} finding(s):\n{report}",
        findings.len()
    )
    .into())
}
