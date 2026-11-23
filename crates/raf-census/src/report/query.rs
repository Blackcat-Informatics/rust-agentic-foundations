// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed native projections of an index's selected policy and physical sites.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_rdf::{Builder, Format, Graph, TermValue};

use super::projection::{Evidence, JobRef, Sites, site_id};
use super::{Index, ReportError as E, mint, reference, shape, write};
use crate::{Error, GateReport};

/// A selector over the complete index, without another policy evaluation.
#[derive(Clone, Copy, Debug)]
pub enum Query<'a> {
    /// One configured job, including all classified physical matches.
    Job(&'a str),
    /// One configured job's exact nonnegative copy count.
    Count(&'a str),
    /// Every physical occurrence of a symbol, including absent fingerprints.
    Fingerprint(&'a str),
}

/// An immutable, strictly validated native Census query result.
#[derive(Clone, Debug)]
pub struct QueryReport {
    graph: Graph,
}

impl QueryReport {
    /// The authoritative native graph, with no parallel assertion collection.
    #[must_use]
    pub const fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Render canonical N-Triples 1.2 within the index-size ceiling.
    ///
    /// # Errors
    /// Returns native serialization and size refusals without partial output.
    pub fn to_ntriples(&self) -> Result<String, Error> {
        self.render(Format::NTriples)
    }

    /// Render canonical Turtle 1.2 with the same snapshot identity.
    ///
    /// # Errors
    /// Returns native serialization and size refusals without partial output.
    pub fn to_turtle(&self) -> Result<String, Error> {
        self.render(Format::Turtle)
    }

    /// Read exactly a canonical, closed N-Triples query result.
    ///
    /// # Errors
    /// Refuses unsupported versions, foreign fields, contradictory roles/counts
    /// and unrelated or missing result nodes.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::NTriples)
    }

    /// Read exactly a canonical, closed Turtle query result.
    ///
    /// # Errors
    /// Refuses unsupported versions, foreign fields, contradictory roles/counts
    /// and unrelated or missing result nodes.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::Turtle)
    }

    fn from_graph(graph: Graph) -> Result<Self, Error> {
        validate(&graph)?;
        Ok(Self { graph })
    }

    fn read(text: &str, format: Format) -> Result<Self, Error> {
        check_size(text, "query")?;
        Self::from_graph(Graph::from_text(text, format).map_err(E::Ground)?)
    }

    fn render(&self, format: Format) -> Result<String, Error> {
        Ok(self
            .graph
            .to_text_bounded(format, crate::MAX_INDEX_BYTES)
            .map_err(E::Ground)?)
    }
}

impl Index {
    /// Select a complete native query result without changing this index.
    ///
    /// Job/count selection verifies physical membership, uniqueness and copies
    /// against Copy roles. Fingerprint selection retains all matching sites.
    ///
    /// # Errors
    /// Returns typed unknown selectors, native allocation failures and invalid
    /// selected records; it never substitutes empty policy or recomputes hashes.
    pub fn query(&self, query: Query<'_>) -> Result<QueryReport, Error> {
        project(&self.into(), self.jobs.iter().map(JobRef::from), query)
    }
}

impl GateReport {
    /// Select native evidence from this evaluation without building an index.
    ///
    /// # Errors
    /// Refuses unknown selectors, inconsistent evidence and storage failures.
    pub fn query(&self, query: Query<'_>) -> Result<QueryReport, Error> {
        project(
            &self.census().into(),
            self.jobs().iter().map(JobRef::from),
            query,
        )
    }
}

fn project<'a>(
    evidence: &Evidence<'a>,
    jobs: impl Iterator<Item = JobRef<'a>>,
    query: Query<'_>,
) -> Result<QueryReport, Error> {
    let mut graph = Builder::new();
    let nf = evidence.normal_form;
    match query {
        Query::Job(id) | Query::Count(id) => {
            let job = select_job(jobs, id)?;
            let selected = job.selected(evidence)?;
            let is_job = matches!(query, Query::Job(_));
            write::root(
                &mut graph,
                if is_job { "JobQuery" } else { "CountQuery" },
                nf,
            )?;
            write::text(&mut graph, "", "jobId", id)?;
            if is_job {
                reference::project(evidence, &mut graph, &selected)?;
                let job = write::job(&mut graph, &job)?;
                write::put(&mut graph, "", "job", TermValue::iri(job))?;
            } else {
                write::num(&mut graph, "", "copies", job.copies)?;
            }
        }
        Query::Fingerprint(symbol) => {
            write::root(&mut graph, "FingerprintQuery", nf)?;
            write::text(&mut graph, "", "symbol", symbol)?;
            let selected = select_symbol(evidence, symbol)?;
            reference::project(evidence, &mut graph, &selected)?;
            for (site, _) in selected.ordered() {
                write::put(&mut graph, "", "member", TermValue::iri(site_id(*site)))?;
            }
        }
    }
    QueryReport::from_graph(graph.finish().map_err(E::Ground)?)
}

fn select_job<'a>(jobs: impl Iterator<Item = JobRef<'a>>, id: &str) -> Result<JobRef<'a>, E> {
    let mut jobs = jobs.filter(|job| job.id == id);
    let job = jobs.next().ok_or_else(|| E::UnknownJob(id.into()))?;
    if jobs.next().is_some() {
        return Err(E::DuplicateJob(id.into()));
    }
    Ok(job)
}

fn select_symbol<'a>(evidence: &Evidence<'a>, symbol: &str) -> Result<Sites<'a>, E> {
    let mut units = Sites::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for unit in evidence.units() {
        let site = unit.site();
        if site.0 == symbol
            && units
                .insert(site, Some(unit), &mut memory)
                .map_err(E::Storage)?
                .is_some()
        {
            return Err(E::Repeated(site_id(site)));
        }
    }
    if units.is_empty() {
        return Err(E::UnknownSymbol(symbol.into()));
    }
    Ok(units)
}

pub(super) fn check_size(text: &str, description: &str) -> Result<(), E> {
    if text.len() > crate::MAX_INDEX_BYTES {
        return Err(E::Value(format!(
            "{description} is {} bytes; maximum is {}",
            text.len(),
            crate::MAX_INDEX_BYTES
        )));
    }
    Ok(())
}

fn validate(graph: &Graph) -> Result<(), E> {
    let root = graph
        .get("")
        .ok_or_else(|| E::Missing("query root".into()))?;
    let class = shape::class(&root)?;
    let report = match class {
        "JobQuery" => "job-query",
        "CountQuery" => "count-query",
        "FingerprintQuery" => "fingerprint-query",
        found => {
            return Err(E::Kind {
                expected: "Census query result".into(),
                found: found.into(),
            });
        }
    };
    let nf = shape::nf(graph, class)?;
    shape::validate(graph, report, nf)?;
    let mut selected = OrderedTable::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let (original, property) = match class {
        "JobQuery" => {
            let job = shape::one(&root, "job")?
                .iri()
                .and_then(|id| graph.get(id))
                .ok_or_else(|| E::Value("query job IRI".into()))?;
            (job, "match")
        }
        "FingerprintQuery" => (root, "member"),
        _ => (root, "match"),
    };
    for id in original
        .values(&shape::property(property))
        .filter_map(raf_rdf::Value::iri)
    {
        selected.insert(id, (), &mut memory).map_err(E::Storage)?;
    }
    for (_, node) in graph
        .nodes()
        .filter(|(_, node)| shape::class(node) == Ok("ReferenceWitness"))
    {
        if !["consumer", "reference"].iter().any(|key| {
            node.values(&shape::property(key))
                .any(|value| value.iri().is_some_and(|id| selected.get(&id).is_some()))
        }) {
            return Err(E::Inconsistent(
                "query reference witness is unrelated to its selected job or symbol occurrences"
                    .into(),
            ));
        }
    }
    match class {
        "JobQuery" => {
            let selected = shape::string(&root, "jobId")?;
            let id = shape::one(&root, "job")?
                .iri()
                .ok_or_else(|| E::Value("query job IRI".into()))?;
            if id != mint::job(&selected)
                || graph
                    .nodes()
                    .filter(|(_, node)| shape::class(node) == Ok("Job"))
                    .count()
                    != 1
            {
                return Err(E::Inconsistent(
                    "query selects exactly its named job".into(),
                ));
            }
        }
        "CountQuery" => {
            shape::count(&root, "copies")?;
        }
        "FingerprintQuery" => {
            let symbol = shape::string(&root, "symbol")?;
            for value in root.values(&shape::property("member")) {
                let node = value
                    .iri()
                    .and_then(|id| graph.get(id))
                    .ok_or_else(|| E::Value("query unit IRI".into()))?;
                if shape::string(&node, "symbol")? != symbol {
                    return Err(E::Inconsistent(
                        "fingerprint query symbol and occurrence".into(),
                    ));
                }
            }
        }
        _ => unreachable!("the query root class was checked above"),
    }
    Ok(())
}
