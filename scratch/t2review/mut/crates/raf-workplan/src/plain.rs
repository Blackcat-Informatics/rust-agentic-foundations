// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Tracked Markdown outside the document directories: the README, the agent
//! guide, the brand page, the pull request template.
//!
//! These files are not capability, change or architecture documents, so they
//! have no schema. They are held to the rules every Markdown file shares, and
//! the code of those rules is the code [`Workplan::check`](crate::Workplan::check)
//! runs over its documents.

use raf_okf::{Body, Document, ErrorTag, Location, Rule};

use crate::finding::kind;
use crate::links::{self, LinkContext};
use crate::{Finding, refs};

/// Whether a repository path is held to the content rules: Markdown at the
/// top level of the repository and under `docs/` and `work/`. Other Markdown,
/// such as the templates under `.github/` and the README of a crate, is held
/// to the link rules and the split-link rule only.
fn is_scanned(path: &str) -> bool {
    match path.split_once('/') {
        None => true,
        Some((first, _)) => first == "docs" || first == "work",
    }
}

/// Check one tracked Markdown file that is not a workplan document.
///
/// * Every file: a link or image split across lines, a link that does not
///   resolve (`link-broken`, `anchor-broken`, `link-outside`, `link-symlink`),
///   and text that does not read as Markdown (an unclosed frontmatter block, a
///   malformed or unclosed generated marker, a malformed PROPOSED marker, a
///   malformed or unattached claim marker, an unknown `raf:` comment).
/// * Files at the top level and under `docs/` and `work/`: an issue or pull
///   request reference (`issue-reference`), an unfinished-work token
///   (`okf-placeholder`), a PROPOSED block, which belongs in a capability
///   document (`okf-proposed-outside-capability`), whose change must also
///   exist (`proposed-missing-change`), and a claim marker, which belongs in
///   a capability document too (`okf-claim-outside-capability`).
///
/// `path` is the repository-relative path with `/` separators, `text` is the
/// file, and `context` supplies the tracked paths.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use raf_source::{MemoryTree, RelPath};
/// use raf_workplan::{LinkContext, plain};
///
/// let tree = MemoryTree::from([("NOTES.md", "")]);
/// let context = LinkContext::new(&tree, BTreeSet::from([RelPath::new("NOTES.md")?]));
/// let findings = plain::check("NOTES.md", "# Notes\n\n[gone](missing.md)\n", &context);
/// assert_eq!(findings[0].kind, "link-broken");
/// assert!(plain::check("NOTES.md", "# Notes\n\nFine.\n", &context).is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn check(path: &str, text: &str, context: &LinkContext<'_>) -> Vec<Finding> {
    let mut out = Vec::new();
    if is_scanned(path) {
        refs::scan(path, text, &mut out);
    }
    rules(path, text, context, &mut out);
    out
}

/// The rules of a Markdown file that are not about issue references.
pub(crate) fn rules(path: &str, text: &str, context: &LinkContext<'_>, out: &mut Vec<Finding>) {
    let scanned = is_scanned(path);
    let parsed = if scanned {
        match Document::parse(text) {
            Ok(document) => Ok(document.body().clone()),
            Err(error) if error.kind().tag() == ErrorTag::MissingFence => Body::parse(text),
            Err(error) => Err(error),
        }
    } else {
        Body::parse(text)
    };
    let body = match parsed {
        Ok(body) => body,
        Err(error) => {
            out.push(Finding::from_parse_error(path, &error));
            return;
        }
    };
    for split in body.split_links() {
        out.push(Finding::from_rule(
            &Rule::SplitLink { form: split.form() },
            path,
            Location::Line(split.line()),
        ));
    }
    if scanned {
        for (line, token) in body.placeholder_lines() {
            out.push(Finding::from_rule(
                &Rule::Placeholder {
                    token: token.to_string(),
                },
                path,
                Location::Line(line),
            ));
        }
        for block in body.proposed_blocks() {
            let line = Location::Line(block.marker().lines.start);
            out.push(Finding::from_rule(
                &Rule::ProposedOutsideCapability {
                    slug: block.slug().to_string(),
                },
                path,
                line.clone(),
            ));
            let record = format!("work/changes/{}.md", block.slug());
            if !context.is_tracked_file(&record) {
                out.push(Finding::new(
                    kind::PROPOSED_MISSING_CHANGE,
                    path,
                    line,
                    format!(
                        "the PROPOSED block names the change `{}`, but {record} does not exist",
                        block.slug()
                    ),
                    "create the change record, or correct the slug in the marker",
                ));
            }
        }
        for claim in body.claims() {
            out.push(Finding::from_rule(
                &Rule::ClaimOutsideCapability,
                path,
                Location::Line(claim.line()),
            ));
        }
    }
    links::check_body(path, &body, context, out);
}
