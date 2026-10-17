// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Where each obligation comes from.
//!
//! Every function here reads a value the model already holds (the graph, the
//! roadmap gaps, the analysis of the generated blocks, the prose lines of a
//! body) and turns what it finds into an [`Obligation`]. None of them walks
//! the documents a second time to decide a rule `check` already decides.

use std::collections::{BTreeMap, BTreeSet};

use raf_okf::{Body, Capability, CapabilitySection, Change, Location, Outcome, State};

use crate::bindings::wrong_role;
use crate::finding::kind;
use crate::generated::{self, INDEX_PATH};
use crate::graph::{Binding, BindingKind};
use crate::lifecycle::Impact;
use crate::lookup::{CHANGE_PREFIX, capability_path};
use crate::model::IndexSlot;
use crate::roadmap::{self, Gap};
use crate::{Loaded, Workplan};

use super::{
    BindingWhy, Found, Needed, Obligation, ObligationKind, OutcomeProblem, Part, Place,
    ReconcileError, Reconciled, Subject, SyncProblem, suppressed, tense,
};

/// The capability documents by id.
type Capabilities<'a> = BTreeMap<&'a str, &'a Loaded<Capability>>;

/// The obligations of `change`.
pub(super) fn reconcile(plan: &Workplan<'_>, change: &str) -> Result<Reconciled, ReconcileError> {
    // The footprint is the one `change impact` prints: the listed
    // capabilities and their bindings are read from it.
    let impact = plan.impact(change)?;
    let record = plan.change(change)?;
    let by_id: Capabilities<'_> = plan
        .capabilities()
        .iter()
        .map(|cap| (cap.value().header.id.as_str(), cap))
        .collect();
    let mut found = Vec::new();
    let listed = listed(plan, &impact, &by_id, &mut found)?;
    found.extend(plan.unresolved_proposals(&impact.change));
    items(plan, record, &mut found);
    bindings(plan, &impact, &listed, &mut found);
    stale_blocks(plan, &listed, &mut found);
    wording(&listed, &mut found);
    super::trace::raise(plan, &listed, &mut found);
    super::trace::assessments(plan, &impact.sections, &mut found);

    let effect = record.value().contract_effect;
    found.retain(|obligation| !suppressed(obligation.kind, obligation.section, effect));
    Ok(Reconciled::new(impact.change, found))
}

/// The loaded documents of the capabilities the change lists, in order of id.
///
/// A listed capability that did not load is an error, with one exception: a
/// delivered capability that only binds nothing in a list it must bind. Its
/// document is otherwise sound and what it owes is that binding, so the
/// obligation is raised here and the capability takes part in nothing else.
fn listed<'a>(
    plan: &Workplan<'_>,
    impact: &Impact,
    by_id: &Capabilities<'a>,
    found: &mut Vec<Obligation>,
) -> Result<Vec<&'a Loaded<Capability>>, ReconcileError> {
    let mut loaded = Vec::new();
    for id in &impact.capabilities {
        if let Some(cap) = by_id.get(id.as_str()) {
            loaded.push(*cap);
            continue;
        }
        let path = capability_path(id);
        let unbound = path.as_ref().and_then(|path| {
            plan.unbound
                .iter()
                .find(|(file, _)| file == path)
                .map(|(_, lists)| lists)
        });
        match (path, unbound) {
            (Some(path), Some(lists)) => {
                for list in lists {
                    found.push(binding_owed(id, &path, *list, None, BindingWhy::Empty));
                }
            }
            (path, _) => {
                return Err(ReconcileError::CapabilityNotLoaded {
                    change: impact.change.clone(),
                    capability: id.clone(),
                    findings: path.map_or_default(|path| plan.findings_of(&path)),
                });
            }
        }
    }
    Ok(loaded)
}

/// A binding a capability still owes, in its `Verification` section where the
/// matrix of its bindings stands.
fn binding_owed(
    capability: &str,
    path: &str,
    list: BindingKind,
    entry: Option<&str>,
    why: BindingWhy,
) -> Obligation {
    Obligation {
        kind: ObligationKind::BindingMissing,
        subject: Subject::Capability(capability.to_string()),
        section: Part::Section(CapabilitySection::Verification),
        location: Place {
            path: path.to_string(),
            line: None,
        },
        evidence: Needed::ResolvedBinding {
            list,
            path: entry.map(str::to_string),
            why,
        },
    }
}

impl Workplan<'_> {
    /// The PROPOSED blocks that name the change `slug`, as `unresolved-proposal`
    /// obligations: those of the loaded capability documents (from the graph)
    /// and those of capability documents that did not load, which the graph
    /// does not hold. A block stays owed wherever it stands, so the source is
    /// the one answer to "no PROPOSED block for the change remains in any
    /// capability document".
    ///
    /// A block of a document that did not load has no section, so its part is
    /// the document, and its subject is the capability the file name gives.
    pub(crate) fn unresolved_proposals(&self, slug: &str) -> Vec<Obligation> {
        let by_id: Capabilities<'_> = self
            .capabilities()
            .iter()
            .map(|cap| (cap.value().header.id.as_str(), cap))
            .collect();
        let mut found = Vec::new();
        for proposal in self
            .graph()
            .proposals()
            .iter()
            .filter(|proposal| proposal.change() == slug)
        {
            let Some(cap) = by_id.get(proposal.capability()) else {
                continue;
            };
            let line = cap
                .body()
                .proposed_blocks()
                .get(proposal.ordinal())
                .map(|block| block.marker().lines.start);
            found.push(proposal_owed(
                proposal.capability(),
                proposal.section().map_or(Part::Document, Part::Section),
                cap.path(),
                line,
            ));
        }
        for (path, block, line) in &self.unvalidated_blocks {
            if block == slug {
                let stem = path.rsplit('/').next().unwrap_or(path);
                let stem = stem.strip_suffix(".md").unwrap_or(stem);
                found.push(proposal_owed(
                    &format!("cap:{stem}"),
                    Part::Document,
                    path,
                    Some(*line),
                ));
            }
        }
        found
    }
}

fn proposal_owed(capability: &str, section: Part, path: &str, line: Option<usize>) -> Obligation {
    Obligation {
        kind: ObligationKind::UnresolvedProposal,
        subject: Subject::Capability(capability.to_string()),
        section,
        location: Place {
            path: path.to_string(),
            line,
        },
        evidence: Needed::ProposalResolved,
    }
}

/// An item of the record with no outcome, or with a move that leads nowhere.
///
/// A `moved` outcome names a change that exists when a record with the slug is
/// there, loaded or not: a record that did not load is explained by its own
/// findings. The slugs are listed once, not looked up for each item.
fn items(plan: &Workplan<'_>, record: &Loaded<Change>, found: &mut Vec<Obligation>) {
    let value = record.value();
    let known: BTreeSet<&str> = plan
        .changes()
        .iter()
        .filter_map(|change| change.value().header.id.strip_prefix(CHANGE_PREFIX))
        .collect();
    let unloaded = plan.unloaded();
    let unloaded: BTreeSet<&str> = unloaded
        .iter()
        .filter_map(|id| id.strip_prefix(CHANGE_PREFIX))
        .collect();
    for (index, item) in value.items.iter().enumerate() {
        let problem = match &item.outcome {
            None => OutcomeProblem::Missing,
            Some(Outcome::Moved(target)) if *target == value.header.slug => {
                OutcomeProblem::MovedToItself
            }
            Some(Outcome::Moved(target))
                if !known.contains(target.as_str()) && !unloaded.contains(target.as_str()) =>
            {
                OutcomeProblem::MovedToMissing(target.clone())
            }
            Some(_) => continue,
        };
        found.push(Obligation {
            kind: ObligationKind::ItemWithoutOutcome,
            subject: Subject::Change(value.header.id.clone()),
            section: Part::Item(index + 1),
            location: Place {
                path: record.path().to_string(),
                line: None,
            },
            evidence: Needed::Outcome(problem),
        });
    }
}

/// The bindings an `implementing` or `delivered` capability still owes.
///
/// The empty lists and the entries that do not resolve are the gaps of the
/// roadmap, so the obligation list and `status` agree. A `delivered`
/// capability binds code and tests and may bind no examples, so an empty
/// `examples` list is owed only while it is `implementing`. An entry that
/// resolves to an item its list does not take is judged by the rule `check`
/// applies.
fn bindings(
    plan: &Workplan<'_>,
    impact: &Impact,
    listed: &[&Loaded<Capability>],
    found: &mut Vec<Obligation>,
) {
    let graph = plan.graph();
    let by_entry: BTreeMap<(&str, BindingKind, &str), &Binding> = impact
        .bindings
        .iter()
        .map(|binding| {
            (
                (binding.capability(), binding.kind(), binding.path()),
                binding,
            )
        })
        .collect();
    for cap in listed {
        let state = cap.value().state;
        if !matches!(state, State::Implementing | State::Delivered) {
            continue;
        }
        let id = cap.value().header.id.as_str();
        let mut owed = |list, entry: Option<&str>, why| {
            found.push(binding_owed(id, cap.path(), list, entry, why));
        };
        for gap in roadmap::gaps(id, graph) {
            match gap {
                Gap::EmptyList(BindingKind::Example) if state == State::Delivered => {}
                Gap::EmptyList(list) => owed(list, None, BindingWhy::Empty),
                Gap::Unresolved { list, binding } => {
                    let reason = by_entry
                        .get(&(id, list, binding.as_str()))
                        .and_then(|entry| entry.resolution().err())
                        .map_or_else(String::new, ToString::to_string);
                    owed(list, Some(&binding), BindingWhy::DoesNotResolve(reason));
                }
            }
        }
        for binding in impact
            .bindings
            .iter()
            .filter(|binding| binding.capability() == id)
        {
            if let Ok(sites) = binding.resolution()
                && let Some(why) = wrong_role(plan.source(), binding, sites)
            {
                owed(
                    binding.kind(),
                    Some(binding.path()),
                    BindingWhy::WrongRole(why),
                );
            }
        }
    }
}

/// The generated blocks of the listed capabilities, and of the index, that a
/// sync would rewrite.
fn stale_blocks(plan: &Workplan<'_>, listed: &[&Loaded<Capability>], found: &mut Vec<Obligation>) {
    let by_path: BTreeMap<&str, &Loaded<Capability>> =
        listed.iter().map(|cap| (cap.path(), *cap)).collect();
    for finding in generated::analyze(plan).repairs {
        let owner = if finding.file == INDEX_PATH {
            None
        } else if let Some(cap) = by_path.get(finding.file.as_str()) {
            Some(*cap)
        } else {
            continue;
        };
        let mut push = |section, line, problem| {
            found.push(Obligation {
                kind: ObligationKind::GeneratedStale,
                subject: owner.map_or(Subject::Index, |cap| {
                    Subject::Capability(cap.value().header.id.clone())
                }),
                section,
                location: Place {
                    path: finding.file.clone(),
                    line,
                },
                evidence: Needed::SyncRun(problem),
            });
        };
        match finding.location {
            Location::Line(line) => {
                // The analysis found the block in this very body, by the line
                // of its opening marker.
                let body = match (owner, &plan.index) {
                    (Some(cap), _) => cap.body(),
                    (None, IndexSlot::Valid(index)) => index.body(),
                    (None, _) => {
                        unreachable!("a stale block was found in an index that is not read")
                    }
                };
                let block = block_kind_at(body, line).unwrap_or_else(|| {
                    unreachable!("the analysis reported a block that is not there")
                });
                let section = owner
                    .and_then(|_| section_at(body, line))
                    .map_or(Part::Document, Part::Section);
                push(section, Some(line), SyncProblem::Stale(block));
            }
            _ if finding.kind == kind::INDEX_MISSING => {
                push(Part::Document, None, SyncProblem::IndexMissing);
            }
            _ => unreachable!("a repair is a stale block or a missing index"),
        }
    }
}

/// The kind of the generated block whose opening marker is on `line`.
fn block_kind_at(body: &Body, line: usize) -> Option<String> {
    body.generated_blocks()
        .iter()
        .find(|block| block.outer().lines.start == line)
        .map(|block| block.kind().to_string())
}

/// The section of a capability body that holds `line`.
fn section_at(body: &Body, line: usize) -> Option<CapabilitySection> {
    body.sections()
        .iter()
        .find(|section| section.line() <= line && line < section.content().lines.end)
        .and_then(|section| CapabilitySection::from_slug(section.slug()))
}

/// Forward-looking wording in an `implementing` or `delivered` capability,
/// and a PROPOSED block in a `delivered` one.
fn wording(listed: &[&Loaded<Capability>], found: &mut Vec<Obligation>) {
    for cap in listed {
        let state = cap.value().state;
        if !matches!(state, State::Implementing | State::Delivered) {
            continue;
        }
        let subject = || Subject::Capability(cap.value().header.id.clone());
        let place = |line| Place {
            path: cap.path().to_string(),
            line: Some(line),
        };
        for hit in tense::find(cap.body()) {
            found.push(Obligation {
                kind: ObligationKind::TenseMismatch,
                subject: subject(),
                section: hit.section.map_or(Part::Document, Part::Section),
                location: place(hit.line),
                evidence: Needed::SupportedBehaviour(Found::Phrase(hit.phrase)),
            });
        }
        if state == State::Delivered {
            for block in cap.body().proposed_blocks() {
                found.push(Obligation {
                    kind: ObligationKind::TenseMismatch,
                    subject: subject(),
                    section: block
                        .section()
                        .and_then(CapabilitySection::from_slug)
                        .map_or(Part::Document, Part::Section),
                    location: place(block.marker().lines.start),
                    evidence: Needed::SupportedBehaviour(Found::ProposedBlock(
                        block.slug().to_string(),
                    )),
                });
            }
        }
    }
}
