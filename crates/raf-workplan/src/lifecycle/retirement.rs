// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Verification and recovery locations for explicit historical removals.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::ops::Range;

use raf_okf::{Body, DecisionSectionRef};
use raf_source::{DEFAULT_MAX_FILE_BYTES, Item, RelPath, SymbolKind, Workspace};

use super::{BaseTree, Lifecycle, LifecycleError, Side};
use crate::escape::shown;
use crate::git;
use crate::identifiers::{Commit, SourcePath};
use crate::lookup::write_findings;
use crate::rationale::{
    architecture_findings, authored_decision_text, authored_headings, decision_blocks, historical,
    retirement_prose,
};
use crate::{Finding, Opened, Workplan};

/// The pinned commit and original locations of the removals verified in the checkout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retirement {
    /// The original commit, equal to HEAD or reachable from it.
    pub base: Commit,
    /// Removed Architecture sections, keyed by their full decision section reference.
    pub sections: BTreeMap<String, RetirementLocation>,
    /// Removed written definitions, keyed by their full indexed path.
    pub definitions: BTreeMap<String, RetirementLocation>,
}

/// A place to recover the original text from the selected commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetirementLocation {
    /// The blob's path from the Git worktree root.
    pub path: SourcePath,
    /// One-based lines, ending just after the retired region.
    pub lines: Range<usize>,
}

impl fmt::Display for Retirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "base {}", self.base)?;
        for (kind, locations) in [
            ("section", &self.sections),
            ("definition", &self.definitions),
        ] {
            for (name, location) in locations {
                writeln!(
                    f,
                    "{kind} {} at {}:{}-{}",
                    shown(name),
                    location.path,
                    location.lines.start,
                    location.lines.end - 1,
                )?;
            }
        }
        Ok(())
    }
}

/// Why an Architecture section prevents retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SectionProblem {
    /// The section is absent from the base or has no replacing edge there.
    NotHistorical,
    /// A heading with the selected anchor still occurs in the page.
    StillPresent,
    /// The selected prose still occurs in the Architecture documents.
    TextRetained,
    /// A previously replaced heading remains without a replacing edge.
    Resurrected,
}

/// Why an indexed source definition prevents retirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceProblem {
    /// The path names no written definition in the base.
    MissingAtBase,
    /// The path names more than one physical definition in the base.
    AmbiguousAtBase,
    /// A module declaration can name implementation in other files.
    ModuleDeclaration,
    /// The path or a matching name in its original file still exists.
    StillPresent,
    /// The file remains on disk but has no source index coverage.
    UnindexedFile,
    /// The original file contains opaque declarations that could hide the definition.
    OpaqueFile,
}

/// Why a retirement cannot be verified.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetirementError {
    /// A native snapshot, source index or historical blob was unavailable.
    Snapshot(LifecycleError),
    /// The selected commit is not reachable from HEAD.
    NotAncestor(Commit),
    /// At least one named section is required.
    NoSections,
    /// Findings prevent comparison or leave invalid references in the checkout.
    Documents {
        /// The version with the findings.
        side: Side,
        /// The original Workplan findings.
        findings: Vec<Finding>,
    },
    /// A section failed the retirement check.
    Section {
        /// The full decision section reference.
        section: String,
        /// The rule it violated.
        problem: SectionProblem,
    },
    /// A page would change the authored text or structure of the current design.
    CurrentDesignChanged(String),
    /// An associated source definition failed the retirement check.
    Definition {
        /// The selected full source index path.
        path: String,
        /// The rule it violated.
        problem: SourceProblem,
    },
    /// The checkout could not establish whether a former source file remains.
    SourceFile(raf_source::TreeError),
}

impl fmt::Display for RetirementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => error.fmt(f),
            Self::NotAncestor(base) => write!(
                f,
                "{base} is not reachable from HEAD; select a commit in this checkout's history"
            ),
            Self::NoSections => f.write_str("name at least one superseded decision section"),
            Self::Documents { side, findings } => {
                write!(
                    f,
                    "fix the Workplan document findings {side} before verifying retirement"
                )?;
                write_findings(f, findings)
            }
            Self::Section { section, problem } => {
                let reason = match problem {
                    SectionProblem::NotHistorical => {
                        "is not superseded at the base; select a historical named section"
                    }
                    SectionProblem::StillPresent => {
                        "still has a heading in the checkout; delete the selected section"
                    }
                    SectionProblem::TextRetained => {
                        "still has its complete text in the checkout; remove the retired wording"
                    }
                    SectionProblem::Resurrected => {
                        "survives without a replacing edge; remove it or keep it superseded"
                    }
                };
                write!(f, "decision section `{section}` {reason}")
            }
            Self::CurrentDesignChanged(page) => write!(
                f,
                "`{page}` would alter the current design; preserve all current decision blocks and remove obsolete prose"
            ),
            Self::Definition { path, problem } => {
                let reason = match problem {
                    SourceProblem::MissingAtBase => {
                        "has no written definition at the base; use its defining path"
                    }
                    SourceProblem::AmbiguousAtBase => {
                        "names multiple written definitions at the base; select an unambiguous definition"
                    }
                    SourceProblem::ModuleDeclaration => {
                        "is a module declaration; select the concrete definitions in its implementation"
                    }
                    SourceProblem::StillPresent => {
                        "still has a matching written declaration; remove the selected definition"
                    }
                    SourceProblem::UnindexedFile => {
                        "left the index while its file remains; delete the file or restore its index coverage"
                    }
                    SourceProblem::OpaqueFile => {
                        "has opaque declarations in its former file; expose the declarations to indexing or remove the file"
                    }
                };
                write!(f, "source definition `{}` {reason}", shown(path))
            }
            Self::SourceFile(error) => {
                write!(f, "cannot verify removal from the checkout: {error}")
            }
        }
    }
}

impl std::error::Error for RetirementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Snapshot(error) => Some(error),
            Self::SourceFile(error) => Some(error),
            _ => None,
        }
    }
}

impl From<LifecycleError> for RetirementError {
    fn from(error: LifecycleError) -> Self {
        Self::Snapshot(error)
    }
}

impl Lifecycle {
    /// Verify removal of named historical sections and explicitly associated
    /// written definitions. The selected revision is HEAD or an ancestor of HEAD.
    ///
    /// Surviving historical headings must remain superseded. The original
    /// complete text of each retired section must occur fewer times in
    /// Architecture bodies. The authored current design must remain the same
    /// apart from source locations and surrounding blank lines. The checkout
    /// must have valid loaded documents and bindings.
    /// Definition checks use private as well as public indexed items.
    ///
    /// # Errors
    /// Refuses invalid inputs, unresolved document references, missing Git data,
    /// current sections, incomplete removals, or an unprovable source removal.
    pub fn verify_retirement(
        &self,
        base: &str,
        sections: &[DecisionSectionRef],
        definitions: &[&str],
    ) -> Result<Retirement, RetirementError> {
        if sections.is_empty() {
            return Err(RetirementError::NoSections);
        }
        let base = git::resolve_revision(&self.repository, base).map_err(LifecycleError::from)?;
        let common =
            git::merge_base(&self.repository, base.commit()).map_err(LifecycleError::from)?;
        if common.commit() != base.commit() {
            return Err(RetirementError::NotAncestor(base.commit().clone()));
        }
        let (tree, source) = self.open_revision(&base)?;
        let before = tree.workplan(&source)?;
        let after = self.opened.workplan();
        require_documents(Side::Revision, architecture_findings(&before))?;
        require_documents(Side::WorkingTree, after.findings().to_vec())?;
        Ok(Retirement {
            base: base.commit().clone(),
            sections: retire_sections(&tree, &before, &after, sections)?,
            definitions: retire_definitions(&tree, &source, &self.opened, definitions)?,
        })
    }
}

fn require_documents(side: Side, findings: Vec<Finding>) -> Result<(), RetirementError> {
    if findings.is_empty() {
        Ok(())
    } else {
        Err(RetirementError::Documents { side, findings })
    }
}

struct Page<'a> {
    path: &'a str,
    body: &'a Body,
    anchors: BTreeSet<&'a str>,
    named: BTreeMap<&'a str, Range<usize>>,
    current: String,
}

fn pages<'a>(
    plan: &'a Workplan<'_>,
    replaced: &BTreeSet<(&str, &str)>,
) -> BTreeMap<&'a str, Page<'a>> {
    let mut pages = BTreeMap::new();
    for page in plan.architecture() {
        let body = page.body();
        let headings = authored_headings(body);
        let anchors = body
            .headings()
            .iter()
            .map(raf_okf::Heading::anchor)
            .collect();
        let mut named = BTreeMap::new();
        for block in decision_blocks(&headings, body.span().lines.end) {
            for (index, &section) in block.sections.iter().enumerate() {
                let end = block
                    .sections
                    .get(index + 1)
                    .map_or(block.end, |next| next.line());
                named.insert(section.anchor(), section.line()..end);
            }
        }
        pages.insert(
            page.value().header.id.as_str(),
            Page {
                path: page.path(),
                body,
                anchors,
                named,
                current: authored_decision_text(page, &headings, replaced),
            },
        );
    }
    pages
}

fn retire_sections(
    tree: &BaseTree<'_>,
    before: &Workplan<'_>,
    after: &Workplan<'_>,
    sections: &[DecisionSectionRef],
) -> Result<BTreeMap<String, RetirementLocation>, RetirementError> {
    let old_historical = historical(before);
    let new_historical = historical(after);
    let old_pages = pages(before, &old_historical);
    let new_pages = pages(after, &new_historical);
    let mut retired = BTreeMap::new();
    let mut removed_text = BTreeMap::new();
    let mut selected_lines = BTreeMap::<_, Vec<_>>::new();
    for section in sections {
        let id = section.architecture();
        let anchor = section.section();
        let Some((page, lines)) = old_pages
            .get(id)
            .and_then(|page| page.named.get(anchor).map(|lines| (page, lines)))
            .filter(|_| old_historical.contains(&(id, anchor)))
        else {
            return Err(section_error(id, anchor, SectionProblem::NotHistorical));
        };
        if has_anchor(&new_pages, id, anchor) {
            return Err(section_error(id, anchor, SectionProblem::StillPresent));
        }
        let original = tree.recovery_file(page.path)?;
        retired.insert(
            section.to_string(),
            RetirementLocation {
                path: tree.git_path(original.as_str())?,
                lines: lines.clone(),
            },
        );
        selected_lines.entry(id).or_default().push(lines.clone());
        for prose in retirement_prose(page.body, std::iter::once(lines.start + 1..lines.end)) {
            if prose != "\n" {
                removed_text.entry(prose).or_insert((id, anchor));
            }
        }
    }
    for &(id, anchor) in &old_historical {
        if has_anchor(&new_pages, id, anchor) && !new_historical.contains(&(id, anchor)) {
            return Err(section_error(id, anchor, SectionProblem::Resurrected));
        }
    }
    for &id in old_pages.keys().chain(new_pages.keys()) {
        let old = old_pages.get(id).map_or("", |page| page.current.as_str());
        let new = new_pages.get(id).map_or("", |page| page.current.as_str());
        if old != new {
            return Err(RetirementError::CurrentDesignChanged(id.to_owned()));
        }
    }
    require_prose_removal(&old_pages, &new_pages, selected_lines, removed_text)?;
    Ok(retired)
}

fn require_prose_removal(
    before: &BTreeMap<&str, Page<'_>>,
    after: &BTreeMap<&str, Page<'_>>,
    mut selected_lines: BTreeMap<&str, Vec<Range<usize>>>,
    removed: BTreeMap<String, (&str, &str)>,
) -> Result<(), RetirementError> {
    if removed.is_empty() {
        return Ok(());
    }
    // A selected section never supplies allowance for another selected
    // section, even when the shorter text occurs in the longer section.
    let old: Vec<_> = before
        .iter()
        .flat_map(|(&id, page)| {
            let mut removed = selected_lines.remove(id).unwrap_or_default();
            removed.sort_unstable_by_key(|range| range.start);
            let whole = &page.body.span().lines;
            let mut start = whole.start;
            let kept = removed
                .into_iter()
                .chain(std::iter::once(whole.end..whole.end))
                .filter_map(move |range| {
                    let keep = start..range.start;
                    start = start.max(range.end);
                    (!keep.is_empty()).then_some(keep)
                });
            retirement_prose(page.body, kept)
        })
        .collect();
    let new: Vec<_> = after
        .values()
        .flat_map(|page| {
            retirement_prose(page.body, std::iter::once(page.body.span().lines.clone()))
        })
        .collect();
    for (prose, (id, anchor)) in removed {
        let count = |pages: &[String]| {
            pages
                .iter()
                .map(|page| complete_occurrences(page, &prose))
                .sum::<usize>()
        };
        if count(&new) > count(&old) {
            return Err(section_error(id, anchor, SectionProblem::TextRetained));
        }
    }
    Ok(())
}

fn complete_occurrences(text: &str, prose: &str) -> usize {
    let Some(first_line) = prose.lines().nth(1) else {
        return 0;
    };
    let start = format!("\n{first_line}");
    text.match_indices(&start)
        .filter(|(offset, _)| {
            text.get(*offset..)
                .is_some_and(|rest| rest.starts_with(prose))
        })
        .count()
}

fn has_anchor(pages: &BTreeMap<&str, Page<'_>>, id: &str, anchor: &str) -> bool {
    pages
        .get(id)
        .is_some_and(|page| page.anchors.contains(anchor))
}

fn section_error(id: &str, anchor: &str, problem: SectionProblem) -> RetirementError {
    RetirementError::Section {
        section: format!("{id}#{anchor}"),
        problem,
    }
}

fn retire_definitions(
    tree: &BaseTree<'_>,
    before: &Workspace,
    after: &Opened,
    definitions: &[&str],
) -> Result<BTreeMap<String, RetirementLocation>, RetirementError> {
    definitions
        .iter()
        .map(|&path| {
            let refused = |problem| definition_error(path, problem);
            let mut candidates = exact_items(before, path);
            let item = candidates
                .next()
                .ok_or_else(|| refused(SourceProblem::MissingAtBase))?;
            if candidates.any(|other| other.file != item.file || other.span != item.span) {
                return Err(refused(SourceProblem::AmbiguousAtBase));
            }
            if item.kind == SymbolKind::Mod {
                return Err(refused(SourceProblem::ModuleDeclaration));
            }
            let current = after.workspace();
            if exact_items(current, path).next().is_some() {
                return Err(refused(SourceProblem::StillPresent));
            }
            let original = tree.recovery_file(item.file.as_str())?;
            require_source_absence(after, item, &item.file, current.source(&original))?;
            if original != item.file {
                require_source_absence(after, item, &original, current.source(&item.file))?;
            }
            Ok((
                path.to_owned(),
                RetirementLocation {
                    path: tree.git_path(original.as_str())?,
                    lines: item.span.lines.clone(),
                },
            ))
        })
        .collect()
}

fn require_source_absence(
    checkout: &Opened,
    item: &Item,
    file: &RelPath,
    alias_text: Option<&str>,
) -> Result<(), RetirementError> {
    let source = checkout.workspace();
    let refuse = |problem| definition_error(item.path.as_str(), problem);
    if source.file(file).is_none() {
        if checkout
            .tree()
            .entry(file)
            .map_err(RetirementError::SourceFile)?
            .is_none()
        {
            return Ok(());
        }
        if let Some(indexed) = alias_text
            && checkout
                .tree()
                .read(file, DEFAULT_MAX_FILE_BYTES)
                .map_err(RetirementError::SourceFile)?
                == indexed
        {
            return Ok(());
        }
        return Err(refuse(SourceProblem::UnindexedFile));
    }
    if source
        .items_in(file)
        .any(|other| other.kind == item.kind && other.name == item.name)
    {
        return Err(refuse(SourceProblem::StillPresent));
    }
    let disabled = source.disabled_syntax();
    let first_disabled = disabled.partition_point(|opaque| &opaque.file < file);
    let unexpanded = source.unexpanded();
    let first_unexpanded = unexpanded.partition_point(|opaque| &opaque.file < file);
    if disabled
        .get(first_disabled)
        .is_some_and(|opaque| &opaque.file == file)
        || unexpanded[first_unexpanded..]
            .iter()
            .take_while(|opaque| &opaque.file == file)
            .any(|opaque| opaque.identifiers.contains(&item.name))
    {
        return Err(refuse(SourceProblem::OpaqueFile));
    }
    Ok(())
}

fn definition_error(path: &str, problem: SourceProblem) -> RetirementError {
    RetirementError::Definition {
        path: path.to_owned(),
        problem,
    }
}

fn exact_items<'a>(source: &'a Workspace, path: &'a str) -> impl Iterator<Item = &'a Item> {
    source
        .items_under(path)
        .take_while(move |item| item.path.as_str() == path)
}
