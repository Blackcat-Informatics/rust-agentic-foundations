// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The commands. Each one calls `raf-workplan` and turns what it returns into
//! a [`Report`]; none decides anything the library does not.

use std::path::{Path, PathBuf};

use raf_workplan::exec::{Programs, SystemRunner};
use raf_workplan::{Gate, RoadmapEntry, SyncError, SyncOptions, Workplan, root};
use serde_json::{Value, json};

use raf_workplan::coverage::GapKind;

use crate::cli::{Args, Change, Command, Coverage, Docs, Evidence, SyncArgs};
use crate::exit::Failure;
use crate::render::{Record, Report};
use crate::{assess, change, coverage, evidence, history};

/// Run the command of `args` and return what it produced.
///
/// # Errors
///
/// A [`Failure`] when the tool could not run: the root cannot be found, the
/// repository cannot be opened, or the command's own input is refused. The
/// command of `evidence record` is checked before the root is looked for, so
/// a refusal of it starts no process.
pub(crate) fn run(args: &Args) -> Result<Report, Failure> {
    let root = || workspace_root(args.root.as_deref());
    match &args.command {
        Command::Check => check(&root()?, args.strict),
        Command::Status => status(&root()?),
        Command::Docs(Docs::Sync(sync_args)) => sync(&root()?, sync_args),
        Command::Docs(Docs::Preview(preview_args)) => preview(&root()?, &preview_args.slug),
        Command::Change(Change::New(new_args)) => change::new(&root()?, new_args),
        Command::Change(Change::Impact(slug)) => change::impact(&root()?, &slug.slug),
        Command::Change(Change::Reconcile(slug)) => {
            change::reconcile(&root()?, &slug.slug, args.strict)
        }
        Command::Change(Change::Complete(slug)) => {
            change::complete(&root()?, &slug.slug, args.strict)
        }
        Command::Verify(verify_args) => change::verify(&root()?, verify_args.slug.as_deref()),
        Command::History(history_args) => history::history(&root()?, history_args),
        Command::Evidence(Evidence::Record(record_args)) => {
            let source = evidence::source(record_args)?;
            evidence::record(&root()?, record_args, source)
        }
        Command::Evidence(Evidence::Claims(id)) => evidence::claims(&root()?, &id.id),
        Command::Evidence(Evidence::For(cap)) => evidence::support(&root()?, &cap.capability),
        Command::Assess(assess_args) => assess::assess(&root()?, assess_args),
        Command::Coverage(Coverage::Code(coverage_args)) => {
            coverage::coverage(&root()?, coverage_args, GapKind::PublicItem)
        }
        Command::Coverage(Coverage::Tests(coverage_args)) => {
            coverage::coverage(&root()?, coverage_args, GapKind::Test)
        }
        Command::Coverage(Coverage::Claims(claims_args)) => coverage::claims(&root()?, claims_args),
    }
}

/// The workspace root: the directory of `--root`, or the root `cargo metadata`
/// names for the current directory.
fn workspace_root(given: Option<&Path>) -> Result<PathBuf, Failure> {
    if let Some(dir) = given {
        return Ok(dir.to_path_buf());
    }
    let here = std::env::current_dir()
        .map_err(|error| Failure::new(format!("cannot read the current directory: {error}")))?;
    Ok(root::discover(&SystemRunner::new(Programs::new(here)))?)
}

/// The gate over `root`, raised to strict mode when `strict`.
pub(crate) fn gate(root: &Path, strict: bool) -> Gate {
    let gate = Gate::workspace(root);
    if strict { gate.strict() } else { gate }
}

/// `check`: every finding of `Gate::check`, as the library orders them,
/// judged under strict mode when `strict`.
fn check(root: &Path, strict: bool) -> Result<Report, Failure> {
    let findings = gate(root, strict).try_check()?;
    Ok(Report::of(findings.iter().map(Record::of_finding)))
}

/// `status`: one record for each state, in the order of the catalogue, and one
/// for the open changes.
fn status(root: &Path) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let roadmap = plan.status();
    let mut records: Vec<Record> = roadmap
        .groups()
        .into_iter()
        .map(|(state, entries)| state_record(state.name(), entries))
        .collect();
    records.push(open_changes_record(&plan));
    let mut report = Report::of(records);
    let findings = plan.findings().len();
    if findings > 0 {
        report.note(format!(
            "the documents have {findings} finding(s), and a document that does not load is not in this roadmap; run `cargo workplan check` to see them"
        ));
    }
    Ok(report)
}

/// The width of the label column of the roadmap, the longest label.
const LABEL: usize = "implementing".len();

/// The row of one state: `implementing  1  name (gap: ...; changes: ...)`.
fn state_record(state: &str, entries: &[RoadmapEntry]) -> Record {
    let names: Vec<String> = entries.iter().map(entry_text).collect();
    let capabilities: Vec<Value> = entries.iter().map(entry_json).collect();
    Record::outcome(
        "state",
        vec![
            ("state", Value::from(state)),
            ("count", Value::from(entries.len())),
            ("capabilities", Value::Array(capabilities)),
        ],
        row(state, entries.len(), &names),
    )
}

/// `label  count  items`, without the trailing spaces of an empty list.
fn row(label: &str, count: usize, items: &[String]) -> String {
    let head = format!("{label:<LABEL$}  {count}");
    if items.is_empty() {
        head
    } else {
        format!("{head}  {}", items.join(", "))
    }
}

/// A capability in a row: its slug, then its gaps and its open changes.
fn entry_text(entry: &RoadmapEntry) -> String {
    let mut notes: Vec<String> = entry
        .gaps()
        .iter()
        .map(|gap| format!("gap: {gap}"))
        .collect();
    if !entry.open_changes().is_empty() {
        notes.push(format!("changes: {}", entry.open_change_slugs().join(", ")));
    }
    if notes.is_empty() {
        entry.slug().to_string()
    } else {
        format!("{} ({})", entry.slug(), notes.join("; "))
    }
}

/// A capability as a JSON object.
fn entry_json(entry: &RoadmapEntry) -> Value {
    let gaps: Vec<Value> = entry
        .gaps()
        .iter()
        .map(|gap| {
            json!({
                "list": gap.list().key(),
                "binding": gap.binding(),
                "message": gap.to_string(),
            })
        })
        .collect();
    json!({
        "id": entry.id(),
        "slug": entry.slug(),
        "title": entry.title(),
        "path": entry.path(),
        "gaps": gaps,
        "open_changes": entry.open_change_slugs(),
    })
}

/// The row of the open changes: each change record that loaded, by slug, with
/// the capabilities it lists.
fn open_changes_record(plan: &Workplan<'_>) -> Record {
    let changes = plan.open_changes();
    let names: Vec<String> = changes
        .iter()
        .map(|open| format!("{} ({})", open.slug(), open.capabilities().join(", ")))
        .collect();
    let objects: Vec<Value> = changes
        .iter()
        .map(|open| {
            json!({
                "id": open.id(),
                "slug": open.slug(),
                "path": open.path(),
                "capabilities": open.capabilities(),
            })
        })
        .collect();
    Record::outcome(
        "open-changes",
        vec![
            ("count", Value::from(changes.len())),
            ("changes", Value::Array(objects)),
        ],
        row("open changes", changes.len(), &names),
    )
}

/// `docs sync`: write the generated blocks and report each file rewritten.
fn sync(root: &Path, args: &SyncArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let mut options = SyncOptions::default();
    options.copyright.clone_from(&args.copyright);
    options.licence.clone_from(&args.licence);
    let report = plan.sync(&options).map_err(|error| match error {
        SyncError::SpdxUnset { field } => Failure::new(format!(
            "docs/capabilities/index.md must be created and --{field} is not given; pass --copyright <text> and --licence <id>"
        )),
        other => Failure::from(other),
    })?;
    Workplan::write(&report, root)
        .map_err(|error| Failure::new(format!("cannot write the generated blocks: {error}")))?;
    Ok(Report::of(report.edits.iter().map(|(path, _)| {
        Record::outcome(
            "rewrote",
            vec![("path", Value::from(path.as_str()))],
            format!("rewrote {path}"),
        )
    })))
}

/// `docs preview <slug>`: each capability of the change as it reads once the
/// change is reconciled. Nothing is written.
fn preview(root: &Path, slug: &str) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let previews = plan.preview(slug)?;
    Ok(Report::of(previews.iter().map(|preview| {
        Record::outcome(
            "preview",
            vec![
                ("path", Value::from(preview.path.as_str())),
                ("capability", Value::from(preview.capability.as_str())),
                ("text", Value::from(preview.text.as_str())),
            ],
            format!("==> {} <==\n{}", preview.path, preview.text),
        )
    })))
}
