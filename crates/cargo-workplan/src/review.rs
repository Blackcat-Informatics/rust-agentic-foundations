// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Review commands delegate source selection, currency and publication to the library.

use crate::{
    cli::{Review, ReviewRecordArgs, ReviewScopeArgs},
    exit::Failure,
    render::{Format, Record, Report},
    result::Result as CommandResult,
};
use raf_okf::{DocType, RecordHeader, ReviewRecord, ReviewScope};
use raf_workplan::{
    Gate, Workplan,
    escape::shown_lines,
    exec::{Programs, SystemRunner},
    git,
};
use std::{fmt::Write as _, path::Path};

pub(crate) fn run(
    root: &Path,
    command: &Review,
    format: Format,
    quiet: bool,
) -> Result<Report, Failure> {
    if quiet && format != Format::Text && matches!(command, Review::List | Review::Show(_)) {
        return Err(Failure::new(
            "review claim RDF output refuses quiet mode; omit --quiet for the complete selection",
        ));
    }
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    match command {
        Review::List => {
            let records = plan.review_list()?;
            let displayed = records
                .iter()
                .map(|loaded| {
                    let record = loaded.value();
                    let date = record.header.timestamp.as_deref().ok_or_else(|| {
                        Failure::new("validated review is missing its required date")
                    })?;
                    let text = format!(
                        "{}  {}  {}  {}  {}",
                        record.header.id,
                        record.kind.name(),
                        record.verdict.name(),
                        record.by.name(),
                        date
                    );
                    Ok(Record::outcome(
                        CommandResult::CodeReview(record.clone()),
                        shown_lines(&text),
                    ))
                })
                .collect::<Result<Vec<_>, Failure>>()?;
            let mut report = Report::of(displayed);
            if records.is_empty() {
                report.note("no review record loaded");
            }
            if format != Format::Text {
                let ids: Vec<_> = records
                    .iter()
                    .map(|record| record.value().header.id.as_str())
                    .collect();
                report = report.semantic_dataset(plan.review_claims_selection(&ids)?);
            }
            Ok(report)
        }
        Review::Show(args) => show(&plan, &args.id, format),
        Review::Status(args) | Review::ChangedSince(args) => status(&plan, args),
        Review::Unreviewed(args) => {
            let items = plan.review_unreviewed(args.kind, args.package.as_deref())?;
            let text = if items.is_empty() {
                "unreviewed: 0".to_string()
            } else {
                items.join("\n")
            };
            let finding = !items.is_empty();
            let result = CommandResult::Unreviewed {
                items,
                package: args.package.clone(),
                kind: args.kind,
            };
            Ok(Report::of([classified(
                result,
                shown_lines(&text),
                finding,
            )]))
        }
        Review::Coverage(args) => {
            if let Some(package) = &args.package {
                plan.review_inputs(&ReviewScope::Crate(package.clone()))?;
            }
            let rows = plan.review_coverage(args.kind)?;
            Ok(Report::of(
                rows.into_iter()
                    .filter(|row| {
                        args.package
                            .as_ref()
                            .is_none_or(|name| row.package == *name)
                    })
                    .map(|row| {
                        let text = format!(
                            "{}  {}  {}/{}  {}.{:02}%",
                            row.package,
                            row.kind.name(),
                            row.reviewed,
                            row.total,
                            row.percent_hundredths / 100,
                            row.percent_hundredths % 100
                        );
                        Record::outcome(CommandResult::ReviewCoverage(row), shown_lines(&text))
                    }),
            ))
        }
        Review::Record(args) => record(root, args),
    }
}

fn show(plan: &Workplan<'_>, id: &str, format: Format) -> Result<Report, Failure> {
    let record = plan.review_show(id)?.value();
    let text = record.to_document()?.render();
    let report = Report::of([Record::outcome(
        CommandResult::CodeReview(record.clone()),
        shown_lines(&text),
    )]);
    if format == Format::Text {
        Ok(report)
    } else {
        Ok(report.semantic_dataset(plan.review_claims_dataset(id)?))
    }
}

fn scope(values: &[String]) -> Result<ReviewScope, Failure> {
    let values: Vec<_> = values.iter().map(String::as_str).collect();
    ReviewScope::from_selectors(&values).ok_or_else(|| Failure::new(
        "give one crate:<package>, module:<path> or cap:<slug>, or distinct item:<path> selectors"))
}

fn status(plan: &Workplan<'_>, args: &ReviewScopeArgs) -> Result<Report, Failure> {
    let status = plan.review_status(&scope(&args.scope)?, args.kind)?;
    let state = if status.review.is_none() {
        "never-reviewed"
    } else if status.is_current() {
        "current"
    } else {
        "stale"
    };
    let mut text = format!("{}  {}  {state}", args.scope.join(" "), args.kind.name());
    for change in &status.changes {
        let kind = change.kind.name();
        let _ = write!(text, "\n{kind}  {}", change.item);
    }
    if let Some(revision) = &status.revision {
        let _ = write!(text, "\ngit diff {revision} --");
    }
    let finding = !status.is_current();
    let result = CommandResult::ReviewStatus {
        scope: args.scope.clone(),
        kind: args.kind,
        status,
    };
    Ok(Report::of([classified(
        result,
        shown_lines(&text),
        finding,
    )]))
}

fn classified(result: CommandResult, text: String, finding: bool) -> Record {
    if finding {
        Record::finding(result, text)
    } else {
        Record::outcome(result, text)
    }
}

fn record(root: &Path, args: &ReviewRecordArgs) -> Result<Report, Failure> {
    let runner = SystemRunner::new(Programs::new(root));
    let revision = match &args.revision {
        Some(revision) => revision.clone(),
        None => git::head(&runner)?.as_str().to_string(),
    };
    let mut header = RecordHeader::new(DocType::Review, &args.id);
    header.timestamp = Some(args.date.clone());
    let mut draft = ReviewRecord::new(
        header,
        scope(&args.scope)?,
        revision,
        args.kind,
        args.verdict,
        args.by.actor(),
        Vec::new(),
    );
    draft.findings_ref.clone_from(&args.findings);
    let mut options = raf_workplan::record::RecordOptions::new();
    if let Some(copyright) = &args.copyright {
        options = options.copyright(copyright);
    }
    if let Some(licence) = &args.licence {
        options = options.licence(licence);
    }
    let recorded = raf_workplan::review::record_review(root, &runner, &draft, &options)?;
    let text = format!("recorded: {}", recorded.path);
    let finding = args.verdict != raf_okf::ReviewVerdict::Clean;
    Ok(Report::of([classified(
        CommandResult::ReviewRecorded(recorded),
        text,
        finding,
    )]))
}
