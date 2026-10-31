// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed reference receipts and full cross-boundary fingerprint witnesses.

use super::{Index, IndexUnit, MemberRow, ReportError as E, mint, shape, write};
use crate::{Fingerprint, GroupKind};
use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_rdf::{Builder, Graph, TermValue as Term};

/// Manifest-verified dependency identity, with its precise observation boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceSource {
    /// Actual Cargo package name, independent of the dependency alias.
    pub package: String,
    /// Actual explicit or inherited package version.
    pub version: String,
    /// `local`, `git`, `registry`, `sparse`, `path`, `local-registry` or `directory`.
    pub source_kind: String,
    /// Exact external repository/index spelling; empty for an unqualified local row.
    pub repository: String,
    /// Requested Git selector, independent of the full locked revision.
    pub selector: String,
    /// Full committed Git revision; empty for a non-Git observation.
    pub revision: String,
    /// Source-relative manifest locator, retaining workspace membership context.
    pub manifest: String,
    /// Native lowercase registry archive digest; empty when the lock row has none.
    pub checksum: String,
    /// `contained-tree` observes local contents; `committed-git` reads sealed bytes.
    pub observation: String,
}

impl ReferenceSource {
    fn fields(&self) -> [(&'static str, &str); 9] {
        [
            ("package", &self.package),
            ("version", &self.version),
            ("sourceKind", &self.source_kind),
            ("repository", &self.repository),
            ("selector", &self.selector),
            ("revision", &self.revision),
            ("manifest", &self.manifest),
            ("checksum", &self.checksum),
            ("observation", &self.observation),
        ]
    }

    /// Native RDF content identity of the complete receipt, without host labels.
    ///
    /// # Errors
    /// Refuses inconsistent observation fields and native graph failures.
    pub fn identity(&self) -> Result<String, E> {
        self.validate()?;
        let mut graph = Builder::new();
        write::class(&mut graph, "", "ReferenceSource")?;
        for (key, value) in self.fields() {
            write::text(&mut graph, "", key, value)?;
        }
        let snapshot = graph
            .finish()
            .map_err(E::Ground)?
            .snapshot()
            .map_err(E::Ground)?;
        Ok(format!("#reference-source/{}", mint::part(&snapshot)))
    }

    fn validate(&self) -> Result<(), E> {
        let manifest = raf_source::RelPath::new(&self.manifest)
            .map_err(|error| E::Value(error.to_string()))?;
        if manifest.file_name() != Some("Cargo.toml")
            || manifest.as_str().split('/').any(|part| part == "..")
        {
            return Err(E::Value(
                "reference manifest must be a source-relative Cargo.toml without parent traversal"
                    .into(),
            ));
        }
        if self.package.is_empty()
            || self.version.is_empty()
            || self.manifest.is_empty()
            || !matches!(
                self.source_kind.as_str(),
                "local" | "git" | "registry" | "sparse" | "path" | "local-registry" | "directory"
            )
            || !matches!(
                self.observation.as_str(),
                "contained-tree" | "committed-git"
            )
        {
            return Err(E::Value(
                "invalid reference identity or observation kind".into(),
            ));
        }
        if (self.source_kind == "git") != (self.observation == "committed-git")
            || (self.source_kind == "git"
                && (self.revision.len() != 40
                    || purrdf_hash::hex::decode_canonical(&self.revision).is_err()))
            || (self.source_kind != "git"
                && (!self.revision.is_empty() || !self.selector.is_empty()))
            || (self.source_kind == "local" && !self.repository.is_empty())
            || (self.source_kind != "local" && self.repository.is_empty())
            || (!self.checksum.is_empty()
                && purrdf_hash::hex::decode_32_canonical(&self.checksum).is_none())
        {
            return Err(E::Value(
                "reference source fields disagree with their observation".into(),
            ));
        }
        Ok(())
    }
}

/// A physical first-party unit matching one exact reference body or shim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceWitness {
    /// First-party endpoint, always present as an ordinary indexed unit.
    pub consumer: MemberRow,
    /// Reference endpoint, always carrying a reference receipt in the index.
    pub reference: MemberRow,
    /// Complete body or shim fingerprint shared by both endpoints.
    pub fingerprint: Fingerprint,
}

fn site(row: &MemberRow) -> String {
    mint::unit(&row.symbol, &row.file, row.line, row.start_byte)
}
pub(super) fn witness_id(row: &ReferenceWitness) -> String {
    format!(
        "#reference-witness/{}/{}/{}",
        mint::part(&site(&row.consumer)),
        mint::part(&site(&row.reference)),
        mint::part(&row.fingerprint.to_string())
    )
}

pub(super) fn source(graph: &mut Builder, source: &ReferenceSource) -> Result<(), E> {
    let id = source.identity()?;
    write::class(graph, &id, "ReferenceSource")?;
    for (key, value) in source.fields() {
        write::text(graph, &id, key, value)?;
    }
    Ok(())
}

pub(super) fn witness(graph: &mut Builder, row: &ReferenceWitness, nf: u32) -> Result<(), E> {
    let id = witness_id(row);
    write::class(graph, &id, "ReferenceWitness")?;
    write::put(graph, &id, "consumer", Term::iri(site(&row.consumer)))?;
    write::put(graph, &id, "reference", Term::iri(site(&row.reference)))?;
    write::put(
        graph,
        &id,
        "fingerprint",
        write::fingerprint(&row.fingerprint, nf)?,
    )
}

pub(super) fn read_source(node: &raf_rdf::Node<'_>) -> Result<ReferenceSource, E> {
    Ok(ReferenceSource {
        package: shape::string(node, "package")?,
        version: shape::string(node, "version")?,
        source_kind: shape::string(node, "sourceKind")?,
        repository: shape::string(node, "repository")?,
        selector: shape::string(node, "selector")?,
        revision: shape::string(node, "revision")?,
        manifest: shape::string(node, "manifest")?,
        checksum: shape::string(node, "checksum")?,
        observation: shape::string(node, "observation")?,
    })
}

pub(super) fn read_witness(
    graph: &Graph,
    node: &raf_rdf::Node<'_>,
    nf: u32,
) -> Result<ReferenceWitness, E> {
    let endpoint = |key| {
        let value = shape::one(node, key)?;
        let id = value
            .iri()
            .ok_or_else(|| E::Value(format!("{key}: unit IRI required")))?;
        shape::member(graph, id)
    };
    let value = shape::one(node, "fingerprint")?;
    let fingerprint =
        shape::fp(value, nf, GroupKind::Body).or_else(|_| shape::fp(value, nf, GroupKind::Shim))?;
    Ok(ReferenceWitness {
        consumer: endpoint("consumer")?,
        reference: endpoint("reference")?,
        fingerprint,
    })
}

fn validate_unit(graph: &Graph, node: &raf_rdf::Node<'_>) -> Result<(), E> {
    let file = shape::string(node, "file")?;
    let receipt = node.values(&shape::property("referenceSource")).next();
    if let Some(receipt) = receipt {
        let source = receipt
            .iri()
            .ok_or_else(|| E::Value("reference receipt IRI required".into()))?;
        let source_node = graph
            .get(source)
            .ok_or_else(|| E::Dangling(source.into()))?;
        if shape::string(node, "package")? != shape::string(&source_node, "package")? {
            return Err(E::Inconsistent(
                "reference unit package differs from its source receipt".into(),
            ));
        }
        let prefix = format!("reference:{}/", mint::part(source));
        let path = file.strip_prefix(&prefix).ok_or_else(|| {
            E::Inconsistent("reference unit locator does not carry its receipt identity".into())
        })?;
        raf_source::RelPath::new(path).map_err(|error| E::Value(error.to_string()))?;
    } else if file.starts_with("reference:") {
        return Err(E::Missing("reference unit source receipt".into()));
    }
    Ok(())
}

pub(super) fn validate(graph: &Graph, nf: u32) -> Result<(), E> {
    for (id, node) in graph.nodes() {
        match shape::class(&node)? {
            "Unit" => validate_unit(graph, &node)?,
            "ReferenceSource" => {
                if read_source(&node)?.identity()? != id {
                    return Err(E::Name(id));
                }
                if !graph.nodes().any(|(_, unit)| {
                    unit.values(&shape::property("referenceSource"))
                        .any(|value| value.iri() == Some(id.as_str()))
                }) {
                    return Err(E::Orphan(id));
                }
            }
            "ReferenceWitness" => {
                let row = read_witness(graph, &node, nf)?;
                if witness_id(&row) != id {
                    return Err(E::Name(id));
                }
                let consumer = graph
                    .get(&site(&row.consumer))
                    .ok_or_else(|| E::Dangling(site(&row.consumer)))?;
                let reference = graph
                    .get(&site(&row.reference))
                    .ok_or_else(|| E::Dangling(site(&row.reference)))?;
                if consumer
                    .values(&shape::property("referenceSource"))
                    .next()
                    .is_some()
                    || reference
                        .values(&shape::property("referenceSource"))
                        .next()
                        .is_none()
                {
                    return Err(E::Inconsistent(
                        "reference witness must cross from first-party to a reference".into(),
                    ));
                }
                let key = if matches!(row.fingerprint, Fingerprint::Body(_)) {
                    "body"
                } else {
                    "shim"
                };
                let expected = write::fingerprint(&row.fingerprint, nf)?;
                for endpoint in [consumer, reference] {
                    if shape::one(&endpoint, key)?.to_owned() != expected {
                        return Err(E::Inconsistent(
                            "reference witness fingerprint disagrees with its endpoints".into(),
                        ));
                    }
                }
            }
            "BodyGroup" | "ShimGroup" => {
                for member in node.values(&shape::property("member")) {
                    if let Some(unit) = member.iri().and_then(|id| graph.get(id))
                        && unit
                            .values(&shape::property("referenceSource"))
                            .next()
                            .is_some()
                    {
                        return Err(E::Inconsistent(
                            "reference units cannot enter first-party structural groups".into(),
                        ));
                    }
                }
            }
            "Match"
                if shape::one(&node, "matchClass")?.iri()
                    == Some(format!("{}Copy", super::vocab::CENSUS).as_str()) =>
            {
                let (_, _, unit) = shape::quoted(shape::one(&node, "reifies")?)?;
                if graph.get(unit).is_some_and(|unit| {
                    unit.values(&shape::property("referenceSource"))
                        .next()
                        .is_some()
                }) {
                    return Err(E::Inconsistent(
                        "a reference unit cannot be a job COPY".into(),
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn project(
    index: &Index,
    graph: &mut Builder,
    selected: &[&IndexUnit],
) -> Result<(), E> {
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut selected_sites = OrderedTable::default();
    for unit in selected {
        selected_sites
            .insert(
                mint::unit(&unit.symbol, &unit.file, unit.line, unit.start_byte),
                (),
                &mut memory,
            )
            .map_err(E::Storage)?;
    }
    let mut witnesses = Vec::new();
    for row in index.reference_witnesses.iter().filter(|row| {
        selected_sites.get(&site(&row.consumer)).is_some()
            || selected_sites.get(&site(&row.reference)).is_some()
    }) {
        memory.push(&mut witnesses, row).map_err(E::Storage)?;
    }
    let mut needed = selected_sites;
    for row in &witnesses {
        needed
            .insert(site(&row.consumer), (), &mut memory)
            .map_err(E::Storage)?;
        needed
            .insert(site(&row.reference), (), &mut memory)
            .map_err(E::Storage)?;
    }
    let mut sources = OrderedTable::default();
    for unit in &index.units {
        let id = mint::unit(&unit.symbol, &unit.file, unit.line, unit.start_byte);
        if needed.get(&id).is_none() {
            continue;
        }
        if !write::has_unit(graph, &id)? {
            write::index_unit(graph, unit, index.normal_form)?;
        }
        if let Some(source) = &unit.reference_source {
            sources
                .insert(source.as_str(), (), &mut memory)
                .map_err(E::Storage)?;
        }
    }
    for (source_id, ()) in sources.ordered() {
        let mut matched = None;
        for receipt in &index.reference_sources {
            if receipt.identity()? == *source_id && matched.replace(receipt).is_some() {
                return Err(E::Repeated((*source_id).into()));
            }
        }
        let receipt = matched.ok_or_else(|| E::Dangling((*source_id).into()))?;
        source(graph, receipt)?;
    }
    for row in witnesses {
        witness(graph, row, index.normal_form)?;
    }
    Ok(())
}
