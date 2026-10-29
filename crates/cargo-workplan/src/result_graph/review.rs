// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Typed query results; authored review claims use the library's semantic dataset.

use super::{Answer, Builder, child, class, number, optional, strings, text};
use raf_workplan::review::{ReviewCoverage, ReviewStatus};

pub(super) fn status_fields(
    g: &mut Builder,
    id: &str,
    scope: &[String],
    kind: raf_okf::ReviewKind,
    status: &ReviewStatus,
) -> Answer<()> {
    class(g, id, "ReviewStatus")?;
    text(g, id, "kind", kind.name())?;
    strings(g, id, "scope", "Selector", "value", scope)?;
    optional(g, id, "review", status.review.as_deref())?;
    optional(g, id, "revision", status.revision.as_deref())?;
    let state = if status.review.is_none() {
        "never-reviewed"
    } else if status.is_current() {
        "current"
    } else {
        "stale"
    };
    text(g, id, "state", state)?;
    if let Some(revision) = &status.revision {
        text(g, id, "diffCommand", &format!("git diff {revision} --"))?;
    }
    for (ordinal, change) in status.changes.iter().enumerate() {
        let node = child(g, id, "change", "ReviewChange", ordinal)?;
        text(g, &node, "item", &change.item)?;
        text(g, &node, "kind", change.kind.name())?;
    }
    Ok(())
}

pub(super) fn coverage_fields(g: &mut Builder, id: &str, row: &ReviewCoverage) -> Answer<()> {
    class(g, id, "ReviewCoverage")?;
    text(g, id, "package", &row.package)?;
    text(g, id, "kind", row.kind.name())?;
    number(g, id, "total", row.total)?;
    number(g, id, "reviewed", row.reviewed)?;
    number(
        g,
        id,
        "percentHundredths",
        usize::from(row.percent_hundredths),
    )
}
