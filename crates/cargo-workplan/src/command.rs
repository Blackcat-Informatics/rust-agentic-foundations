// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The commands call `raf-workplan` and return a [`Report`] or the documented
//! mdBook transport response; semantic decisions remain in the library.

use std::path::{Path, PathBuf};

use crate::result::Result as CommandResult;
use raf_workplan::{Gate, RoadmapEntry, SyncError, SyncOptions, Workplan, root};

use raf_workplan::coverage::GapKind;
use raf_workplan::diagram::{DiagramKind, DiagramScope, NamedDiagramView};

use crate::cli::{
    Args, Change, Command, Coverage, Docs, DocsBuild, Evidence, StandaloneArgs, SyncArgs,
};
use crate::exit::Failure;
use crate::render::{Format, Record, Report};
use crate::{assess, change, coverage, debt, evidence, history, mdbook};

/// A normal report or the documented mdBook transport response.
pub(crate) enum Output {
    Report(Report),
    Mdbook(mdbook::Response),
}

/// Run the command of `args` and return what it produced.
///
/// # Errors
///
/// A [`Failure`] when the tool could not run: the root cannot be found, the
/// repository cannot be opened, or the command's own input is refused. The
/// command of `evidence record` is checked before the root is looked for, so
/// a refusal of it starts no process.
pub(crate) fn run(args: &Args) -> Result<Output, Failure> {
    check_format(args)?;
    let root = || workspace_root(args.root.as_deref());
    let report = match &args.command {
        Command::Check => check(&root()?),
        Command::Status => status(&root()?),
        Command::Diagram(view) => {
            if args.quiet {
                return Err(Failure::new(
                    "diagram refuses --quiet; request a complete artifact",
                ));
            }
            diagram(&root()?, view)
        }
        Command::Docs(Docs::Mdbook(book_args)) => {
            return mdbook::run(args, book_args).map(Output::Mdbook);
        }
        Command::Docs(Docs::Build(DocsBuild::Standalone(build_args))) => {
            standalone(&root()?, build_args)
        }
        Command::Docs(Docs::Sync(sync_args)) => sync(&root()?, sync_args),
        Command::Docs(Docs::Preview(preview_args)) => {
            preview(&root()?, &preview_args.slug, &preview_args.assets)
        }
        Command::Docs(Docs::Export(export_args)) => {
            if args.quiet || args.format == Format::Text {
                return Err(Failure::new(
                    "docs export requires --format turtle or --format ntriples and refuses --quiet",
                ));
            }
            export(&root()?, &export_args.path)
        }
        Command::Change(Change::New(new_args)) => change::new(&root()?, new_args),
        Command::Change(Change::Impact(slug)) => change::impact(&root()?, &slug.slug),
        Command::Change(Change::Reconcile(slug)) => change::reconcile(&root()?, &slug.slug),
        Command::Change(Change::Complete(slug)) => change::complete(&root()?, &slug.slug),
        Command::Verify(verify_args) => change::verify(&root()?, verify_args.slug.as_deref()),
        Command::History(history_args) => history::history(&root()?, history_args),
        Command::Evidence(Evidence::Record(record_args)) => {
            let source = evidence::source(record_args)?;
            evidence::record(&root()?, record_args, source)
        }
        Command::Evidence(Evidence::Claims(id)) => {
            if args.quiet && args.format != Format::Text {
                return Err(Failure::new(
                    "evidence claims RDF output refuses --quiet; omit --quiet for the complete semantic selection",
                ));
            }
            evidence::claims(&root()?, &id.id, args.format)
        }
        Command::Evidence(Evidence::For(cap)) => evidence::support(&root()?, &cap.capability),
        Command::Assess(assess_args) => assess::assess(&root()?, assess_args),
        Command::Review(review) => crate::review::run(&root()?, review, args.format, args.quiet),
        Command::Coverage(Coverage::Code(coverage_args)) => {
            coverage::coverage(&root()?, coverage_args, GapKind::PublicItem)
        }
        Command::Coverage(Coverage::Tests(coverage_args)) => {
            coverage::coverage(&root()?, coverage_args, GapKind::Test)
        }
        Command::Coverage(Coverage::Claims(claims_args)) => coverage::claims(&root()?, claims_args),
        Command::Coverage(Coverage::Lines(lines_args)) => coverage::lines(&root()?, lines_args),
        Command::Debt(debt_args) => debt::debt(&root()?, debt_args),
    }?;
    let name = match &args.command {
        Command::Check => "check",
        Command::Status => "status",
        Command::Diagram(_) => "diagram",
        Command::Docs(Docs::Mdbook(_)) => "docs mdbook",
        Command::Docs(Docs::Build(_)) => "docs build standalone",
        Command::Docs(Docs::Sync(_)) => "docs sync",
        Command::Docs(Docs::Preview(_)) => "docs preview",
        Command::Docs(Docs::Export(_)) => "docs export",
        Command::Change(Change::New(_)) => "change new",
        Command::Change(Change::Impact(_)) => "change impact",
        Command::Change(Change::Reconcile(_)) => "change reconcile",
        Command::Change(Change::Complete(_)) => "change complete",
        Command::Verify(_) => "verify",
        Command::History(_) => "history",
        Command::Evidence(Evidence::Record(_)) => "evidence record",
        Command::Evidence(Evidence::Claims(_)) => "evidence claims",
        Command::Evidence(Evidence::For(_)) => "evidence for",
        Command::Assess(args) if args.list => "assess list",
        Command::Assess(args) if args.verdict.is_some() => "assess record",
        Command::Assess(_) => "assess context",
        Command::Review(crate::cli::Review::List) => "review list",
        Command::Review(crate::cli::Review::Show(_)) => "review show",
        Command::Review(crate::cli::Review::Status(_)) => "review status",
        Command::Review(crate::cli::Review::ChangedSince(_)) => "review changed-since",
        Command::Review(crate::cli::Review::Unreviewed(_)) => "review unreviewed",
        Command::Review(crate::cli::Review::Coverage(_)) => "review coverage",
        Command::Review(crate::cli::Review::Record(_)) => "review record",
        Command::Coverage(Coverage::Code(_)) => "coverage code",
        Command::Coverage(Coverage::Tests(_)) => "coverage tests",
        Command::Coverage(Coverage::Claims(_)) => "coverage claims",
        Command::Coverage(Coverage::Lines(_)) => "coverage lines",
        Command::Debt(_) => "debt write",
    };
    Ok(Output::Report(report.command(name)))
}

/// Admit carriers before opening a root, retaining mdBook's own protocol refusal.
fn check_format(args: &Args) -> Result<(), Failure> {
    if args.format == Format::Json
        && !matches!(
            &args.command,
            Command::Diagram(_) | Command::Review(_) | Command::Docs(Docs::Mdbook(_))
        )
    {
        return Err(Failure::new(
            "JSON output requires the diagram or review command",
        ));
    }
    if args.format == Format::Svg
        && !matches!(
            &args.command,
            Command::Diagram(_) | Command::Docs(Docs::Mdbook(_))
        )
    {
        return Err(Failure::new("SVG output requires the diagram command"));
    }
    Ok(())
}

/// The publication library owns the bundle, version identity and file admission.
fn standalone(root: &Path, args: &StandaloneArgs) -> Result<Report, Failure> {
    let output = std::fs::canonicalize(&args.output_dir)?;
    if output.to_str().is_none() {
        return Err(Failure::new(
            "documentation output root must be representable as UTF-8",
        ));
    }
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let bundle = plan.named_diagram(&args.view)?;
    let directory = raf_workplan::diagram::publication::publish(&bundle, &output)?;
    let entry = directory.join(format!("{}.html", bundle.view().name()));
    let directory = directory
        .into_os_string()
        .into_string()
        .map_err(|_| Failure::new("documentation directory cannot be represented as UTF-8"))?;
    let entry = entry
        .into_os_string()
        .into_string()
        .map_err(|_| Failure::new("documentation entry cannot be represented as UTF-8"))?;
    let message = format!("published {} at {entry}", args.view);
    Ok(Report::of([Record::outcome(
        CommandResult::DocumentationBuilt {
            view: args.view.clone(),
            directory,
            entry,
        },
        message,
    )]))
}

/// The public authoring codec owns the profile and native dataset shape.
fn export(root: &Path, path: &str) -> Result<Report, Failure> {
    use raf_source::{DiskTree, RelPath, Tree};
    let path = RelPath::new(path)?;
    if raf_okf::is_capability_index(path.as_str()) {
        return Err(raf_okf::DatasetError::NavigationIndex.into());
    }
    let tree = DiskTree::new(root)?;
    let text = tree.read(&path, raf_okf::MAX_TEXT_BYTES as u64 + 1)?;
    let document = raf_okf::Document::parse(&text)
        .map_err(|error| Failure::new(format!("{path}: {error}")))?;
    let dataset = document
        .to_dataset(path.as_str())
        .map_err(|error| Failure::new(format!("{path}: {error}")))?;
    Ok(Report::document(dataset))
}

/// The workspace root: the directory of `--root`, or the discovered root.
pub(crate) fn workspace_root(given: Option<&Path>) -> Result<PathBuf, Failure> {
    if let Some(dir) = given {
        return Ok(dir.to_path_buf());
    }
    let here = std::env::current_dir()
        .map_err(|error| Failure::new(format!("cannot read the current directory: {error}")))?;
    Ok(root::discover(&here)?)
}

/// The complete repository gate over `root`.
pub(crate) fn gate(root: &Path) -> Gate {
    Gate::workspace(root)
}

/// `check`: every finding of `Gate::check`, as the library orders them,
/// judged with the complete repository policy.
fn check(root: &Path) -> Result<Report, Failure> {
    let findings = gate(root).try_check()?;
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
    Record::outcome(
        CommandResult::State {
            state: state.into(),
            capabilities: entries.to_vec(),
        },
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

/// The row of the open changes: each change record that loaded, by slug, with
/// the capabilities it lists.
fn open_changes_record(plan: &Workplan<'_>) -> Record {
    let changes = plan.open_changes();
    let names: Vec<String> = changes
        .iter()
        .map(|open| format!("{} ({})", open.slug(), open.capabilities().join(", ")))
        .collect();
    let text = row("open changes", changes.len(), &names);
    Record::outcome(CommandResult::OpenChanges(changes), text)
}

fn diagram(root: &Path, view: &crate::cli::DiagramArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    if let Some(reference) = &view.view {
        return Ok(Report::diagram(plan.named_diagram(reference)?));
    }
    let kind = view
        .kind
        .ok_or_else(|| Failure::new("diagram requires --kind"))?;
    let scope = view
        .scope
        .clone()
        .ok_or_else(|| Failure::new("diagram requires --scope"))?;
    let selection = match kind {
        DiagramKind::EvidenceClaims => DiagramScope::Evidence { id: scope },
        kind => DiagramScope::Automatic { kind, scope },
    };
    let grouping = match view.grouping.as_deref() {
        Some("package") => raf_workplan::diagram::DiagramGrouping::Package,
        Some("capability") => raf_workplan::diagram::DiagramGrouping::Capability,
        None | Some("none") => raf_workplan::diagram::DiagramGrouping::None,
        Some(other) => {
            return Err(Failure::new(format!(
                "unsupported diagram grouping {other}; use none, package or capability"
            )));
        }
    };
    let view = NamedDiagramView::new(
        view.name
            .as_deref()
            .ok_or_else(|| Failure::new("diagram requires --name"))?,
        selection,
    )?
    .with_focus(view.focus.clone())
    .with_grouping(grouping)
    .with_relations(view.relations.clone());
    Ok(Report::diagram(std::sync::Arc::new(plan.diagram(&view)?)))
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
    let mut result = Report::of(report.edits.iter().map(|(path, _)| {
        Record::outcome(
            CommandResult::Rewritten {
                path: path.to_string(),
            },
            format!("rewrote {path}"),
        )
    }));
    for path in report.removals() {
        result.note(format!("removed owned diagram {path}"));
    }
    Ok(result)
}

/// `docs preview <slug>`: each capability of the change as it reads once the
/// change is reconciled. Nothing is written.
fn preview(root: &Path, slug: &str, args: &SyncArgs) -> Result<Report, Failure> {
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    let mut previews: Vec<_> = plan
        .preview(slug)?
        .into_iter()
        .map(|preview| (preview.path, preview.capability, preview.text))
        .collect();
    let mut options = SyncOptions::default();
    options.copyright.clone_from(&args.copyright);
    options.licence.clone_from(&args.licence);
    let generated = opened
        .preview_generated(slug, &options)
        .map_err(|error| Failure::new(error.to_string()))?;
    for (path, text) in &generated.edits {
        if path.as_str().starts_with("docs/diagrams/") {
            continue;
        }
        if let Some((_, _, body)) = previews
            .iter_mut()
            .find(|(existing, _, _)| existing == path)
        {
            body.clone_from(text);
        } else {
            let owner = plan
                .capabilities()
                .iter()
                .map(|doc| (doc.path(), &doc.value().header.id))
                .chain(
                    plan.architecture()
                        .iter()
                        .map(|doc| (doc.path(), &doc.value().header.id)),
                )
                .find_map(|(existing, id)| (existing == path.as_str()).then_some(id))
                .ok_or_else(|| {
                    Failure::new(format!(
                        "preview generated document {path} has no loaded owner"
                    ))
                })?;
            previews.push((path.clone(), owner.clone(), text.clone()));
        }
    }
    purrdf_lex::walk::try_sort_unstable_by(&mut previews, |a, b| Ok::<_, Failure>(a.0.cmp(&b.0)))?;
    let mut report = Report::of(previews.iter().map(|(path, subject, text)| {
        Record::outcome(
            CommandResult::Preview {
                path: path.to_string(),
                subject: subject.clone(),
                text: text.clone(),
            },
            format!("==> {path} <==\n{text}"),
        )
    }));
    for (path, text) in &generated.edits {
        if path.as_str().starts_with("docs/diagrams/") {
            report.note(format!(
                "would write owned diagram {path}: {} bytes",
                text.len()
            ));
        }
    }
    for path in generated.removals() {
        report.note(format!("would remove owned diagram {path}"));
    }
    Ok(report)
}
