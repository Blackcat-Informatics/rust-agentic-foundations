// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::error::Error;
use std::path::PathBuf;
use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::{Gate, assessment::AssessmentRequest, record::RecordOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().ok_or("expected root")?);
    let contexts = PathBuf::from(args.next().ok_or("expected reviewed context directory")?);
    let subjects: Vec<_> = args.collect();
    if subjects.is_empty() { return Err("name the individually reviewed subjects explicitly".into()); }
    let opened = Gate::workspace(&root).open()?;
    let plan = opened.workplan();
    // Verify every reviewed snapshot before any write. This executable is an
    // explicit assessment action, never called by a normal check or gate.
    for subject in &subjects {
        let path = contexts.join(subject.replace([':', '#'], "-") + ".txt");
        let reviewed = std::fs::read_to_string(&path)?;
        let current = plan.assess_context(subject)?.to_string();
        if reviewed != current {
            return Err(format!("reviewed context moved for {subject}; recapture and reassess it").into());
        }
    }
    for subject in subjects {
        let mut request = AssessmentRequest::new(subject, AssessmentVerdict::Accepted, Actor::Agent);
        request.options = RecordOptions::new()
            .copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>")
            .licence("CC-BY-4.0");
        let result = plan.record_assessment(&root, &request)?;
        println!("accepted {} wrote {}", request.subject, result.path);
    }
    Ok(())
}
