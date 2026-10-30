// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Prose assessment: the verdict of a human or an agent on one authored
//! section of a capability, kept as a record that is current only while the
//! inputs it was given are unchanged.
//!
//! An assessment covers a whole section, `cap:<slug>#<section>`. Its inputs
//! are, each with its [`Digest`]:
//!
//! * the section: [`Digest::section`] of its authored text, the content of
//!   its generated blocks left out, so a `docs sync` does not move it;
//! * the bound symbols. A code span of the section (outside fenced code,
//!   PROPOSED blocks and generated blocks) names a `code` binding strongly
//!   when it holds, as a whole path, the binding's path or a `::`-suffix of it
//!   of two segments or more (`raf_workplan::Workplan::check` or
//!   `Workplan::check`), and weakly when it holds the last segment alone
//!   (`check`), a word that may stand for something else. When the section
//!   names a binding strongly, its inputs are the bindings it names strongly
//!   or weakly; otherwise they are every `code` binding of the capability. A
//!   weak name only ever adds an input, never takes the section out of the
//!   every-binding rule, and a suffix two bindings end in names both. Each is
//!   [`Digest::symbol`], the text of the binding's own item, so the binding of
//!   a type covers the type's definition and not the bodies of its methods: a
//!   section about a method names the method's binding. The binding of a
//!   module below a crate root stands for its whole content, the files of the
//!   module and of the modules below it;
//! * the evidence: each evidence record a claim marker of the section names,
//!   by [`Digest::record`] of its text.
//!
//! A section whose text outside its generated blocks and HTML comments is
//! blank is generated only. It has nothing an author wrote, so it would never
//! be assessed and the completion gate would not ask for an assessment of it;
//! the `section-empty` rule of the profile refuses such a section, so every
//! section of a capability that loads is authored.
//!
//! The record of an assessment is `work/assessments/<capability slug>-<section>.md`
//! with the id `assessment:<capability slug>-<section>` ([`record_slug`]):
//! one id for each section, so a section has at most one assessment and a
//! new verdict replaces the record of the section. A record whose id does not
//! follow the rule is the `check` finding `assessment-id-mismatch` and is not
//! the assessment of its subject. The record names who assessed only by kind,
//! `human` or `agent`, never by name, and an assessment by an agent counts as
//! much as one by a human.
//!
//! [`Workplan::assess_context`] gives the assessor the section, the bound
//! symbols with their source text and the evidence with its verdicts: the
//! same context whoever assesses, since nothing in it depends on who asks.
//! [`Workplan::record_assessment`] writes the record, and
//! [`Workplan::list_assessments`] says of each record whether it is current
//! and, when it is not, which inputs moved. An input moved when its digest now
//! differs from the one the record keeps, when it no longer resolves, when it
//! is no longer an input (the section no longer names the symbol, the claim is
//! gone) or when something is an input now that was not one then.
//!
//! The reconcile obligations `assessment-missing`, `assessment-stale` and
//! `assessment-rejected`, and condition 6 of the completion gate, are judged
//! by the same comparison.

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::path::Path;

use raf_okf::{
    Actor, AssessmentRecord, AssessmentVerdict, Body, Capability, CapabilitySection, DocType,
    EvidenceVerdict, Input, InputKind, Measure, SectionRef, is_kebab,
};
use raf_source::{EntryKind, RelPath};

use crate::digest::{Digest, SymbolDigestError, symbol_texts};
use crate::graph::{BindingKind, SectionId, code_span_paths, prose_of};
use crate::lifecycle::section_digest;
use crate::lookup::write_findings;
use crate::record::{self, Record, RecordError, RecordOptions};
use crate::work::{Work, add};
use crate::{Finding, Loaded, Workplan};

/// The prefix of a capability id.
const CAPABILITY_PREFIX: &str = "cap:";

/// The slug of the assessment record of the section `section` of the
/// capability `capability` (`cap:<slug>`): `<slug>-<section>`, or `None` when
/// `capability` is not a capability id.
///
/// The section slugs hold no `-`, so the text after the last `-` is the
/// section and two sections never share a slug.
///
/// ```rust
/// use raf_okf::CapabilitySection;
/// use raf_workplan::assessment::record_slug;
///
/// assert_eq!(
///     record_slug("cap:workplan-reconcile", CapabilitySection::Explanation).as_deref(),
///     Some("workplan-reconcile-explanation")
/// );
/// assert_eq!(record_slug("workplan-reconcile", CapabilitySection::Usage), None);
/// ```
#[must_use]
pub fn record_slug(capability: &str, section: CapabilitySection) -> Option<String> {
    let slug = capability.strip_prefix(CAPABILITY_PREFIX)?;
    is_kebab(slug).then(|| format!("{slug}-{}", section.slug()))
}

/// The section `text` names, written `cap:<slug>#<section>`.
///
/// # Errors
///
/// [`AssessError::BadSubject`] when the text is not of that form, and
/// [`AssessError::NoSuchSection`] when the part after `#` is not one of the
/// seven sections.
pub fn parse_subject(text: &str) -> Result<SectionId, AssessError> {
    let bad = || AssessError::BadSubject {
        found: text.to_string(),
    };
    let (capability, slug) = text.split_once('#').ok_or_else(bad)?;
    let bare = capability.strip_prefix(CAPABILITY_PREFIX).ok_or_else(bad)?;
    if !is_kebab(bare) {
        return Err(bad());
    }
    let section = CapabilitySection::from_slug(slug).ok_or_else(|| AssessError::NoSuchSection {
        subject: text.to_string(),
    })?;
    Ok(SectionId::new(capability, section))
}

/// What an input names now.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputNow {
    /// The digest of its text now.
    Digest(Digest),
    /// It names nothing that resolves now; why.
    Unresolved(String),
}

/// One input of an assessment of a section, as it is now.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CurrentInput {
    /// What the input names.
    pub kind: InputKind,
    /// The section `cap:<slug>#<section>`, the symbol path or `evidence:<slug>`.
    pub name: String,
    /// Its digest now, or why it has none.
    pub now: InputNow,
}

/// How an input of an assessment moved since the assessment was made.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputChange {
    /// The text it names has another digest now.
    Changed,
    /// It names nothing that resolves now; why.
    Unresolved(String),
    /// It was an input and is not one now: the section no longer names the
    /// symbol, the capability no longer binds it, or no claim of the section
    /// names the record.
    NoLongerInput,
    /// It is an input now and was not one when the assessment was made.
    NewInput,
}

/// An input of an assessment that moved.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MovedInput {
    /// What the input names.
    pub kind: InputKind,
    /// The section, symbol path or evidence id.
    pub name: String,
    /// How it moved.
    pub change: InputChange,
}

impl fmt::Display for MovedInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            InputKind::Declaration => "model declaration",
            InputKind::Section => "section",
            InputKind::Symbol => "symbol",
            InputKind::Record => "evidence record",
        };
        write!(f, "the {what} `{}` ", self.name)?;
        match &self.change {
            InputChange::Changed => f.write_str("has changed since"),
            InputChange::Unresolved(reason) => write!(f, "does not resolve now: {reason}"),
            InputChange::NoLongerInput => f.write_str("is no longer an input"),
            InputChange::NewInput => f.write_str("is an input now and was not then"),
        }
    }
}

/// Whether an assessment is current.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Currency {
    /// Every input it keeps has the digest it keeps, and it keeps every input
    /// the section has now.
    Current,
    /// The inputs that moved, in the order of the inputs: the section, the
    /// symbols by path, the evidence records by id.
    Stale(Vec<MovedInput>),
}

impl Currency {
    /// Whether the assessment is current.
    #[must_use]
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current)
    }
}

/// An assessment record and whether it is current.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssessmentState {
    /// The id, `assessment:<slug>`.
    pub id: String,
    /// The repository-relative path of the record.
    pub path: String,
    /// The section assessed, `cap:<slug>#<section>`.
    pub subject: String,
    /// The verdict.
    pub verdict: AssessmentVerdict,
    /// Who assessed.
    pub by: Actor,
    /// Whether it is current.
    pub currency: Currency,
}

impl fmt::Display for AssessmentState {
    /// One line: the subject, the id, the verdict, who assessed, and
    /// `current`, or `stale:` and each input that moved.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}  {}  {}  {}  ",
            self.subject,
            self.id,
            self.verdict.name(),
            self.by.name()
        )?;
        match &self.currency {
            Currency::Current => f.write_str("current"),
            Currency::Stale(moved) => {
                f.write_str("stale: ")?;
                let lines: Vec<String> = moved.iter().map(ToString::to_string).collect();
                f.write_str(&lines.join("; "))
            }
        }
    }
}

/// One definition of a bound symbol and its source text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Definition {
    /// The workspace-relative path of the file.
    pub file: String,
    /// The text of the definition, attributes and documentation included,
    /// with CRLF read as LF.
    pub text: String,
}

/// A bound symbol in the context of an assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContextSymbol {
    /// The source binding path selected by the prose or model.
    pub path: String,
    /// Whether a code span of the section names it, strongly or weakly;
    /// `false` for a binding that is an input only because the section names
    /// no bound symbol strongly, or because the model requires it.
    pub named: bool,
    /// Its definitions, in file order; empty when it does not resolve.
    pub definitions: Vec<Definition>,
    /// Why it does not resolve, when it does not.
    pub unresolved: Option<String>,
}

/// An evidence record in the context of an assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ContextEvidence {
    /// The id, `evidence:<slug>`.
    pub id: String,
    /// The repository-relative path of the record.
    pub path: String,
    /// The verdict, or `None` when no record loaded under the id.
    pub verdict: Option<EvidenceVerdict>,
    /// The measured values.
    pub values: Vec<Measure>,
    /// The limits.
    pub limits: Vec<Measure>,
}

/// What an assessor reads before giving a verdict on a section: the same for
/// a human and an agent.
///
/// `Display` writes it as text: a line for the subject, the record and the
/// document, then the authored text of the section, each bound symbol with the
/// source text of its definitions, each evidence record with its verdict, and
/// the inputs with their digests. Every line of quoted text is indented by four
/// spaces, so no text of a document can pass for a line of the context.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssessContext {
    /// The section, `cap:<slug>#<section>`.
    pub subject: String,
    /// The id of the record an assessment of it is written as.
    pub record: String,
    /// The repository-relative path of the record.
    pub record_path: String,
    /// The repository-relative path of the capability document.
    pub document: String,
    /// The authored text of the section: its heading and its content, with
    /// the content of each generated block left out and CRLF read as LF.
    pub section: String,
    /// The bound symbols that are inputs, by path.
    pub symbols: Vec<ContextSymbol>,
    /// The evidence records the claims of the section name, by id.
    pub evidence: Vec<ContextEvidence>,
    /// The inputs and their digests now, in the order a record keeps them.
    pub inputs: Vec<CurrentInput>,
}

/// Write each line of `text` indented by four spaces.
fn quoted(f: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    for line in text.lines() {
        writeln!(f, "    {line}")?;
    }
    Ok(())
}

impl fmt::Display for AssessContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "assess {}", self.subject)?;
        writeln!(f, "record {} ({})", self.record, self.record_path)?;
        writeln!(f, "section {}", self.document)?;
        quoted(f, &self.section)?;
        for symbol in &self.symbols {
            let why = if symbol.named {
                "named in the section"
            } else {
                "bound by the capability; the section names no bound symbol by a path"
            };
            writeln!(f, "symbol {} ({why})", symbol.path)?;
            if let Some(reason) = &symbol.unresolved {
                writeln!(f, "  does not resolve: {reason}")?;
            }
            for definition in &symbol.definitions {
                writeln!(f, "  in {}", definition.file)?;
                quoted(f, &definition.text)?;
            }
        }
        for evidence in &self.evidence {
            match evidence.verdict {
                Some(verdict) => writeln!(
                    f,
                    "evidence {} {} ({})",
                    evidence.id,
                    verdict.name(),
                    evidence.path
                )?,
                None => writeln!(
                    f,
                    "evidence {} is not a loaded evidence record ({})",
                    evidence.id, evidence.path
                )?,
            }
            for value in &evidence.values {
                writeln!(f, "  value {}={}", value.name, value.value.text())?;
            }
            for limit in &evidence.limits {
                writeln!(f, "  limit {}={}", limit.name, limit.value.text())?;
            }
        }
        writeln!(f, "inputs")?;
        for input in &self.inputs {
            match &input.now {
                InputNow::Digest(digest) => {
                    writeln!(f, "  {} {} {digest}", input.kind.name(), input.name)?;
                }
                InputNow::Unresolved(reason) => writeln!(
                    f,
                    "  {} {} does not resolve: {reason}",
                    input.kind.name(),
                    input.name
                )?,
            }
        }
        Ok(())
    }
}

/// What to record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AssessmentRequest {
    /// The section assessed, `cap:<slug>#<section>`.
    pub subject: String,
    /// The verdict.
    pub verdict: AssessmentVerdict,
    /// Who assessed.
    pub by: Actor,
    /// The SPDX comment lines of the record.
    pub options: RecordOptions,
}

impl AssessmentRequest {
    /// A request to record `verdict` by `by` on `subject`, with no SPDX lines.
    #[must_use]
    pub fn new(subject: impl Into<String>, verdict: AssessmentVerdict, by: Actor) -> Self {
        Self {
            subject: subject.into(),
            verdict,
            by,
            options: RecordOptions::new(),
        }
    }

    /// The same request with these SPDX options.
    #[must_use]
    pub fn options(mut self, options: RecordOptions) -> Self {
        self.options = options;
        self
    }
}

/// An assessment that was written.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Assessed {
    /// The repository-relative path of the record.
    pub path: RelPath,
    /// The record as written.
    pub record: AssessmentRecord,
    /// Whether it replaced the record of the section that was there.
    pub replaced: bool,
}

impl fmt::Display for Assessed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({} by {})",
            if self.replaced {
                "replaced"
            } else {
                "recorded"
            },
            self.path,
            self.record.verdict.name(),
            self.record.by.name()
        )
    }
}

/// Why there is no context or no record. Nothing is written in any case.
#[derive(Debug)]
#[non_exhaustive]
pub enum AssessError {
    /// The subject is not `cap:<slug>#<section>`.
    BadSubject {
        /// The text as given.
        found: String,
    },
    /// The subject names no section of the capability: the part after `#` is
    /// not one of the seven sections, or the document has no such section.
    NoSuchSection {
        /// The subject as given.
        subject: String,
    },
    /// No capability document has the id.
    UnknownCapability {
        /// The id.
        id: String,
    },
    /// The capability document exists and did not load.
    CapabilityNotLoaded {
        /// The id.
        id: String,
        /// The findings about its document.
        findings: Vec<Finding>,
    },
    /// The section has no authored text, only generated blocks, so it is not
    /// assessed.
    GeneratedOnly {
        /// The subject.
        subject: String,
    },
    /// A bound symbol that is an input does not resolve, so the record would
    /// keep no digest of it.
    SymbolUnresolved {
        /// The binding path.
        path: String,
        /// Why.
        reason: String,
    },
    /// An assessment input names an evidence record that does not load.
    EvidenceNotLoaded {
        /// The evidence id.
        id: String,
    },
    /// A model declaration input unexpectedly lacks a current digest.
    /// This is a defensive error: the closed input producers currently create
    /// declaration inputs only with digests of loaded documents.
    DeclarationNotLoaded {
        /// Owning capability or architecture document id.
        id: String,
    },
    /// The record cannot be written.
    Record(RecordError),
}

impl fmt::Display for AssessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSubject { found } => write!(
                f,
                "`{found}` is not a section; write `cap:<slug>#<section>`, such as `cap:workplan-reconcile#explanation`"
            ),
            Self::NoSuchSection { subject } => {
                let names: Vec<&str> = CapabilitySection::ALL.iter().map(|s| s.slug()).collect();
                write!(
                    f,
                    "`{subject}` names no section of the capability; the sections are {}",
                    names.join(", ")
                )
            }
            Self::UnknownCapability { id } => write!(f, "`{id}` names no capability document"),
            Self::CapabilityNotLoaded { id, findings } => {
                write!(
                    f,
                    "{id} did not load, so its sections cannot be assessed; fix its document first"
                )?;
                write_findings(f, findings)
            }
            Self::GeneratedOnly { subject } => write!(
                f,
                "`{subject}` holds only generated blocks, which `docs sync` writes; there is no authored text to assess, and the completion gate asks for no assessment of it"
            ),
            Self::SymbolUnresolved { path, reason } => write!(
                f,
                "the bound symbol `{path}` does not resolve: {reason}; fix the binding first, so the assessment keeps a digest of what it read; nothing was written"
            ),
            Self::EvidenceNotLoaded { id } => write!(
                f,
                "an assessment input names `{id}`, which is not a loaded evidence record; record the evidence or fix the reference first; nothing was written"
            ),
            Self::DeclarationNotLoaded { id } => write!(
                f,
                "the model declarations of `{id}` are unavailable; repair the document and regenerate its views before assessing it; nothing was written"
            ),
            Self::Record(error) => error.fmt(f),
        }
    }
}

impl Error for AssessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Record(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RecordError> for AssessError {
    fn from(error: RecordError) -> Self {
        Self::Record(error)
    }
}

/// Whether the section `section` of `body` holds authored text: something
/// other than blank space outside its generated blocks and HTML comments. A
/// section the body does not hold counts as authored, so it is judged and
/// owed rather than passed over.
///
/// This is the rule by which the `section-empty` rule of the `raf-okf` profile
/// refuses a section, so no capability that loads holds a section it is false
/// for; a unit test holds the two to the same verdict on every kind of content.
pub(crate) fn is_authored(body: &Body, section: CapabilitySection) -> bool {
    let Some(found) = body.sections().iter().find(|s| s.slug() == section.slug()) else {
        return true;
    };
    let Some(text) = body.section_text(found) else {
        return true;
    };
    let start = found.content().bytes.start;
    let end = found.content().bytes.end;
    let mut kept = String::with_capacity(text.len());
    let mut at = start;
    for block in body.generated_blocks() {
        let outer = &block.outer().bytes;
        if outer.start < at || outer.end > end {
            continue;
        }
        kept.push_str(&text[at - start..outer.start - start]);
        at = outer.end;
    }
    kept.push_str(&text[at - start..]);
    !without_comments(&kept).trim().is_empty()
}

/// `text` without its HTML comments; a comment that is never closed removes
/// the rest of the text.
fn without_comments(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find("<!--") {
        out.push_str(&rest[..open]);
        match rest[open + 4..].find("-->") {
            Some(close) => rest = &rest[open + 4 + close + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// How the code spans of a section name a `code` binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Naming {
    /// No code span names it.
    Unnamed,
    /// A code span holds its last segment alone, such as `check` for
    /// `raf_workplan::Workplan::check`. A word that common may stand for
    /// something else, so a weak name adds the binding as an input and never
    /// takes the other bindings out.
    Weak,
    /// A code span holds its whole path or a `::`-suffix of two segments or
    /// more, such as `Workplan::check`.
    Strong,
}

/// How the paths `named` in the code spans of a section name the binding
/// `path` (see [`Naming`]): the strongest of its whole path and each of its
/// `::`-suffixes that a span holds as a whole path.
pub(crate) fn naming(named: &BTreeSet<String>, path: &str) -> Naming {
    add(Work::NamingLookups, 1);
    let named = |text: &str| {
        add(Work::NamedLookups, 1);
        named.contains(text)
    };
    let mut rest = path;
    loop {
        match rest.split_once("::") {
            Some((_, tail)) => {
                if named(rest) {
                    return Naming::Strong;
                }
                rest = tail;
            }
            // The last segment alone; a path of one segment is whole.
            None if named(rest) && rest.len() < path.len() => return Naming::Weak,
            None if named(rest) => return Naming::Strong,
            None => return Naming::Unnamed,
        }
    }
}

/// The `code` bindings of `paths` that are inputs of a section whose code
/// spans hold `named`, each with how it is named: every binding a span names
/// strongly or weakly when one is named strongly, and every binding when none
/// is. A weak name therefore only ever adds an input: it never takes the
/// section out of the every-binding rule.
pub(crate) fn symbol_inputs<'p>(
    named: &BTreeSet<String>,
    paths: impl Iterator<Item = &'p str> + Clone,
) -> Vec<(&'p str, Naming)> {
    let any_strong = paths
        .clone()
        .any(|path| naming(named, path) == Naming::Strong);
    paths
        .map(|path| (path, naming(named, path)))
        .filter(|(_, how)| !any_strong || *how != Naming::Unnamed)
        .collect()
}

/// The inputs of an assessment of `section` of `cap` as they are now, keyed
/// by kind and name, in the order a record keeps them; and the bound symbols
/// that are inputs, each with whether the section names it.
struct Inputs {
    rows: BTreeMap<(InputKind, String), InputNow>,
    symbols: Vec<(String, bool)>,
    evidence: Vec<String>,
    refusal: Option<SymbolDigestError>,
}

fn symbol_input_now(
    source: &raf_source::Workspace,
    binding: &crate::Binding,
    refusal: &mut Option<SymbolDigestError>,
) -> InputNow {
    let mut unresolved = |error: SymbolDigestError, reason: String| {
        if error.is_query_refusal() && refusal.is_none() {
            *refusal = Some(error);
        }
        InputNow::Unresolved(reason)
    };
    match binding.resolution() {
        Err(error) => unresolved(
            SymbolDigestError::Resolve {
                path: binding.path().into(),
                error: error.clone(),
            },
            error.to_string(),
        ),
        Ok(_) => match Digest::symbol(source, binding.path()) {
            Ok(digest) => InputNow::Digest(digest),
            Err(error) => {
                let reason = error.to_string();
                unresolved(error, reason)
            }
        },
    }
}

impl Workplan<'_> {
    /// The loaded capability `id`.
    fn assessed_capability(&self, id: &str) -> Result<&Loaded<Capability>, AssessError> {
        if let Some(cap) = self
            .capabilities()
            .iter()
            .find(|c| c.value().header.id == id)
        {
            return Ok(cap);
        }
        let path = crate::lookup::capability_path(id).unwrap_or_default();
        if self.is_present(id) {
            Err(AssessError::CapabilityNotLoaded {
                id: id.to_string(),
                findings: self.findings_of(&path),
            })
        } else {
            Err(AssessError::UnknownCapability { id: id.to_string() })
        }
    }

    /// The inputs of an assessment of `section` of `cap` now.
    fn inputs_now(&self, cap: &Loaded<Capability>, section: CapabilitySection) -> Inputs {
        let id = cap.value().header.id.as_str();
        let body = cap.body();
        let mut rows = BTreeMap::new();
        let subject = SectionId::new(id, section).to_string();
        let digest = Digest::section(body, section).map_or_else(
            || InputNow::Unresolved("the capability document has no such section".to_string()),
            InputNow::Digest,
        );
        rows.insert((InputKind::Section, subject), digest);
        if cap.value().model != raf_okf::ArchitectureModel::default() {
            rows.insert(
                (InputKind::Declaration, id.to_owned()),
                InputNow::Digest(Digest::capability_declaration(cap.value())),
            );
        }

        let named = body
            .sections()
            .iter()
            .find(|s| s.slug() == section.slug())
            .and_then(|s| prose_of(body, s))
            .map_or_default(|prose| code_span_paths(&prose));
        let code: Vec<_> = self
            .graph()
            .bindings_of(id)
            .filter(|binding| binding.kind() == BindingKind::Code)
            .collect();
        let wanted: BTreeMap<&str, Naming> =
            symbol_inputs(&named, code.iter().map(|binding| binding.path()))
                .into_iter()
                .collect();
        let mut symbols = Vec::new();
        let mut refusal = None;
        for binding in code {
            let Some(how) = wanted.get(binding.path()) else {
                continue;
            };
            let now = symbol_input_now(self.source(), binding, &mut refusal);
            if rows
                .insert((InputKind::Symbol, binding.path().to_string()), now)
                .is_none()
            {
                symbols.push((binding.path().to_string(), *how != Naming::Unnamed));
            }
        }
        symbols.sort();

        let prefix = DocType::Evidence.id_prefix();
        let mut evidence = Vec::new();
        for claim in body.claims() {
            if claim.section() != Some(section.slug()) {
                continue;
            }
            for slug in claim.evidence() {
                let evidence_id = format!("{prefix}{slug}");
                if rows.contains_key(&(InputKind::Record, evidence_id.clone())) {
                    continue;
                }
                let now = self
                    .evidence()
                    .iter()
                    .find(|record| record.value().header.id == evidence_id)
                    .map_or_else(
                        || InputNow::Unresolved("it is not a loaded evidence record".to_string()),
                        |record| InputNow::Digest(Digest::record(&record.text())),
                    );
                rows.insert((InputKind::Record, evidence_id.clone()), now);
                evidence.push(evidence_id);
            }
        }
        evidence.sort();
        let mut inputs = Inputs {
            rows,
            symbols,
            evidence,
            refusal,
        };
        self.model_inputs(&cap.value().model, &named, &mut inputs);
        inputs
    }

    fn model_symbol_input(&self, path: &str, named: &BTreeSet<String>, inputs: &mut Inputs) {
        let now = match Digest::symbol(self.source(), path) {
            Ok(digest) => InputNow::Digest(digest),
            Err(error) => {
                let reason = error.to_string();
                if error.is_query_refusal() && inputs.refusal.is_none() {
                    inputs.refusal = Some(error);
                }
                InputNow::Unresolved(reason)
            }
        };
        if inputs
            .rows
            .insert((InputKind::Symbol, path.to_owned()), now)
            .is_none()
        {
            inputs
                .symbols
                .push((path.to_owned(), naming(named, path) != Naming::Unnamed));
        }
    }

    fn model_inputs(
        &self,
        model: &raf_okf::ArchitectureModel,
        named: &BTreeSet<String>,
        inputs: &mut Inputs,
    ) {
        for bindings in model.bindings() {
            for path in bindings
                .code
                .iter()
                .chain(&bindings.interfaces)
                .chain(&bindings.tests)
                .chain(&bindings.examples)
            {
                self.model_symbol_input(path, named, inputs);
            }
            for capability in &bindings.capabilities {
                let target = self
                    .capabilities()
                    .iter()
                    .find(|target| target.value().header.id == *capability);
                let now = target
                    .and_then(|target| Digest::section(target.body(), CapabilitySection::Contract))
                    .map_or_else(
                        || {
                            InputNow::Unresolved(
                                "model-bound capability contract did not load".into(),
                            )
                        },
                        InputNow::Digest,
                    );
                inputs
                    .rows
                    .insert((InputKind::Section, format!("{capability}#contract")), now);
                if let Some(target) = target {
                    for path in target
                        .value()
                        .bindings
                        .code
                        .iter()
                        .chain(&target.value().bindings.tests)
                        .chain(&target.value().bindings.examples)
                    {
                        self.model_symbol_input(path, named, inputs);
                    }
                    inputs.rows.insert(
                        (InputKind::Declaration, capability.clone()),
                        InputNow::Digest(Digest::capability_declaration(target.value())),
                    );
                }
            }
        }
        for evidence_id in model.evidence() {
            let record = self
                .evidence()
                .iter()
                .find(|record| record.value().header.id == evidence_id);
            let now = record.map_or_else(
                || InputNow::Unresolved("model-associated verification record did not load".into()),
                |record| InputNow::Digest(Digest::record(&record.text())),
            );
            if inputs
                .rows
                .insert((InputKind::Record, evidence_id.to_owned()), now)
                .is_none()
            {
                inputs.evidence.push(evidence_id.to_owned());
            }
            if let Some(record) = record {
                for input in &record.value().inputs {
                    self.model_symbol_input(&input.name, named, inputs);
                }
            }
        }
        inputs.symbols.sort();
        inputs.evidence.sort();
    }

    /// The loaded capability and the section of `subject`, which must hold
    /// authored text.
    fn assessed_section(
        &self,
        subject: &str,
    ) -> Result<(&Loaded<Capability>, SectionId), AssessError> {
        let id = parse_subject(subject)?;
        let cap = self.assessed_capability(id.capability())?;
        let body = cap.body();
        if !body
            .sections()
            .iter()
            .any(|s| s.slug() == id.section().slug())
        {
            return Err(AssessError::NoSuchSection {
                subject: subject.to_string(),
            });
        }
        if !is_authored(body, id.section()) {
            return Err(AssessError::GeneratedOnly {
                subject: subject.to_string(),
            });
        }
        Ok((cap, id))
    }

    /// The context of an assessment of `subject` (`cap:<slug>#<section>`):
    /// the authored text of the section, the bound symbols that are its inputs
    /// with the source text of their definitions, the evidence records its
    /// claims name with their verdicts, and the inputs with their digests now
    /// (see the [module](self)). It takes nothing about who assesses, so a
    /// human and an agent read the same context.
    ///
    /// A symbol that does not resolve and a claim on a record that does not
    /// load are shown as such; [`Workplan::record_assessment`] refuses them.
    /// Nothing is written and nothing is started.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    /// use raf_workplan::assessment::AssessError;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// assert!(matches!(
    ///     plan.assess_context("cap:parser#intro"),
    ///     Err(AssessError::NoSuchSection { .. })
    /// ));
    /// assert!(matches!(
    ///     plan.assess_context("cap:parser#usage"),
    ///     Err(AssessError::UnknownCapability { .. })
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`AssessError::BadSubject`], [`AssessError::NoSuchSection`],
    /// [`AssessError::UnknownCapability`], [`AssessError::CapabilityNotLoaded`]
    /// and [`AssessError::GeneratedOnly`].
    pub fn assess_context(&self, subject: &str) -> Result<AssessContext, AssessError> {
        let (cap, id) = self.assessed_section(subject)?;
        let body = cap.body();
        let section =
            section_digest(body, id.section()).map_or_default(|digest| digest.text().to_string());
        let inputs = self.inputs_now(cap, id.section());
        let symbols = inputs
            .symbols
            .iter()
            .map(|(path, named)| {
                let (definitions, unresolved) =
                    match &inputs.rows[&(InputKind::Symbol, path.clone())] {
                        InputNow::Unresolved(reason) => (Vec::new(), Some(reason.clone())),
                        InputNow::Digest(_) => match symbol_texts(self.source(), path) {
                            Ok(texts) => (
                                texts
                                    .into_iter()
                                    .map(|(file, text)| Definition { file, text })
                                    .collect(),
                                None,
                            ),
                            Err(error) => (Vec::new(), Some(error.to_string())),
                        },
                    };
                ContextSymbol {
                    path: path.clone(),
                    named: *named,
                    definitions,
                    unresolved,
                }
            })
            .collect();
        let evidence = inputs
            .evidence
            .iter()
            .map(|evidence_id| {
                let loaded = self
                    .evidence()
                    .iter()
                    .find(|record| record.value().header.id == *evidence_id);
                ContextEvidence {
                    id: evidence_id.clone(),
                    path: crate::evidence::record_path(evidence_id),
                    verdict: loaded.map(|record| record.value().verdict),
                    values: loaded.map_or_default(|record| record.value().values.clone()),
                    limits: loaded.map_or_default(|record| record.value().limits.clone()),
                }
            })
            .collect();
        let slug = record_slug(id.capability(), id.section()).unwrap_or_default();
        Ok(AssessContext {
            subject: id.to_string(),
            record: format!("{}{slug}", DocType::Assessment.id_prefix()),
            record_path: format!("{}/{slug}.md", DocType::Assessment.directory()),
            document: cap.path().to_string(),
            section,
            symbols,
            evidence,
            inputs: inputs
                .rows
                .into_iter()
                .map(|((kind, name), now)| CurrentInput { kind, name, now })
                .collect(),
        })
    }

    /// The record an assessment of `request.subject` is written as, built
    /// and judged as [`Workplan::record_assessment`] writes it, without
    /// writing anything: the subject, the verdict, `by` (`human` or `agent`)
    /// and one input row for each input of the section now, with its digest
    /// (see the [module](self)). [`Record::text`] with the request's options is
    /// the text of the file and [`Record::path`] its path.
    ///
    /// # Errors
    ///
    /// Every error of [`Workplan::assess_context`];
    /// [`AssessError::SymbolUnresolved`] when a bound symbol that is an input
    /// does not resolve; [`AssessError::EvidenceNotLoaded`] when an assessment
    /// input names a record that does not load; [`AssessError::UnknownCapability`]
    /// or [`AssessError::CapabilityNotLoaded`] when a model-bound capability
    /// does not load; [`AssessError::Record`] when the record or its SPDX lines
    /// are refused.
    pub fn assessment_record(&self, request: &AssessmentRequest) -> Result<Record, AssessError> {
        let (cap, id) = self.assessed_section(&request.subject)?;
        let inputs = self.inputs_now(cap, id.section());
        let mut rows = Vec::with_capacity(inputs.rows.len());
        for ((kind, name), now) in inputs.rows {
            match now {
                InputNow::Digest(digest) => rows.push(Input::new(kind, name, digest.to_string())),
                InputNow::Unresolved(reason) => {
                    return Err(match kind {
                        InputKind::Declaration => AssessError::DeclarationNotLoaded { id: name },
                        InputKind::Record => AssessError::EvidenceNotLoaded { id: name },
                        InputKind::Symbol => AssessError::SymbolUnresolved { path: name, reason },
                        InputKind::Section => {
                            let section = parse_subject(&name)?;
                            self.assessed_capability(section.capability())?;
                            AssessError::NoSuchSection { subject: name }
                        }
                    });
                }
            }
        }
        let slug = record_slug(id.capability(), id.section()).unwrap_or_default();
        let record = Record::Assessment(AssessmentRecord::new(
            slug,
            SectionRef::new(id.capability(), id.section()),
            rows,
            request.verdict,
            request.by,
        ));
        record.text(&request.options)?;
        Ok(record)
    }

    /// Write the assessment of `request.subject` as the record
    /// `work/assessments/<capability slug>-<section>.md` below `root`, the
    /// directory that holds `work/`: the record of
    /// [`Workplan::assessment_record`].
    ///
    /// When a record of the section is there, the new one replaces it,
    /// through a temporary file and a rename, and nothing of the old one is
    /// kept; otherwise the file is created exclusively. `work` and
    /// `work/assessments` must each be a directory, and a symbolic link there,
    /// inside the root or out of it, or a file is refused. Every check is made
    /// before anything is written, and the record is the one file written: no
    /// document is touched.
    ///
    /// # Errors
    ///
    /// Every error of [`Workplan::assessment_record`], before anything is
    /// written, and [`AssessError::Record`] when a directory on the way is
    /// refused, the file in the way is not a regular file, or the write fails.
    pub fn record_assessment(
        &self,
        root: &Path,
        request: &AssessmentRequest,
    ) -> Result<Assessed, AssessError> {
        self.record_assessment_in(&mut record::DiskRecordStore::new(root), request)
    }

    /// Publish an assessment through a checked store, selecting create or replace.
    ///
    /// # Errors
    /// The errors of [`Workplan::record_assessment`], preserving store causes.
    pub fn record_assessment_in(
        &self,
        store: &mut dyn record::RecordStore,
        request: &AssessmentRequest,
    ) -> Result<Assessed, AssessError> {
        let record = self.assessment_record(request)?;
        let path = record.path()?;
        // The way to the record is judged as every record write judges it
        // (`work` and `work/assessments` are directories, never a link), so
        // the file found is the record of the section and nothing else.
        let replaced = match store.inspect(&path)? {
            Some(EntryKind::File) => true,
            Some(_) => return Err(RecordError::Exists { path }.into()),
            None => false,
        };
        let path = if replaced {
            record::replace_in(store, &record, &request.options)?
        } else {
            record::create_in(store, &record, &request.options)?
        };
        let Record::Assessment(record) = record else {
            unreachable!("an assessment record was built")
        };
        Ok(Assessed {
            path,
            record,
            replaced,
        })
    }

    /// Whether the assessment `record` is current: its inputs compared with
    /// the inputs of its subject now.
    fn currency_inputs(&self, record: &AssessmentRecord) -> Option<Inputs> {
        self.capabilities()
            .iter()
            .find(|cap| cap.value().header.id == record.subject.capability)
            .map(|cap| self.inputs_now(cap, record.subject.section))
    }

    pub(crate) fn currency(&self, record: &AssessmentRecord) -> Currency {
        currency_from_rows(
            record,
            self.currency_inputs(record).map(|inputs| inputs.rows),
        )
    }

    pub(crate) fn currency_checked(
        &self,
        record: &AssessmentRecord,
    ) -> Result<Currency, SymbolDigestError> {
        let now = self.currency_inputs(record);
        if let Some(Inputs {
            refusal: Some(error),
            ..
        }) = &now
        {
            return Err(error.clone());
        }
        Ok(currency_from_rows(record, now.map(|inputs| inputs.rows)))
    }

    /// The assessment of the section `section` of `cap`, as the completion
    /// gate reads it.
    pub(crate) fn judge_section(
        &self,
        cap: &Loaded<Capability>,
        section: CapabilitySection,
    ) -> Judged<'_> {
        if !is_authored(cap.body(), section) {
            return Judged::Exempt;
        }
        let capability = cap.value().header.id.as_str();
        let slug = record_slug(capability, section).unwrap_or_default();
        let found = self.assessments().iter().find(|record| {
            let value = record.value();
            value.header.slug == slug
                && value.subject.capability == capability
                && value.subject.section == section
        });
        match found {
            None => Judged::Missing {
                id: format!("{}{slug}", DocType::Assessment.id_prefix()),
            },
            Some(record) => match self.currency(record.value()) {
                Currency::Current => Judged::Current(record),
                Currency::Stale(moved) => Judged::Stale(record, moved),
            },
        }
    }

    /// Every assessment record that loaded and whether it is current, by
    /// subject (the capability, then the sections in the order a capability
    /// holds them) and then by id. A record that did not load is a finding of
    /// [`Workplan::check`], not an entry. A record whose subject is a section
    /// of a capability that is not loaded is stale, its section input
    /// unresolved. Nothing is written and nothing is started.
    #[must_use]
    pub fn list_assessments(&self) -> Vec<AssessmentState> {
        let mut states: Vec<(SectionId, AssessmentState)> = self
            .assessments()
            .iter()
            .map(|record| {
                let value = record.value();
                (
                    SectionId::new(&value.subject.capability, value.subject.section),
                    AssessmentState {
                        id: value.header.id.clone(),
                        path: record.path().to_string(),
                        subject: value.subject.text(),
                        verdict: value.verdict,
                        by: value.by,
                        currency: self.currency(value),
                    },
                )
            })
            .collect();
        states.sort_by(|a, b| (&a.0, &a.1.id).cmp(&(&b.0, &b.1.id)));
        states.into_iter().map(|(_, state)| state).collect()
    }
}

/// What the completion gate finds for one affected section.
pub(crate) enum Judged<'a> {
    /// The section has no authored text.
    Exempt,
    /// No assessment of it: the id it would have.
    Missing { id: String },
    /// A current assessment.
    Current(&'a Loaded<AssessmentRecord>),
    /// An assessment whose inputs moved.
    Stale(&'a Loaded<AssessmentRecord>, Vec<MovedInput>),
}

fn currency_from_rows(
    record: &AssessmentRecord,
    now: Option<BTreeMap<(InputKind, String), InputNow>>,
) -> Currency {
    let subject = &record.subject;
    let Some(mut now) = now else {
        return Currency::Stale(vec![MovedInput {
            kind: InputKind::Section,
            name: subject.text(),
            change: InputChange::Unresolved(format!(
                "`{}` is not a loaded capability document",
                subject.capability
            )),
        }]);
    };
    let mut moved = Vec::new();
    for input in &record.inputs {
        let change = match now.remove(&(input.kind, input.name.clone())) {
            None => InputChange::NoLongerInput,
            Some(InputNow::Unresolved(reason)) => InputChange::Unresolved(reason),
            Some(InputNow::Digest(digest)) if digest.to_string() == input.digest => continue,
            Some(InputNow::Digest(_)) => InputChange::Changed,
        };
        moved.push(MovedInput {
            kind: input.kind,
            name: input.name.clone(),
            change,
        });
    }
    moved.extend(now.into_keys().map(|(kind, name)| MovedInput {
        kind,
        name,
        change: InputChange::NewInput,
    }));
    if moved.is_empty() {
        Currency::Current
    } else {
        moved.sort_by(|a, b| (a.kind, &a.name).cmp(&(b.kind, &b.name)));
        Currency::Stale(moved)
    }
}
