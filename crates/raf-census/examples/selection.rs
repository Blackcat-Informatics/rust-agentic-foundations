// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Count selection alternatives without changing the production normal form.
//!
//! Arguments are exact `label=directory#package-root` corpus roots; the package
//! root defaults to `.` when omitted. The fixture registry is
//! always included. Source roots are parsed once through `Workspace` and `Tree`;
//! every candidate uses the production `Tower` forms. Output is RDF 1.2 Turtle.
//! A refused source root is recorded and the process fails after all roots have
//! been attempted. No substitute corpus is selected. No clocks are read.

#[path = "../tests/support/fixtures.rs"]
mod fixtures;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;

use raf_census::{Level, Slice, Tower};
use raf_source::{DiskTree, SymbolKind, Workspace};

const FLOORS: [usize; 8] = [4, 8, 12, 16, 20, 24, 30, 40];
const TIERS: [usize; 6] = [8, 16, 24, 30, 40, 64];
const SHIM_MIN: [usize; 3] = [3, 6, 8];
const SHIM_MAX: [usize; 3] = [20, 30, 40];

struct Body {
    id: String,
    tokens: usize,
    retained: Vec<String>,
    abstracted: Vec<String>,
    shim: Vec<String>,
    forwarder: bool,
    shim_eligible: bool,
}

impl Body {
    fn new(id: String, tower: &Tower) -> Self {
        let (retained, abstracted) = tower.literal_values();
        Self {
            id,
            tokens: tower.value_tokens(),
            retained: retained.texts(),
            abstracted: abstracted.texts(),
            shim: tower.form(Level::Shim).texts(),
            forwarder: tower.shim_candidate(),
            shim_eligible: !tower.calls_through_self(),
        }
    }
}

#[derive(Clone, Copy)]
struct Policy {
    floor: usize,
    tier: usize,
    shim_min: usize,
    shim_max: usize,
}

impl Policy {
    fn key(self, body: &Body) -> Option<(bool, &[String])> {
        // As in production grouping, a selected forwarder never joins a body
        // group even when it also has a body print.
        if body.forwarder && (self.shim_min..self.shim_max).contains(&body.tokens) {
            body.shim_eligible.then_some((true, &body.shim))
        } else if body.tokens >= self.floor {
            Some((
                false,
                if body.tokens < self.tier {
                    &body.retained
                } else {
                    &body.abstracted
                },
            ))
        } else {
            None
        }
    }

    fn id(self) -> String {
        format!(
            "f{}-t{}-s{}-{}",
            self.floor, self.tier, self.shim_min, self.shim_max
        )
    }
}

fn literal(value: &str) -> String {
    let mut out = String::new();
    purrdf_lex::term_syntax::write_literal(
        value,
        "http://www.w3.org/2001/XMLSchema#string",
        None,
        None,
        &mut out,
    );
    out
}

fn source(path: &str, root: &str) -> Result<Vec<Body>, Box<dyn Error>> {
    let tree = DiskTree::new(path)?;
    let workspace = Workspace::load(&tree, &[root])?;
    let mut bodies = Vec::new();
    for (file, offset, ids) in workspace.items_by_source() {
        let mut reaches = ids
            .iter()
            .map(|id| workspace.item(*id).ok_or("source item does not resolve"))
            .collect::<Result<Vec<_>, _>>()?;
        reaches.sort_by(|a, b| {
            (!workspace.is_shipping(a), &a.path).cmp(&(!workspace.is_shipping(b), &b.path))
        });
        let item = reaches.first().ok_or("source item has no reach")?;
        if !matches!(
            item.kind,
            SymbolKind::Fn
                | SymbolKind::InherentMethod
                | SymbolKind::TraitDefaultMethod
                | SymbolKind::TraitImplMethod
        ) {
            continue;
        }
        let Some(body) = &item.body else { continue };
        let contents = workspace
            .source(file)
            .and_then(|text| text.get(body.start_byte..body.end_byte))
            .ok_or("body span is not text of its file")?;
        let contents = contents
            .strip_prefix('{')
            .and_then(|text| text.strip_suffix('}'))
            .ok_or("body is not a braced block")?;
        let slice = Slice::new(file.as_str(), body.lines.start, contents).params(
            item.parameter_bindings()
                .iter()
                .filter(|name| *name != "self")
                .cloned(),
        );
        let tower = Tower::new(&slice)?;
        let mut measured = Body::new(
            format!("{}@{}:{}:{}", item.path, file, body.lines.start, offset),
            &tower,
        );
        measured.shim_eligible &= item.kind != SymbolKind::TraitImplMethod;
        bodies.push(measured);
        // Only owned strings and source-index byte ranges survive each tower.
        proc_macro2::extra::invalidate_current_thread_spans();
    }
    Ok(bodies)
}

fn groups(bodies: &[Body], policy: Policy) -> BTreeMap<(bool, &[String]), Vec<usize>> {
    let mut groups = BTreeMap::new();
    for (index, body) in bodies.iter().enumerate() {
        if let Some(key) = policy.key(body) {
            groups.entry(key).or_insert_with(Vec::new).push(index);
        }
    }
    groups
}

fn summarize(out: &mut String, corpus: &str, bodies: &[Body], labels: &[(&str, &str, bool)]) {
    let by_id: BTreeMap<&str, &Body> = bodies.iter().map(|body| (body.id.as_str(), body)).collect();
    let corpus_id =
        purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(corpus.as_bytes()).as_bytes())
            .to_string();
    writeln!(
        out,
        "<urn:selection:corpus:{corpus_id}> a s:Corpus ; s:label {} ; s:bodies {} ; s:tokens {} .",
        literal(corpus),
        bodies.len(),
        bodies.iter().map(|body| body.tokens as u128).sum::<u128>()
    )
    .expect("string write");
    writeln!(
        out,
        "<urn:selection:corpus:{corpus_id}> s:labelledEqual {} ; s:labelledDistinct {} .",
        labels.iter().filter(|label| label.2).count(),
        labels.iter().filter(|label| !label.2).count()
    )
    .expect("string write");
    distribution(out, &corpus_id, bodies);
    for floor in FLOORS {
        for tier in TIERS {
            for shim_min in SHIM_MIN {
                for shim_max in SHIM_MAX {
                    let policy = Policy {
                        floor,
                        tier,
                        shim_min,
                        shim_max,
                    };
                    let groups = groups(bodies, policy);
                    let selected: usize = groups.values().map(Vec::len).sum();
                    let duplicate_groups = groups.values().filter(|group| group.len() > 1).count();
                    let shim_groups = groups
                        .iter()
                        .filter(|((shim, _), group)| *shim && group.len() > 1)
                        .count();
                    let body_groups = duplicate_groups - shim_groups;
                    let duplicate_pairs: u128 = groups
                        .values()
                        .map(|group| {
                            let n = group.len() as u128;
                            n * n.saturating_sub(1) / 2
                        })
                        .sum();
                    let mut equal_selected = 0;
                    let mut equal_missed = 0;
                    let mut distinct_merged = 0;
                    for &(a, b, equal) in labels {
                        let (a, b) = (policy.key(by_id[a]), policy.key(by_id[b]));
                        let merged = a.is_some() && a == b;
                        match (equal, merged) {
                            (true, true) => equal_selected += 1,
                            (true, false) => equal_missed += 1,
                            (false, true) => distinct_merged += 1,
                            (false, false) => {}
                        }
                    }
                    writeln!(out, "<urn:selection:{corpus_id}:{}> a s:Count ; s:corpus <urn:selection:corpus:{corpus_id}> ; s:floor {floor} ; s:tier {tier} ; s:shimMin {shim_min} ; s:shimMax {shim_max} ; s:selected {selected} ; s:groups {duplicate_groups} ; s:bodyGroups {body_groups} ; s:shimGroups {shim_groups} ; s:pairs {duplicate_pairs} ; s:equalSelected {equal_selected} ; s:equalMissed {equal_missed} ; s:distinctMerged {distinct_merged} .", policy.id()).expect("string write");
                }
            }
        }
    }
    inspect(out, &corpus_id, bodies);
}

fn distribution(out: &mut String, corpus_id: &str, bodies: &[Body]) {
    let bounds = [0, 4, 6, 8, 12, 16, 20, 24, 30, 40, 64, usize::MAX];
    for pair in bounds.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        let count = bodies
            .iter()
            .filter(|body| (start..end).contains(&body.tokens))
            .count();
        writeln!(out, "<urn:selection:{corpus_id}:bin{start}> a s:Bin ; s:corpus <urn:selection:corpus:{corpus_id}> ; s:from {start} ; s:below {end} ; s:bodies {count} .").expect("string write");
    }
}

fn inspect(out: &mut String, corpus_id: &str, bodies: &[Body]) {
    // Retain concrete default and neighbouring-policy disagreement groups for
    // human inspection. These names are not normal-form fingerprints.
    let default = Policy {
        floor: 20,
        tier: 30,
        shim_min: 6,
        shim_max: 30,
    };
    let defaults = groups(bodies, default);
    for policy in [
        default,
        Policy {
            floor: 12,
            ..default
        },
        Policy {
            floor: 16,
            ..default
        },
        Policy {
            floor: 24,
            ..default
        },
        Policy {
            tier: 24,
            ..default
        },
        Policy {
            tier: 40,
            ..default
        },
        Policy {
            shim_min: 3,
            ..default
        },
        Policy {
            shim_min: 8,
            ..default
        },
        Policy {
            shim_max: 20,
            ..default
        },
        Policy {
            shim_max: 40,
            ..default
        },
    ] {
        for ((shim, form), members) in groups(bodies, policy) {
            if members.len() < 2 {
                continue;
            }
            let is_default = policy.id() == default.id();
            if !is_default && defaults.get(&(shim, form)) == Some(&members) {
                continue;
            }
            let id = format!(
                "urn:selection:{corpus_id}:{}:group{}",
                policy.id(),
                members[0]
            );
            writeln!(
                out,
                "<{id}> a s:Inspection ; s:policy {} ; s:shim {shim} ; s:form {}",
                literal(&policy.id()),
                literal(&form.join(" "))
            )
            .expect("string write");
            for index in members {
                write!(out, " ; s:member {}", literal(&bodies[index].id)).expect("string write");
            }
            out.push_str(" .\n");
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let registry = fixtures::registry();
    let bodies = registry
        .fixtures
        .iter()
        .map(|fixture| {
            Tower::new(&fixture.slice()).map(|tower| Body::new(fixture.id.clone(), &tower))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let labels = registry
        .pairs
        .iter()
        .map(|pair| {
            (
                pair.a.as_str(),
                pair.b.as_str(),
                matches!(pair.expect, fixtures::Expect::Equal(_)),
            )
        })
        .collect::<Vec<_>>();
    let mut out = String::from("VERSION \"1.2\"\nPREFIX s: <urn:raf:census:selection:>\n");
    summarize(&mut out, "labelled-fixtures", &bodies, &labels);
    let mut refused = false;
    for argument in std::env::args().skip(1) {
        let (label, path) = argument
            .split_once('=')
            .ok_or("expected exact label=directory argument")?;
        if label.is_empty() || path.is_empty() {
            return Err("empty corpus label or path".into());
        }
        let (path, root) = path.split_once('#').unwrap_or((path, "."));
        match source(path, root) {
            Ok(bodies) => {
                writeln!(
                    out,
                    "<urn:selection:input:{}> a s:Input ; s:label {} ; s:path {} ; s:root {} .",
                    purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(label.as_bytes()).as_bytes()),
                    literal(label),
                    literal(path),
                    literal(root)
                )
                .expect("string write");
                summarize(&mut out, label, &bodies, &[]);
            }
            Err(error) => {
                refused = true;
                writeln!(out, "<urn:selection:refusal:{}> a s:Refusal ; s:label {} ; s:path {} ; s:root {} ; s:reason {} .", purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(label.as_bytes()).as_bytes()), literal(label), literal(path), literal(root), literal(&error.to_string())).expect("string write");
            }
        }
    }
    print!("{out}");
    if refused {
        return Err(
            "one or more exact source corpora were refused; inspect s:Refusal records".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_counts_are_pinned_and_default_selection_matches_production() {
        let registry = fixtures::registry();
        let mut bodies = Vec::new();
        let policy = Policy {
            floor: raf_census::DEFAULT_MIN_TOKENS,
            tier: raf_census::LITERAL_TIER,
            shim_min: raf_census::MIN_SHIM_TOKENS,
            shim_max: raf_census::SHIM_MAX_TOKENS,
        };
        for fixture in &registry.fixtures {
            let tower = Tower::new(&fixture.slice()).expect("fixture tower");
            let body = Body::new(fixture.id.clone(), &tower);
            let print = tower.print(raf_census::DEFAULT_MIN_TOKENS);
            let expected = if print.shim.is_some() {
                (!tower.calls_through_self()).then(|| (true, tower.form(Level::Shim).texts()))
            } else {
                print
                    .body
                    .as_ref()
                    .map(|_| (false, tower.form(Level::Value).texts()))
            };
            assert_eq!(
                policy.key(&body).map(|(shim, form)| (shim, form.to_vec())),
                expected,
                "{}",
                fixture.id
            );
            bodies.push(body);
        }
        assert_eq!(
            (
                bodies.len(),
                bodies.iter().map(|body| body.tokens).sum::<usize>()
            ),
            (261, 3900)
        );
        let counts = groups(&bodies, policy);
        assert_eq!(counts.values().filter(|group| group.len() > 1).count(), 22);
        assert_eq!(
            counts
                .values()
                .map(|group| group.len() * group.len().saturating_sub(1) / 2)
                .sum::<usize>(),
            54
        );
    }
}
