// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Workflow execution budgets reserve headroom below the ten-minute ceiling.

use purrdf_lex::{json::Value, yaml};

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
            (!matches!(limit, Some(1..=9))).then(|| {
                format!("job {name} must declare timeout-minutes as an integer from one to nine")
            })
        })
        .collect())
}

#[test]
fn every_workflow_job_has_an_explicit_ten_minute_ceiling() {
    assert_eq!(
        excessive_jobs("jobs: {fast: {timeout-minutes: 9}}"),
        Ok(vec![])
    );
    for job in [
        "{}",
        "{timeout-minutes: 0}",
        "{timeout-minutes: 10}",
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
