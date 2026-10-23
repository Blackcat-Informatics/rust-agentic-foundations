use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::{Gate, Workplan, SyncOptions, assessment::AssessmentRequest, record::RecordOptions};
use std::{error::Error, fs, path::Path};
fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(".");
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let sync = plan.sync(&SyncOptions::default())?;
    Workplan::write(&sync, root)?;
    for (path, _) in &sync.edits { println!("rewrote {path}"); }
    let mut subjects = Vec::new();
    for capability in ["census-patterns", "census-normal-form"] {
      for section in ["purpose", "contract", "usage", "examples", "constraints", "explanation", "verification"] {
        subjects.push(format!("cap:{capability}#{section}"));
      }
    }
    for subject in subjects {
        let context = plan.assess_context(&subject)?;
        fs::write(format!(".stage/raf-census-detectors-and-extensibility/raw/context-{}.log", subject.replace([':', '#'], "-")), context.to_string())?;
        let request = AssessmentRequest::new(subject, AssessmentVerdict::Accepted, Actor::Agent)
            .options(RecordOptions::new().copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>").licence("CC-BY-4.0"));
        println!("{}", plan.record_assessment(root, &request)?);
    }
    Ok(())
}
