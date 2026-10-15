// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! [`Workplan::new_change`]: the record of a new change, and what the author
//! does next.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write as _};
use std::path::Path;

use raf_okf::{
    CapabilitySection, ConstructError, DocType, Document, Entry, Frontmatter, List, ListItem, Map,
    Scalar, State, Value, is_kebab,
};
use raf_source::{DiskTree, EntryKind, RelPath, Tree};

use crate::generated;
use crate::graph::SectionId;
use crate::{Finding, Workplan};

/// The tag of the copyright line of an SPDX header. It is written in two pieces
/// so that this file does not carry the tag itself, which a licence checker
/// reads in any file.
const COPYRIGHT_TAG: &str = concat!("SPDX-", "FileCopyrightText: ");

/// The tag of the licence line of an SPDX header, in two pieces for the same
/// reason.
const LICENCE_TAG: &str = concat!("SPDX-", "License-Identifier: ");

/// The prefix of a capability id.
const CAPABILITY_PREFIX: &str = "cap:";

/// The title of a new record, for the author to replace.
const TITLE: &str = "Name this change";

/// The description of a new record, for the author to replace.
const DESCRIPTION: &str = "Say in one line what this change does to the capabilities it lists.";

/// The body of a new record: the `Summary` section the profile requires.
const BODY: &str =
    "\n## Summary\n\nDescribe what this change does and why, in a short paragraph.\n";

/// The text of the placeholder line of the PROPOSED block template.
const TEMPLATE_LINE: &str = "> State the proposed text here.";

/// What `change new` is asked to create: the slug, the capabilities the change
/// lists in `changes` and the `affects` entries.
///
/// Build it with [`NewChangeRequest::new`] and the adders; `capability` and
/// `affects` may be called more than once.
///
/// A repository that checks the licence of every file (REUSE) needs the record
/// to open with two SPDX comment lines. `copyright` and `licence` give their
/// text; they are written together or not at all.
///
/// ```rust
/// use raf_workplan::lifecycle::NewChangeRequest;
///
/// let request = NewChangeRequest::new("incremental-parsing")
///     .capability("cap:streaming-parser")
///     .affects("cap:streaming-parser#usage")
///     .copyright("2026 Example Org <dev@example.org>")
///     .licence("CC-BY-4.0");
/// assert_eq!(request.capabilities, ["cap:streaming-parser"]);
/// assert_eq!(request.affects, ["cap:streaming-parser#usage"]);
/// assert_eq!(request.licence.as_deref(), Some("CC-BY-4.0"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NewChangeRequest {
    /// The slug of the change, which names the file `work/changes/<slug>.md`.
    pub slug: String,
    /// The ids of the capabilities the change lists, `cap:<name>`. At least one.
    pub capabilities: Vec<String>,
    /// The `affects` entries, `cap:<name>#<section>`. When there are none, the
    /// record affects the `contract` section of each listed capability.
    pub affects: Vec<String>,
    /// The text of the `SPDX-FileCopyrightText` comment line the record opens
    /// with. Required when `licence` is set.
    pub copyright: Option<String>,
    /// The identifier of the `SPDX-License-Identifier` comment line the record
    /// opens with. Required when `copyright` is set.
    pub licence: Option<String>,
}

impl NewChangeRequest {
    /// A request for the change `slug`, listing no capability yet.
    #[must_use]
    pub fn new(slug: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            capabilities: Vec::new(),
            affects: Vec::new(),
            copyright: None,
            licence: None,
        }
    }

    /// List the capability `id` (`cap:<name>`) in `changes`.
    #[must_use]
    pub fn capability(mut self, id: impl Into<String>) -> Self {
        self.capabilities.push(id.into());
        self
    }

    /// Add the `affects` entry `entry` (`cap:<name>#<section>`).
    #[must_use]
    pub fn affects(mut self, entry: impl Into<String>) -> Self {
        self.affects.push(entry.into());
        self
    }

    /// Open the record with the SPDX copyright comment line `text`. Needs
    /// [`licence`](Self::licence) as well.
    #[must_use]
    pub fn copyright(mut self, text: impl Into<String>) -> Self {
        self.copyright = Some(text.into());
        self
    }

    /// Open the record with the SPDX licence comment line `id`. Needs
    /// [`copyright`](Self::copyright) as well.
    #[must_use]
    pub fn licence(mut self, id: impl Into<String>) -> Self {
        self.licence = Some(id.into());
        self
    }
}

/// What the author does to the state of a listed capability in the commit that
/// holds the PROPOSED block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StateAdvice {
    /// The state allows a PROPOSED block and an open record as it is.
    Keep,
    /// Set `state: implementing` in the same commit: a PROPOSED block is a
    /// finding of `check` in a `delivered` and in a `designed` capability.
    SetImplementing,
}

impl StateAdvice {
    /// The stable name of the advice: `keep` or `set-implementing`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Keep => "keep",
            Self::SetImplementing => "set-implementing",
        }
    }

    /// The advice for a capability in `state`.
    #[must_use]
    pub fn for_state(state: State) -> Self {
        match state {
            State::Delivered | State::Designed => Self::SetImplementing,
            State::Considered | State::Implementing => Self::Keep,
        }
    }
}

/// The state guidance for one listed capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CapabilityGuidance {
    /// The id of the capability.
    pub capability: String,
    /// Its state in the loaded documents.
    pub state: State,
    /// What the author does about the state.
    pub advice: StateAdvice,
}

/// A change record that `change new` wrote, and what to do next.
///
/// Its `Display` is the text a command prints: the path, the PROPOSED block
/// template to place, and the guidance for each listed capability.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct NewChange {
    /// The slug of the change.
    pub slug: String,
    /// The repository-relative path of the record, `work/changes/<slug>.md`.
    pub path: RelPath,
    /// The capabilities in `changes`, in the order given, without repeats.
    pub capabilities: Vec<String>,
    /// The sections in `affects`, in the order given, without repeats.
    pub affects: Vec<SectionId>,
    /// The PROPOSED block to place in a capability document: the marker line
    /// and one placeholder line, each ending in a line break.
    pub template: String,
    /// One entry per listed capability, in the order of `capabilities`.
    pub guidance: Vec<CapabilityGuidance>,
    /// Whether the record changes what a sync writes: a generated block it
    /// makes stale (the relationship tables name the changes that list a
    /// capability), or one that was already stale and now reads differently.
    /// The generated blocks must then be synced.
    pub sync_needed: bool,
}

impl fmt::Display for NewChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "created {}", self.path)?;
        writeln!(f, "changes: {}", self.capabilities.join(", "))?;
        let affects: Vec<String> = self.affects.iter().map(ToString::to_string).collect();
        writeln!(f, "affects: {}", affects.join(", "))?;
        writeln!(f)?;
        writeln!(
            f,
            "Place this block in a section named in `affects`, in the document of a listed capability, and replace its second line with the proposed text:"
        )?;
        writeln!(f)?;
        write!(f, "{}", self.template)?;
        writeln!(f)?;
        writeln!(f, "State of the listed capabilities:")?;
        for guidance in &self.guidance {
            let advice = match (guidance.advice, guidance.state) {
                (StateAdvice::SetImplementing, State::Delivered) => {
                    "set `state: implementing` in the same commit as the block, since in this state a PROPOSED block is a finding of `check` (`state-delivered-proposal`) and so is the open record itself (`state-delivered-open-record`)"
                }
                (StateAdvice::SetImplementing, _) => {
                    "set `state: implementing` in the same commit as the block, since a PROPOSED block is a finding of `check` in this state (`state-designed-proposal`)"
                }
                (StateAdvice::Keep, _) => "no change of state is needed",
            };
            writeln!(
                f,
                "  {}  {}  {advice}",
                guidance.capability,
                guidance.state.name()
            )?;
        }
        if self.sync_needed {
            writeln!(f)?;
            writeln!(
                f,
                "The record makes generated blocks stale. Stage the record first, since a link to a file git does not track is a finding of `check`, then bring the blocks up to date:"
            )?;
            writeln!(f, "  git add {}", self.path)?;
            writeln!(f, "  docs sync")?;
        }
        Ok(())
    }
}

/// Why an `affects` entry is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AffectsProblem {
    /// The entry is not `cap:<name>#<section>`: it has no `#`, or the part
    /// before it is not a capability id.
    Malformed,
    /// The part after `#` is not one of the seven sections.
    UnknownSection {
        /// The text after `#`.
        section: String,
    },
    /// The capability is not one the change lists.
    NotListed,
}

/// Why `change new` created nothing, or created a record it then removed.
#[derive(Debug)]
#[non_exhaustive]
pub enum NewChangeError {
    /// The slug is not a lowercase kebab-case slug, so it is not a file name
    /// the profile accepts and may not begin with `-`.
    BadSlug {
        /// The slug as given.
        slug: String,
    },
    /// No capability was named; a change lists at least one.
    NoCapability,
    /// A listed capability is not a capability of the loaded documents.
    UnknownCapability {
        /// The text as given.
        capability: String,
        /// The ids of the capabilities that loaded, sorted.
        known: Vec<String>,
    },
    /// A listed capability has a document that did not load, so its state is
    /// unknown.
    CapabilityNotLoaded {
        /// The id of the capability.
        capability: String,
        /// The findings about the document.
        findings: Vec<Finding>,
    },
    /// An `affects` entry is refused.
    BadAffects {
        /// The entry as given.
        entry: String,
        /// Why.
        problem: AffectsProblem,
    },
    /// A record, or at least a file, is already at the path. It is untouched.
    Exists {
        /// The path of the record.
        path: RelPath,
    },
    /// One of `copyright` and `licence` is given and the other is not; the
    /// record opens with both SPDX lines or with neither.
    SpdxUnset {
        /// The field that is missing: `copyright` or `licence`.
        field: &'static str,
    },
    /// An SPDX field cannot be written as one comment line.
    SpdxInvalid {
        /// The field: `copyright` or `licence`.
        field: &'static str,
        /// Why.
        reason: &'static str,
    },
    /// The record could not be built from the request.
    Build(ConstructError),
    /// The record written does not load clean. It was removed again.
    Invalid {
        /// The path of the record.
        path: RelPath,
        /// The findings about it, when it loaded with findings; none when it
        /// did not load as a change record at all.
        findings: Vec<Finding>,
    },
    /// The record could not be written, or a directory it needs is refused.
    Write {
        /// The path that was being written or judged.
        path: String,
        /// Why.
        source: io::Error,
    },
}

impl fmt::Display for NewChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSlug { slug } => write!(
                f,
                "`{slug}` is not a change slug; use lowercase letters and digits joined by single `-`, starting with a letter or digit, such as `incremental-parsing`"
            ),
            Self::NoCapability => {
                f.write_str("a change lists at least one capability; name one as `cap:<name>`")
            }
            Self::UnknownCapability { capability, known } => {
                write!(f, "`{capability}` is not a capability")?;
                if !capability.starts_with(CAPABILITY_PREFIX) {
                    write!(f, " id (capability ids read `cap:<name>`)")?;
                }
                if known.is_empty() {
                    write!(f, "; there is no capability document")
                } else {
                    write!(f, "; the capabilities are {}", known.join(", "))
                }
            }
            Self::CapabilityNotLoaded {
                capability,
                findings,
            } => {
                write!(
                    f,
                    "{capability} has a document that did not load, so its state is unknown; fix its findings first"
                )?;
                crate::lookup::write_findings(f, findings)
            }
            Self::BadAffects { entry, problem } => {
                write!(f, "the `affects` entry `{entry}` is refused: ")?;
                match problem {
                    AffectsProblem::Malformed => f.write_str("it must read `cap:<name>#<section>`"),
                    AffectsProblem::UnknownSection { section } => write!(
                        f,
                        "`{section}` is not a section; the sections are {}",
                        CapabilitySection::ALL
                            .map(CapabilitySection::slug)
                            .join(", ")
                    ),
                    AffectsProblem::NotListed => {
                        f.write_str("the capability is not one the change lists")
                    }
                }
            }
            Self::Exists { path } => write!(
                f,
                "{path} already exists and is left as it is; choose another slug"
            ),
            Self::SpdxUnset { field } => write!(
                f,
                "the record needs both SPDX lines or neither, and the {field} is not given; pass --copyright <text> and --licence <id> together"
            ),
            Self::SpdxInvalid { field, reason } => write!(
                f,
                "the {field} of the SPDX header of the record is refused: {reason}"
            ),
            Self::Build(error) => write!(f, "the record cannot be built: {error}"),
            Self::Invalid { path, findings } => {
                write!(
                    f,
                    "the record written to {path} did not load clean and was removed"
                )?;
                crate::lookup::write_findings(f, findings)
            }
            Self::Write { path, source } => write!(f, "cannot write {path}: {source}"),
        }
    }
}

impl Error for NewChangeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::Write { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<ConstructError> for NewChangeError {
    fn from(error: ConstructError) -> Self {
        Self::Build(error)
    }
}

/// `items` without repeats, in the order of first appearance.
fn unique<'a>(items: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = BTreeSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(*item))
        .collect()
}

impl Workplan<'_> {
    /// Create the record `work/changes/<slug>.md` of a new change below
    /// `root`, and say what the author does next.
    ///
    /// `root` is the directory that holds `docs/` and `work/`, the directory
    /// the documents were loaded from. The slug is validated by the one slug
    /// pattern of the profile ([`raf_okf::is_kebab`]), every listed capability
    /// must be a loaded capability, and every `affects` entry must name a
    /// listed capability and one of the seven sections. A capability or an
    /// `affects` entry given twice is listed once. With no `affects` entry the
    /// record affects the `contract` section of each listed capability.
    ///
    /// The record has the identity `change:<slug>`, `contract_effect: changes`,
    /// an empty `items` list, a title and a description for the author to
    /// replace, and a non-empty `## Summary`. With `copyright` and `licence`
    /// given it opens with the two SPDX comment lines of the frontmatter,
    /// the copyright tag with `copyright` and the licence tag with `licence`,
    /// each as a `#` comment, as the change records of a
    /// repository that checks licences do; with neither given it opens with
    /// none, and with one only the call is refused before anything is read or
    /// written. It is built with the writer of
    /// `raf-okf` and is the only file written: it is created exclusively, so
    /// a file already at the path is never overwritten, and `work/changes` is
    /// created when it does not exist. The record is then read back through the
    /// loader and must have no finding; a record that has one is removed again
    /// and reported.
    ///
    /// The result carries the PROPOSED block template and the state guidance:
    /// a `delivered` or `designed` capability is set to `state: implementing`
    /// in the commit that holds the block, a `considered` or `implementing`
    /// one needs no change.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    /// use raf_workplan::lifecycle::NewChangeRequest;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let plan = Workplan::load(&MemoryTree::from([("docs/architecture/.keep", "")]), &source);
    /// // The slug is judged first, so nothing is read from disk.
    /// let error = plan
    ///     .new_change(
    ///         std::path::Path::new("."),
    ///         &NewChangeRequest::new("--since=x").capability("cap:parser"),
    ///     )
    ///     .expect_err("a slug that starts with `-` is refused");
    /// assert!(error.to_string().contains("not a change slug"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A [`NewChangeError`]: `BadSlug`, `NoCapability`, `UnknownCapability`,
    /// `CapabilityNotLoaded`, `BadAffects`, `SpdxUnset` or `SpdxInvalid` before
    /// anything is written;
    /// `Exists` when a file is already at the path (also when the record of the
    /// slug is loaded or is there and did not load); `Write` when the file or
    /// `work/changes` cannot be created, in which case no file is left; and
    /// `Invalid` when the record written does not load clean.
    pub fn new_change(
        &self,
        root: &Path,
        request: &NewChangeRequest,
    ) -> Result<NewChange, NewChangeError> {
        let slug = request.slug.as_str();
        if !is_kebab(slug) {
            return Err(NewChangeError::BadSlug {
                slug: slug.to_string(),
            });
        }
        let spdx = spdx_of(request)?;
        let guidance = self.guidance_for(&request.capabilities)?;
        let capabilities: Vec<String> = guidance
            .iter()
            .map(|entry| entry.capability.clone())
            .collect();
        let affects = affects_of(&capabilities, &request.affects)?;
        let path = record_path(slug)?;
        if self.change(slug).is_ok() || self.unloaded().contains(&format!("change:{slug}")) {
            return Err(NewChangeError::Exists { path });
        }
        let document = Document::new(frontmatter(slug, &capabilities, &affects)?, BODY)?;
        let before = pending_sync(self);
        create(root, &path, &with_spdx(document.render(), spdx))?;
        let after = match self.reload(root, &path, slug) {
            Ok(after) => after,
            Err(error) => {
                // The file was created by this call just above; nothing else
                // is removed.
                let _ = fs::remove_file(root.join(path.as_str()));
                return Err(error);
            }
        };
        Ok(NewChange {
            slug: slug.to_string(),
            path,
            capabilities,
            affects,
            template: template(slug),
            guidance,
            sync_needed: after != before,
        })
    }

    /// The guidance of each capability in `ids`, without repeats.
    fn guidance_for(&self, ids: &[String]) -> Result<Vec<CapabilityGuidance>, NewChangeError> {
        let ids = unique(ids.iter().map(String::as_str));
        if ids.is_empty() {
            return Err(NewChangeError::NoCapability);
        }
        ids.into_iter()
            .map(|id| {
                if let Some(cap) = self
                    .capabilities
                    .iter()
                    .find(|cap| cap.value().header.id == id)
                {
                    let state = cap.value().state;
                    return Ok(CapabilityGuidance {
                        capability: id.to_string(),
                        state,
                        advice: StateAdvice::for_state(state),
                    });
                }
                if self.unloaded().iter().any(|unloaded| unloaded == id) {
                    let name = id.strip_prefix(CAPABILITY_PREFIX).unwrap_or(id);
                    return Err(NewChangeError::CapabilityNotLoaded {
                        capability: id.to_string(),
                        findings: self
                            .findings_of(&format!("{}/{name}.md", DocType::Capability.directory())),
                    });
                }
                Err(NewChangeError::UnknownCapability {
                    capability: id.to_string(),
                    known: self
                        .capabilities
                        .iter()
                        .map(|cap| cap.value().header.id.clone())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                })
            })
            .collect()
    }

    /// Load the documents again from `root` and require that the record at
    /// `path` loaded with no finding. The result is what a sync would write
    /// after the load.
    fn reload(&self, root: &Path, path: &RelPath, slug: &str) -> Result<Pending, NewChangeError> {
        let invalid = |findings| NewChangeError::Invalid {
            path: path.clone(),
            findings,
        };
        let tree = DiskTree::new(root).map_err(|error| write_error(path.as_str(), to_io(error)))?;
        let reloaded = Workplan::load(&tree, self.source());
        let findings = reloaded.findings_of(path.as_str());
        if reloaded.change(slug).is_err() || !findings.is_empty() {
            return Err(invalid(findings));
        }
        Ok(pending_sync(&reloaded))
    }
}

/// The path of the record of `slug`, a kebab-case slug. A name the tree
/// refuses (a reserved device name on Windows) is a bad slug.
fn record_path(slug: &str) -> Result<RelPath, NewChangeError> {
    RelPath::new(&format!("{}/{slug}.md", DocType::Change.directory())).map_err(|_| {
        NewChangeError::BadSlug {
            slug: slug.to_string(),
        }
    })
}

/// What a sync would write: the new text of every stale file, and the text of
/// the index when it has to be created.
type Pending = (Vec<(String, String)>, Option<String>);

/// What a sync of `plan` would write. Two plans with the same value need the
/// same sync, so a record that changes it makes a sync necessary, whether the
/// file was current before or already stale.
fn pending_sync(plan: &Workplan<'_>) -> Pending {
    let analysis = generated::analyze(plan);
    (analysis.edits, analysis.new_index)
}

/// The SPDX lines of a request: both, or neither.
fn spdx_of(request: &NewChangeRequest) -> Result<Option<(&str, &str)>, NewChangeError> {
    match (request.copyright.as_deref(), request.licence.as_deref()) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(NewChangeError::SpdxUnset { field: "licence" }),
        (None, Some(_)) => Err(NewChangeError::SpdxUnset { field: "copyright" }),
        (Some(copyright), Some(licence)) => Ok(Some((
            spdx_line("copyright", copyright)?,
            spdx_line("licence", licence)?,
        ))),
    }
}

/// A value of the SPDX header: not empty and one line of text.
fn spdx_line<'a>(field: &'static str, value: &'a str) -> Result<&'a str, NewChangeError> {
    // A control character, and the two separators Unicode adds to the line
    // breaks (U+2028 and U+2029), would end or split the comment line. The
    // value is judged as given, so a line ending at its edge is refused too and
    // not trimmed away.
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(NewChangeError::SpdxInvalid {
            field,
            reason: "it must be one line of text, with no control character and no line or paragraph separator",
        });
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(NewChangeError::SpdxInvalid {
            field,
            reason: "it is empty",
        });
    }
    Ok(value)
}

/// `rendered` with the two SPDX comment lines directly after the opening
/// fence of the frontmatter, where the change records of a repository that
/// checks licences carry them.
fn with_spdx(rendered: String, spdx: Option<(&str, &str)>) -> String {
    let Some((copyright, licence)) = spdx else {
        return rendered;
    };
    let fence = rendered.find('\n').map_or(rendered.len(), |end| end + 1);
    let (head, tail) = rendered.split_at(fence);
    format!("{head}# {COPYRIGHT_TAG}{copyright}\n# {LICENCE_TAG}{licence}\n{tail}")
}

/// The `affects` sections of a request: the entries given, or the `contract`
/// section of each listed capability.
fn affects_of(listed: &[String], entries: &[String]) -> Result<Vec<SectionId>, NewChangeError> {
    if entries.is_empty() {
        return Ok(listed
            .iter()
            .map(|id| SectionId::new(id, CapabilitySection::Contract))
            .collect());
    }
    let mut parsed = Vec::new();
    for entry in unique(entries.iter().map(String::as_str)) {
        let refuse = |problem| NewChangeError::BadAffects {
            entry: entry.to_string(),
            problem,
        };
        let (capability, section) = entry
            .split_once('#')
            .filter(|(capability, _)| {
                capability
                    .strip_prefix(CAPABILITY_PREFIX)
                    .is_some_and(is_kebab)
            })
            .ok_or_else(|| refuse(AffectsProblem::Malformed))?;
        let section = CapabilitySection::from_slug(section).ok_or_else(|| {
            refuse(AffectsProblem::UnknownSection {
                section: section.to_string(),
            })
        })?;
        if !listed.iter().any(|id| id == capability) {
            return Err(refuse(AffectsProblem::NotListed));
        }
        parsed.push(SectionId::new(capability, section));
    }
    Ok(parsed)
}

/// The PROPOSED block template of the change `slug`.
fn template(slug: &str) -> String {
    format!("> [!PROPOSED change:{slug}]\n{TEMPLATE_LINE}\n")
}

/// The frontmatter of a new record.
fn frontmatter(
    slug: &str,
    capabilities: &[String],
    affects: &[SectionId],
) -> Result<Frontmatter, ConstructError> {
    let text = |text: &str| Scalar::new(text).map(Value::Scalar);
    let changes = List::flow(
        capabilities
            .iter()
            .map(Scalar::new)
            .collect::<Result<_, _>>()?,
    )?;
    let affects = List::block(
        affects
            .iter()
            .map(|section| Scalar::new(section.to_string()).and_then(ListItem::scalar))
            .collect::<Result<_, _>>()?,
    )?;
    let entries = vec![
        Entry::new("type", text(DocType::Change.name())?)?,
        Entry::new(
            "id",
            text(&format!("{}{slug}", DocType::Change.id_prefix()))?,
        )?,
        Entry::new("title", text(TITLE)?)?,
        Entry::new("description", text(DESCRIPTION)?)?,
        Entry::new("changes", Value::List(changes))?,
        Entry::new("affects", Value::List(affects))?,
        Entry::new("contract_effect", text("changes")?)?,
        Entry::new("items", Value::List(List::flow(Vec::new())?))?,
    ];
    Ok(Frontmatter::new(Map::new(entries)?))
}

fn to_io(error: raf_source::TreeError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, error)
}

fn write_error(path: &str, source: io::Error) -> NewChangeError {
    NewChangeError::Write {
        path: path.to_string(),
        source,
    }
}

/// Create `path` below `root` with `text`, exclusively.
///
/// `work` and `work/changes` must each be a directory or not exist; a
/// symbolic link or a file there is refused, and a missing directory is
/// created. The file is created with `create_new`, so an existing file or
/// link at the path is an [`NewChangeError::Exists`] and is not touched. A
/// failure after the file was created removes it again.
fn create(root: &Path, path: &RelPath, text: &str) -> Result<(), NewChangeError> {
    let tree = DiskTree::new(root).map_err(|error| write_error(path.as_str(), to_io(error)))?;
    let directory = DocType::Change.directory();
    let mut missing = false;
    let mut prefix = String::new();
    for part in directory.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        let rel = RelPath::new(&prefix).map_err(|error| {
            write_error(&prefix, io::Error::new(io::ErrorKind::InvalidInput, error))
        })?;
        match tree.entry(&rel) {
            Ok(Some(EntryKind::Dir)) => {}
            Ok(None) => missing = true,
            Ok(Some(_)) => {
                return Err(write_error(
                    &prefix,
                    io::Error::new(io::ErrorKind::InvalidInput, "it is not a directory"),
                ));
            }
            Err(error) => return Err(write_error(&prefix, to_io(error))),
        }
    }
    if missing {
        fs::create_dir_all(root.join(directory)).map_err(|error| write_error(directory, error))?;
    }
    let target = root.join(path.as_str());
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(NewChangeError::Exists { path: path.clone() });
        }
        Err(error) => return Err(write_error(path.as_str(), error)),
    };
    let written = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    if let Err(error) = written {
        drop(file);
        // The file was created by this call just above; nothing else is
        // removed.
        let _ = fs::remove_file(&target);
        return Err(write_error(path.as_str(), error));
    }
    Ok(())
}
