// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reader admission for a retained Cap'n Proto persistence declaration.

use std::{collections::BTreeSet, error, fmt, path::Path};

use purrdf_core::ContentDigest;
use raf_rdf::Graph;
use raf_source::{PinnedGitTree, RelPath, Tree};

use crate::{ReportError, report::shape};

use super::{Declaration, Finding, Options, continuity, read_bound_policy, valid_id};
use crate::capnp_history::{
    CaptureError, Snapshot, TypeId,
    capture::{canonical_path, digest, fields},
    compare, decode_descriptor, descriptor, open_tree, reader_evidence,
};

const PROFILE: &str = "strict-schema-history-v1";

/// A caller-selected reader artifact and trusted source of admission evidence.
#[derive(Clone, Copy, Debug)]
pub struct Claim<'a> {
    /// Expected SHA-256 of the reader artifact.
    pub artifact: ContentDigest,
    /// Explicit evidence commit, detached signature, and caller-trusted key.
    pub evidence: reader_evidence::Evidence<'a>,
}

/// Both inspected revisions and one stable declaration in their policies.
#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    /// Old committed descriptor and policy revision.
    pub base: Snapshot<'a>,
    /// Current committed descriptor and policy revision.
    pub current: Snapshot<'a>,
    /// Shared committed policy path and optional strict comparison settings.
    pub options: Options<'a>,
    /// Stable declaration slug without its RDF fragment delimiter.
    pub declaration: &'a str,
    /// Evidence to verify, if selected.
    pub claim: Option<Claim<'a>>,
}

/// Admission disposition for the selected declaration.
#[derive(Debug)]
pub enum Verdict {
    /// The retained declaration has no strict findings.
    Clean,
    /// Strict findings remain without admitted reader coverage.
    Uncovered,
    /// A trusted signer attested to old-shape acceptance and the specimens agree.
    Admitted {
        /// Verified committed base specimen paths in repository order.
        specimens: Vec<RelPath>,
    },
}

/// The strict findings alongside the independent reader admission verdict.
#[derive(Debug)]
pub struct Report {
    /// Declaration continuity or strict findings for this declaration.
    pub findings: Vec<Finding>,
    /// Reader admission disposition.
    pub verdict: Verdict,
}

/// A refusal to establish a declaration or authenticate its reader coverage.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The selected declaration or its policy lacks required scope.
    Selection(&'static str),
    /// A pinned policy, Rust binding, or compiled shape was refused.
    Persistence(Box<super::Error>),
    /// A signature, signed statement or committed specimen was refused.
    Claim(Box<reader_evidence::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Selection(message) => f.write_str(message),
            Self::Persistence(source) => source.fmt(f),
            Self::Claim(source) => source.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Selection(_) => None,
            Self::Persistence(source) => Some(source.as_ref()),
            Self::Claim(source) => Some(source.as_ref()),
        }
    }
}

impl From<super::Error> for Error {
    fn from(source: super::Error) -> Self {
        Self::Persistence(Box::new(source))
    }
}

impl From<crate::capnp_history::Error> for Error {
    fn from(source: crate::capnp_history::Error) -> Self {
        super::Error::History(source).into()
    }
}

impl From<reader_evidence::Error> for Error {
    fn from(source: reader_evidence::Error) -> Self {
        Self::Claim(Box::new(source))
    }
}

/// Compare the selected retained declaration and verify its reader admission.
///
/// The signer attests to application acceptance of the old shape. Census
/// corroborates the pinned scope and exact specimens without executing a
/// reader. A removed or rebound declaration remains an uncovered finding.
///
/// # Errors
/// Refuses invalid selections, pinned inputs, evidence or specimens.
pub fn check(repository: impl AsRef<Path>, request: Request<'_>) -> Result<Report, Error> {
    let repository = repository.as_ref();
    let scope = Scope {
        base: request.base,
        current: request.current,
        options: request.options,
        declaration: request.declaration,
    };
    let retained = match scope.select(repository)? {
        Selection::Retained(retained) => retained,
        Selection::Discontinuous(finding) => {
            return Ok(Report {
                findings: vec![finding],
                verdict: Verdict::Uncovered,
            });
        }
    };
    if let Some(claim) = request.claim {
        reader_evidence::check_evidence_inputs(
            request.base.commit,
            request.current.commit,
            claim.evidence,
        )?;
    }

    let strict = compare_layouts(
        (&retained.base, &retained.current),
        &scope,
        retained.before.root,
    )?;
    let verdict = if let Some(claim) = request.claim {
        let reader = retained.after.reader.as_deref().ok_or(Error::Selection(
            "declare readerEntry on the current persistence declaration",
        ))?;
        if retained.before.specimens.is_empty() {
            return Err(Error::Selection(
                "declare a nonempty specimenPath set on the base persistence declaration",
            ));
        }
        let expected = reader_evidence::Expected {
            base: request.base.commit,
            current: request.current.commit,
            root: retained.before.root,
            entry_point: reader,
            artifact: claim.artifact,
            payloads: &retained.before.specimens,
        };
        let graph = reader_evidence::authenticate(repository, claim.evidence)?;
        let reported = admission_payloads(&graph, &scope, &expected, strict.digests)
            .map_err(reader_evidence::Error::Evidence)?;
        let selected: Vec<_> = expected.payloads.iter().collect();
        reader_evidence::match_specimens(&selected, &reported)?;
        let specimens = reader_evidence::verify_specimens(&retained.base, reported)?;
        Verdict::Admitted { specimens }
    } else if strict.findings.is_empty() {
        Verdict::Clean
    } else {
        Verdict::Uncovered
    };
    Ok(Report {
        findings: strict.findings,
        verdict,
    })
}

pub(super) struct Scope<'a> {
    pub(super) base: Snapshot<'a>,
    pub(super) current: Snapshot<'a>,
    pub(super) options: Options<'a>,
    pub(super) declaration: &'a str,
}

pub(super) enum Selection {
    Retained(Box<Retained>),
    Discontinuous(Finding),
}

pub(super) struct Retained {
    pub(super) base: PinnedGitTree<'static>,
    pub(super) current: PinnedGitTree<'static>,
    pub(super) before: Declaration,
    pub(super) after: Declaration,
}

impl Scope<'_> {
    pub(super) fn select(&self, repository: &Path) -> Result<Selection, Error> {
        if !valid_id(self.declaration) {
            return Err(Error::Selection(
                "select a canonical persistence declaration slug",
            ));
        }
        let path = canonical_path(self.options.policy, false).map_err(super::Error::PolicyPath)?;
        let base = open_tree(repository, self.base.commit)?;
        let current = open_tree(repository, self.current.commit)?;
        let selected = Some(self.declaration);
        let mut base_policy = read_bound_policy(&base, self.base.commit, &path, selected)?;
        let mut current_policy = read_bound_policy(&current, self.current.commit, &path, selected)?;
        let before = base_policy
            .remove(self.declaration)
            .ok_or(Error::Selection(
                "select a declaration present in the committed base policy",
            ))?;
        let after = current_policy.remove(self.declaration);
        if let Some(finding) = continuity(self.declaration, &before, after.as_ref()) {
            return Ok(Selection::Discontinuous(finding));
        }
        let Some(after) = after else {
            unreachable!("INVARIANT: declaration continuity requires a current declaration")
        };
        Ok(Selection::Retained(Box::new(Retained {
            base,
            current,
            before,
            after,
        })))
    }
}

pub(super) struct Strict {
    pub(super) findings: Vec<Finding>,
    pub(super) digests: [ContentDigest; 2],
}

pub(super) fn compare_layouts(
    trees: (&dyn Tree, &dyn Tree),
    request: &Scope<'_>,
    root: TypeId,
) -> Result<Strict, crate::capnp_history::Error> {
    let roots = BTreeSet::from([root]);
    let mode = descriptor::Roots::Declared(&roots);
    let capture = request.options.capture;
    let (base, base_digest) = decode_descriptor(
        trees.0,
        request.base,
        &roots,
        request.options.annotations(),
        mode,
        capture.map(|capture| (capture.key, capture.base)),
        ContentDigest::of,
    )?;
    let (current, current_digest) = decode_descriptor(
        trees.1,
        request.current,
        &roots,
        request.options.annotations(),
        mode,
        capture.map(|capture| (capture.key, capture.current)),
        ContentDigest::of,
    )?;
    let findings = compare::changes(&base, &current)?
        .into_iter()
        .map(Finding::Layout)
        .collect();
    Ok(Strict {
        findings,
        digests: [base_digest, current_digest],
    })
}

fn admission_payloads(
    graph: &Graph,
    request: &Scope<'_>,
    expected: &reader_evidence::Expected<'_>,
    digests: [ContentDigest; 2],
) -> Result<std::collections::BTreeMap<RelPath, [u8; 32]>, CaptureError> {
    let root = graph
        .get("")
        .ok_or_else(|| ReportError::Missing("reader admission root".into()))?;
    fields(
        &root,
        "CapnpReaderAdmissionV1",
        &[
            "policyFile",
            "declaration",
            "baseCommit",
            "currentCommit",
            "baseDescriptorFile",
            "baseDescriptorSha256",
            "currentDescriptorFile",
            "currentDescriptorSha256",
            "rootType",
            "comparisonProfile",
            "requiredAnnotation",
            "pairedPresenceAnnotation",
            "captureVerified",
            "readerEntry",
            "readerSha256",
            "acceptsOldShape",
            "acceptsSpecimens",
            "payload",
        ],
    )?;
    match_scope(&root, request, expected.root, digests)?;
    reader_evidence::match_artifact(
        &root,
        ("readerEntry", expected.entry_point),
        ("readerSha256", expected.artifact),
        "reader",
    )?;
    for key in ["acceptsOldShape", "acceptsSpecimens"] {
        require_claim(&root, key, true)?;
    }
    reader_evidence::payloads(graph, &root)
}

pub(super) fn match_scope(
    root: &raf_rdf::Node<'_>,
    request: &Scope<'_>,
    root_type: TypeId,
    [base_digest, current_digest]: [ContentDigest; 2],
) -> Result<(), CaptureError> {
    let marker = request
        .options
        .required_annotation
        .map_or_else(|| "none".into(), |id| id.to_string());
    for (key, value) in [
        ("policyFile", request.options.policy),
        ("declaration", request.declaration),
        ("baseCommit", request.base.commit),
        ("currentCommit", request.current.commit),
        ("rootType", &root_type.to_string()),
        ("baseDescriptorFile", request.base.descriptor.as_str()),
        ("currentDescriptorFile", request.current.descriptor.as_str()),
        ("comparisonProfile", PROFILE),
        ("requiredAnnotation", &marker),
    ] {
        reader_evidence::require_text(root, key, value)?;
    }
    let pair_key = "pairedPresenceAnnotation";
    if let Some(id) = request.options.paired_presence_annotation {
        reader_evidence::require_text(root, pair_key, &id.to_string())?;
    } else if root.values(&shape::property(pair_key)).next().is_some() {
        return Err(CaptureError::Mismatch(
            "signed pairedPresenceAnnotation must be absent when the comparison omits it".into(),
        ));
    }
    for (key, expected) in [
        ("baseDescriptorSha256", base_digest),
        ("currentDescriptorSha256", current_digest),
    ] {
        if &digest(root, key)? != expected.as_bytes() {
            return Err(CaptureError::Mismatch(format!(
                "signed {key} differs from the compared descriptor"
            )));
        }
    }
    require_claim(root, "captureVerified", request.options.capture.is_some())
}

pub(super) fn require_claim(
    root: &raf_rdf::Node<'_>,
    key: &str,
    expected: bool,
) -> Result<(), CaptureError> {
    if shape::boolean_value(shape::one(root, key)?) != Some(expected) {
        return Err(CaptureError::Mismatch(format!(
            "signed {key} must be {expected}"
        )));
    }
    Ok(())
}
