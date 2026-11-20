// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Committed declarations selecting strict history for persisted Rust types.

pub mod admission;

use std::{
    collections::{BTreeMap, BTreeSet},
    error, fmt,
    path::Path,
};

use raf_rdf::{Format, Graph};
use raf_source::{
    LoadError, LoadOptions, Namespace, RelPath, SourceAuthorityId, SymbolKind, TargetKind, Tree,
    Workspace,
};

use crate::{ReportError, report::shape};

use super::{
    CaptureError, CaptureVerification, Change, Snapshot, TypeId,
    capture::{canonical_path, fields, owned_node},
    compare, descriptor, load_descriptor, open_tree, reader_evidence,
};

const MAX_POLICY_BYTES: u64 = 128 * 1024;
const MAX_DECLARATIONS: usize = 96;
const MAX_ID_BYTES: usize = 64;
const MAX_SOURCE_NAME_BYTES: usize = 1024;

/// Committed policy and optional compiler checks for both revisions.
#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    /// The same canonical repository-relative policy path in both commits.
    pub policy: &'a str,
    /// Void field annotation identifying required pointer fields, if selected.
    pub required_annotation: Option<TypeId>,
    /// Authenticate and corroborate both compiled inputs, if selected.
    pub capture: Option<CaptureVerification<'a>>,
}

/// A violation of declaration continuity or strict compiled history.
#[derive(Debug)]
#[non_exhaustive]
pub enum Finding {
    /// A stable declaration in the base policy disappeared.
    Removed {
        /// Stable declaration slug.
        declaration: String,
        /// Previously protected compiled root.
        root: TypeId,
    },
    /// A stable declaration now selects a different compiled root.
    Rebound {
        /// Stable declaration slug.
        declaration: String,
        /// Previously protected compiled root.
        base: TypeId,
        /// Replacement compiled root.
        current: TypeId,
    },
    /// A strict layout change in the retained roots' reachable schema.
    Layout(Change),
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Removed { declaration, root } => write!(
                f,
                "Cap'n Proto persistence declaration {declaration}: removed protection of {root}"
            ),
            Self::Rebound {
                declaration,
                base,
                current,
            } => write!(
                f,
                "Cap'n Proto persistence declaration {declaration}: root changed from {base} to {current}"
            ),
            Self::Layout(change) => change.fmt(f),
        }
    }
}

/// A refusal to establish the committed persistence comparison.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An invalid policy path was selected.
    PolicyPath(CaptureError),
    /// A committed policy could not be read or admitted.
    Policy {
        /// Revision holding the policy.
        commit: String,
        /// Selected committed policy path.
        path: RelPath,
        /// Read or profile failure.
        source: CaptureError,
    },
    /// A pinned Cargo workspace could not be indexed.
    Workspace {
        /// Revision holding the workspace.
        commit: String,
        /// Native source failure.
        source: LoadError,
    },
    /// An exact Rust declaration could not be established.
    Binding {
        /// Revision holding the declaration.
        commit: String,
        /// Stable declaration slug.
        declaration: String,
        /// Declared Cargo package.
        package: String,
        /// Declared canonical Rust symbol.
        symbol: String,
    },
    /// A pinned revision, descriptor, or capture could not support a verdict.
    History(super::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyPath(source) => write!(f, "persistence policy path: {source}"),
            Self::Policy {
                commit,
                path,
                source,
            } => write!(f, "persistence policy {path} at {commit}: {source}"),
            Self::Workspace { commit, source } => {
                write!(f, "persistence Cargo workspace at {commit}: {source}")
            }
            Self::Binding {
                commit,
                declaration,
                package,
                symbol,
            } => write!(
                f,
                "persistence declaration {declaration} at {commit}: {package}/{symbol} must name one direct first-party library or binary struct or enum"
            ),
            Self::History(source) => source.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::PolicyPath(source) | Self::Policy { source, .. } => Some(source),
            Self::Workspace { source, .. } => Some(source),
            Self::Binding { .. } => None,
            Self::History(source) => Some(source),
        }
    }
}

impl From<super::Error> for Error {
    fn from(source: super::Error) -> Self {
        Self::History(source)
    }
}

struct Declaration {
    package: String,
    symbol: String,
    root: TypeId,
    reader: Option<String>,
    specimens: Vec<RelPath>,
}

type Policy = BTreeMap<String, Declaration>;

/// Compare declarations and the strict layouts of retained compiled roots.
///
/// Both policies and Rust workspace bindings are read from explicit Git
/// commits. The written policies associate indexed type declarations with
/// compiled schema identities. This operation establishes no reader or
/// migration behavior.
///
/// # Errors
/// Refuses unadmitted revisions, invalid policies or source bindings, and
/// missing, malformed, unsupported, or oversized descriptor and capture inputs.
pub fn check(
    repository: impl AsRef<Path>,
    base: Snapshot<'_>,
    current: Snapshot<'_>,
    options: Options<'_>,
) -> Result<Vec<Finding>, Error> {
    let policy_path = canonical_path(options.policy, false).map_err(Error::PolicyPath)?;
    let repository = repository.as_ref();
    let base_tree = open_tree(repository, base.commit)?;
    let base_policy = read_bound_policy(&base_tree, base.commit, &policy_path, None)?;
    let current_tree = open_tree(repository, current.commit)?;
    let current_policy = read_bound_policy(&current_tree, current.commit, &policy_path, None)?;
    let (retained, mut findings) = compare_policies(&base_policy, &current_policy);
    let base_schema = load_policy_descriptor(
        &base_tree,
        base,
        &base_policy,
        &retained,
        options.required_annotation,
        options.capture.map(|capture| (capture.key, capture.base)),
    )?;
    let current_schema = load_policy_descriptor(
        &current_tree,
        current,
        &current_policy,
        &retained,
        options.required_annotation,
        options
            .capture
            .map(|capture| (capture.key, capture.current)),
    )?;
    findings.extend(
        compare::changes(&base_schema, &current_schema)?
            .into_iter()
            .map(Finding::Layout),
    );
    Ok(findings)
}

fn load_policy_descriptor(
    tree: &dyn Tree,
    snapshot: Snapshot<'_>,
    policy: &Policy,
    retained: &BTreeSet<TypeId>,
    required_annotation: Option<TypeId>,
    capture: Option<(&super::CaptureKey, super::CaptureFiles<'_>)>,
) -> Result<descriptor::Schema, super::Error> {
    let declared = policy.values().map(|row| row.root).collect();
    load_descriptor(
        tree,
        snapshot,
        retained,
        required_annotation,
        descriptor::Roots::Declared(&declared),
        capture,
    )
}

fn read_bound_policy(
    tree: &dyn Tree,
    commit: &str,
    path: &RelPath,
    selected: Option<&str>,
) -> Result<Policy, Error> {
    let policy = read_policy(tree, path).map_err(|source| Error::Policy {
        commit: commit.into(),
        path: path.clone(),
        source,
    })?;
    let rows = policy
        .iter()
        .filter(|(id, _)| selected.is_none_or(|selection| selection == id.as_str()));
    if rows.clone().next().is_none() {
        return Ok(policy);
    }
    let workspace = Workspace::load_members(tree, &RelPath::root(), &LoadOptions::default())
        .map_err(|source| Error::Workspace {
            commit: commit.into(),
            source,
        })?;
    for (id, row) in rows {
        if !is_direct_binding(&workspace, row) {
            return Err(Error::Binding {
                commit: commit.into(),
                declaration: id.clone(),
                package: row.package.clone(),
                symbol: row.symbol.clone(),
            });
        }
    }
    Ok(policy)
}

fn is_direct_binding(workspace: &Workspace, row: &Declaration) -> bool {
    let mut packages = workspace
        .packages()
        .iter()
        .filter(|package| package.name == row.package);
    if packages.next().is_none() || packages.next().is_some() {
        return false;
    }
    let mut sites = workspace
        .items_under(&row.symbol)
        .take_while(|item| item.path.as_str() == row.symbol)
        .filter(|item| {
            item.namespace == Namespace::Type
                && item.file.authority() == SourceAuthorityId::FIRST_PARTY
                && workspace.is_shipping(item)
                && workspace.target(item.target).is_some_and(|target| {
                    target.package == row.package
                        && matches!(target.kind, TargetKind::Lib | TargetKind::Bin)
                })
        });
    let Some(item) = sites.next() else {
        return false;
    };
    sites.next().is_none() && matches!(item.kind, SymbolKind::Struct | SymbolKind::Enum)
}

fn read_policy(tree: &dyn Tree, path: &RelPath) -> Result<Policy, CaptureError> {
    let bytes = tree
        .read_bytes(path, MAX_POLICY_BYTES)
        .map_err(CaptureError::Read)?;
    let text = str::from_utf8(&bytes).map_err(CaptureError::Encoding)?;
    let graph = Graph::from_text(text, Format::NTriples).map_err(ReportError::Ground)?;
    let root = graph
        .get("")
        .ok_or_else(|| ReportError::Missing("persistence policy root".into()))?;
    fields(&root, "CapnpPersistencePolicyV1", &["persistedType"])?;
    let mut policy = Policy::new();
    for reference in root.values(&shape::property("persistedType")) {
        if policy.len() == MAX_DECLARATIONS {
            return Err(ReportError::Value(format!(
                "persistence policy admits at most {MAX_DECLARATIONS} declarations"
            ))
            .into());
        }
        let row = owned_node(&graph, reference, "persistedType")?;
        fields(
            &row,
            "CapnpPersistedType",
            &[
                "package",
                "symbol",
                "rootType",
                "readerEntry",
                "specimenPath",
            ],
        )?;
        let id = reference
            .iri()
            .and_then(|iri| iri.strip_prefix('#'))
            .filter(|id| valid_id(id))
            .ok_or_else(|| {
                ReportError::Value(
                    "persistence declaration must have a lowercase slug of at most 64 bytes".into(),
                )
            })?;
        let root_id = shape::string(&row, "rootType")?.parse().map_err(|error| {
            ReportError::Value(format!("persistence declaration {id} rootType: {error}"))
        })?;
        policy.insert(
            id.to_owned(),
            Declaration {
                package: source_name(&row, "package")?,
                symbol: source_name(&row, "symbol")?,
                root: root_id,
                reader: optional_reader(&row)?,
                specimens: specimen_paths(&row)?,
            },
        );
    }
    if graph.nodes().count() != 1 + policy.len() {
        return Err(ReportError::Value(
            "persistence policy may contain only its referenced declarations".into(),
        )
        .into());
    }
    Ok(policy)
}

fn optional_reader(row: &raf_rdf::Node<'_>) -> Result<Option<String>, ReportError> {
    if row.values(&shape::property("readerEntry")).next().is_some() {
        source_name(row, "readerEntry").map(Some)
    } else {
        Ok(None)
    }
}

fn specimen_paths(row: &raf_rdf::Node<'_>) -> Result<Vec<RelPath>, CaptureError> {
    let mut paths = BTreeSet::new();
    for value in row.values(&shape::property("specimenPath")) {
        if paths.len() == reader_evidence::MAX_PAYLOADS {
            return Err(ReportError::Value(
                "persistence declaration admits at most 128 specimen paths".into(),
            )
            .into());
        }
        let Some((path, raf_rdf::vocab::STRING)) = value.literal() else {
            return Err(
                ReportError::Value("persistence specimenPath must be a string".into()).into(),
            );
        };
        paths.insert(canonical_path(path, false)?);
    }
    Ok(paths.into_iter().collect())
}

fn valid_id(id: &str) -> bool {
    id.len() <= MAX_ID_BYTES
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn source_name(row: &raf_rdf::Node<'_>, key: &str) -> Result<String, ReportError> {
    let name = shape::string(row, key)?;
    if name.is_empty() || name.len() > MAX_SOURCE_NAME_BYTES || name.chars().any(char::is_control) {
        return Err(ReportError::Value(format!(
            "persistence {key} must be nonempty, at most 1024 bytes, and contain no control characters"
        )));
    }
    Ok(name)
}

fn compare_policies(base: &Policy, current: &Policy) -> (BTreeSet<TypeId>, Vec<Finding>) {
    let mut retained = BTreeSet::new();
    let mut findings = Vec::new();
    for (id, before) in base {
        if let Some(finding) = continuity(id, before, current.get(id)) {
            findings.push(finding);
        } else {
            retained.insert(before.root);
        }
    }
    (retained, findings)
}

fn continuity(id: &str, before: &Declaration, after: Option<&Declaration>) -> Option<Finding> {
    match after {
        None => Some(Finding::Removed {
            declaration: id.into(),
            root: before.root,
        }),
        Some(after) if before.root != after.root => Some(Finding::Rebound {
            declaration: id.into(),
            base: before.root,
            current: after.root,
        }),
        Some(_) => None,
    }
}
