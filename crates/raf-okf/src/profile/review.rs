// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Code review claims and findings, independent of their storage or renderer.

use super::typed::{Actor, Input, RecordHeader};
use super::{Check, Rule, is_capability_id, is_kebab, records, scalar_text, strings};
use crate::frontmatter::value::{Entry, Map};

/// The question a code review answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewKind {
    /// Security boundaries and trust.
    Security,
    /// Behavior and invariants.
    Correctness,
    /// Architecture and interfaces.
    Design,
    /// Resource use and scalability.
    Performance,
    /// Code documentation and its accuracy.
    Documentation,
}

impl ReviewKind {
    /// Every admitted value in deterministic presentation order.
    pub const ALL: [Self; 5] = [
        Self::Security,
        Self::Correctness,
        Self::Design,
        Self::Performance,
        Self::Documentation,
    ];

    /// The closed profile spelling.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Correctness => "correctness",
            Self::Design => "design",
            Self::Performance => "performance",
            Self::Documentation => "documentation",
        }
    }

    /// Read one closed profile spelling.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "security" => Some(Self::Security),
            "correctness" => Some(Self::Correctness),
            "design" => Some(Self::Design),
            "performance" => Some(Self::Performance),
            "documentation" => Some(Self::Documentation),
            _ => None,
        }
    }
}

/// The conclusion of a code review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewVerdict {
    /// No findings were identified.
    Clean,
    /// Findings are recorded.
    Findings,
    /// The review could not reach a conclusion.
    Blocked,
}

impl ReviewVerdict {
    /// Every admitted value in deterministic presentation order.
    pub const ALL: [Self; 3] = [Self::Clean, Self::Findings, Self::Blocked];

    /// The closed profile spelling.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Blocked => "blocked",
        }
    }

    /// Read one closed profile spelling.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "clean" => Some(Self::Clean),
            "findings" => Some(Self::Findings),
            "blocked" => Some(Self::Blocked),
            _ => None,
        }
    }
}

/// The severity assigned to a structured finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewSeverity {
    /// A critical defect.
    Critical,
    /// A high severity defect.
    High,
    /// A medium severity defect.
    Medium,
    /// A low severity defect.
    Low,
    /// An observation without a defect claim.
    Informational,
}

impl ReviewSeverity {
    /// Every admitted value in deterministic presentation order.
    pub const ALL: [Self; 5] = [
        Self::Critical,
        Self::High,
        Self::Medium,
        Self::Low,
        Self::Informational,
    ];

    /// The closed profile spelling.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Informational => "informational",
        }
    }

    /// Read one closed profile spelling.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "informational" => Some(Self::Informational),
            _ => None,
        }
    }
}

/// The source set a review resolves at its reviewed revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewScope {
    /// All indexed definitions in a Cargo package.
    Crate(String),
    /// The named module and its descendants.
    Module(String),
    /// The explicitly named definitions, without inferred call expansion.
    Items(Vec<String>),
    /// The code bindings of the named capability at the reviewed revision.
    Capability(String),
}

/// A finding's disposition. Reasons and fixing revisions are part of the claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewDisposition {
    /// The finding remains unresolved.
    Open,
    /// The fixing commit, validated when the disposition is recorded.
    Fixed(String),
    /// Accepted debt with its explicit rationale.
    Accepted(String),
    /// A rejected defect claim with its explicit rationale.
    FalsePositive(String),
}

/// A finding's location at the reviewed revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewLocation {
    /// An indexed item path.
    Item(String),
    /// A repository-relative file and inclusive, positive line range.
    Lines {
        /// The repository-relative file.
        file: String,
        /// First line, starting at one.
        first: usize,
        /// Last line, including the endpoint.
        last: usize,
    },
}

/// A structured finding stated by a review.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReviewFinding {
    /// A unique local finding slug.
    pub id: String,
    /// The severity assigned by the reviewer.
    pub severity: ReviewSeverity,
    /// The source location at the reviewed revision.
    pub location: ReviewLocation,
    /// The finding's current disposition.
    pub disposition: ReviewDisposition,
}

impl ReviewFinding {
    /// Build a structured finding, validated by the shared profile on publication.
    #[must_use]
    pub fn new(
        id: String,
        severity: ReviewSeverity,
        location: ReviewLocation,
        disposition: ReviewDisposition,
    ) -> Self {
        Self {
            id,
            severity,
            location,
            disposition,
        }
    }
}

/// A review claim tied to the source definitions it actually judged.
///
/// The revision is provenance. Currency compares the persisted item inputs with
/// current source and never requires that revision to remain reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ReviewRecord {
    /// Stable record identity, tags and the required review date.
    pub header: RecordHeader,
    /// The named source set and its module-tree expansion policy.
    pub scope: ReviewScope,
    /// The validated full commit identifier reviewed.
    pub revision: String,
    /// The question the review answered.
    pub kind: ReviewKind,
    /// The reviewer's conclusion.
    pub verdict: ReviewVerdict,
    /// A reviewer category, without personal names or tool identities.
    pub by: Actor,
    /// Per-definition digests, persisted in deterministic identity order.
    pub inputs: Vec<Input>,
    /// Structured findings, in local identifier order.
    pub findings: Vec<ReviewFinding>,
    /// A repository path or an opaque external findings URL.
    pub findings_ref: Option<String>,
}

impl ReviewRecord {
    /// Build a review claim; the shared profile validates its date, revision,
    /// scope and input digests before the record store publishes it.
    #[must_use]
    pub fn new(
        header: RecordHeader,
        scope: ReviewScope,
        revision: String,
        kind: ReviewKind,
        verdict: ReviewVerdict,
        by: Actor,
        inputs: Vec<Input>,
    ) -> Self {
        Self {
            header,
            scope,
            revision,
            kind,
            verdict,
            by,
            inputs,
            findings: Vec::new(),
            findings_ref: None,
        }
    }
}

pub(super) fn check_scope(check: &mut Check<'_>, entry: &Entry) -> Option<ReviewScope> {
    let parsed = strings(entry.value()).and_then(|values| ReviewScope::from_selectors(&values));
    if parsed.is_none() {
        check.key("scope", Rule::WrongShape { expected: "one crate:<package>, module:<path> or cap:<slug>, or a nonempty list of distinct item:<path> values" });
    }
    parsed
}

impl ReviewScope {
    /// Parse the same closed scope selectors accepted by the record profile.
    #[must_use]
    pub fn from_selectors(values: &[&str]) -> Option<Self> {
        let first = *values.first()?;
        if values.len() == 1 {
            if let Some(name) = first
                .strip_prefix("crate:")
                .filter(|name| records::is_package_name(name))
            {
                return Some(ReviewScope::Crate(name.to_string()));
            }
            if let Some(path) = first
                .strip_prefix("module:")
                .filter(|path| symbol_name(path))
            {
                return Some(ReviewScope::Module(path.to_string()));
            }
            if is_capability_id(first) {
                return Some(ReviewScope::Capability(first.to_string()));
            }
        }
        let mut paths = values
            .iter()
            .map(|value| {
                value
                    .strip_prefix("item:")
                    .filter(|path| symbol_name(path))
                    .map(str::to_string)
            })
            .collect::<Option<Vec<_>>>()?;
        purrdf_lex::walk::try_sort_unstable_by(&mut paths, |left, right| {
            Ok::<_, std::convert::Infallible>(left.cmp(right))
        })
        .unwrap_or_else(|never| match never {});
        if paths.windows(2).any(|pair| pair[0] == pair[1]) {
            return None;
        }
        Some(ReviewScope::Items(paths))
    }
}

pub(super) fn check_kind(check: &mut Check<'_>, entry: &Entry) -> Option<ReviewKind> {
    let kind = scalar_text(entry).and_then(ReviewKind::from_name);
    if kind.is_none() {
        check.key(
            "kind",
            Rule::WrongShape {
                expected: "security, correctness, design, performance or documentation",
            },
        );
    }
    kind
}

/// Capability policy accepts only distinct members of the same closed kind set.
pub(super) fn check_required(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<ReviewKind>> {
    let values = strings(entry.value()).and_then(|values| {
        values
            .into_iter()
            .map(ReviewKind::from_name)
            .collect::<Option<Vec<_>>>()
    });
    if let Some(values) = values {
        let mut seen = Vec::new();
        for kind in &values {
            if seen.contains(kind) {
                check.key("review_required", Rule::WrongShape {
                    expected: "a list of distinct security, correctness, design, performance or documentation kinds",
                });
                return None;
            }
            seen.push(*kind);
        }
        Some(values)
    } else {
        check.key("review_required", Rule::WrongShape {
            expected: "a list of distinct security, correctness, design, performance or documentation kinds",
        });
        None
    }
}

/// Keep findings as closed data; no location or disposition is executable.
pub(super) fn check_findings(check: &mut Check<'_>, entry: &Entry) -> Option<Vec<ReviewFinding>> {
    let rows = records::rows(check, entry, "a list of structured review findings")?;
    let mut found = Vec::new();
    let mut valid = true;
    for row in rows {
        if let Some(finding) = finding(row) {
            found.push(finding);
        } else {
            check.key("findings", Rule::WrongShape { expected: "closed finding rows with id, severity, location and status; locations use item:path or file:path#Lfirst-Llast with positive ordered lines; fixed requires revision, accepted and false-positive require reason" });
            valid = false;
        }
    }
    purrdf_lex::walk::try_sort_unstable_by(&mut found, |left, right| {
        Ok::<_, std::convert::Infallible>(left.id.cmp(&right.id))
    })
    .unwrap_or_else(|never| match never {});
    if found.windows(2).any(|pair| pair[0].id == pair[1].id) {
        check.key(
            "findings",
            Rule::WrongShape {
                expected: "findings with distinct kebab-case identifiers",
            },
        );
        valid = false;
    }
    valid.then_some(found)
}

fn finding(row: &Map) -> Option<ReviewFinding> {
    let field = |key| row.get(key).and_then(scalar_text);
    let id = field("id").filter(|id| is_kebab(id))?.to_string();
    let severity = ReviewSeverity::from_name(field("severity")?)?;
    let location = location(field("location")?)?;
    let status = field("status")?;
    let (extra, disposition) = match status {
        "open" => (None, ReviewDisposition::Open),
        "fixed" => (
            Some("revision"),
            ReviewDisposition::Fixed(
                field("revision")
                    .filter(|revision| records::is_commit(revision))?
                    .to_string(),
            ),
        ),
        "accepted" => (
            Some("reason"),
            ReviewDisposition::Accepted(
                field("reason")
                    .filter(|reason| !reason.trim().is_empty())?
                    .to_string(),
            ),
        ),
        "false-positive" => (
            Some("reason"),
            ReviewDisposition::FalsePositive(
                field("reason")
                    .filter(|reason| !reason.trim().is_empty())?
                    .to_string(),
            ),
        ),
        _ => return None,
    };
    if row.entries().iter().any(|entry| {
        !["id", "severity", "location", "status"].contains(&entry.key())
            && Some(entry.key()) != extra
    }) {
        return None;
    }
    Some(ReviewFinding {
        id,
        severity,
        location,
        disposition,
    })
}

fn location(text: &str) -> Option<ReviewLocation> {
    if let Some(path) = text.strip_prefix("item:") {
        return symbol_name(path).then(|| ReviewLocation::Item(path.to_string()));
    }
    let (file, range) = text.strip_prefix("file:")?.rsplit_once('#')?;
    let (first, last) = range.strip_prefix('L')?.split_once("-L")?;
    if file.is_empty()
        || file.starts_with('/')
        || file.contains('\\')
        || file
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    let first = first.parse::<usize>().ok()?;
    let last = last.parse::<usize>().ok()?;
    (first > 0 && last >= first).then(|| ReviewLocation::Lines {
        file: file.to_string(),
        first,
        last,
    })
}

/// Native source-index paths are opaque data here, including Unicode and
/// generic impl keys. Source resolution, not a second Rust parser, judges them.
pub(super) fn symbol_name(text: &str) -> bool {
    !text.is_empty() && text.trim() == text && !text.chars().any(char::is_control)
}

pub(super) fn check_consistency(
    check: &mut Check<'_>,
    verdict: Option<records::Verdict>,
    findings: Option<&[ReviewFinding]>,
    has_reference: bool,
) -> bool {
    let Some(findings) = findings else {
        return false;
    };
    let invalid = match verdict {
        Some(records::Verdict::Review(ReviewVerdict::Clean)) => !findings.is_empty(),
        Some(records::Verdict::Review(ReviewVerdict::Findings)) => {
            findings.is_empty() && !has_reference
        }
        _ => false,
    };
    if invalid {
        check.key("findings", Rule::WrongShape { expected: "no structured findings for clean; at least one finding or a findings reference for verdict findings" });
    }
    !invalid
}
