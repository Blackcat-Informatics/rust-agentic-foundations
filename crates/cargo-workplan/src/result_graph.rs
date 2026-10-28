// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed projections of command domain results into the shared report graph.

use crate::result::Result as CommandResult;
use raf_rdf::{Builder, Error, Graph, TermBox, TermValue, Triple, vocab};
use raf_workplan::{assessment, evidence, history};

#[cfg(test)]
mod tests;

type Answer<T> = Result<T, Error>;
fn iri(name: &str) -> TermValue {
    TermValue::iri(format!("{}{name}", vocab::REPORT))
}
fn local(name: &str) -> TermValue {
    TermValue::iri(name)
}
fn value(graph: &mut Builder, id: &str, key: &str, atom: TermValue) -> Answer<()> {
    graph.insert(Triple {
        subject: local(id),
        predicate: iri(key),
        object: atom,
    })
}
fn text(graph: &mut Builder, id: &str, key: &str, value_: &str) -> Answer<()> {
    value(graph, id, key, TermValue::simple_literal(value_))
}
fn reifies(
    graph: &mut Builder,
    observation: &str,
    subject: &str,
    property: &str,
    object: TermValue,
) -> Answer<()> {
    graph.insert(Triple {
        subject: local(observation),
        predicate: TermValue::iri(vocab::REIFIES),
        object: TermValue::Triple {
            s: TermBox::new(local(subject)),
            p: TermBox::new(iri(property)),
            o: TermBox::new(object),
        },
    })
}
fn number(graph: &mut Builder, id: &str, key: &str, value_: usize) -> Answer<()> {
    value(
        graph,
        id,
        key,
        TermValue::integer(i128::try_from(value_).expect("supported pointer width fits i128")),
    )
}
fn boolean(graph: &mut Builder, id: &str, key: &str, value_: bool) -> Answer<()> {
    value(graph, id, key, TermValue::boolean(value_))
}
fn class(graph: &mut Builder, id: &str, name: &str) -> Answer<()> {
    graph.insert(Triple {
        subject: local(id),
        predicate: TermValue::iri(vocab::TYPE),
        object: iri(name),
    })
}
fn child(
    graph: &mut Builder,
    parent: &str,
    key: &str,
    name: &str,
    ordinal: usize,
) -> Answer<String> {
    let id = format!("{parent}/{key}/{ordinal}");
    value(graph, parent, key, local(&id))?;
    class(graph, &id, name)?;
    number(graph, &id, "ordinal", ordinal)?;
    Ok(id)
}
fn strings(
    graph: &mut Builder,
    id: &str,
    key: &str,
    name: &str,
    field: &str,
    values: &[String],
) -> Answer<()> {
    for (ordinal, entry) in values.iter().enumerate() {
        let node = child(graph, id, key, name, ordinal)?;
        text(graph, &node, field, entry)?;
    }
    Ok(())
}
fn optional(graph: &mut Builder, id: &str, key: &str, text_: Option<&str>) -> Answer<()> {
    if let Some(text_) = text_ {
        text(graph, id, key, text_)?;
    }
    Ok(())
}
fn measures(graph: &mut Builder, id: &str, key: &str, values: &[raf_okf::Measure]) -> Answer<()> {
    for (ordinal, measure) in values.iter().enumerate() {
        let node = child(graph, id, key, "Measure", ordinal)?;
        text(graph, &node, "name", &measure.name)?;
        text(graph, &node, "decimal", measure.value.text())?;
    }
    Ok(())
}
fn inputs(graph: &mut Builder, id: &str, values: &[raf_okf::Input]) -> Answer<()> {
    for (ordinal, input) in values.iter().enumerate() {
        input_fields(graph, id, ordinal, input)?;
    }
    Ok(())
}
fn input_fields(
    graph: &mut Builder,
    id: &str,
    ordinal: usize,
    input: &raf_okf::Input,
) -> Answer<String> {
    let node = child(graph, id, "input", "Input", ordinal)?;
    text(graph, &node, "kind", input.kind.name())?;
    text(graph, &node, "name", &input.name)?;
    text(graph, &node, "digest", &input.digest)?;
    Ok(node)
}

pub(crate) fn graph(command: &str, results: &[&CommandResult]) -> Answer<Graph> {
    if let [CommandResult::Coverage { report, direction }] = results {
        return raf_workplan::report::coverage_graph(report, *direction);
    }
    let mut graph = Builder::new();
    class(&mut graph, "", "Report")?;
    number(&mut graph, "", "reportForm", 1)?;
    text(&mut graph, "", "command", command)?;
    for (ordinal, result) in results.iter().enumerate() {
        let id = format!("#result/{ordinal}");
        value(&mut graph, "", "result", local(&id))?;
        number(&mut graph, &id, "ordinal", ordinal)?;
        project(&mut graph, &id, result)?;
    }
    graph.finish()
}

fn project(g: &mut Builder, id: &str, result: &CommandResult) -> Answer<()> {
    match result {
        CommandResult::Finding(finding) => raf_workplan::report::finding_into(g, id, finding)?,
        CommandResult::Rewritten { path } => {
            class(g, id, "Rewritten")?;
            text(g, id, "path", path)?;
        }
        CommandResult::Completed { path, evidence } => {
            class(g, id, "Completed")?;
            text(g, id, "path", path)?;
            completion_bindings(g, id, evidence)?;
        }
        CommandResult::Preview {
            path,
            capability,
            text: body,
        } => {
            class(g, id, "Preview")?;
            text(g, id, "path", path)?;
            text(g, id, "capability", capability)?;
            text(g, id, "text", body)?;
        }
        CommandResult::State {
            state,
            capabilities,
        } => state_fields(g, id, state, capabilities)?,
        CommandResult::OpenChanges(changes) => open_changes_fields(g, id, changes)?,
        CommandResult::NewChange(change) => new_change_fields(g, id, change)?,
        CommandResult::Impact(impact) => impact_fields(g, id, impact)?,
        CommandResult::Obligation(obligation) => {
            class(g, id, "Obligation")?;
            obligation_fields(g, id, obligation)?;
        }
        CommandResult::Refused {
            failures,
            message,
            evidence,
        } => {
            class(g, id, "Refused")?;
            number(g, id, "failures", *failures)?;
            text(g, id, "message", message)?;
            completion_bindings(g, id, evidence)?;
        }
        CommandResult::Refusal { failure, repeated } => refusal_fields(g, id, failure, *repeated)?,
        CommandResult::Binding { evidence, message } => binding_fields(g, id, evidence, message)?,
        CommandResult::EvidenceRecorded(recorded) => recorded_fields(g, id, recorded)?,
        CommandResult::EvidenceSupport {
            capability,
            support,
        } => support_fields(g, id, capability, support)?,
        CommandResult::AssessmentContext(context) => context_fields(g, id, context)?,
        CommandResult::AssessmentRecorded(assessed) => assessed_fields(g, id, assessed)?,
        CommandResult::Assessment(state) => assessment_fields(g, id, state)?,
        CommandResult::History(history) => history_fields(g, id, history)?,
        CommandResult::Claims(report) => claims_fields(g, id, report)?,
        CommandResult::Lines(report) => lines_fields(g, id, report)?,
        CommandResult::DebtWritten(written) => {
            class(g, id, "DebtWritten")?;
            text(g, id, "path", written.path())?;
            text(g, id, "change", written.change().name())?;
            number(g, id, "findings", written.findings())?;
            number(g, id, "lines", written.lines())?;
        }
        CommandResult::Coverage { .. } => {
            return Err(Error::Value(
                "item coverage must be the sole command result".into(),
            ));
        }
    }
    Ok(())
}
fn obligation_fields(
    g: &mut Builder,
    id: &str,
    o: &raf_workplan::obligation::Obligation,
) -> Answer<()> {
    for (key, v) in [
        ("kind", o.kind.name().to_owned()),
        ("subject", o.subject.to_string()),
        ("section", o.section.to_string()),
        ("path", o.location.path.clone()),
        ("evidence", o.evidence.to_string()),
    ] {
        text(g, id, key, &v)?;
    }
    if let Some(line) = o.location.line {
        number(g, id, "line", line)?;
    }
    Ok(())
}
fn recorded_fields(g: &mut Builder, id: &str, recorded: &evidence::Recorded) -> Answer<()> {
    class(g, id, "EvidenceRecorded")?;
    text(g, id, "path", recorded.path.as_str())?;
    let r = &recorded.record;
    for (key, v) in [
        ("id", r.header.id.as_str()),
        ("verdict", r.verdict.name()),
        ("subject", &r.subject.text()),
        ("by", r.by.name()),
    ] {
        text(g, id, key, v)?;
    }
    optional(g, id, "command", r.command.as_deref())?;
    optional(g, id, "revision", r.revision.as_deref())?;
    strings(g, id, "for", "CapabilityReference", "id", &r.covers)?;
    measures(g, id, "value", &r.values)?;
    measures(g, id, "limit", &r.limits)?;
    for (ordinal, input) in r.inputs.iter().enumerate() {
        let node = input_fields(g, id, ordinal, input)?;
        reifies(
            g,
            &node,
            id,
            "verdict",
            TermValue::simple_literal(r.verdict.name()),
        )?;
    }
    for (i, failure) in recorded.failed.iter().enumerate() {
        let node = child(g, id, "failed", "EvidenceFailure", i)?;
        let name = match failure {
            evidence::Failed::Exited(code) => {
                if let Some(code) = code {
                    value(g, &node, "exitCode", TermValue::integer(*code))?;
                }
                "exited"
            }
            evidence::Failed::TimedOut => "timed-out",
            evidence::Failed::OverLimit(names) => {
                strings(g, &node, "measure", "MeasureReference", "name", names)?;
                "over-limit"
            }
            evidence::Failed::Binding(outcome) => {
                text(g, &node, "outcome", outcome.name())?;
                optional(g, &node, "message", outcome.reason().as_deref())?;
                "binding"
            }
            _ => return Err(Error::Value("unknown evidence failure".into())),
        };
        text(g, &node, "reason", name)?;
    }
    text(g, id, "message", &recorded.to_string())?;
    Ok(())
}
fn context_fields(g: &mut Builder, id: &str, context: &assessment::AssessContext) -> Answer<()> {
    class(g, id, "AssessmentContext")?;
    for (key, v) in [
        ("subject", context.subject.as_str()),
        ("record", context.record.as_str()),
        ("recordPath", context.record_path.as_str()),
        ("document", context.document.as_str()),
        ("section", context.section.as_str()),
    ] {
        text(g, id, key, v)?;
    }
    for (i, symbol) in context.symbols.iter().enumerate() {
        let node = child(g, id, "symbol", "ContextSymbol", i)?;
        text(g, &node, "path", &symbol.path)?;
        boolean(g, &node, "named", symbol.named)?;
        optional(g, &node, "unresolved", symbol.unresolved.as_deref())?;
        for (j, definition) in symbol.definitions.iter().enumerate() {
            let def = child(g, &node, "definition", "Definition", j)?;
            text(g, &def, "file", &definition.file)?;
            text(g, &def, "text", &definition.text)?;
        }
    }
    for (i, evidence) in context.evidence.iter().enumerate() {
        let node = child(g, id, "evidence", "ContextEvidence", i)?;
        text(g, &node, "id", &evidence.id)?;
        text(g, &node, "path", &evidence.path)?;
        optional(
            g,
            &node,
            "verdict",
            evidence.verdict.map(raf_okf::EvidenceVerdict::name),
        )?;
        measures(g, &node, "value", &evidence.values)?;
        measures(g, &node, "limit", &evidence.limits)?;
    }
    for (i, input) in context.inputs.iter().enumerate() {
        let node = child(g, id, "input", "CurrentInput", i)?;
        text(g, &node, "kind", input.kind.name())?;
        text(g, &node, "name", &input.name)?;
        match &input.now {
            assessment::InputNow::Digest(digest) => text(g, &node, "digest", &digest.to_string())?,
            assessment::InputNow::Unresolved(reason) => text(g, &node, "unresolved", reason)?,
            _ => return Err(Error::Value("unknown current input state".into())),
        }
    }
    Ok(())
}
fn history_fields(g: &mut Builder, id: &str, history: &history::History) -> Answer<()> {
    class(g, id, "History")?;
    text(g, id, "id", history.id())?;
    text(g, id, "path", history.path())?;
    for (i, entry) in history.entries().iter().enumerate() {
        let node = child(g, id, "entry", "HistoryEntry", i)?;
        text(g, &node, "commit", entry.commit().as_str())?;
        text(g, &node, "date", &entry.date().to_string())?;
        text(g, &node, "subject", entry.subject())?;
        strings(g, &node, "path", "FileReference", "path", entry.paths())?;
        for (j, source) in entry.sources().iter().enumerate() {
            let source_node = child(g, &node, "source", "HistorySource", j)?;
            match source {
                history::Source::Document => text(g, &source_node, "kind", "document")?,
                history::Source::Symbol(binding) => {
                    text(g, &source_node, "kind", "symbol")?;
                    text(g, &source_node, "binding", binding)?;
                }
                _ => return Err(Error::Value("unknown history source".into())),
            }
        }
    }
    for (i, removed) in history.removed().iter().enumerate() {
        let node = child(g, id, "removed", "RemovedBinding", i)?;
        text(g, &node, "binding", removed.binding())?;
        text(g, &node, "identifier", removed.identifier())?;
        strings(
            g,
            &node,
            "directory",
            "DirectoryReference",
            "path",
            removed.dirs(),
        )?;
        match removed.removal() {
            history::Removal::Found {
                commit,
                date,
                subject,
                last_resolved,
            } => {
                text(g, &node, "removal", "found")?;
                text(g, &node, "commit", commit.as_str())?;
                text(g, &node, "date", &date.to_string())?;
                text(g, &node, "subject", subject)?;
                optional(
                    g,
                    &node,
                    "lastResolved",
                    last_resolved
                        .as_ref()
                        .map(raf_workplan::exec::Commit::as_str),
                )?;
            }
            history::Removal::NoCommit => text(g, &node, "removal", "no-commit")?,
            history::Removal::NoCrate => text(g, &node, "removal", "no-crate")?,
            _ => return Err(Error::Value("unknown binding removal".into())),
        }
    }
    Ok(())
}
fn lines_fields(
    g: &mut Builder,
    id: &str,
    report: &raf_workplan::coverage::lines::LinesReport,
) -> Answer<()> {
    class(g, id, "LinesReport")?;
    for (i, row) in report.crates().iter().enumerate() {
        let n = child(g, id, "package", "CrateLines", i)?;
        text(g, &n, "package", row.package())?;
        for (key, v) in [
            ("counted", row.counted()),
            ("covered", row.covered()),
            ("excepted", row.excepted()),
            ("orphan", row.orphan()),
            ("shippingCounted", row.shipping_counted()),
            ("testCounted", row.test_counted()),
            ("shippingOrphan", row.shipping_orphan()),
            ("testOrphan", row.test_orphan()),
            ("debt", row.debt()),
        ] {
            number(g, &n, key, v)?;
        }
        text(g, &n, "ceiling", &row.ceiling_text())?;
        for (j, ceiling) in row.ceilings().iter().enumerate() {
            let c = child(g, &n, "ceilingRow", "Ceiling", j)?;
            text(g, &c, "of", ceiling.of.name())?;
            match &ceiling.limit {
                raf_okf::CeilingLimit::Lines(v) => text(g, &c, "lines", v.text())?,
                raf_okf::CeilingLimit::Percent(v) => text(g, &c, "percent", v.text())?,
            }
        }
    }
    for (i, row) in report.modules().iter().enumerate() {
        let n = child(g, id, "module", "ModuleLines", i)?;
        text(g, &n, "package", row.package())?;
        text(g, &n, "module", row.module())?;
        number(g, &n, "counted", row.counted())?;
        number(g, &n, "orphan", row.orphan())?;
    }
    for (i, row) in report.orphans().iter().enumerate() {
        let n = child(g, id, "orphan", "UnitLines", i)?;
        unit_fields(g, &n, row)?;
    }
    for (i, row) in report.excepted().iter().enumerate() {
        let n = child(g, id, "excepted", "ExceptedLines", i)?;
        unit_fields(g, &n, row.lines())?;
        text(g, &n, "exception", row.exception())?;
        text(g, &n, "reason", row.reason())?;
    }
    for (i, error) in report.errors().iter().enumerate() {
        let n = child(g, id, "error", "LinesError", i)?;
        text(g, &n, "message", error)?;
    }
    Ok(())
}
fn unit_fields(
    g: &mut Builder,
    id: &str,
    row: &raf_workplan::coverage::lines::UnitLines,
) -> Answer<()> {
    for (key, v) in [
        ("package", row.package()),
        ("module", row.module()),
        ("item", row.item()),
        ("unit", row.kind()),
        ("file", row.file()),
    ] {
        text(g, id, key, v)?;
    }
    for (i, (first, last)) in row.ranges().iter().enumerate() {
        let n = child(g, id, "range", "LineRange", i)?;
        number(g, &n, "first", *first)?;
        number(g, &n, "last", *last)?;
    }
    Ok(())
}

fn state_fields(
    g: &mut Builder,
    id: &str,
    state: &str,
    capabilities: &[raf_workplan::RoadmapEntry],
) -> Answer<()> {
    class(g, id, "State")?;
    text(g, id, "state", state)?;
    number(g, id, "count", capabilities.len())?;
    for (i, capability) in capabilities.iter().enumerate() {
        let c = child(g, id, "capability", "Capability", i)?;
        for (key, v) in [
            ("id", capability.id()),
            ("slug", capability.slug()),
            ("title", capability.title()),
            ("path", capability.path()),
        ] {
            text(g, &c, key, v)?;
        }
        strings(
            g,
            &c,
            "openChange",
            "ChangeReference",
            "slug",
            &capability
                .open_change_slugs()
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>(),
        )?;
        for (j, gap) in capability.gaps().iter().enumerate() {
            let node = child(g, &c, "gap", "BindingGap", j)?;
            text(g, &node, "list", gap.list().key())?;
            optional(g, &node, "binding", gap.binding())?;
            text(g, &node, "message", &gap.to_string())?;
        }
    }

    Ok(())
}

fn open_changes_fields(
    g: &mut Builder,
    id: &str,
    changes: &[raf_workplan::OpenChange],
) -> Answer<()> {
    class(g, id, "OpenChanges")?;
    number(g, id, "count", changes.len())?;
    for (i, change) in changes.iter().enumerate() {
        let c = child(g, id, "change", "OpenChange", i)?;
        for (key, v) in [
            ("id", change.id()),
            ("slug", change.slug()),
            ("path", change.path()),
        ] {
            text(g, &c, key, v)?;
        }
        strings(
            g,
            &c,
            "capability",
            "CapabilityReference",
            "id",
            change.capabilities(),
        )?;
    }

    Ok(())
}

fn new_change_fields(
    g: &mut Builder,
    id: &str,
    change: &raf_workplan::lifecycle::NewChange,
) -> Answer<()> {
    class(g, id, "NewChange")?;
    for (key, v) in [
        ("slug", change.slug.as_str()),
        ("path", change.path.as_str()),
        ("template", change.template.as_str()),
    ] {
        text(g, id, key, v)?;
    }
    boolean(g, id, "syncNeeded", change.sync_needed)?;
    strings(
        g,
        id,
        "capability",
        "CapabilityReference",
        "id",
        &change.capabilities,
    )?;
    strings(
        g,
        id,
        "affects",
        "SectionReference",
        "subject",
        &change
            .affects
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )?;
    for (i, guidance) in change.guidance.iter().enumerate() {
        let node = child(g, id, "guidance", "Guidance", i)?;
        text(g, &node, "capability", &guidance.capability)?;
        text(g, &node, "state", guidance.state.name())?;
        text(g, &node, "advice", guidance.advice.name())?;
    }

    Ok(())
}

fn impact_fields(
    g: &mut Builder,
    id: &str,
    impact: &raf_workplan::lifecycle::Impact,
) -> Answer<()> {
    class(g, id, "Impact")?;
    text(g, id, "change", &impact.change)?;
    strings(
        g,
        id,
        "capability",
        "CapabilityReference",
        "id",
        &impact.capabilities,
    )?;
    strings(
        g,
        id,
        "section",
        "SectionReference",
        "subject",
        &impact
            .sections
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    )?;
    for (i, binding) in impact.bindings.iter().enumerate() {
        let n = child(g, id, "binding", "BindingResolution", i)?;
        text(g, &n, "capability", binding.capability())?;
        text(g, &n, "list", binding.kind().key())?;
        text(g, &n, "path", binding.path())?;
        boolean(g, &n, "resolves", binding.resolution().is_ok())?;
        optional(
            g,
            &n,
            "reason",
            binding
                .resolution()
                .err()
                .map(ToString::to_string)
                .as_deref(),
        )?;
    }
    for (i, dependant) in impact.dependants.iter().enumerate() {
        let n = child(g, id, "dependant", "Dependant", i)?;
        text(g, &n, "capability", &dependant.capability)?;
        text(g, &n, "dependsOn", &dependant.depends_on)?;
    }
    for (i, link) in impact.linking.iter().enumerate() {
        let n = child(g, id, "link", "SectionLink", i)?;
        text(g, &n, "from", &link.from.to_string())?;
        text(g, &n, "target", &link.target.to_string())?;
    }

    Ok(())
}

fn refusal_fields(
    g: &mut Builder,
    id: &str,
    failure: &raf_workplan::lifecycle::Failure,
    repeated: bool,
) -> Answer<()> {
    class(g, id, "Refusal")?;
    text(g, id, "condition", failure.condition().name())?;
    if let Some(n) = failure.condition().number() {
        number(g, id, "number", usize::from(n))?;
    }
    text(g, id, "source", failure.name())?;
    text(g, id, "message", &failure.to_string())?;
    boolean(g, id, "repeated", repeated)?;
    if let Some(obligation) = failure.obligation() {
        let n = child(g, id, "obligation", "Obligation", 0)?;
        obligation_fields(g, &n, obligation)?;
    }

    Ok(())
}

fn completion_bindings(
    g: &mut Builder,
    id: &str,
    evidence: &[raf_workplan::bound::Evidence],
) -> Answer<()> {
    for (ordinal, evidence) in evidence
        .iter()
        .filter(|row| !row.runs().is_empty())
        .enumerate()
    {
        let node = format!("{id}/binding/{ordinal}");
        value(g, id, "binding", local(&node))?;
        number(g, &node, "ordinal", ordinal)?;
        binding_fields(g, &node, evidence, evidence.outcome().name())?;
    }
    Ok(())
}

fn binding_fields(
    g: &mut Builder,
    id: &str,
    evidence: &raf_workplan::bound::Evidence,
    message: &str,
) -> Answer<()> {
    class(g, id, "Binding")?;
    for (key, v) in [
        ("outcome", evidence.outcome().name()),
        ("capability", evidence.capability()),
        ("list", evidence.kind().key()),
        ("path", evidence.path()),
        ("message", message),
    ] {
        text(g, id, key, v)?;
    }
    optional(g, id, "reason", evidence.outcome().reason().as_deref())?;
    let proof = evidence.proof();
    for (key, count) in [
        ("listed", proof.listed),
        ("selected", proof.selected),
        ("passed", proof.passed),
        ("failed", proof.failed),
        ("ignored", proof.ignored),
        ("okLines", proof.ok_lines),
    ] {
        if let Some(count) = count {
            number(g, id, key, count)?;
        }
    }
    for (ordinal, run) in evidence.runs().iter().enumerate() {
        let node = child(g, id, "run", "VerificationRun", ordinal)?;
        reifies(
            g,
            &node,
            id,
            "outcome",
            TermValue::simple_literal(evidence.outcome().name()),
        )?;
        text(
            g,
            &node,
            "programKind",
            &run.invocation().program().to_string(),
        )?;
        strings(g, &node, "argument", "Argument", "text", &run.argv())?;
        if let Some(view) = run.invocation().view() {
            view_fields(g, &node, view)?;
        }
        match run.ended() {
            raf_workplan::bound::Ended::Exited(code) => {
                text(g, &node, "ended", "exited")?;
                if let Some(code) = code {
                    value(g, &node, "exitCode", TermValue::integer(code))?;
                }
            }
            raf_workplan::bound::Ended::TimedOut => text(g, &node, "ended", "timed-out")?,
            raf_workplan::bound::Ended::Overflowed => text(g, &node, "ended", "overflowed")?,
            _ => return Err(Error::Value("unknown verification process outcome".into())),
        }
    }
    Ok(())
}

fn support_fields(
    g: &mut Builder,
    id: &str,
    capability: &str,
    support: &evidence::Support,
) -> Answer<()> {
    class(g, id, "EvidenceSupport")?;
    for (key, v) in [
        ("capability", capability),
        ("evidence", support.evidence.as_str()),
        ("path", support.path.as_str()),
    ] {
        text(g, id, key, v)?;
    }
    optional(
        g,
        id,
        "verdict",
        support.verdict.map(raf_okf::EvidenceVerdict::name),
    )?;
    boolean(g, id, "listed", support.listed)?;
    for (i, line) in support.claims.iter().enumerate() {
        let n = child(g, id, "claim", "ClaimReference", i)?;
        number(g, &n, "line", *line)?;
    }

    Ok(())
}

fn assessed_fields(g: &mut Builder, id: &str, assessed: &assessment::Assessed) -> Answer<()> {
    class(g, id, "AssessmentRecorded")?;
    text(g, id, "path", assessed.path.as_str())?;
    let r = &assessed.record;
    for (key, v) in [
        ("id", r.header.id.as_str()),
        ("subject", &r.subject.text()),
        ("verdict", r.verdict.name()),
        ("by", r.by.name()),
    ] {
        text(g, id, key, v)?;
    }
    boolean(g, id, "replaced", assessed.replaced)?;
    inputs(g, id, &r.inputs)?;

    Ok(())
}

fn assessment_fields(g: &mut Builder, id: &str, state: &assessment::AssessmentState) -> Answer<()> {
    class(g, id, "Assessment")?;
    for (key, v) in [
        ("id", state.id.as_str()),
        ("path", state.path.as_str()),
        ("subject", state.subject.as_str()),
        ("verdict", state.verdict.name()),
        ("by", state.by.name()),
    ] {
        text(g, id, key, v)?;
    }
    boolean(g, id, "current", state.currency.is_current())?;
    if let assessment::Currency::Stale(changes) = &state.currency {
        for (i, input) in changes.iter().enumerate() {
            let n = child(g, id, "moved", "MovedInput", i)?;
            text(g, &n, "kind", input.kind.name())?;
            text(g, &n, "name", &input.name)?;
            let change = match &input.change {
                assessment::InputChange::Changed => "changed",
                assessment::InputChange::Unresolved(reason) => {
                    text(g, &n, "unresolved", reason)?;
                    "unresolved"
                }
                assessment::InputChange::NoLongerInput => "no-longer-input",
                assessment::InputChange::NewInput => "new-input",
                _ => return Err(Error::Value("unknown input change".into())),
            };
            text(g, &n, "change", change)?;
            text(g, &n, "message", &input.to_string())?;
        }
    }

    Ok(())
}

fn claims_fields(
    g: &mut Builder,
    id: &str,
    report: &raf_workplan::strict::ClaimsReport,
) -> Answer<()> {
    class(g, id, "ClaimCoverageReport")?;
    for (i, claim) in report.claims().iter().enumerate() {
        let n = child(g, id, "claim", "ClaimCoverage", i)?;
        text(g, &n, "capability", claim.capability())?;
        text(g, &n, "section", claim.section().slug())?;
        text(g, &n, "claim", claim.claim())?;
        number(g, &n, "line", claim.line())?;
        strings(g, &n, "test", "TestReference", "path", claim.tests())?;
    }

    Ok(())
}

fn view_fields(g: &mut Builder, node: &str, view: &raf_workplan::exec::ViewContext) -> Answer<()> {
    text(g, node, "viewDigest", view.digest())?;
    if let Some(backend) = view.filesystem_backend() {
        text(g, node, "filesystemBackend", backend)?;
        let owned_index = view.worktree_index();
        for (key, path) in [
            ("isolatedRoot", view.isolated_root()),
            ("commonGitDirectory", view.common_git_dir()),
            ("ownedGitDirectory", view.worktree_git_dir()),
            ("ownedIndexPath", owned_index.as_deref()),
        ] {
            let path = path.ok_or_else(|| Error::Value(format!("missing native context {key}")))?;
            text(
                g,
                node,
                key,
                path.to_str()
                    .ok_or_else(|| Error::Value(format!("non-UTF-8 native context {key}")))?,
            )?;
        }
    }
    text(
        g,
        node,
        "logicalRoot",
        view.logical_root()
            .to_str()
            .ok_or_else(|| Error::Value("non-UTF-8 logical execution root".into()))?,
    )?;
    if let Some(root) = view.physical_root() {
        text(g, node, "viewKind", "materialized")?;
        text(
            g,
            node,
            "physicalRoot",
            root.to_str()
                .ok_or_else(|| Error::Value("non-UTF-8 physical execution root".into()))?,
        )?;
    } else {
        text(g, node, "viewKind", "virtual")?;
    }
    match view.proof().index() {
        raf_workplan::exec::IndexProvenance::Native {
            tree,
            head,
            default_ref,
            default_commit,
            merge_base,
            index_path,
            git_dir,
            common_dir,
        } => {
            for (key, path) in [
                ("originalIndexPath", index_path),
                ("originalGitDirectory", git_dir),
                ("originalCommonDirectory", common_dir),
            ] {
                text(
                    g,
                    node,
                    key,
                    path.to_str()
                        .ok_or_else(|| Error::Value(format!("non-UTF-8 {key}")))?,
                )?;
            }
            for (key, value) in [
                ("originalIndex", tree),
                ("originalHead", head),
                ("defaultReference", default_ref),
                ("defaultCommit", default_commit),
                ("mergeBase", merge_base),
            ] {
                text(g, node, key, value)?;
            }
        }
        raf_workplan::exec::IndexProvenance::Supplied => {}
    }
    if let Some(paths) = view.proof().tracked() {
        number(g, node, "originalTracked", paths.len())?;
    }
    Ok(())
}
