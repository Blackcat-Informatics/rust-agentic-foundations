// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact present debt, its native RDF representation and monotone comparison.

use std::collections::{BTreeMap, BTreeSet};

use raf_rdf::{Builder, Format, Graph, TermValue};

use super::{ReportError as E, mint, shape, vocab, write::put};
use crate::{Error, GroupKind, check_normal_form};

/// Maximum serialized debt record size, shared by readers and writers: 16 MiB.
pub const MAX_DEBT_BYTES: usize = 16 * 1024 * 1024;

fn check_size(text: &str) -> Result<(), Error> {
    if text.len() > MAX_DEBT_BYTES {
        return Err(E::Value(format!(
            "debt record is {} bytes; maximum is {MAX_DEBT_BYTES}",
            text.len()
        ))
        .into());
    }
    Ok(())
}

/// A duplicated body or shim, identified by its exact counted member symbols.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DebtGroup {
    kind: GroupKind,
    members: BTreeMap<String, usize>,
}

impl DebtGroup {
    /// Construct a group with at least two distinct, nonempty symbols.
    ///
    /// # Errors
    /// Refuses empty or repeated symbols and groups smaller than two members.
    pub fn new(kind: GroupKind, members: impl IntoIterator<Item = String>) -> Result<Self, Error> {
        let mut unique = BTreeSet::new();
        for member in members {
            if member.is_empty() || !unique.insert(member.clone()) {
                return Err(E::Value(format!("debt member {member:?}: empty or repeated")).into());
            }
        }
        if unique.len() < 2 {
            return Err(E::Value("debt group requires at least two members".into()).into());
        }
        Self::with_counts(kind, unique.into_iter().map(|symbol| (symbol, 1)).collect())
    }

    /// Construct exact symbol multiplicities, retaining distinct cfg source units.
    /// Ordinary symbols have count one; counts never depend on source line numbers.
    ///
    /// # Errors
    /// Refuses empty symbols, zero counts, overflow or fewer than two total members.
    pub fn with_counts(kind: GroupKind, members: BTreeMap<String, usize>) -> Result<Self, Error> {
        let mut total = 0usize;
        for (symbol, count) in &members {
            if symbol.is_empty() || *count == 0 {
                return Err(E::Value(
                    "debt members require nonempty symbols and positive counts".into(),
                )
                .into());
            }
            total = total
                .checked_add(*count)
                .ok_or_else(|| E::Value("debt member count overflow".into()))?;
        }
        if total < 2 {
            return Err(E::Value("debt group requires at least two members".into()).into());
        }
        Ok(Self { kind, members })
    }

    /// The kind of duplication.
    #[must_use]
    pub const fn kind(&self) -> GroupKind {
        self.kind
    }

    /// The exact symbol multiplicities, in lexical order.
    #[must_use]
    pub fn members(&self) -> &BTreeMap<String, usize> {
        &self.members
    }

    fn contained_in(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.members.iter().all(|(symbol, count)| {
                other
                    .members
                    .get(symbol)
                    .is_some_and(|other| count <= other)
            })
    }

    fn node(&self) -> String {
        // A comma separates encoded members; commas within a symbol must not
        // alias that separator. Other escaping remains the shared native codec.
        let members = self
            .members
            .iter()
            .map(|(member, count)| {
                let member = mint::part(member).replace(',', "%2C");
                if *count == 1 {
                    member
                } else {
                    format!("{member}@{count}")
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let kind = match self.kind {
            GroupKind::Body => "body",
            GroupKind::Shim => "shim",
        };
        format!("#debt/{kind}/{members}")
    }
}

/// The exact duplication recorded for one normal-form version.
///
/// Zero-copy jobs are absent. Older normal-form versions can be read for the
/// explicit adoption path; comparison requires the implemented version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebtRecord {
    normal_form: u32,
    groups: BTreeSet<DebtGroup>,
    job_copies: BTreeMap<String, usize>,
}

impl DebtRecord {
    /// Construct a record, refusing repeated groups and empty or zero-copy jobs.
    ///
    /// # Errors
    /// Refuses invalid group or job identities and a zero normal-form version.
    pub fn new(
        normal_form: u32,
        groups: impl IntoIterator<Item = DebtGroup>,
        job_copies: BTreeMap<String, usize>,
    ) -> Result<Self, Error> {
        if normal_form == 0 {
            return Err(E::Value("debt normalForm must be positive".into()).into());
        }
        let mut unique = BTreeSet::new();
        for group in groups {
            if !unique.insert(group) {
                return Err(E::Repeated("debt group".into()).into());
            }
        }
        if job_copies
            .iter()
            .any(|(id, count)| id.is_empty() || *count == 0)
        {
            return Err(
                E::Value("debt jobs require a nonempty id and positive copies".into()).into(),
            );
        }
        Ok(Self {
            normal_form,
            groups: unique,
            job_copies,
        })
    }

    /// The version in which this debt was recorded.
    #[must_use]
    pub const fn normal_form(&self) -> u32 {
        self.normal_form
    }

    /// The recorded exact groups, in deterministic order.
    #[must_use]
    pub fn groups(&self) -> &BTreeSet<DebtGroup> {
        &self.groups
    }

    /// Positive copy counts, keyed by job id.
    #[must_use]
    pub fn job_copies(&self) -> &BTreeMap<String, usize> {
        &self.job_copies
    }

    /// Compare the present debt to this record. Every difference requires action.
    /// A strict subset replacing an absent recorded group is shrinkage. An
    /// unchanged recorded group cannot sponsor additional subset groups.
    ///
    /// # Errors
    /// Refuses a recorded or present normal form different from the implemented one.
    pub fn compare(&self, present: &Self) -> Result<Vec<DebtChange>, Error> {
        check_normal_form(self.normal_form)?;
        check_normal_form(present.normal_form)?;
        let mut changes = Vec::new();
        for group in &present.groups {
            if !self.groups.iter().any(|recorded| {
                group == recorded
                    || (group.contained_in(recorded) && !present.groups.contains(recorded))
            }) {
                changes.push(DebtChange::NewGroup(group.clone()));
            }
        }
        for group in &self.groups {
            if !present.groups.contains(group) {
                let subsets = present
                    .groups
                    .iter()
                    .filter(|current| current.contained_in(group))
                    .cloned()
                    .collect();
                changes.push(DebtChange::MissingGroup {
                    recorded: group.clone(),
                    present: subsets,
                });
            }
        }
        let ids: BTreeSet<_> = self
            .job_copies
            .keys()
            .chain(present.job_copies.keys())
            .collect();
        for id in ids {
            let recorded = self.job_copies.get(id).copied().unwrap_or(0);
            let current = present.job_copies.get(id).copied().unwrap_or(0);
            if recorded != current {
                changes.push(DebtChange::JobCopies {
                    id: id.clone(),
                    recorded,
                    present: current,
                });
            }
        }
        Ok(changes)
    }

    /// Render the stored canonical Turtle 1.2 form.
    ///
    /// # Errors
    /// Refuses a native graph construction, serialization or size failure.
    pub fn to_turtle(&self) -> Result<String, Error> {
        self.render(Format::Turtle)
    }

    /// Render the equivalent canonical N-Triples 1.2 form.
    ///
    /// # Errors
    /// Refuses a native graph construction, serialization or size failure.
    pub fn to_ntriples(&self) -> Result<String, Error> {
        self.render(Format::NTriples)
    }

    /// Read the closed canonical Turtle 1.2 debt profile.
    ///
    /// # Errors
    /// Refuses invalid syntax, digest, layout, cardinality, identities or values.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::Turtle)
    }

    /// Read the equivalent closed canonical N-Triples 1.2 debt profile.
    ///
    /// # Errors
    /// Refuses invalid syntax, digest, layout, cardinality, identities or values.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::NTriples)
    }

    fn render(&self, format: Format) -> Result<String, Error> {
        let mut graph = Builder::new();
        put(&mut graph, "", "type", shape::iri("DebtRecord"))?;
        put(
            &mut graph,
            "",
            "normalForm",
            shape::integer(u128::from(self.normal_form)),
        )?;
        put(
            &mut graph,
            "",
            "reportForm",
            shape::integer(u128::from(vocab::REPORT_FORM)),
        )?;
        for group in &self.groups {
            let id = group.node();
            let class = match group.kind {
                GroupKind::Body => "BodyDebt",
                GroupKind::Shim => "ShimDebt",
            };
            put(&mut graph, &id, "type", shape::iri(class))?;
            for (member, count) in &group.members {
                put(
                    &mut graph,
                    &id,
                    "memberSymbol",
                    TermValue::simple_literal(member),
                )?;
                if *count > 1 {
                    let counted = format!("{id}/multiplicity/{}", mint::part(member));
                    put(&mut graph, &id, "multiplicity", TermValue::iri(&counted))?;
                    put(
                        &mut graph,
                        &counted,
                        "type",
                        shape::iri("MemberMultiplicity"),
                    )?;
                    put(
                        &mut graph,
                        &counted,
                        "symbol",
                        TermValue::simple_literal(member),
                    )?;
                    put(
                        &mut graph,
                        &counted,
                        "copies",
                        shape::integer(*count as u128),
                    )?;
                }
            }
        }
        for (id, count) in &self.job_copies {
            let node = mint::job(id);
            put(&mut graph, &node, "type", shape::iri("JobDebt"))?;
            put(&mut graph, &node, "jobId", TermValue::simple_literal(id))?;
            put(&mut graph, &node, "copies", shape::integer(*count as u128))?;
        }
        let text = graph
            .finish()
            .map_err(E::Ground)?
            .to_text(format)
            .map_err(E::Ground)?;
        check_size(&text)?;
        Ok(text)
    }

    fn read(text: &str, format: Format) -> Result<Self, Error> {
        check_size(text)?;
        let graph = Graph::from_text(text, format).map_err(E::Ground)?;
        Self::from_graph(&graph)
    }

    pub(super) fn from_graph(graph: &Graph) -> Result<Self, Error> {
        let nf = shape::nf(graph, "DebtRecord")?;
        shape::validate(graph, "debt", nf)?;
        let mut groups = Vec::new();
        let mut jobs = BTreeMap::new();
        for (id, node) in graph.nodes() {
            match shape::class(&node)? {
                "DebtRecord" if id.is_empty() => {}
                "BodyDebt" | "ShimDebt" => {
                    let kind = if shape::class(&node)? == "BodyDebt" {
                        GroupKind::Body
                    } else {
                        GroupKind::Shim
                    };
                    let mut members: BTreeMap<_, _> = node
                        .values(&shape::property("memberSymbol"))
                        .map(|value| (value.literal().expect("validated string").0.to_owned(), 1))
                        .collect();
                    for value in node.values(&shape::property("multiplicity")) {
                        let target = value.iri().expect("validated multiplicity reference");
                        let row = graph.get(target).expect("validated multiplicity node");
                        let symbol = shape::string(&row, "symbol")?;
                        let count = shape::count(&row, "copies")?;
                        if target != format!("{id}/multiplicity/{}", mint::part(&symbol))
                            || count < 2
                        {
                            return Err(E::Name(target.into()).into());
                        }
                        let member = members.get_mut(&symbol).ok_or_else(|| {
                            E::Value(format!("multiplicity names absent member {symbol:?}"))
                        })?;
                        if *member != 1 {
                            return Err(E::Repeated(format!("multiplicity {symbol:?}")).into());
                        }
                        *member = count;
                    }
                    let group = DebtGroup::with_counts(kind, members)?;
                    if id != group.node() {
                        return Err(E::Name(id).into());
                    }
                    groups.push(group);
                }
                "JobDebt" => {
                    let job = shape::string(&node, "jobId")?;
                    if id != mint::job(&job) {
                        return Err(E::Name(id).into());
                    }
                    let count = shape::count(&node, "copies")?;
                    if jobs.insert(job.clone(), count).is_some() {
                        return Err(E::DuplicateJob(job).into());
                    }
                }
                "MemberMultiplicity" => {}
                _ => return Err(E::Name(id).into()),
            }
        }
        Self::new(nf, groups, jobs)
    }
}

/// A difference between recorded and present duplication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DebtChange {
    /// A group is neither recorded nor a subset replacing an absent recorded
    /// group of the same kind.
    NewGroup(DebtGroup),
    /// An exact recorded group is absent, even if another group contains it.
    MissingGroup {
        /// The recorded group.
        recorded: DebtGroup,
        /// Present strict subsets; empty when no such subset remains.
        present: Vec<DebtGroup>,
    },
    /// A job's positive copy count changed, appeared or disappeared.
    JobCopies {
        /// The job id.
        id: String,
        /// Previously recorded count; zero when absent.
        recorded: usize,
        /// Current count; zero when absent.
        present: usize,
    },
}

impl DebtChange {
    /// Whether recording this difference would admit growth.
    #[must_use]
    pub const fn is_growth(&self) -> bool {
        match self {
            Self::NewGroup(_) => true,
            Self::MissingGroup { .. } => false,
            Self::JobCopies {
                recorded, present, ..
            } => *present > *recorded,
        }
    }
}
