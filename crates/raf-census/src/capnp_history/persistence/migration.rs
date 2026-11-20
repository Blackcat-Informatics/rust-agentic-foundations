// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Signed migration admission for a retained Cap'n Proto declaration.

use std::{collections::BTreeMap, error, fmt, path::Path};

use purrdf_core::ContentDigest;
use raf_rdf::Graph;
use raf_source::{RelPath, Tree};

use crate::{ReportError, report::shape};

use super::{
    Finding, Options,
    admission::{self, Scope, Selection, compare_layouts, match_scope, require_claim},
};
use crate::capnp_history::{
    CaptureError, Snapshot, TypeId,
    capture::{digest, fields, owned_node, signed_graph},
    reader_evidence::{self, Evidence, MAX_PAYLOADS, PayloadError},
};

/// The externally selected migration artifact and trust inputs.
#[derive(Clone, Copy, Debug)]
pub struct Claim<'a> {
    /// Expected SHA-256 of the migration artifact.
    pub artifact: ContentDigest,
    /// Explicit evidence commit, detached signature, and caller-trusted key.
    pub evidence: Evidence<'a>,
}

/// A selected declaration and both of its committed schema revisions.
#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    /// Old committed descriptor and policy revision.
    pub base: Snapshot<'a>,
    /// Current committed descriptor and policy revision.
    pub current: Snapshot<'a>,
    /// Committed policy path and strict comparison settings.
    pub options: Options<'a>,
    /// Stable declaration slug without its RDF fragment delimiter.
    pub declaration: &'a str,
    /// Evidence to verify, if selected.
    pub claim: Option<Claim<'a>>,
}

/// Exact paths of a verified pair.
#[derive(Debug)]
pub struct SpecimenPair {
    /// Input blob in the base commit.
    pub input: RelPath,
    /// Result blob in the evidence commit.
    pub output: RelPath,
}

/// Disposition of the selected declaration.
#[derive(Debug)]
pub enum Verdict {
    /// The retained declaration has no strict findings.
    Clean,
    /// A strict finding remains without admitted migration coverage.
    Uncovered,
    /// A trusted signer's migration claim agrees with the committed pairs.
    Admitted {
        /// Verified pairs sorted by input path.
        pairs: Vec<SpecimenPair>,
    },
}

/// Strict findings alongside the migration admission disposition.
#[derive(Debug)]
pub struct Report {
    /// Declaration continuity or strict layout findings.
    pub findings: Vec<Finding>,
    /// Migration admission disposition.
    pub verdict: Verdict,
}

/// Which revision supplies a migration specimen.
#[derive(Clone, Copy, Debug)]
pub enum SpecimenRole {
    /// Old record in the base revision.
    Input,
    /// Migrated record in the evidence revision.
    Output,
}

/// A refusal to establish migration admission.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Required committed policy data is absent.
    Selection(&'static str),
    /// The declaration or strict comparison was refused.
    Scope(Box<admission::Error>),
    /// Evidence selection or the pinned evidence revision was refused.
    Evidence(Box<reader_evidence::Error>),
    /// Authentication, statement profile or concordance failed.
    Statement(Box<CaptureError>),
    /// Committed specimen verification failed.
    Specimen {
        /// Role and revision of the refused specimen.
        role: SpecimenRole,
        /// Exact path of the refused specimen.
        path: RelPath,
        /// Reason the committed blob was refused.
        source: PayloadError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Selection(message) => f.write_str(message),
            Self::Scope(source) => source.fmt(f),
            Self::Evidence(source) => write!(f, "migration evidence selection: {source}"),
            Self::Statement(source) => write!(f, "migration statement: {source}"),
            Self::Specimen { role, path, source } => {
                let role = match role {
                    SpecimenRole::Input => "base input",
                    SpecimenRole::Output => "evidence output",
                };
                write!(f, "{role} {path}: {source}")
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Selection(_) => None,
            Self::Scope(source) => Some(source.as_ref()),
            Self::Evidence(source) => Some(source.as_ref()),
            Self::Statement(source) => Some(source.as_ref()),
            Self::Specimen { source, .. } => Some(source),
        }
    }
}

/// Compare one retained declaration and verify a selected migration claim.
///
/// The trusted signer attests that the migrator accepts the old shape, produced
/// the listed results, and satisfies the destination application contract.
/// Census corroborates the pinned scope and committed pairs.
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
    let retained = match scope.select(repository).map_err(scope_error)? {
        Selection::Retained(retained) => retained,
        Selection::Discontinuous(finding) => {
            return Ok(Report {
                findings: vec![finding],
                verdict: Verdict::Uncovered,
            });
        }
    };
    let root = retained.before.root;
    let strict = compare_layouts((&retained.base, &retained.current), &scope, root)
        .map_err(|source| scope_error(source.into()))?;
    let verdict = if let Some(claim) = request.claim {
        let entry = retained.after.migration.as_deref().ok_or(Error::Selection(
            "declare migrationEntry on the current persistence declaration",
        ))?;
        let inputs = &retained.before.specimens;
        if inputs.is_empty() {
            return Err(Error::Selection(
                "declare a nonempty specimenPath set on the base persistence declaration",
            ));
        }
        reader_evidence::check_evidence_selection(
            request.base.commit,
            request.current.commit,
            claim.evidence,
            "select a separate commit for the migration evidence",
        )
        .map_err(evidence_error)?;
        let evidence = reader_evidence::pinned(repository, claim.evidence.commit, "evidence")
            .map_err(evidence_error)?;
        let graph = signed_graph(&evidence, claim.evidence.files, claim.evidence.key)
            .map_err(statement_error)?;
        let pairs = statement_pairs(
            &graph,
            &scope,
            (root, strict.digests),
            (entry, claim.artifact),
        )
        .map_err(statement_error)?;
        let pairs = verify_pairs((&retained.base, &evidence), inputs, pairs)?;
        Verdict::Admitted { pairs }
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

fn scope_error(source: admission::Error) -> Error {
    Error::Scope(Box::new(source))
}

fn evidence_error(source: reader_evidence::Error) -> Error {
    Error::Evidence(Box::new(source))
}

fn statement_error(source: CaptureError) -> Error {
    Error::Statement(Box::new(source))
}

struct Digests {
    input: [u8; 32],
    output: [u8; 32],
}

type SignedPairs = BTreeMap<RelPath, (RelPath, Digests)>;

fn statement_pairs(
    graph: &Graph,
    scope: &Scope<'_>,
    (root_type, digests): (TypeId, [ContentDigest; 2]),
    (entry, artifact): (&str, ContentDigest),
) -> Result<SignedPairs, CaptureError> {
    let root = graph
        .get("")
        .ok_or_else(|| ReportError::Missing("migration admission root".into()))?;
    fields(
        &root,
        "CapnpMigrationAdmissionV1",
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
            "migrationEntry",
            "migrationSha256",
            "acceptsOldShape",
            "producedOutputs",
            "outputsValid",
            "specimenPair",
        ],
    )?;
    match_scope(&root, scope, root_type, digests)?;
    reader_evidence::match_artifact(
        &root,
        ("migrationEntry", entry),
        ("migrationSha256", artifact),
        "migration",
    )?;
    for key in ["acceptsOldShape", "producedOutputs", "outputsValid"] {
        require_claim(&root, key, true)?;
    }
    let mut pairs = BTreeMap::new();
    for reference in root.values(&shape::property("specimenPair")) {
        if pairs.len() == MAX_PAYLOADS {
            return Err(CaptureError::Mismatch(format!(
                "migration evidence admits at most {MAX_PAYLOADS} specimen pairs"
            )));
        }
        let row = owned_node(graph, reference, "specimenPair")?;
        fields(
            &row,
            "CapnpMigrationSpecimenPair",
            &["inputFile", "inputSha256", "outputFile", "outputSha256"],
        )?;
        let input = reader_evidence::path(&shape::string(&row, "inputFile")?)?;
        let output = reader_evidence::path(&shape::string(&row, "outputFile")?)?;
        let digests = Digests {
            input: digest(&row, "inputSha256")?,
            output: digest(&row, "outputSha256")?,
        };
        if pairs.insert(input.clone(), (output, digests)).is_some() {
            return Err(CaptureError::Mismatch(format!(
                "migration evidence repeats input path {input}"
            )));
        }
    }
    if graph.nodes().count() != 1 + pairs.len() {
        return Err(CaptureError::Mismatch(
            "migration evidence may contain only its referenced subjects".into(),
        ));
    }
    Ok(pairs)
}

fn verify_pairs(
    (base, evidence): (&dyn Tree, &dyn Tree),
    inputs: &[RelPath],
    pairs: SignedPairs,
) -> Result<Vec<SpecimenPair>, Error> {
    if inputs.iter().ne(pairs.keys()) {
        return Err(statement_error(CaptureError::Mismatch(
            "signed input paths differ from the exact base specimen set".into(),
        )));
    }
    let mut outputs = BTreeMap::new();
    for (output, digests) in pairs.values() {
        if let Some(previous) = outputs.insert(output, &digests.output)
            && previous != &digests.output
        {
            return Err(statement_error(CaptureError::Mismatch(format!(
                "migration evidence gives conflicting digests for output {output}"
            ))));
        }
    }
    for (input, (_, digests)) in &pairs {
        verify_specimen(base, input, &digests.input, SpecimenRole::Input)?;
    }
    for (output, digest) in outputs {
        verify_specimen(evidence, output, digest, SpecimenRole::Output)?;
    }
    Ok(pairs
        .into_iter()
        .map(|(input, (output, _))| SpecimenPair { input, output })
        .collect())
}

fn verify_specimen(
    tree: &dyn Tree,
    path: &RelPath,
    digest: &[u8; 32],
    role: SpecimenRole,
) -> Result<(), Error> {
    reader_evidence::verify_payload(tree, path, digest).map_err(|source| Error::Specimen {
        role,
        path: path.clone(),
        source,
    })
}
