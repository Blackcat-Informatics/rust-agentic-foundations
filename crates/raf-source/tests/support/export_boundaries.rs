// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact external-export and resource observations, separate from generic oracle classes.

use crate::support::rustc;
use raf_source::{
    Budget, ExportError, Exports, LoadErrorKind, LoadOptions, MemoryTree, TargetKind, Workspace,
};
use std::fmt::Write as _;

/// These names identify only the exact cases below, never a tolerated error class.
const INVENTORY: [Boundary; 4] = [
    Boundary {
        name: "external-carried-value-shadow",
        shape: Shape::Shadowed,
        source: Expected::Omitted,
    },
    Boundary {
        name: "external-carried-free-type-route",
        shape: Shape::FreeRoute,
        source: Expected::Listed { count: 1 },
    },
    Boundary {
        name: "external-carry-budget-refusal",
        shape: Shape::Carrier,
        source: Expected::CarryBudget,
    },
    Boundary {
        name: "external-carry-budget-neighbor",
        shape: Shape::Carrier,
        source: Expected::Listed { count: 200 },
    },
];

struct Boundary {
    name: &'static str,
    shape: Shape,
    source: Expected,
}
#[derive(Clone, Copy)]
enum Shape {
    Shadowed,
    FreeRoute,
    Carrier,
}
enum Expected {
    Omitted,
    Listed { count: usize },
    CarryBudget,
}

pub(super) fn assert_inventory() {
    // One metadata-only observation serves the two grants over identical bytes.
    let carrier = source(Shape::Carrier);
    assert_compiler("external-carry-budget", &carrier);
    for boundary in &INVENTORY {
        let source = match boundary.shape {
            Shape::Carrier => carrier.as_str(),
            Shape::Shadowed | Shape::FreeRoute => &source(boundary.shape),
        };
        if !matches!(boundary.shape, Shape::Carrier) {
            assert_compiler(boundary.name, source);
        }
        let tree = tree(source);
        let mut workspace = Workspace::load(&tree, &["."]).unwrap_or_else(|error| {
            panic!("{}: unexpected source load failure: {error}", boundary.name)
        });
        let mut options = LoadOptions::default();
        if matches!(boundary.source, Expected::CarryBudget) {
            // The index fits first. This fixed query headroom admits setup but
            // refuses the competing-binder searches, without changing defaults.
            options.max_item_bytes = workspace.usage().item_bytes + 256 * 1024;
            workspace = Workspace::load_with(&tree, &["."], &options)
                .expect("the declared tight profile admits the identical source load");
        }
        let target = workspace
            .target_ids()
            .find(|id| {
                workspace
                    .target(*id)
                    .is_some_and(|target| target.kind == TargetKind::Lib)
            })
            .expect("the controlled fixture declares a library");
        let outcome = workspace.exported_items(target);
        match boundary.source {
            Expected::Omitted => {
                let exports = outcome.unwrap_or_else(|error| panic!("{}: {error}", boundary.name));
                assert!(
                    exports.reexports.is_empty(),
                    "{}: the current namespace boundary changed: {exports:?}",
                    boundary.name
                );
            }
            Expected::Listed { count } => {
                let exports = outcome.unwrap_or_else(|error| panic!("{}: {error}", boundary.name));
                assert_listed(boundary, &exports, count);
            }
            Expected::CarryBudget => {
                let error = outcome.expect_err("the exact carrier shape exhausts its item grant");
                assert_carrier_refusal(boundary.name, source, &error, options.max_item_bytes);
            }
        }
    }
    // Preserve the original default-budget Source traversal once. Its giant
    // source has no successful compiler observation; it is never compiled here.
    let source = chain(5000, 2000);
    let workspace = Workspace::load(&tree(&source), &["."]).expect("the original Source load fits");
    let target = workspace
        .target_ids()
        .find(|id| {
            workspace
                .target(*id)
                .is_some_and(|target| target.kind == TargetKind::Lib)
        })
        .expect("the library target");
    let error = workspace
        .exported_items(target)
        .expect_err("the original long carrier exceeds the default item grant");
    assert_carrier_refusal(
        "source-only-default-carrier",
        &source,
        &error,
        LoadOptions::default().max_item_bytes,
    );
}

fn tree(source: &str) -> MemoryTree {
    MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n",
        ),
        ("src/lib.rs", source),
    ])
}

fn assert_compiler(name: &str, source: &str) {
    let diagnostics = rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{name}: compiler refuses the controlled export: {diagnostics:?}"
    );
}

fn assert_listed(boundary: &Boundary, exports: &Exports, count: usize) {
    assert_eq!(exports.reexports.len(), count, "{}", boundary.name);
    for index in 0..count {
        let name = match boundary.shape {
            Shape::Shadowed | Shape::FreeRoute => "Cell".to_owned(),
            Shape::Carrier => format!("C{index}"),
        };
        assert!(
            exports
                .reexports
                .iter()
                .any(|export| export.path == format!("demo::{name}")
                    && export.written == "std::cell::Cell"
                    && export.item.is_none()),
            "{}: missing structural external export {name}",
            boundary.name,
        );
    }
}

fn assert_carrier_refusal(name: &str, source: &str, error: &ExportError, expected_limit: usize) {
    let ExportError::CarryBudget {
        path,
        error: refusal,
    } = error
    else {
        panic!("{name}: expected the exact typed carrier refusal, got {error}");
    };
    assert!(
        matches!(
            refusal.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                limit
            } if limit == expected_limit
        ),
        "{name}: {refusal}",
    );
    assert!(path.starts_with("demo::C"), "{name}: {path}");
    let export = path
        .strip_prefix("demo::")
        .expect("the reader owns the exported path");
    let declaration = format!("pub use std::cell::Cell as {export};");
    let line = source
        .lines()
        .position(|text| text.trim_start_matches("mod m {").trim() == declaration)
        .expect("the refused carrier retains its original declaration")
        + 1;
    assert_eq!(
        (refusal.file.as_str(), refusal.line),
        ("src/lib.rs", Some(line)),
        "{name}"
    );
    assert!(
        error.to_string().contains(&format!("`{path}`")),
        "{name}: {error}"
    );
}

fn source(shape: Shape) -> String {
    let mut source = match shape {
        Shape::Shadowed | Shape::FreeRoute => {
            let mut source = String::from(
                "mod c { pub use std::cell::Cell; }\nmod z {}\n\
                 mod a { fn Cell() {} pub use crate::c::*; }\npub use a::*;\n",
            );
            if matches!(shape, Shape::FreeRoute) {
                source.push_str("pub use c::*;\n");
            }
            source
        }
        Shape::Carrier => chain(50, 200),
    };
    let exported = match shape {
        Shape::Shadowed | Shape::FreeRoute => "Cell",
        Shape::Carrier => "C0",
    };
    writeln!(
        source,
        "pub fn observe_external_type(_: {exported}<u8>) {{}}"
    )
    .expect("write controlled source");
    source
}

/// The original carrier shape: long globs reach names behind competing binders.
fn chain(links: usize, names: usize) -> String {
    let mut source = String::from("mod z {}\npub use c0::*;\n");
    for index in 0..links {
        writeln!(
            source,
            "mod c{index} {{ pub use crate::c{}::*; }}",
            index + 1
        )
        .expect("write");
    }
    writeln!(
        source,
        "mod c{links} {{ pub use crate::m::*; pub use crate::a::*; pub use crate::b::*; }}"
    )
    .expect("write");
    let (mut carried, mut bound) = (String::new(), String::new());
    for index in 0..names {
        writeln!(carried, "pub use std::cell::Cell as C{index};").expect("write");
        writeln!(bound, "fn C{index}() {{}}").expect("write");
    }
    writeln!(source, "mod m {{ {carried} }}\nmod a {{ {bound} pub use crate::z::*; }}\nmod b {{ {bound} pub use crate::z::*; }}").expect("write");
    source
}
