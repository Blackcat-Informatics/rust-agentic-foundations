// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Read-only diagnostics of the stronger delivered-state candidate.
//! Product files and the ordinary gate policy are never rewritten here.

use raf_rdf::Format;
use raf_source::{DiskTree, Entry, EntryKind, FileId, Opened, RelPath, Tree, TreeError};
use raf_workplan::{Gate, Workplan, coverage::{GapKind, render}, report};
use std::error::Error;

struct Candidate<'a>(&'a DiskTree);

fn delivered(path: &RelPath, text: String) -> String {
    let header = text.strip_prefix("---\n").and_then(|rest| rest.split_once("\n---\n")).map(|(header, _)| header);
    if path.as_str().starts_with("docs/capabilities/")
        && header.is_some_and(|header| header.lines().any(|line| line == "state: implementing")) {
        text.replacen("\nstate: implementing\n", "\nstate: delivered\n", 1)
    } else {
        text
    }
}

impl Tree for Candidate<'_> {
    fn read(&self, path: &RelPath, max_bytes: u64) -> Result<String, TreeError> {
        self.0.read(path, max_bytes).map(|text| delivered(path, text))
    }
    fn read_unseen(&self, path: &RelPath, max_bytes: u64, seen: &dyn Fn(&FileId) -> bool) -> Result<Opened, TreeError> {
        match self.0.read_unseen(path, max_bytes, seen)? {
            Opened::Read { id, text } => Ok(Opened::Read { id, text: delivered(path, text) }),
            value @ Opened::Seen(_) => Ok(value),
            other => panic!("unsupported tree result: {other:?}"),
        }
    }
    fn list(&self, path: &RelPath) -> Result<Vec<Entry>, TreeError> { self.0.list(path) }
    fn entry(&self, path: &RelPath) -> Result<Option<EntryKind>, TreeError> { self.0.entry(path) }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let root = args.next().ok_or("expected repository root")?;
    let output = std::path::PathBuf::from(args.next().ok_or("expected output directory")?);
    if args.next().is_some() { return Err("unexpected argument".into()); }
    std::fs::create_dir_all(&output)?;
    let opened = Gate::workspace(root).open()?;
    let tree = Candidate(opened.tree());
    let plan = Workplan::load_with(&tree, opened.workspace(), &opened.links());
    let findings = plan.check();
    std::fs::write(output.join("findings.ttl"), report::findings(&findings, Format::Turtle)?)?;
    std::fs::write(output.join("findings.txt"), findings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))?;
    let coverage = plan.coverage_report();
    for (name, direction) in [("code", GapKind::PublicItem), ("tests", GapKind::Test)] {
        std::fs::write(output.join(format!("{name}.ttl")), report::coverage(&coverage, direction, Format::Turtle)?)?;
        std::fs::write(output.join(format!("{name}.md")), render::table(&coverage, direction))?;
    }
    let claims = plan.claims_report();
    std::fs::write(output.join("claims.md"), render::claims_table(&claims))?;
    let lines = plan.lines_report();
    std::fs::write(output.join("lines.md"), render::lines_table(&lines))?;
    eprintln!("candidate diagnostics only: {} findings, {} code gaps, {} test gaps, {} uncovered claims; product states unchanged", findings.len(), coverage.code().len(), coverage.tests().len(), claims.claims().iter().filter(|claim| !claim.is_covered()).count());
    Ok(())
}
