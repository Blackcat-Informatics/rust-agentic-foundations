// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Validate the explicitly declared replacements of named decision sections.

use std::collections::BTreeMap;

use raf_okf::{Architecture, Body, Entry, Heading, Location};

use super::DECISION;
use crate::finding::kind;
use crate::graph::strongly_connected;
use crate::{Finding, Loaded, Workplan};

type Section<'a> = (&'a str, &'a str);
type Replacements<'a> = BTreeMap<Section<'a>, (&'a str, usize, Vec<Section<'a>>)>;

#[derive(Default)]
enum Endpoint {
    #[default]
    Missing,
    DecisionSection,
    OtherHeading,
    Ambiguous,
}

pub(super) fn check(plan: &Workplan<'_>, findings: &mut Vec<Finding>) {
    let mut endpoints = BTreeMap::<&str, BTreeMap<&str, Endpoint>>::new();
    for page in plan.architecture() {
        for replacement in &page.value().supersedes {
            endpoints
                .entry(&page.value().header.id)
                .or_default()
                .entry(replacement.section())
                .or_default();
            for target in replacement.targets() {
                endpoints
                    .entry(target.architecture())
                    .or_default()
                    .entry(target.section())
                    .or_default();
            }
        }
    }
    if endpoints.is_empty() {
        return;
    }
    for page in plan.architecture() {
        if let Some(sections) = endpoints.get_mut(page.value().header.id.as_str()) {
            read_headings(page.body(), sections);
        }
    }
    let mut replacements = BTreeMap::new();
    for page in plan.architecture() {
        check_page(plan, page, &endpoints, &mut replacements, findings);
    }
    check_cycles(&replacements, findings);
}

pub(super) fn headings_with_authorship(body: &Body) -> impl Iterator<Item = (&Heading, bool)> {
    let mut generated = body.generated_blocks().iter().peekable();
    body.headings().iter().map(move |heading| {
        while generated
            .peek()
            .is_some_and(|block| block.outer().lines.end <= heading.line())
        {
            generated.next();
        }
        let authored = !generated
            .peek()
            .is_some_and(|block| block.outer().lines.contains(&heading.line()));
        (heading, authored)
    })
}

fn read_headings(body: &Body, sections: &mut BTreeMap<&str, Endpoint>) {
    let mut in_decision = false;
    for (heading, authored) in headings_with_authorship(body) {
        if authored && heading.level() <= 2 {
            in_decision = heading.level() == 2 && heading.text() == DECISION;
        }
        if let Some(endpoint) = sections.get_mut(heading.anchor()) {
            *endpoint = match endpoint {
                Endpoint::Missing if authored && in_decision && heading.level() == 3 => {
                    Endpoint::DecisionSection
                }
                Endpoint::Missing => Endpoint::OtherHeading,
                _ => Endpoint::Ambiguous,
            };
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "profile admission retains the authored entry for every typed replacement"
)]
fn declaration<'a>(page: &'a Loaded<Architecture>, section: &str) -> &'a Entry {
    page.profiled()
        .document()
        .frontmatter()
        .map()
        .get("supersedes")
        .and_then(|entry| entry.value().as_map())
        .and_then(|map| map.get(section))
        .expect("INVARIANT: a typed supersession retains its original map entry")
}

fn check_page<'a>(
    plan: &Workplan<'_>,
    page: &'a Loaded<Architecture>,
    endpoints: &BTreeMap<&str, BTreeMap<&str, Endpoint>>,
    replacements: &mut Replacements<'a>,
    findings: &mut Vec<Finding>,
) {
    let id = page.value().header.id.as_str();
    for replacement in &page.value().supersedes {
        let section = replacement.section();
        let row = declaration(page, section);
        let line = row.line();
        let source = (id, section);
        let valid_source = matches!(endpoints[id][section], Endpoint::DecisionSection);
        if !valid_source {
            findings.push(Finding::new(
                kind::SUPERSESSION_SOURCE,
                page.path(),
                Location::Line(line),
                format!("`{id}#{section}` does not name an unambiguous decision section"),
                "name the anchor of one authored third-level heading under Decision, or remove the entry",
            ));
        }
        let mut targets = Vec::new();
        #[expect(
            clippy::expect_used,
            reason = "profile admission establishes that every typed replacement is an authored list"
        )]
        let authored = row
            .value()
            .as_list()
            .expect("INVARIANT: a typed supersession has an authored target list");
        for (index, target) in replacement.targets().iter().enumerate() {
            let target_id = target.architecture();
            let section = target.section();
            if target_id != id && matches!(endpoints[target_id][section], Endpoint::DecisionSection)
            {
                targets.push((target_id, section));
                continue;
            }
            let detail = if target_id == id {
                "names the declaring decision"
            } else if !plan.is_present(target_id) {
                "names a missing Architecture decision"
            } else {
                "does not name an unambiguous decision section"
            };
            findings.push(Finding::new(
                kind::SUPERSESSION_TARGET,
                page.path(),
                Location::Line(authored.items()[index].line()),
                format!("`{target}` {detail}"),
                "name the anchor of one authored third-level heading under Decision in another Architecture page, or remove the reference",
            ));
        }
        if valid_source {
            replacements.insert(source, (page.path(), line, targets));
        }
    }
}

fn check_cycles(replacements: &Replacements<'_>, findings: &mut Vec<Finding>) {
    let sections: Vec<_> = replacements.keys().copied().collect();
    let adjacent: Vec<Vec<usize>> = replacements
        .values()
        .map(|(_, _, targets)| {
            targets
                .iter()
                .filter_map(|target| sections.binary_search(target).ok())
                .collect()
        })
        .collect();
    for group in strongly_connected(&adjacent)
        .into_iter()
        .filter(|group| group.len() > 1)
    {
        let (file, line, _) = &replacements[&sections[group[0]]];
        let members = group
            .iter()
            .map(|index| {
                let (id, section) = sections[*index];
                format!("`{id}#{section}`")
            })
            .collect::<Vec<_>>()
            .join(", ");
        findings.push(Finding::new(
            kind::SUPERSESSION_CYCLE,
            file,
            Location::Line(*line),
            format!("decision sections {members} supersede one another in a cycle"),
            "remove a supersedes reference so the decision sections form no cycle",
        ));
    }
}
