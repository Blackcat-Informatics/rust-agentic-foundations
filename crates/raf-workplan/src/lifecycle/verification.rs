// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Static source bindings and document findings, without execution evidence.

use std::collections::BTreeSet;
use std::fmt;

use super::session::LifecycleError;
use crate::{Binding, Finding, Opened};

/// The declared bindings and canonical static findings of a verification.
/// No entry represents a test run, compiler observation or runtime verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    change: Option<String>,
    bindings: Vec<Binding>,
    findings: Vec<Finding>,
}

impl VerifyReport {
    /// The selected change, or all capabilities when absent.
    #[must_use]
    pub fn change(&self) -> Option<&str> {
        self.change.as_deref()
    }

    /// The selected declared source bindings in deterministic order.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// The canonical static findings of the repository.
    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// The number of selected declared bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether the selection contains no declared binding.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Whether the canonical static check has no findings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.findings.is_empty()
    }

    /// Human-readable declared bindings followed by static findings.
    /// Control characters are escaped; no line claims execution success.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<_> = self
            .bindings
            .iter()
            .map(|binding| {
                crate::escape::shown(&format!(
                    "declared {} {} `{}`",
                    binding.capability(),
                    binding.kind().key(),
                    binding.path()
                ))
            })
            .collect();
        lines.extend(self.findings.iter().map(|finding| {
            crate::escape::shown(&format!(
                "finding {} {} {} ({})",
                finding.kind, finding.file, finding.message, finding.fix
            ))
        }));
        lines
    }
}

impl fmt::Display for VerifyReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.lines() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

/// Check source and document structure without starting a process or writing.
/// The optional change selects the displayed bindings; the canonical static
/// repository check remains complete so unrelated faults are not hidden.
///
/// # Errors
/// Refuses an unknown change or a required capability that did not load.
pub fn verify(opened: &Opened, change: Option<&str>) -> Result<VerifyReport, LifecycleError> {
    let plan = opened.workplan();
    verify_plan(&plan, change)
}

pub(super) fn verify_plan(
    plan: &crate::Workplan<'_>,
    change: Option<&str>,
) -> Result<VerifyReport, LifecycleError> {
    let findings = plan.check();
    let selected = change.map(|change| plan.impact(change)).transpose()?;
    if let Some(impact) = &selected {
        let loaded: BTreeSet<_> = plan
            .capabilities()
            .iter()
            .map(|cap| cap.value().header.id.as_str())
            .collect();
        let missing: Vec<_> = impact
            .capabilities
            .iter()
            .filter(|id| !loaded.contains(id.as_str()))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Err(LifecycleError::Unloaded {
                capabilities: missing,
                findings,
            });
        }
    }
    let mut bindings: Vec<_> = plan
        .graph()
        .bindings()
        .iter()
        .filter(|binding| {
            selected.as_ref().is_none_or(|impact| {
                impact
                    .capabilities
                    .iter()
                    .any(|id| id == binding.capability())
            })
        })
        .cloned()
        .collect();
    bindings.sort_by(|a, b| {
        (a.capability(), a.kind(), a.path()).cmp(&(b.capability(), b.kind(), b.path()))
    });
    Ok(VerifyReport {
        change: selected.map(|impact| impact.change),
        bindings,
        findings,
    })
}
