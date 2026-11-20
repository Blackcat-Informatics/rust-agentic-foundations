// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Reader admission for a retained Cap'n Proto persistence declaration.

use std::{collections::BTreeSet, error, fmt, path::Path};

use purrdf_core::ContentDigest;
use raf_rdf::Graph;
use raf_source::{RelPath, Tree};

use crate::{ReportError, report::shape};

use super::{Finding, Options, continuity, read_bound_policy, valid_id};
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
    if !valid_id(request.declaration) {
        return Err(Error::Selection(
            "select a canonical persistence declaration slug",
        ));
    }
    let repository = repository.as_ref();
    let path = canonical_path(request.options.policy, false).map_err(super::Error::PolicyPath)?;
    let base = open_tree(repository, request.base.commit)?;
    let current = open_tree(repository, request.current.commit)?;
    let select = Some(request.declaration);
    let base_policy = read_bound_policy(&base, request.base.commit, &path, select)?;
    let current_policy = read_bound_policy(&current, request.current.commit, &path, select)?;
    let before = base_policy
        .get(request.declaration)
        .ok_or(Error::Selection(
            "select a declaration present in the committed base policy",
        ))?;
    let after = current_policy.get(request.declaration);
    if let Some(finding) = continuity(request.declaration, before, after) {
        return Ok(Report {
            findings: vec![finding],
            verdict: Verdict::Uncovered,
        });
    }
    let Some(after) = after else {
        unreachable!("INVARIANT: declaration continuity requires a current declaration")
    };
    if let Some(claim) = request.claim {
        reader_evidence::check_evidence_inputs(
            request.base.commit,
            request.current.commit,
            claim.evidence,
        )?;
    }

    let strict = compare_layouts((&base, &current), &request, before.root)?;
    let verdict = if let Some(claim) = request.claim {
        let reader = after.reader.as_deref().ok_or(Error::Selection(
            "declare readerEntry on the current persistence declaration",
        ))?;
        if before.specimens.is_empty() {
            return Err(Error::Selection(
                "declare a nonempty specimenPath set on the base persistence declaration",
            ));
        }
        let expected = reader_evidence::Expected {
            base: request.base.commit,
            current: request.current.commit,
            root: before.root,
            entry_point: reader,
            artifact: claim.artifact,
            payloads: &before.specimens,
        };
        let graph = reader_evidence::authenticate(repository, claim.evidence)?;
        let reported = admission_payloads(&graph, &request, &expected, strict.digests)
            .map_err(reader_evidence::Error::Evidence)?;
        let selected: Vec<_> = expected.payloads.iter().collect();
        reader_evidence::match_specimens(&selected, &reported)?;
        let specimens = reader_evidence::verify_specimens(&base, reported)?;
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

struct Strict {
    findings: Vec<Finding>,
    digests: [ContentDigest; 2],
}

fn compare_layouts(
    trees: (&dyn Tree, &dyn Tree),
    request: &Request<'_>,
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
    request: &Request<'_>,
    expected: &reader_evidence::Expected<'_>,
    [base_digest, current_digest]: [ContentDigest; 2],
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
    reader_evidence::match_reader(&root, expected)?;
    let marker = request
        .options
        .required_annotation
        .map_or_else(|| "none".into(), |id| id.to_string());
    for (key, value) in [
        ("policyFile", request.options.policy),
        ("declaration", request.declaration),
        ("baseDescriptorFile", request.base.descriptor.as_str()),
        ("currentDescriptorFile", request.current.descriptor.as_str()),
        ("comparisonProfile", PROFILE),
        ("requiredAnnotation", &marker),
    ] {
        reader_evidence::require_text(&root, key, value)?;
    }
    let pair_key = "pairedPresenceAnnotation";
    if let Some(id) = request.options.paired_presence_annotation {
        reader_evidence::require_text(&root, pair_key, &id.to_string())?;
    } else if root.values(&shape::property(pair_key)).next().is_some() {
        return Err(CaptureError::Mismatch(
            "signed pairedPresenceAnnotation must be absent when the comparison omits it".into(),
        ));
    }
    for (key, expected) in [
        ("baseDescriptorSha256", base_digest),
        ("currentDescriptorSha256", current_digest),
    ] {
        if &digest(&root, key)? != expected.as_bytes() {
            return Err(CaptureError::Mismatch(format!(
                "signed {key} differs from the compared descriptor"
            )));
        }
    }
    for (key, expected) in [
        ("captureVerified", request.options.capture.is_some()),
        ("acceptsOldShape", true),
        ("acceptsSpecimens", true),
    ] {
        if shape::boolean_value(shape::one(&root, key)?) != Some(expected) {
            return Err(CaptureError::Mismatch(format!(
                "signed {key} must be {expected}"
            )));
        }
    }
    reader_evidence::payloads(graph, &root)
}
