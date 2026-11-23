// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Native cross-boundary evidence from committed Git reference observations.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_rdf::{Builder, Format, Graph, Node, TermValue};

use super::projection::{Evidence, JobRef, Sites, UnitRef, member_site, required_unit, site_id};
use super::{Index, ReferenceSource, ReportError as E, mint, query, reference, shape, write};
use crate::{Error, GateReport, MatchClass};

const CLASS: &str = "ReferenceMatches";
const REPORT: &str = "reference-matches";
const ROLE: &str = "ReferenceMatchRole";

/// Native evidence of physical matches against committed Git sources.
#[derive(Clone, Debug)]
pub struct ReferenceMatchReport {
    graph: Graph,
}

impl ReferenceMatchReport {
    /// The authoritative native graph of witnesses, their endpoints, source
    /// receipts and available per-job endpoint classifications.
    #[must_use]
    pub const fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Render canonical N-Triples 1.2 within the index-size ceiling.
    ///
    /// # Errors
    /// Refuses serialization or output exceeding the ceiling.
    pub fn to_ntriples(&self) -> Result<String, Error> {
        self.render(Format::NTriples)
    }

    /// Render canonical Turtle 1.2 with the same snapshot identity.
    ///
    /// # Errors
    /// Refuses serialization or output exceeding the ceiling.
    pub fn to_turtle(&self) -> Result<String, Error> {
        self.render(Format::Turtle)
    }

    /// Read a canonical N-Triples 1.2 match report.
    ///
    /// # Errors
    /// Refuses noncanonical, oversized or inconsistent report evidence.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::NTriples)
    }

    /// Read a canonical Turtle 1.2 match report.
    ///
    /// # Errors
    /// Refuses noncanonical, oversized or inconsistent report evidence.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::Turtle)
    }

    fn read(text: &str, format: Format) -> Result<Self, Error> {
        query::check_size(text, "reference matches report")?;
        Self::from_graph(Graph::from_text(text, format).map_err(E::Ground)?)
    }

    fn render(&self, format: Format) -> Result<String, Error> {
        Ok(self
            .graph
            .to_text_bounded(format, crate::MAX_INDEX_BYTES)
            .map_err(E::Ground)?)
    }

    fn from_graph(graph: Graph) -> Result<Self, Error> {
        validate(&graph)?;
        Ok(Self { graph })
    }
}

impl GateReport {
    /// Project only committed Git witnesses from this evaluation.
    ///
    /// An evaluation without such witnesses produces an empty match report.
    ///
    /// # Errors
    /// Refuses inconsistent evidence and storage failures.
    pub fn reference_matches(&self) -> Result<ReferenceMatchReport, Error> {
        project(&self.census().into(), self.jobs().iter().map(JobRef::from))
    }
}

impl Index {
    /// Project committed Git witnesses and available endpoint roles from this index.
    ///
    /// An index without such witnesses produces an empty match report.
    ///
    /// # Errors
    /// Refuses inconsistent evidence and storage failures.
    pub fn reference_matches(&self) -> Result<ReferenceMatchReport, Error> {
        project(&self.into(), self.jobs.iter().map(JobRef::from))
    }
}

fn pinned(source: &ReferenceSource) -> bool {
    source.source_kind == "git" && source.observation == "committed-git"
}

fn project<'a>(
    evidence: &Evidence<'a>,
    jobs: impl Iterator<Item = JobRef<'a>>,
) -> Result<ReferenceMatchReport, Error> {
    let mut graph = Builder::new();
    write::root(&mut graph, CLASS, evidence.normal_form)?;
    let selected = project_witnesses(evidence, &mut graph)?;
    if !selected.is_empty() {
        reference::project_units(
            evidence,
            &mut graph,
            selected
                .iter()
                .map(|(site, unit)| required_unit(*site, *unit)),
        )?;
        for job in jobs {
            for (site, class) in job.matches() {
                if selected.get(&site).is_some() {
                    write_role(&mut graph, job.id, &site_id(site), class)?;
                }
            }
        }
    }
    ReferenceMatchReport::from_graph(graph.finish().map_err(E::Ground)?)
}

fn project_witnesses<'a>(evidence: &Evidence<'a>, graph: &mut Builder) -> Result<Sites<'a>, E> {
    let mut sites = Sites::default();
    if evidence.witnesses.is_empty() {
        return Ok(sites);
    }
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut receipts = OrderedTable::default();
    for source in evidence.sources {
        let id = source.identity()?;
        if receipts
            .insert(id, pinned(source), &mut memory)
            .map_err(E::Storage)?
            .is_some()
        {
            return Err(E::Repeated("reference source receipt identity".into()));
        }
    }
    let mut references = Sites::default();
    for row in evidence.witnesses {
        references
            .insert(member_site(&row.reference), None, &mut memory)
            .map_err(E::Storage)?;
    }
    evidence.locate(&mut references)?;
    for row in evidence.witnesses {
        let site = member_site(&row.reference);
        let source = references
            .get(&site)
            .copied()
            .flatten()
            .and_then(UnitRef::source)
            .ok_or_else(|| E::Missing(format!("{}: reference source receipt", site_id(site))))?;
        let committed = receipts
            .get_by(|id: &String| source.cmp(id.as_str()))
            .ok_or_else(|| E::Dangling(source.into()))?;
        if *committed {
            let id = reference::witness(graph, row, evidence.normal_form)?;
            write::put(graph, "", "witness", TermValue::iri(id))?;
            for endpoint in [member_site(&row.consumer), site] {
                sites
                    .insert(endpoint, None, &mut memory)
                    .map_err(E::Storage)?;
            }
        }
    }
    evidence.locate(&mut sites)?;
    Ok(sites)
}

fn write_role(graph: &mut Builder, job: &str, unit: &str, class: MatchClass) -> Result<(), E> {
    let id = mint::reifier(job, unit);
    write::class(graph, &id, ROLE)?;
    write::text(graph, &id, "jobId", job)?;
    write::put(graph, &id, "member", TermValue::iri(unit))?;
    write::put(graph, &id, "matchClass", write::match_class(class))?;
    write::put(graph, "", "matchRole", TermValue::iri(id))
}

fn linked<'a>(node: &Node<'a>, key: &str) -> Result<&'a str, E> {
    shape::one(node, key)?
        .iri()
        .ok_or_else(|| E::Value(format!("census:{key}: IRI required")))
}

fn validate(graph: &Graph) -> Result<(), E> {
    let nf = shape::nf(graph, CLASS)?;
    shape::validate(graph, REPORT, nf)?;
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut endpoints = OrderedTable::default();
    for (_, witness) in graph
        .nodes()
        .filter(|(_, node)| shape::class(node) == Ok("ReferenceWitness"))
    {
        let consumer = linked(&witness, "consumer")?;
        let reference = linked(&witness, "reference")?;
        endpoints
            .insert(consumer, false, &mut memory)
            .map_err(E::Storage)?;
        endpoints
            .insert(reference, true, &mut memory)
            .map_err(E::Storage)?;
    }
    for (id, node) in graph.nodes() {
        match shape::class(&node)? {
            "Unit" if endpoints.get(&id.as_str()).is_none() => return Err(E::Orphan(id)),
            "ReferenceSource" if !pinned(&reference::read_source(&node)?) => {
                return Err(E::Inconsistent(format!(
                    "{id}: expected a committed Git source"
                )));
            }
            ROLE => {
                let unit = linked(&node, "member")?;
                let Some(reference) = endpoints.get(&unit) else {
                    return Err(E::Inconsistent(format!(
                        "{id}: role is unrelated to a witness endpoint"
                    )));
                };
                let job = shape::string(&node, "jobId")?;
                if id != mint::reifier(&job, unit) {
                    return Err(E::Name(id));
                }
                if *reference
                    && shape::one(&node, "matchClass")?.to_owned()
                        == write::match_class(MatchClass::Copy)
                {
                    return Err(E::Inconsistent(format!(
                        "{id}: a reference unit cannot be a job COPY"
                    )));
                }
            }
            _ => {}
        }
    }
    Ok(())
}
