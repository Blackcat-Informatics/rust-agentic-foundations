// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Every answer the index gives about one program of the differential test
//! against `rustc`, written as text, so that two resolvers, or two modes of
//! one, can be compared answer by answer.
//!
//! The answers of a program are: each of its probes read from its module in
//! the three name spaces; the names [`NAMES`] read as `self::<name>` in the
//! type and the value name space from every module with a `use`
//! declaration; and the exported surface of the library. A load that is
//! refused is answered by its error, and a surface that is refused by its
//! error.
//!
//! The including module provides `MemoryTree`, `Namespace`, `TargetKind`
//! and `Workspace` of `raf_source`, and the generator as its module
//! `program`, so the unit tests of the crate and its examples include the
//! same text.

use std::fmt::Write as _;

use super::program::{CRATE, Program};
use super::{MemoryTree, Namespace, TargetKind, Workspace};

/// The names read from every module with a `use` declaration: the names of
/// the generated items and modules.
pub(crate) const NAMES: [&str; 6] = ["A", "B", "C", "s0", "s1", "s2"];

/// What the answers of some programs held.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Counted {
    /// The loads made.
    pub(crate) loads: usize,
    /// The loads that were refused.
    pub(crate) refused_loads: usize,
    /// The answers written, a surface counting as one.
    pub(crate) answers: usize,
    /// The answers that are an ambiguity.
    pub(crate) ambiguous: usize,
    /// The surfaces that were refused.
    pub(crate) refused_surfaces: usize,
}

impl Counted {
    /// The sum of two counts.
    pub(crate) const fn add(self, other: Self) -> Self {
        Self {
            loads: self.loads + other.loads,
            refused_loads: self.refused_loads + other.refused_loads,
            answers: self.answers + other.answers,
            ambiguous: self.ambiguous + other.ambiguous,
            refused_surfaces: self.refused_surfaces + other.refused_surfaces,
        }
    }
}

/// The library root of `program` with every probe written, as the crate of
/// one package.
pub(crate) fn tree(program: &Program) -> MemoryTree {
    let all: Vec<usize> = (0..program.probes.len()).collect();
    let source = program
        .render(&all, &std::collections::BTreeMap::new())
        .source;
    let manifest =
        format!("[package]\nname = \"{CRATE}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
    [
        ("Cargo.toml", manifest.as_str()),
        ("src/lib.rs", source.as_str()),
    ]
    .into_iter()
    .collect()
}

/// Load `program` and write every answer of the index to it, one a line,
/// returning the text and what it held.
pub(crate) fn answers(program: &Program) -> (String, Counted) {
    let mut out = String::new();
    let mut counted = Counted {
        loads: 1,
        ..Counted::default()
    };
    let ws = match Workspace::load(&tree(program), &["."]) {
        Ok(ws) => ws,
        Err(error) => {
            writeln!(out, "load refused: {error}").expect("write");
            counted.refused_loads = 1;
            return (out, counted);
        }
    };
    let mut note = |out: &mut String, line: String| {
        counted.answers += 1;
        counted.ambiguous += usize::from(line.contains("Ambiguous"));
        writeln!(out, "{line}").expect("write");
    };
    for probe in &program.probes {
        for ns in [Namespace::Type, Namespace::Value, Namespace::Macro] {
            let answer = match ws.resolve_from_in(&probe.module, &probe.path, ns) {
                Ok(found) => format!(
                    "ok {} {:?} glob={} chain={:?} sites={}",
                    found.path,
                    found.namespace,
                    found.via_glob,
                    found.chain,
                    found.sites.len()
                ),
                Err(error) => format!("err {error:?}"),
            };
            note(
                &mut out,
                format!("{} {} {ns:?}: {answer}", probe.module, probe.path),
            );
        }
    }
    for decl in program.uses() {
        for name in NAMES {
            for ns in [Namespace::Type, Namespace::Value] {
                let answer = match ws.resolve_from_in(&decl.module, &format!("self::{name}"), ns) {
                    Ok(found) => format!("ok {} glob={}", found.path, found.via_glob),
                    Err(error) => format!("err {error:?}"),
                };
                note(
                    &mut out,
                    format!("  {} self::{name} {ns:?}: {answer}", decl.module),
                );
            }
        }
    }
    let (surface, refused) = surface(&ws);
    note(&mut out, format!("surface {surface}"));
    counted.refused_surfaces = usize::from(refused);
    (out, counted)
}

/// The exported surface of the library of `ws`, sorted, and whether it was
/// refused.
fn surface(ws: &Workspace) -> (String, bool) {
    let id = ws
        .target_ids()
        .find(|id| {
            ws.target(*id)
                .is_some_and(|target| target.kind == TargetKind::Lib)
        })
        .expect("the program is a library");
    match ws.exported_items(id) {
        Ok(exports) => {
            let mut items: Vec<String> = exports
                .items
                .iter()
                .map(|exported| {
                    let defined = ws
                        .item(exported.item)
                        .map_or_else(String::new, |item| item.path.to_string());
                    format!(
                        "{} = {defined} ({:?}) hidden={:?}",
                        exported.path, exported.kind, exported.doc_hidden
                    )
                })
                .collect();
            items.sort();
            let mut reexports: Vec<String> = exports
                .reexports
                .iter()
                .map(|reexport| format!("{} = {}", reexport.path, reexport.written))
                .collect();
            reexports.sort();
            (format!("items {items:?} reexports {reexports:?}"), false)
        }
        Err(error) => (format!("refused: {error:?}"), true),
    }
}
