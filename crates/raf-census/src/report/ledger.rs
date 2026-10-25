// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The census policy model over a closed native RDF graph.

mod declarations;
mod extensions;
pub use declarations::{LedgerPattern, PatternMatcher, PreferredProvider};
pub use extensions::LedgerExtensionGraph;

use std::collections::{BTreeMap, BTreeSet};

use purrdf_jsonschema::ecma::{self, CompiledPattern};
use raf_rdf::{Builder, Format, Graph, Node, TermValue, Triple};

use super::{ReportError as E, shape};
use crate::{
    CensusOptions, DebtGroup, Error, Fingerprint, GroupKind, Job, JobForbidden, check_normal_form,
};

/// The maximum size of a ledger source: one MiB.
pub const MAX_LEDGER_BYTES: usize = 1024 * 1024;

/// A compiled forbidden-name expression in the native bounded ECMA engine.
#[derive(Clone, Debug)]
pub struct NamePattern {
    /// The exact ECMA-262 Unicode-mode expression from the ledger.
    source: String,
    compiled: CompiledPattern,
}

impl NamePattern {
    /// The exact expression whose compiled matcher this value owns.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Match a name through the native engine and the caller's shared budget.
    ///
    /// # Errors
    /// Returns the native resource refusal rather than reporting no match.
    pub fn is_match(
        &self,
        name: &str,
        limits: &mut ecma::MatchLimits,
    ) -> Result<bool, ecma::PatternError> {
        self.compiled.is_match(name, limits)
    }
}

/// A deliberately different implementation and the documentation that explains it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerVariant {
    /// The item's source-index path.
    pub symbol: String,
    /// Its repository-relative source file.
    pub file: String,
    /// The detector whose match this variant sanctions.
    pub detector: String,
    /// The criterion that distinguishes this implementation.
    pub criterion: String,
    /// Absolute IRI of its documented justification.
    pub anchor: String,
    /// Why the variant is needed.
    pub reason: String,
}

/// One ledger job and the additional matches and justifications it declares.
#[derive(Clone, Debug)]
pub struct LedgerJob {
    /// The fingerprint/constant matcher, with unresolved source-index paths.
    pub job: Job,
    /// A nonempty description of the functionality.
    pub summary: String,
    /// Whether copies fail the gate rather than contribute to debt.
    pub enforced: bool,
    /// Forbidden function and constant names, compiled by `PurRDF`.
    pub names: Vec<NamePattern>,
    /// Declarative pattern ids; the gate must resolve every one before running.
    pub patterns: Vec<String>,
    /// Sanctioned implementations and their reasons.
    pub variants: Vec<LedgerVariant>,
    /// The native graph node offered to a registered extension, when present.
    pub extension: Option<String>,
    /// Declared preferred provider; execution requires grounded source policy.
    pub preferred_provider: Option<PreferredProvider>,
}

/// One exact sanctioned member set and its documentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerDistinct {
    /// The exact member set; a new member invalidates the sanction.
    pub group: DebtGroup,
    /// The criterion that distinguishes these items.
    pub criterion: String,
    /// Absolute IRI of the documented justification.
    pub anchor: String,
    /// Why this set is deliberately distinct.
    pub reason: String,
}

/// The one validated ledger shape. Native storage retains extension assertions.
#[derive(Clone, Debug)]
pub struct Ledger {
    graph: Graph,
    /// The census normalization and source-read limits.
    options: CensusOptions,
    /// Repository-relative scan roots; `.` when not stated.
    roots: Vec<String>,
    /// Reference packages to load for detector comparison.
    reference: Vec<String>,
    /// Jobs, ordered by id.
    jobs: Vec<LedgerJob>,
    /// Sanctioned exact member sets.
    distinct: Vec<LedgerDistinct>,
    patterns: Vec<LedgerPattern>,
    extensions: BTreeMap<String, BTreeSet<String>>,
}

impl Ledger {
    /// Borrow the uniquely owned extension with this normalized local root id.
    /// Absolute snapshot aliases and ids outside an extension return `None`.
    #[must_use]
    pub fn extension(&self, root: &str) -> Option<LedgerExtensionGraph<'_>> {
        let (root, members) = self.extensions.get_key_value(root)?;
        Some(LedgerExtensionGraph::new(&self.graph, root, members))
    }

    /// The validated declared patterns, in id order.
    #[must_use]
    pub fn patterns(&self) -> &[LedgerPattern] {
        &self.patterns
    }
    /// The census normalization and source-read limits.
    #[must_use]
    pub const fn options(&self) -> &CensusOptions {
        &self.options
    }

    /// Repository-relative scan roots, in lexical order.
    #[must_use]
    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    /// Reference packages, in lexical order.
    #[must_use]
    pub fn reference(&self) -> &[String] {
        &self.reference
    }

    /// Jobs, ordered by id.
    #[must_use]
    pub fn jobs(&self) -> &[LedgerJob] {
        &self.jobs
    }

    /// Exact member sets that the ledger deliberately sanctions.
    #[must_use]
    pub fn distinct(&self) -> &[LedgerDistinct] {
        &self.distinct
    }

    /// Read the closed canonical Turtle 1.2 ledger profile.
    ///
    /// # Errors
    /// Refuses oversized input, native RDF errors, a different normal form,
    /// unknown fields, invalid cardinalities, names or values.
    pub fn from_turtle(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::Turtle)
    }

    /// Read the equivalent closed canonical N-Triples 1.2 ledger profile.
    ///
    /// # Errors
    /// Refuses oversized input, native RDF errors, a different normal form,
    /// unknown fields, invalid cardinalities, names or values.
    pub fn from_ntriples(text: &str) -> Result<Self, Error> {
        Self::read(text, Format::NTriples)
    }

    /// The authoritative native RDF graph, including extension assertions.
    #[must_use]
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    fn read(text: &str, format: Format) -> Result<Self, Error> {
        let (graph, nf) = Self::parse(text, format)?;
        // Version refusal precedes inspection of jobs or extension content.
        check_normal_form(nf)?;
        Self::from_graph(graph)
    }

    pub(crate) fn import_index_ledger(text: &str) -> Result<(Self, u32), Error> {
        let (graph, old) = Self::parse(text, Format::Turtle)?;
        if old >= crate::NORMAL_FORM {
            return Err(E::Value(format!(
                "ledger normal form {old} is not older than {}; nothing to migrate",
                crate::NORMAL_FORM
            ))
            .into());
        }
        let mut updated = Builder::from_graph(&graph);
        if !updated
            .remove(Triple {
                subject: TermValue::iri(""),
                predicate: TermValue::iri(shape::property("normalForm")),
                object: shape::integer(u128::from(old)),
            })
            .map_err(E::Ground)?
        {
            return Err(E::Missing("ledger normalForm".into()).into());
        }
        super::write::put(
            &mut updated,
            "",
            "normalForm",
            shape::integer(u128::from(crate::NORMAL_FORM)),
        )?;
        Ok((Self::from_graph(updated.finish().map_err(E::Ground)?)?, old))
    }

    fn parse(text: &str, format: Format) -> Result<(Graph, u32), Error> {
        if text.len() > MAX_LEDGER_BYTES {
            return Err(E::Value(format!(
                "ledger is {} bytes; maximum is {MAX_LEDGER_BYTES}",
                text.len()
            ))
            .into());
        }
        let graph = Graph::from_text(text, format).map_err(E::Ground)?;
        let nf = shape::nf(&graph, "Ledger")?;
        Ok((graph, nf))
    }

    fn from_graph(graph: Graph) -> Result<Self, Error> {
        let root = graph.get("").expect("validated root");
        fields(
            &root,
            &[
                "normalForm",
                "reportForm",
                "minTokens",
                "maxFileBytes",
                "root",
                "reference",
                "job",
                "distinct",
                "pattern",
            ],
        )?;
        let mut options = CensusOptions::default();
        if let Some(value) = optional_count(&root, "minTokens")? {
            options = options.min_tokens(value)?;
        }
        if root
            .values(&shape::property("maxFileBytes"))
            .next()
            .is_some()
        {
            options.max_file_bytes = u64::try_from(shape::number(&root, "maxFileBytes")?)
                .map_err(|_| E::Value("ledger maxFileBytes exceeds u64".into()))?;
        }
        let mut roots = strings(&root, "root")?;
        if roots.is_empty() {
            roots.push(".".into());
        }
        let reference = strings(&root, "reference")?;
        let mut reached = BTreeSet::from([String::new()]);
        let mut jobs = Vec::new();
        let mut job_ids = BTreeSet::new();
        for id in targets(&root, "job")? {
            reached.insert(id.clone());
            let job = read_job(&graph, &id, &mut reached)?;
            if !job_ids.insert(job.job.id.clone()) {
                return Err(E::DuplicateJob(job.job.id).into());
            }
            jobs.push(job);
        }
        jobs.sort_by(|a, b| a.job.id.cmp(&b.job.id));
        let patterns = declarations::patterns(&graph, &root, &mut reached, &jobs)?;
        let mut distinct = Vec::new();
        let mut distinct_sets = BTreeSet::new();
        for id in targets(&root, "distinct")? {
            reached.insert(id.clone());
            let node = node_at(&graph, &id, "Distinct")?;
            fields(
                &node,
                &[
                    "memberSymbol",
                    "multiplicity",
                    "kind",
                    "criterion",
                    "anchor",
                    "reason",
                ],
            )?;
            let kind = match required_text(&node, "kind")?.as_str() {
                "body" => GroupKind::Body,
                "shim" => GroupKind::Shim,
                other => return Err(E::Value(format!("{id}: distinct kind {other:?}")).into()),
            };
            let group = read_distinct_members(&graph, &node, kind, &mut reached)?;
            if !distinct_sets.insert(group.clone()) {
                return Err(E::Repeated("ledger distinct member set".into()).into());
            }
            distinct.push(LedgerDistinct {
                group,
                criterion: required_text(&node, "criterion")?,
                anchor: anchor(&node)?,
                reason: required_text(&node, "reason")?,
            });
        }
        let extensions = extensions::collect(&graph, &jobs, &reached)?;
        Ok(Self {
            graph,
            options,
            roots,
            reference,
            jobs,
            distinct,
            patterns,
            extensions,
        })
    }
}

fn read_distinct_members(
    graph: &Graph,
    node: &Node<'_>,
    kind: GroupKind,
    reached: &mut BTreeSet<String>,
) -> Result<DebtGroup, Error> {
    let mut counts: BTreeMap<_, _> = strings(node, "memberSymbol")?
        .into_iter()
        .map(|symbol| (symbol, 1))
        .collect();
    for id in targets(node, "multiplicity")? {
        if !reached.insert(id.clone()) {
            return Err(E::Repeated(format!("distinct multiplicity {id}")).into());
        }
        let row = node_at(graph, &id, "MemberMultiplicity")?;
        fields(&row, &["symbol", "copies"])?;
        let symbol = required_text(&row, "symbol")?;
        let count = shape::count(&row, "copies")?;
        let member = counts
            .get_mut(&symbol)
            .ok_or_else(|| E::Value(format!("multiplicity names absent member {symbol:?}")))?;
        if *member != 1 || count < 2 {
            return Err(E::Value(format!(
                "distinct multiplicity {symbol:?} requires one count greater than one"
            ))
            .into());
        }
        *member = count;
    }
    DebtGroup::with_counts(kind, counts)
}

fn read_job(graph: &Graph, id: &str, reached: &mut BTreeSet<String>) -> Result<LedgerJob, Error> {
    let node = node_at(graph, id, "LedgerJob")?;
    fields(
        &node,
        &[
            "jobId",
            "summary",
            "homeSymbol",
            "entryPoint",
            "enforced",
            "forbiddenName",
            "forbiddenConstant",
            "forbiddenFingerprint",
            "forbiddenPattern",
            "variant",
            "extension",
            "preferredProvider",
        ],
    )?;
    let job_id = required_text(&node, "jobId")?;
    let names = read_names(&node, &job_id)?;
    let preferred_provider = declarations::provider(graph, &node, reached)?;
    let fingerprints = strings(&node, "forbiddenFingerprint")?
        .into_iter()
        .map(|text| {
            text.parse::<Fingerprint>()
                .map_err(|error| E::Value(format!("job {job_id:?}: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let constants = node
        .values(&shape::property("forbiddenConstant"))
        .map(|value| {
            shape::integer_value(value).ok_or_else(|| {
                E::Value(format!(
                    "job {job_id:?}: forbiddenConstant requires a canonical u128"
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut variants = Vec::new();
    for variant in targets(&node, "variant")? {
        if !reached.insert(variant.clone()) {
            return Err(E::Repeated(format!("ledger variant {variant}")).into());
        }
        let row = node_at(graph, &variant, "Variant")?;
        fields(
            &row,
            &[
                "symbol",
                "file",
                "detector",
                "criterion",
                "anchor",
                "reason",
            ],
        )?;
        variants.push(LedgerVariant {
            symbol: required_text(&row, "symbol")?,
            file: required_text(&row, "file")?,
            detector: required_text(&row, "detector")?,
            criterion: required_text(&row, "criterion")?,
            anchor: anchor(&row)?,
            reason: required_text(&row, "reason")?,
        });
    }
    let extension = targets(&node, "extension")?;
    if extension.len() > 1 {
        return Err(E::Repeated(format!("job {job_id:?}: extension")).into());
    }
    if let Some(id) = extension.first() {
        // Ownership is settled after every core subject and extension root is
        // known, so input/job order cannot conceal an ownership conflict.
        node_at(graph, id, "Extension")?;
    }
    let enforced = shape::boolean_value(shape::one(&node, "enforced")?).ok_or_else(|| {
        E::Value(format!(
            "job {job_id:?}: enforced requires a canonical boolean"
        ))
    })?;
    let job = Job::new(job_id, required_text(&node, "homeSymbol")?)
        .entry_points(strings(&node, "entryPoint")?)
        .variants(variants.iter().map(|variant| variant.symbol.clone()))
        .forbidding(JobForbidden {
            fingerprints,
            constants,
        });
    let summary = required_text(&node, "summary")?;
    if summary.contains(['\r', '\n']) {
        return Err(E::Value(format!("job {:?}: summary must be one line", job.id)).into());
    }
    Ok(LedgerJob {
        job,
        summary,
        enforced,
        names,
        patterns: strings(&node, "forbiddenPattern")?,
        variants,
        extension: extension.into_iter().next(),
        preferred_provider,
    })
}

fn read_names(node: &Node<'_>, job_id: &str) -> Result<Vec<NamePattern>, E> {
    strings(node, "forbiddenName")?
        .into_iter()
        .map(|source| {
            let compiled = ecma::compile(&source).map_err(|error| {
                E::Value(format!("job {job_id:?} forbiddenName {source:?}: {error}"))
            })?;
            Ok(NamePattern { source, compiled })
        })
        .collect()
}

fn node_at<'a>(graph: &'a Graph, id: &str, class: &str) -> Result<Node<'a>, E> {
    let node = graph.get(id).ok_or_else(|| E::Dangling(id.into()))?;
    if shape::class(&node)? != class {
        return Err(E::Class(format!("ledger {id}: expected {class}")));
    }
    Ok(node)
}

fn fields(node: &Node<'_>, allowed: &[&str]) -> Result<(), E> {
    for property in node.properties() {
        if property != raf_rdf::vocab::TYPE
            && !allowed.iter().any(|key| property == shape::property(key))
        {
            return Err(E::UnknownPredicate(format!("ledger {property}")));
        }
    }
    Ok(())
}

fn required_text(node: &Node<'_>, key: &str) -> Result<String, E> {
    let value = shape::string(node, key)?;
    if value.trim().is_empty() {
        return Err(E::Value(format!("ledger {key}: nonempty string required")));
    }
    Ok(value)
}

fn strings(node: &Node<'_>, key: &str) -> Result<Vec<String>, E> {
    let mut values = node
        .values(&shape::property(key))
        .map(|value| {
            let (text, datatype) = value
                .literal()
                .ok_or_else(|| E::Value(format!("ledger {key}: string required")))?;
            if datatype != raf_rdf::vocab::STRING || text.trim().is_empty() {
                return Err(E::Value(format!("ledger {key}: nonempty string required")));
            }
            Ok(text.to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    Ok(values)
}

fn targets(node: &Node<'_>, key: &str) -> Result<Vec<String>, E> {
    node.values(&shape::property(key))
        .map(|value| {
            value
                .iri()
                .filter(|iri| iri.starts_with('#'))
                .map(str::to_owned)
                .ok_or_else(|| E::Value(format!("ledger {key}: owned node required")))
        })
        .collect()
}

fn anchor(node: &Node<'_>) -> Result<String, E> {
    shape::one(node, "anchor")?
        .iri()
        .filter(|iri| !iri.is_empty() && !iri.starts_with('#'))
        .map(str::to_owned)
        .ok_or_else(|| E::Value("ledger anchor: absolute documentation IRI required".into()))
}

fn optional_count(node: &Node<'_>, key: &str) -> Result<Option<usize>, E> {
    if node.values(&shape::property(key)).next().is_none() {
        Ok(None)
    } else {
        shape::count(node, key).map(Some)
    }
}
