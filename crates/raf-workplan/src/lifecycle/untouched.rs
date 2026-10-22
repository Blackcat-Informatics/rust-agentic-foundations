// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! `section-untouched`: a section the change affects whose authored text is
//! the text it had at the merge base.
//!
//! "Authored text" is what a person writes. The text of a generated block
//! (`<!-- raf:generated:... -->` up to its end marker) is written by
//! `docs sync` and moves whenever a binding or a relation moves, so it is left
//! out of the comparison on both sides; the marker lines, fences, PROPOSED
//! blocks and every other line stay. A section whose only change is a
//! regenerated block is therefore still owed.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

use raf_okf::{Body, CapabilitySection, ContractEffect, MAX_TEXT_BYTES};
use raf_source::{RelPath, Tree};

use crate::SectionId;
use crate::exec::{Runner, SourcePath};
use crate::git::{self, MergeBase, Prefix};
use crate::lookup::capability_path;
use crate::obligation::{Needed, Obligation, ObligationKind, Part, Place, Subject, suppressed};
use crate::{Opened, lifecycle::Impact};

use super::session::{LifecycleError, Side};

/// The authored text of one section of a body: its heading and its content
/// with the content of every generated block taken out, in LF line endings.
///
/// Two digests are equal exactly when a person wrote the same text. The value
/// holds the text itself and not a hash, so equal digests never collide; its
/// `Debug` shows the length only.
#[derive(Clone, PartialEq, Eq)]
pub struct SectionDigest(String);

impl SectionDigest {
    /// The authored text the digest stands for.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SectionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SectionDigest({} bytes)", self.0.len())
    }
}

/// The digest of the section `section` of `body`, or `None` when the body has
/// no such section.
///
/// The text is the heading and the content of the section. The content of
/// each generated block inside it is taken out, between its two marker lines
/// and not the markers, so a regenerated block does not change the digest and
/// an added or removed marker does. CRLF line endings count as LF, so a
/// checkout that converts line endings does not touch every section.
///
/// Every other difference is a difference of authored text: a whitespace-only
/// edit (trailing spaces, a blank line, a final newline) and a section moved to
/// another place in the file both make the digest differ, so either clears a
/// `section-untouched` obligation. The body is read with [`Body::parse`] and is
/// not validated against the profile, so a heading that appears twice (a
/// document at the merge base that the loader would refuse) is digested from its
/// first occurrence.
///
/// ```rust
/// use raf_okf::{Body, CapabilitySection};
/// use raf_workplan::lifecycle::section_digest;
///
/// let before = Body::parse(
///     "# T\n\n## Contract\n\nIt reads.\n<!-- raf:generated:matrix -->\nold row\n<!-- raf:generated:end -->\n",
/// )?;
/// let synced = Body::parse(
///     "# T\n\n## Contract\n\nIt reads.\n<!-- raf:generated:matrix -->\nnew row\n<!-- raf:generated:end -->\n",
/// )?;
/// let edited = Body::parse(
///     "# T\n\n## Contract\n\nIt reads and writes.\n<!-- raf:generated:matrix -->\nold row\n<!-- raf:generated:end -->\n",
/// )?;
/// let digest = |body: &Body| section_digest(body, CapabilitySection::Contract);
/// assert_eq!(digest(&before), digest(&synced));
/// assert_ne!(digest(&before), digest(&edited));
/// assert_eq!(section_digest(&before, CapabilitySection::Usage), None);
/// # Ok::<(), raf_okf::ParseError>(())
/// ```
#[must_use]
pub fn section_digest(body: &Body, section: CapabilitySection) -> Option<SectionDigest> {
    let found = body
        .sections()
        .iter()
        .find(|candidate| candidate.slug() == section.slug())?;
    let text = body.section_text(found)?;
    let base = body.span().bytes.start;
    let start = found.content().bytes.start.checked_sub(base)?;
    let end = found.content().bytes.end.checked_sub(base)?;

    let mut blanked: Vec<Range<usize>> = body
        .generated_blocks()
        .iter()
        .filter_map(|block| {
            let bytes = &block.content().bytes;
            let from = bytes.start.checked_sub(base)?.max(start);
            let to = bytes.end.checked_sub(base)?.min(end);
            (from < to).then_some(from..to)
        })
        .collect();
    blanked.sort_by_key(|range| range.start);

    let mut authored = String::with_capacity(text.len());
    authored.push_str(found.heading());
    authored.push('\n');
    let mut at = start;
    for range in blanked {
        if range.start >= at {
            authored.push_str(body.text().get(at..range.start)?);
            at = range.end;
        }
    }
    authored.push_str(body.text().get(at..end)?);
    Some(SectionDigest(authored.replace("\r\n", "\n")))
}

/// The `section-untouched` obligations of a change: the affected sections
/// whose authored text is the text at the merge base.
///
/// Nothing is asked of `git` when every section of `affects` is left out by
/// the effect of the change ([`suppressed`]). Otherwise the merge base is found once
/// ([`git::default_merge_base`], kept in `base` for the other process-backed
/// kinds) and each capability file is read once at it.
pub(super) fn raise(
    opened: &Opened,
    runner: &dyn Runner,
    prefix: &Prefix,
    impact: &Impact,
    effect: ContractEffect,
    base: &mut Option<MergeBase>,
) -> Result<Vec<Obligation>, LifecycleError> {
    let wanted: Vec<&SectionId> = impact
        .sections
        .iter()
        .filter(|id| {
            !suppressed(
                ObligationKind::SectionUntouched,
                Part::Section(id.section()),
                effect,
            )
        })
        .collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    let base = merge_base(base, runner)?;
    let mut files: BTreeMap<String, Files> = BTreeMap::new();
    let mut owed = Vec::new();
    for id in wanted {
        let path = capability_path(id.capability()).ok_or_else(|| LifecycleError::Document {
            path: id.capability().to_string(),
            side: Side::WorkingTree,
            reason: "is not a capability id".to_string(),
        })?;
        if !files.contains_key(&path) {
            let read = Files::read(opened, runner, base, prefix, &path)?;
            files.insert(path.clone(), read);
        }
        let Some(file) = files.get(&path) else {
            continue;
        };
        if file.untouched(id.section(), &path)? {
            owed.push(Obligation {
                kind: ObligationKind::SectionUntouched,
                subject: Subject::Capability(id.capability().to_string()),
                section: Part::Section(id.section()),
                location: Place { path, line: None },
                evidence: Needed::SectionDiff,
            });
        }
    }
    Ok(owed)
}

/// The merge base of `HEAD` and the default branch, asked of `git` the first
/// time a kind needs it and kept in `slot` for the next.
pub(super) fn merge_base<'b>(
    slot: &'b mut Option<MergeBase>,
    runner: &dyn Runner,
) -> Result<&'b MergeBase, LifecycleError> {
    let base = match slot.take() {
        Some(base) => base,
        None => git::default_merge_base(runner)?,
    };
    Ok(slot.insert(base))
}

/// The file `path` (repository-relative, as the loaded documents name it) at
/// the merge base, named from the top of the work tree, or `None` when it is
/// not there.
pub(super) fn blob(
    runner: &dyn Runner,
    base: &MergeBase,
    prefix: &Prefix,
    path: &str,
) -> Result<Option<Vec<u8>>, LifecycleError> {
    let document = |reason: String| LifecycleError::Document {
        path: path.to_string(),
        side: Side::WorkingTree,
        reason,
    };
    let source = SourcePath::new(path)
        .map_err(|error| document(format!("is not a repository path: {error}")))?;
    // `git` names the file from the top of the work tree.
    let named = prefix.join(&source).map_err(|error| {
        document(format!(
            "cannot be named below `{}`: {error}",
            prefix.as_str()
        ))
    })?;
    Ok(git::blob_at(runner, base, &named)?)
}

/// One capability file at the merge base and in the working tree.
struct Files {
    /// The body at the merge base, or `None` when the file is not there.
    base: Option<Body>,
    now: Body,
}

impl Files {
    fn read(
        opened: &Opened,
        runner: &dyn Runner,
        base: &MergeBase,
        prefix: &Prefix,
        path: &str,
    ) -> Result<Self, LifecycleError> {
        let document = |side, reason: String| LifecycleError::Document {
            path: path.to_string(),
            side,
            reason,
        };
        let base = blob(runner, base, prefix, path)?
            .map(|bytes| {
                let text = String::from_utf8(bytes)
                    .map_err(|_| document(Side::MergeBase, "is not UTF-8 text".to_string()))?;
                Body::parse(&text).map_err(|error| document(Side::MergeBase, error.to_string()))
            })
            .transpose()?;
        let relative = RelPath::new(path)
            .map_err(|error| document(Side::WorkingTree, format!("is not a tree path: {error}")))?;
        let text = opened
            .tree()
            .read(&relative, MAX_TEXT_BYTES as u64)
            .map_err(|error| document(Side::WorkingTree, error.to_string()))?;
        let now =
            Body::parse(&text).map_err(|error| document(Side::WorkingTree, error.to_string()))?;
        Ok(Self { base, now })
    }

    /// Whether the section has the authored text it had at the merge base. A
    /// file or a section that is not there counts as touched.
    fn untouched(&self, section: CapabilitySection, path: &str) -> Result<bool, LifecycleError> {
        let Some(now) = section_digest(&self.now, section) else {
            return Err(LifecycleError::Document {
                path: path.to_string(),
                side: Side::WorkingTree,
                reason: format!("has no `{}` section", section.slug()),
            });
        };
        let Some(base) = &self.base else {
            return Ok(false);
        };
        Ok(section_digest(base, section).is_some_and(|then| then == now))
    }
}
