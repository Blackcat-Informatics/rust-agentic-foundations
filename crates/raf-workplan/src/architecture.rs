// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Authored component and scenario models over the existing Workplan documents.

mod check;

use crate::Workplan;
use raf_okf::ArchitectureModel;

/// One document-owned model; local identifiers retain this document authority.
#[derive(Debug, Clone, Copy)]
pub struct AuthoredModel<'a> {
    /// Checked capability or architecture document identifier.
    pub owner: &'a str,
    /// Authoring path, used for findings and navigation.
    pub path: &'a str,
    /// Typed declarations, independent of observed verification facts.
    pub declarations: &'a ArchitectureModel,
}

impl Workplan<'_> {
    /// All loaded document-owned architecture models, including empty declarations.
    ///
    /// Returned values borrow the native typed documents. This does not infer
    /// interactions, observed order, branches or concurrency from source bindings.
    pub fn architecture_models(&self) -> impl Iterator<Item = AuthoredModel<'_>> {
        self.capabilities()
            .iter()
            .map(|document| AuthoredModel {
                owner: &document.value().header.id,
                path: document.path(),
                declarations: &document.value().model,
            })
            .chain(self.architecture().iter().map(|document| AuthoredModel {
                owner: &document.value().header.id,
                path: document.path(),
                declarations: &document.value().model,
            }))
    }
}

pub(crate) fn findings(plan: &Workplan<'_>) -> Vec<crate::Finding> {
    check::models(plan)
}
