// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

use std::error::Error;
use std::path::PathBuf;
use raf_workplan::Gate;
use raf_workplan::assessment::Currency;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let root = args.next().ok_or("expected root")?;
    let output = PathBuf::from(args.next().ok_or("expected output directory")?);
    let subjects: Vec<_> = args.collect();
    std::fs::create_dir_all(&output)?;
    let opened = Gate::workspace(&root).open()?;
    let plan = opened.workplan();
    for state in plan.list_assessments() {
        match &state.currency {
            Currency::Current => println!("{} {:?} current", state.subject, state.verdict),
            Currency::Stale(moved) => {
                println!("{} {:?} stale {}", state.subject, state.verdict, moved.len());
                for input in moved {
                    println!("  {input}");
                }
            }
            other => panic!("unsupported assessment currency: {other:?}"),
        }
    }
    for subject in subjects {
        let context = plan.assess_context(&subject)?;
        let path = output.join(subject.replace([':', '#'], "-") + ".txt");
        std::fs::write(&path, context.to_string())?;
        println!("context {} {}", subject, path.display());
    }
    Ok(())
}
