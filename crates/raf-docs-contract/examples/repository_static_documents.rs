// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Check the tracked text and document links of a staged checkout.

#[path = "../checks/documents.rs"]
mod documents;

use std::collections::BTreeSet;
use std::error::Error;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use raf_source::RelPath;
use raf_workplan::refs::banned_in_line;
use rust_agentic_foundations::gate_files::{decode_text, read_regular};

struct Tracked {
    paths: Vec<PathBuf>,
    symlinks: Vec<PathBuf>,
}

impl Tracked {
    fn parse(inventory: &str) -> Result<Self, String> {
        let records = inventory
            .strip_suffix('\0')
            .ok_or("expected a nonempty NUL-delimited Git tree inventory")?;
        let mut paths = BTreeSet::new();
        let mut symlinks = Vec::new();
        for entry in records.split('\0') {
            let (metadata, path) = entry
                .split_once('\t')
                .ok_or("missing path in Git tree inventory")?;
            let (mode_and_kind, object) = metadata
                .rsplit_once(' ')
                .ok_or("missing object ID in Git tree inventory")?;
            if object.is_empty() || !object.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(format!("invalid Git object ID for {path}"));
            }
            let relative = RelPath::new(path).map_err(|error| format!("{path}: {error}"))?;
            let path = PathBuf::from(relative.as_str());
            match mode_and_kind {
                "100644 blob" | "100755 blob" => {}
                "120000 blob" => symlinks.push(path.clone()),
                _ => return Err(documents::violation(&path, "unsupported staged file mode")),
            }
            if !paths.insert(path) {
                return Err(format!("duplicate path in Git tree inventory: {relative}"));
            }
        }
        Ok(Self {
            paths: paths.into_iter().collect(),
            symlinks,
        })
    }
}

fn reference_findings(root: &Path, checkout: &documents::Checkout) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    for path in checkout
        .paths
        .iter()
        .filter(|path| !checkout.symlinks.contains(path))
    {
        let bytes = read_regular(root, path)?;
        if let Some(text) = decode_text(&bytes) {
            findings.extend(text.lines().enumerate().filter_map(|(line, text)| {
                banned_in_line(text)
                    .map(|rule| format!("{}:{}: {}", path.display(), line + 1, rule.description()))
            }));
        }
    }
    Ok(findings)
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let [root, inventory] = arguments.as_slice() else {
        return Err("usage: repository_static_documents <snapshot> <Git tree inventory>".into());
    };
    let root = Path::new(root);
    let tracked = Tracked::parse(&std::fs::read_to_string(inventory)?)?;
    let checkout = documents::load_checkout(root, tracked.paths, tracked.symlinks)?;
    let mut findings = documents::static_findings(&checkout)?;
    findings.extend(reference_findings(root, &checkout)?);
    if findings.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }
    let mut diagnostics = io::stderr().lock();
    for finding in findings {
        writeln!(diagnostics, "{finding}")?;
    }
    Ok(ExitCode::FAILURE)
}
