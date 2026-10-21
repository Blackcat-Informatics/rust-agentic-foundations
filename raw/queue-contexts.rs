// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Temporary capture and production refresh of changed census assessment contexts.

use std::error::Error;
use std::path::Path;

use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::Gate;
use raf_workplan::assessment::AssessmentRequest;
use raf_workplan::record::RecordOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::current_dir()?;
    let opened = Gate::workspace(&root).open()?;
    let plan = opened.workplan();
    let write = std::env::args().any(|arg| arg == "write");
    for state in plan.list_assessments() {
        if state.currency.is_current() {
            continue;
        }
        let context = plan.assess_context(&state.subject)?;
        let path = Path::new(".stage/raf-census-core-normal-form-v1/raw").join(format!(
            "queue-span-context-{}.txt",
            context.record.trim_start_matches("assessment:")
        ));
        std::fs::write(path, context.to_string())?;
        println!("{}", context.subject);
        if write {
            let options = RecordOptions::new()
                .copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>")
                .licence("CC-BY-4.0");
            let request = AssessmentRequest::new(context.subject, AssessmentVerdict::Accepted, Actor::Agent)
                .options(options);
            plan.record_assessment(&root, &request)?;
        }
    }
    Ok(())
}
