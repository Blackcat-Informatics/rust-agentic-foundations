// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The optional target-kind and path scope of a declared pattern.

use super::{E, strings};
use crate::FileCtx;
use raf_rdf::Node;
use raf_source::{RelPath, TargetKind};
use std::collections::BTreeSet;

/// Where a pattern applies. An empty scope applies everywhere.
///
/// A target kind is admitted when it is in `include_kinds` (or that set is
/// empty), is not in `exclude_kinds`, and, under `shipping_only`, is a library
/// or a binary. Build scripts are never shipping code for a scope. A path is
/// admitted when it lies under an include prefix (or there is none) and under
/// no exclude prefix.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct PatternScope {
    /// Target kinds the pattern is limited to; empty means every kind.
    pub include_kinds: BTreeSet<TargetKind>,
    /// Target kinds the pattern skips.
    pub exclude_kinds: BTreeSet<TargetKind>,
    /// True when the pattern applies only to non-test library and binary code.
    pub shipping_only: bool,
    /// Workspace-relative path prefixes the pattern is limited to; empty means every path.
    pub include_paths: Vec<RelPath>,
    /// Workspace-relative path prefixes the pattern skips.
    pub exclude_paths: Vec<RelPath>,
}

const KINDS: [(&str, TargetKind); 6] = [
    ("lib", TargetKind::Lib),
    ("bin", TargetKind::Bin),
    ("test", TargetKind::Test),
    ("bench", TargetKind::Bench),
    ("example", TargetKind::Example),
    ("buildScript", TargetKind::BuildScript),
];

fn under(path: &RelPath, prefix: &RelPath) -> bool {
    let mut segments = path.segments();
    prefix
        .segments()
        .all(|expected| segments.next() == Some(expected))
}

impl PatternScope {
    /// True when no property narrows the pattern.
    #[must_use]
    pub fn is_unscoped(&self) -> bool {
        *self == Self::default()
    }

    fn admits_kind(&self, kind: TargetKind) -> bool {
        (self.include_kinds.is_empty() || self.include_kinds.contains(&kind))
            && !self.exclude_kinds.contains(&kind)
            && (!self.shipping_only || matches!(kind, TargetKind::Lib | TargetKind::Bin))
    }

    /// True when a file with this context is in scope: some target kind that
    /// reaches it is admitted, and its path is admitted.
    #[must_use]
    pub fn admits_file(&self, file: &FileCtx, path: &RelPath) -> bool {
        (self.include_paths.is_empty() || self.include_paths.iter().any(|p| under(path, p)))
            && !self.exclude_paths.iter().any(|p| under(path, p))
            && file.target_kinds.iter().any(|kind| self.admits_kind(*kind))
            && (!self.shipping_only || file.shipping)
    }

    /// True when an item of the given target kind is in scope; `shipping` is
    /// whether the item is non-test code of a shipping target.
    #[must_use]
    pub fn admits_item(&self, kind: TargetKind, shipping: bool) -> bool {
        self.admits_kind(kind) && (!self.shipping_only || shipping)
    }
}

pub(super) fn read(node: &Node<'_>, id: &str) -> Result<PatternScope, E> {
    let mut scope = PatternScope {
        include_kinds: kinds(node, "targetKind", id)?,
        exclude_kinds: kinds(node, "excludeTargetKind", id)?,
        include_paths: paths(node, "includePath", id)?,
        exclude_paths: paths(node, "excludePath", id)?,
        ..PatternScope::default()
    };
    let mut flags = node.values(&super::super::shape::property("shippingOnly"));
    match (flags.next(), flags.next()) {
        (None, _) => {}
        (Some(value), None) if super::super::shape::boolean_value(value) == Some(true) => {
            scope.shipping_only = true;
        }
        (Some(_), None) => {
            return Err(E::Value(format!(
                "pattern {id:?}: shippingOnly is stated only as true; omit it to apply to every target kind"
            )));
        }
        _ => return Err(E::Repeated(format!("pattern {id:?} shippingOnly"))),
    }
    contradictions(&scope, id)?;
    Ok(scope)
}

fn kinds(node: &Node<'_>, key: &str, id: &str) -> Result<BTreeSet<TargetKind>, E> {
    strings(node, key)?
        .into_iter()
        .map(|name| {
            KINDS
                .iter()
                .find(|(spelling, _)| *spelling == name)
                .map(|(_, kind)| *kind)
                .ok_or_else(|| {
                    E::Value(format!(
                        "pattern {id:?}: {key} {name:?} is not a target kind; use one of lib, bin, test, bench, example or buildScript"
                    ))
                })
        })
        .collect()
}

fn paths(node: &Node<'_>, key: &str, id: &str) -> Result<Vec<RelPath>, E> {
    strings(node, key)?
        .into_iter()
        .map(|text| {
            let path = RelPath::new(&text).map_err(|error| {
                E::Value(format!(
                    "pattern {id:?}: {key} {text:?} must be a path relative to the workspace root: {error}"
                ))
            })?;
            if path.is_root() {
                return Err(E::Value(format!(
                    "pattern {id:?}: {key} {text:?} names the workspace root; name a directory or file below it, or omit the property"
                )));
            }
            Ok(path)
        })
        .collect()
}

fn contradictions(scope: &PatternScope, id: &str) -> Result<(), E> {
    if let Some(kind) = scope
        .include_kinds
        .intersection(&scope.exclude_kinds)
        .next()
    {
        let name = KINDS
            .iter()
            .find(|(_, k)| k == kind)
            .map_or("?", |(n, _)| n);
        return Err(E::Value(format!(
            "pattern {id:?}: target kind {name} is both included and excluded; remove it from one of targetKind and excludeTargetKind"
        )));
    }
    if !KINDS.iter().any(|(_, kind)| scope.admits_kind(*kind)) {
        return Err(E::Value(format!(
            "pattern {id:?}: the target kind properties admit no target kind; widen targetKind, narrow excludeTargetKind, or drop shippingOnly (it admits only lib and bin)"
        )));
    }
    for include in &scope.include_paths {
        if let Some(exclude) = scope.exclude_paths.iter().find(|e| under(include, e)) {
            return Err(E::Value(format!(
                "pattern {id:?}: includePath {include} lies inside excludePath {exclude}, so it admits nothing; remove one of them"
            )));
        }
    }
    Ok(())
}
