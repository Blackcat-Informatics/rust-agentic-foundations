// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! [`Workplan::preview`]: a capability document as it reads once a change is
//! reconciled.
//!
//! The result is a pure function of the loaded documents. It returns text and
//! writes nothing; a caller that wants a file writes it.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use raf_okf::{Body, ProposedBlock, Span, SpliceError};
use raf_source::RelPath;

use crate::{Finding, Workplan};

/// The prefix a change id carries.
const CHANGE_PREFIX: &str = "change:";

/// One capability document as it reads once a change is reconciled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// The id of the capability, such as `cap:parser`.
    pub capability: String,
    /// The repository-relative path of the capability document.
    pub path: RelPath,
    /// The whole text of the document with every PROPOSED block of the change
    /// replaced by its content. Everything else is the document, byte for byte.
    pub text: String,
}

/// Why a preview cannot be produced. Nothing is written in any case.
#[derive(Debug)]
#[non_exhaustive]
pub enum PreviewError {
    /// No change record has the slug.
    UnknownChange {
        /// The slug as asked for, without a `change:` prefix.
        slug: String,
        /// The slugs of the change records that loaded, sorted.
        known: Vec<String>,
    },
    /// The change record exists and did not load, so what it lists is unknown.
    ChangeNotLoaded {
        /// The slug.
        slug: String,
        /// The findings about the record.
        findings: Vec<Finding>,
    },
    /// A capability the change lists exists and did not load, so its preview
    /// would be missing.
    CapabilityNotLoaded {
        /// The slug of the change.
        slug: String,
        /// The id of the capability.
        capability: String,
        /// The findings about the capability document.
        findings: Vec<Finding>,
    },
    /// A capability the change lists has no document.
    CapabilityMissing {
        /// The slug of the change.
        slug: String,
        /// The id of the capability.
        capability: String,
    },
    /// The quote prefix of a PROPOSED block cannot be removed unambiguously.
    Prefix {
        /// The capability document.
        file: String,
        /// The 1-based line of the offending line.
        line: usize,
        /// Why the prefix is ambiguous.
        reason: &'static str,
    },
    /// The reconciled text does not read back as a document.
    Rewrite {
        /// The capability document.
        file: String,
        /// Why.
        reason: String,
    },
}

impl fmt::Display for PreviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownChange { slug, known } => {
                write!(f, "no change record `{slug}`; ")?;
                if known.is_empty() {
                    write!(f, "there is no open change")
                } else {
                    write!(f, "the open changes are {}", known.join(", "))
                }
            }
            Self::ChangeNotLoaded { slug, findings } => {
                write!(
                    f,
                    "the record of change `{slug}` did not load, so what it lists is unknown; fix its findings first"
                )?;
                list(f, findings)
            }
            Self::CapabilityNotLoaded {
                slug,
                capability,
                findings,
            } => {
                write!(
                    f,
                    "change `{slug}` lists {capability}, which did not load; fix its findings first"
                )?;
                list(f, findings)
            }
            Self::CapabilityMissing { slug, capability } => write!(
                f,
                "change `{slug}` lists {capability}, which has no document"
            ),
            Self::Prefix { file, line, reason } => write!(
                f,
                "{file}:{line}: the quote prefix of a PROPOSED block cannot be removed: {reason}"
            ),
            Self::Rewrite { file, reason } => {
                write!(f, "the reconciled {file} does not read back: {reason}")
            }
        }
    }
}

/// The findings, one per line.
fn list(f: &mut fmt::Formatter<'_>, findings: &[Finding]) -> fmt::Result {
    for finding in findings {
        write!(f, "\n  {finding}")?;
    }
    Ok(())
}

impl Error for PreviewError {}

impl Workplan<'_> {
    /// Each capability document that the change `change` lists, as it reads
    /// once that change is reconciled, sorted by capability id.
    ///
    /// `change` is the slug of a change record, with or without the `change:`
    /// prefix. In each listed capability every PROPOSED block of this change
    /// is replaced by its content: the marker line goes, and so does the
    /// quote marker (`>` and one space after it, if there is one) of each
    /// following line, after the block's own indentation, which stays. The
    /// blocks of other changes and all text outside blocks are kept byte for
    /// byte, and every line keeps its own line ending. A listed capability
    /// with no block of the change is returned unchanged.
    ///
    /// The result is a pure function of the loaded documents: it returns text
    /// and writes nothing.
    ///
    /// ```rust
    /// use raf_source::{MemoryTree, Workspace};
    /// use raf_workplan::Workplan;
    ///
    /// let source = Workspace::load(
    ///     &MemoryTree::from([
    ///         ("Cargo.toml", "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n"),
    ///         ("src/lib.rs", ""),
    ///     ]),
    ///     &["."],
    /// )?;
    /// let docs = MemoryTree::from([(
    ///     "docs/architecture/overview.md",
    ///     "---\ntype: Architecture\nid: arch:overview\ntitle: Overview\n\
    ///      description: How it fits.\n---\n\n# Overview\n\nBody text.\n",
    /// )]);
    /// let plan = Workplan::load(&docs, &source);
    /// // There is no change record, so there is nothing to preview.
    /// assert!(plan.preview("grow").is_err());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`PreviewError::UnknownChange`] when no change record has the slug;
    /// [`PreviewError::ChangeNotLoaded`] when the record did not load;
    /// [`PreviewError::CapabilityNotLoaded`] or
    /// [`PreviewError::CapabilityMissing`] when a listed capability has no
    /// loaded document; [`PreviewError::Prefix`] when the quote prefix of a
    /// line of a block cannot be removed unambiguously (a line that is not
    /// indented as the marker line is, or a quote that goes on into a fenced
    /// code block, which ends a block in the document model); [`PreviewError::Rewrite`] when a
    /// reconciled text does not read back. No text is returned with an error.
    pub fn preview(&self, change: &str) -> Result<Vec<Preview>, PreviewError> {
        let slug = change.strip_prefix(CHANGE_PREFIX).unwrap_or(change);
        let id = format!("{CHANGE_PREFIX}{slug}");
        let Some(record) = self
            .changes
            .iter()
            .find(|record| record.value().header.id == id)
        else {
            return Err(self.unknown_change(slug, &id));
        };
        let listed: BTreeSet<&String> = record.value().changes.iter().collect();
        let mut previews = Vec::new();
        for capability in listed {
            let Some(cap) = self
                .capabilities
                .iter()
                .find(|cap| &cap.value().header.id == capability)
            else {
                return Err(self.missing_capability(slug, capability));
            };
            let blocks: Vec<&ProposedBlock> = cap
                .body()
                .proposed_blocks()
                .iter()
                .filter(|block| block.slug() == slug)
                .collect();
            let mut replacements = Vec::with_capacity(blocks.len());
            for block in &blocks {
                let text = reconcile(cap.path(), cap.body(), block)?;
                replacements.push((block.span().clone(), text));
            }
            let edits: Vec<(Span, &str)> = replacements
                .iter()
                .map(|(span, text)| (span.clone(), text.as_str()))
                .collect();
            let mut body = cap.body().clone();
            body.replace_spans(&edits)
                .map_err(|error: SpliceError| PreviewError::Rewrite {
                    file: cap.path().to_string(),
                    reason: error.to_string(),
                })?;
            let path = RelPath::new(cap.path()).map_err(|error| PreviewError::Rewrite {
                file: cap.path().to_string(),
                reason: error.to_string(),
            })?;
            previews.push(Preview {
                capability: capability.clone(),
                path,
                text: format!("{}{}", cap.head(), body.text()),
            });
        }
        Ok(previews)
    }

    /// The error for a slug that no loaded record has.
    fn unknown_change(&self, slug: &str, id: &str) -> PreviewError {
        if self.unloaded().iter().any(|unloaded| unloaded == id) {
            return PreviewError::ChangeNotLoaded {
                slug: slug.to_string(),
                findings: self.findings_of(&format!("work/changes/{slug}.md")),
            };
        }
        let mut known: Vec<String> = self
            .changes
            .iter()
            .filter_map(|record| record.value().header.id.strip_prefix(CHANGE_PREFIX))
            .map(str::to_string)
            .collect();
        known.sort();
        PreviewError::UnknownChange {
            slug: slug.to_string(),
            known,
        }
    }

    /// The error for a listed capability that is not among the loaded ones.
    fn missing_capability(&self, slug: &str, capability: &str) -> PreviewError {
        if self
            .unloaded()
            .iter()
            .any(|unloaded| unloaded == capability)
        {
            let stem = capability.strip_prefix("cap:").unwrap_or(capability);
            return PreviewError::CapabilityNotLoaded {
                slug: slug.to_string(),
                capability: capability.to_string(),
                findings: self.findings_of(&format!("docs/capabilities/{stem}.md")),
            };
        }
        PreviewError::CapabilityMissing {
            slug: slug.to_string(),
            capability: capability.to_string(),
        }
    }

    /// The findings of the load about the file `path`.
    fn findings_of(&self, path: &str) -> Vec<Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.file == path)
            .cloned()
            .collect()
    }
}

/// The text a block becomes: its lines after the marker, each without the
/// quote marker that follows the marker line's indentation.
fn reconcile(file: &str, body: &Body, block: &ProposedBlock) -> Result<String, PreviewError> {
    let (body_text, base) = (body.text(), body.span().bytes.start);
    let whole = &body_text[block.span().bytes.start - base..block.span().bytes.end - base];
    let marker = &whole[..block.marker().bytes.len()];
    let indent = &marker[..marker.len() - marker.trim_start().len()];
    let first = block.span().lines.start;
    // The document model ends a block at a fenced code block, so a quote
    // that goes on after the block is a block whose end cannot be told.
    let after = &body_text[block.span().bytes.end - base..];
    let next = after.lines().next().unwrap_or("");
    let starts_block = body
        .proposed_blocks()
        .iter()
        .any(|other| other.span().bytes.start == block.span().bytes.end);
    if next.trim_start().starts_with('>') && !starts_block {
        return Err(PreviewError::Prefix {
            file: file.to_string(),
            line: block.span().lines.end,
            reason: "the quote goes on after the block, into a fenced code block, which ends a block in the document model; the end of the block is ambiguous",
        });
    }
    let mut out = String::new();
    for (offset, line) in whole[marker.len()..].split_inclusive('\n').enumerate() {
        let at = first + 1 + offset;
        let prefix = |reason| PreviewError::Prefix {
            file: file.to_string(),
            line: at,
            reason,
        };
        let (content, ending) = match line.strip_suffix("\r\n") {
            Some(content) => (content, "\r\n"),
            None => match line.strip_suffix('\n') {
                Some(content) => (content, "\n"),
                None => (line, ""),
            },
        };
        let rest = content
            .strip_prefix(indent)
            .ok_or_else(|| prefix("the line is not indented as the marker line is"))?;
        let quoted = rest
            .strip_prefix('>')
            .ok_or_else(|| prefix("the line has no quote marker after the block's indentation"))?;
        let text = quoted.strip_prefix(' ').unwrap_or(quoted);
        if !text.is_empty() {
            out.push_str(indent);
            out.push_str(text);
        }
        out.push_str(ending);
    }
    Ok(out)
}
