// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Workflow execution budgets reserve headroom below the fifteen-minute ceiling.
//! The release-profile job `test-release` has its own thirty-minute ceiling.

use purrdf_lex::{json::Value, yaml};

/// The largest `timeout-minutes` a job named `name` may declare.
fn ceiling(name: &str) -> u64 {
    if name == "test-release" { 30 } else { 15 }
}

fn excessive_jobs(text: &str) -> Result<Vec<String>, String> {
    let document = yaml::read(text).map_err(|error| error.to_string())?;
    let jobs = document
        .as_object()
        .and_then(|root| root.get("jobs"))
        .and_then(Value::as_object)
        .filter(|jobs| !jobs.is_empty())
        .ok_or("workflow must declare a nonempty jobs mapping")?;
    Ok(jobs
        .iter()
        .filter_map(|(name, job)| {
            let limit = job
                .as_object()
                .and_then(|job| job.get("timeout-minutes"))
                .and_then(Value::as_u64);
            let ceiling = ceiling(name);
            (!matches!(limit, Some(minutes) if (1..=ceiling).contains(&minutes))).then(|| {
                format!(
                    "job {name} must declare timeout-minutes as an integer from one to {ceiling}"
                )
            })
        })
        .collect())
}

#[test]
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn every_workflow_job_has_an_explicit_execution_budget() {
    assert_eq!(
        excessive_jobs("jobs: {fast: {timeout-minutes: 15}}"),
        Ok(vec![])
    );
    // `test-release` runs the release-profile suite and may declare thirty minutes.
    assert_eq!(
        excessive_jobs("jobs: {test-release: {timeout-minutes: 30}}"),
        Ok(vec![])
    );
    let refused = excessive_jobs("jobs: {test-release: {timeout-minutes: 31}}").unwrap();
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("job test-release"));
    for job in [
        "{}",
        "{timeout-minutes: 0}",
        "{timeout-minutes: 16}",
        "{timeout-minutes: '${{ matrix.limit }}'}",
    ] {
        let failures = excessive_jobs(&format!("jobs: {{slow: {job}}}")).unwrap();
        assert_eq!(failures.len(), 1, "{job}");
        assert!(failures[0].contains("job slow"));
    }
    assert!(excessive_jobs("jobs: {}").is_err());
    assert!(excessive_jobs("jobs: [").is_err());
    let root = super::gate_root();
    let mut workflows = 0;
    for path in super::tracked_paths() {
        if path.starts_with(".github/workflows")
            && matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yaml" | "yml")
            )
        {
            workflows += 1;
            let text = String::from_utf8(super::read_tracked(&root, &path)).unwrap();
            let failures =
                excessive_jobs(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert_eq!(failures.len(), 0, "{}: {failures:?}", path.display());
        }
    }
    assert_ne!(workflows, 0, "no tracked workflow was checked");
}
