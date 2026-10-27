// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Original-meter admission through actual impl type-path traversal.

use super::*;
use crate::{Budget, LoadOptions, MemoryTree, Workspace};
use std::fmt::Write;

fn workspace(source: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", source),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("source fixture")
}

fn admitted_nominal(
    index: &Index,
    name: &str,
    limit: usize,
) -> (Result<Vec<ItemId>, LoadError>, usize) {
    let meter = crate::meter::Meter::new(&LoadOptions {
        max_glob_bytes: limit,
        ..LoadOptions::default()
    });
    let file = RelPath::new("src/lib.rs").expect("path");
    let result = {
        let memory = Transient::new(&meter);
        index.nominal_types(
            0,
            &[name.to_owned()],
            false,
            Heads::Code { test: false },
            Steps::LoadAt {
                memory: &memory,
                file: &file,
                line: 1,
            },
        )
    };
    let peak = meter.usage().glob_bytes;
    // The production traversal returned and released its actual native spill.
    Transient::new(&meter)
        .charge(limit, &file, Some(1))
        .expect("all scratch released");
    (result, peak)
}

#[test]
fn missing_impl_head_admits_pending_glob_spill_and_keeps_the_missing_neighbor() {
    let mut source = String::from("pub struct Base;\n");
    for n in 0..9 {
        writeln!(
            source,
            "pub mod m{n} {{ pub use crate::Base as Other; }} use m{n}::*;"
        )
        .expect("text");
    }
    let ws = workspace(&source);
    // The absent head walks all nine import-bearing glob targets before any
    // target is popped, exceeding read_pending_at's inline eight cells.
    let (answer, peak) = admitted_nominal(ws.index(), "Missing", usize::MAX);
    assert_eq!(answer.expect("missing self type is unresolved"), []);
    assert!(peak > 0);
    assert_eq!(
        admitted_nominal(ws.index(), "Missing", peak)
            .0
            .expect("exact grant"),
        []
    );
    let error = admitted_nominal(ws.index(), "Missing", peak - 1)
        .0
        .expect_err("one byte less");
    assert!(
        matches!(error.kind, LoadErrorKind::Budget { budget: Budget::GlobBytes, limit } if limit == peak - 1)
    );
    assert_eq!((error.file.as_str(), error.line), ("src/lib.rs", Some(1)));
    assert_eq!(
        admitted_nominal(ws.index(), "Base", peak - 1)
            .0
            .expect("direct valid neighbor under the refused grant")
            .len(),
        1
    );
    source.push_str("impl Missing { pub fn member() {} }\nimpl Base { pub fn valid() {} }\n");
    let ws = workspace(&source);
    assert!(
        ws.resolve("demo::Base::valid").is_ok(),
        "actual attach caller"
    );
}

#[test]
fn alternate_aliases_spill_the_nominal_stack_under_its_original_grant() {
    let mut source = String::from("pub struct Base;\n");
    for n in 0..9 {
        writeln!(source, "#[cfg(feature = \"f{n}\")] pub type Alias = Base;").expect("text");
    }
    let ws = workspace(&source);
    let (answer, peak) = admitted_nominal(ws.index(), "Alias", usize::MAX);
    assert_eq!(answer.expect("all alternate alias routes").len(), 1);
    assert!(admitted_nominal(ws.index(), "Alias", peak).0.is_ok());
    assert!(matches!(
        admitted_nominal(ws.index(), "Alias", peak - 1)
            .0
            .expect_err("short grant")
            .kind,
        LoadErrorKind::Budget {
            budget: Budget::GlobBytes,
            ..
        }
    ));
}
