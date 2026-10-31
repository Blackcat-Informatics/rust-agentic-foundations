// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed native projections of an index's selected policy and physical sites.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_rdf::{Builder, Format, Graph, TermValue};

use super::{Index, IndexUnit, JobRow, ReportError as E, mint, shape, write};
use crate::{Error, MatchClass};

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
        check_size(text)?;
        Self::from_graph(Graph::from_text(text, format).map_err(E::Ground)?)
    }

    fn render(&self, format: Format) -> Result<String, Error> {
        let text = self.graph.to_text(format).map_err(E::Ground)?;
        check_size(&text)?;
        Ok(text)
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
        let mut graph = Builder::new();
        match query {
            Query::Job(id) | Query::Count(id) => {
                let job = select_job(self, id)?;
                let units = selected_units(self, job)?;
                let is_job = matches!(query, Query::Job(_));
                write::root(
                    &mut graph,
                    if is_job { "JobQuery" } else { "CountQuery" },
                    self.normal_form,
                )?;
                write::text(&mut graph, "", "jobId", id)?;
                if is_job {
                    let mut selected = Vec::new();
                    for (site, unit) in units.ordered() {
                        let unit = unit.ok_or_else(|| {
                            E::Dangling(mint::unit(site.0, site.1, site.2, site.3))
                        })?;
                        write::index_unit(&mut graph, unit, self.normal_form)?;
                        selected.push(unit);
                    }
                    super::reference::project(self, &mut graph, &selected)?;
                    let job = write::job(&mut graph, job)?;
                    write::put(&mut graph, "", "job", TermValue::iri(job))?;
                } else {
                    write::num(&mut graph, "", "copies", job.copies)?;
                }
            }
            Query::Fingerprint(symbol) => {
                write::root(&mut graph, "FingerprintQuery", self.normal_form)?;
                write::text(&mut graph, "", "symbol", symbol)?;
                let mut found = false;
                let mut selected = Vec::new();
                for unit in self.units.iter().filter(|unit| unit.symbol == symbol) {
                    let id = write::index_unit(&mut graph, unit, self.normal_form)?;
                    write::put(&mut graph, "", "member", TermValue::iri(id))?;
                    found = true;
                    selected.push(unit);
                }
                if !found {
                    return Err(E::UnknownSymbol(symbol.into()).into());
                }
                super::reference::project(self, &mut graph, &selected)?;
            }
        }
        QueryReport::from_graph(graph.finish().map_err(E::Ground)?)
    }
}

fn select_job<'a>(index: &'a Index, id: &str) -> Result<&'a JobRow, E> {
    let mut jobs = index.jobs.iter().filter(|job| job.id == id);
    let job = jobs.next().ok_or_else(|| E::UnknownJob(id.into()))?;
    if jobs.next().is_some() {
        return Err(E::DuplicateJob(id.into()));
    }
    Ok(job)
}

type Site<'a> = (&'a str, &'a str, usize, usize);
type Units<'a> = OrderedTable<Site<'a>, Option<&'a IndexUnit>>;

fn selected_units<'a>(index: &'a Index, job: &'a JobRow) -> Result<Units<'a>, Error> {
    let mut units = Units::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for row in &job.matches {
        let key = (
            row.symbol.as_str(),
            row.file.as_str(),
            row.line,
            row.start_byte,
        );
        if units
            .insert(key, None, &mut memory)
            .map_err(|source| Error::QueryStorage { source })?
            .is_some()
        {
            return Err(E::Repeated(mint::unit(key.0, key.1, key.2, key.3)).into());
        }
    }
    if !units.is_empty() {
        for unit in &index.units {
            let key = (
                unit.symbol.as_str(),
                unit.file.as_str(),
                unit.line,
                unit.start_byte,
            );
            if let Some(selected) = units.get_mut_by(|known| key.cmp(known))
                && selected.replace(unit).is_some()
            {
                return Err(E::Repeated(mint::unit(key.0, key.1, key.2, key.3)).into());
            }
        }
        for (site, selected) in units.iter() {
            if selected.is_none() {
                return Err(E::Dangling(mint::unit(site.0, site.1, site.2, site.3)).into());
            }
        }
    }
    if job.copies
        != job
            .matches
            .iter()
            .filter(|row| row.class == MatchClass::Copy)
            .count()
    {
        return Err(E::Inconsistent(format!("job {}: copies", job.id)).into());
    }
    Ok(units)
}

fn check_size(text: &str) -> Result<(), E> {
    if text.len() > crate::MAX_INDEX_BYTES {
        return Err(E::Value(format!(
            "query is {} bytes; maximum is {}",
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
