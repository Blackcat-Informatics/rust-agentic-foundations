// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual import declarations distinguish determinacy from settled lookup.

mod support;

use raf_source::{ExportError, MemoryTree, Namespace, ResolveError, Workspace};
use support::program::Program;
use support::random::Draw;

fn load(source: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", source),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).expect("source loads")
}

fn returning_private_glob_sources() -> [(&'static str, usize); 2] {
    [
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/import_cycle_a_private_glob_does_not_finalize_a_returning_type.rs"
            ),
            3,
        ),
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/import_cycle_a_private_glob_does_not_finalize_a_returning_value.rs"
            ),
            1,
        ),
    ]
}

#[test]
fn a_late_namespace_refuses_the_whole_import_including_its_bound_peer() {
    for (source, at) in returning_private_glob_sources() {
        let program = Program::parse(source).expect("fixture parses");
        let judged = support::rustc::judge(&program).expect("native fixture loads");
        assert_eq!(
            judged.dropped, 0,
            "the retained program needs no declaration repair"
        );
        assert!(
            matches!(&judged.verdicts[at], support::rustc::Verdict::Rejected(codes)
            if codes.iter().any(|code| code == support::rustc::UNDETERMINABLE)),
            "{judged:?}"
        );
        let probes: Vec<_> = (0..program.probes.len()).collect();
        let rendered = program.render(&probes, &std::collections::BTreeMap::new());
        let ws = load(&rendered.source);
        for space in [Namespace::Type, Namespace::Value] {
            let found = ws.resolve_from_in(
                &program.probes[at].module,
                &format!("self::probe{at}"),
                space,
            );
            assert!(
                matches!(found, Err(ResolveError::IndeterminateImport { .. })),
                "{space:?}: {found:?}"
            );
        }
    }
}

#[test]
fn a_private_type_shadow_does_not_poison_its_independent_value_glob() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_type_shadow_does_not_poison_its_independent_value_glob.rs"
    );
    let program = Program::parse(source).expect("fixture parses");
    let judged = support::rustc::judge(&program).expect("native fixture loads");
    assert_eq!(judged.dropped, 0, "no declaration repair");
    assert_eq!(
        judged.verdicts[1],
        support::rustc::Verdict::Accepted {
            ty: None,
            value: Some(10)
        }
    );
    let probes: Vec<_> = (0..program.probes.len()).collect();
    let rendered = program.render(&probes, &std::collections::BTreeMap::new());
    let ws = load(&rendered.source);
    assert_eq!(
        ws.resolve_from_in("demo::m1::s0", "self::probe1", Namespace::Value)
            .expect("independent value route")
            .path,
        "demo::m2::s0::C"
    );
    assert!(matches!(
        ws.resolve_from_in("demo::m1::s0", "self::probe1", Namespace::Type),
        Err(ResolveError::NotFound(_))
    ));
    assert_eq!(
        ws.resolve_in("demo::m2::s0::C", Namespace::Value)
            .expect("genuine independent source")
            .path,
        "demo::m2::s0::C"
    );
    let visible = source.replace("use crate::m1::s1::C;", "pub use crate::m1::s1::C;");
    let program = Program::parse(&visible).expect("visible neighbor parses");
    let judged = support::rustc::judge(&program).expect("native visible neighbor");
    assert_eq!(judged.dropped, 0);
    assert_eq!(
        judged.verdicts[1],
        support::rustc::Verdict::Accepted {
            ty: Some(2),
            value: Some(10)
        }
    );
    let rendered = program.render(&probes, &std::collections::BTreeMap::new());
    let ws = load(&rendered.source);
    for (space, expected) in [
        (Namespace::Type, "demo::C"),
        (Namespace::Value, "demo::m2::s0::C"),
    ] {
        assert_eq!(
            ws.resolve_from_in("demo::m1::s0", "self::probe1", space)
                .expect("visible namespace")
                .path,
            expected
        );
    }
}

fn assert_named_reference(ws: &Workspace, expected: bool) {
    let inspect = ws
        .resolve_in("demo::inspect", Namespace::Value)
        .expect("inspect")
        .sites[0];
    let definition = ws
        .resolve_in("demo::m1::A", Namespace::Value)
        .expect("definition")
        .sites[0];
    let stable = ws
        .resolve_in("demo::stable", Namespace::Value)
        .expect("stable")
        .sites[0];
    let reached = ws
        .reachable(&[inspect.into()])
        .expect("semantic refusal is no reference edge");
    assert_eq!(reached.contains(definition.into()), expected);
    assert!(reached.contains(stable.into()));
}

#[test]
fn a_late_bound_peer_cannot_escape_through_named_and_glob_copies_or_exports() {
    let source = returning_private_glob_sources()[0]
        .0
        .replace(
            "pub(crate) const fn A() -> u32 { 1 }",
            "pub const fn A() -> u32 { 1 }",
        )
        .replace(
            "pub mod m2 {",
            "pub mod m2 {\npub use crate::m1::A as Bound;",
        );
    let source = format!(
        "{source}\npub mod copies {{ pub use crate::m2::Bound as Named; }}\npub use copies::*;\npub fn inspect() {{ crate::copies::Named(); stable(); }}\npub fn stable() {{}}\n"
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .text
            .contains("cannot determine resolution for the import")),
        "{diagnostics:?}"
    );
    let ws = load(&source);
    for path in ["demo::m2::Bound", "demo::copies::Named", "demo::Named"] {
        assert!(
            matches!(
                ws.resolve_in(path, Namespace::Value),
                Err(ResolveError::IndeterminateImport { .. })
            ),
            "{path}: {:?}",
            ws.resolve_in(path, Namespace::Value)
        );
        let local = path.strip_prefix("demo::").expect("crate path");
        assert!(
            matches!(
                ws.resolve_from_in("demo", &format!("crate::{local}"), Namespace::Value),
                Err(ResolveError::IndeterminateImport { .. })
            ),
            "{path}"
        );
    }
    let target = ws.target_ids().next().expect("library target");
    assert!(
        matches!(
            ws.exported_items(target),
            Err(ExportError::Resolve(
                ResolveError::IndeterminateImport { .. }
            ))
        ),
        "{:?}",
        ws.exported_items(target)
    );
    // The genuine definition remains independently usable in the same index.
    assert_named_reference(&ws, false);
    assert_eq!(
        ws.resolve_in("demo::m1::A", Namespace::Value)
            .expect("direct definition")
            .path,
        "demo::m1::A"
    );
    let valid = source.replace(
        "pub mod m1 {",
        "pub mod m1 {\npub struct A {} impl A { pub const TAG: u32 = 90; }",
    );
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let ws = load(&valid);
    assert_named_reference(&ws, true);
    for path in ["demo::m2::Bound", "demo::copies::Named", "demo::Named"] {
        assert_eq!(
            ws.resolve_in(path, Namespace::Value)
                .expect("grounded neighbor")
                .path,
            "demo::m1::A"
        );
    }
    assert!(
        ws.exported_items(ws.target_ids().next().expect("library target"))
            .is_ok()
    );
}

#[test]
fn a_direct_visible_definition_makes_the_neighboring_import_determinate() {
    for (source, at, before, after, ty, value, expected) in [
        (
            returning_private_glob_sources()[0].0,
            3,
            "pub mod m1 {",
            "pub mod m1 {\npub struct A {} impl A { pub const TAG: u32 = 90; }",
            90,
            1,
            "demo::m1::A",
        ),
        (
            returning_private_glob_sources()[1].0,
            1,
            "pub mod s0 {",
            "pub mod s0 {\npub const fn B() -> u32 { 90 }",
            5,
            90,
            "demo::m2::s0::B",
        ),
    ] {
        let source = source.replace(before, after);
        let program = Program::parse(&source).expect("neighbor parses");
        let judged = support::rustc::judge(&program).expect("native neighbor loads");
        assert_eq!(
            judged.dropped, 0,
            "the neighbor needs no declaration repair"
        );
        assert_eq!(
            judged.verdicts[at],
            support::rustc::Verdict::Accepted {
                ty: Some(ty),
                value: Some(value)
            }
        );
        let probes: Vec<_> = (0..program.probes.len()).collect();
        let ws = load(
            &program
                .render(&probes, &std::collections::BTreeMap::new())
                .source,
        );
        for space in [Namespace::Type, Namespace::Value] {
            assert_eq!(
                ws.resolve_from_in(
                    &program.probes[at].module,
                    &format!("self::probe{at}"),
                    space
                )
                .expect("determinate neighbor")
                .path,
                expected
            );
        }
    }
}

#[test]
fn a_fixed_module_prefix_decides_absence_before_an_unrelated_glob_widens_a_name() {
    let source = "
pub mod m2 {
    use crate::m1::*;
    pub use super::*;
}
use crate::m3::C;
pub use crate::m1::*;
mod m1 { pub const fn C() -> u32 { 7 } }
pub mod m3 { PRIVATE_GLOB pub struct C {} impl C { pub const TAG: u32 = 14; } }
use crate::m2::C as Probe;
const _: () = assert!(Probe::TAG == 14);
const _: () = assert!(Probe() == 7);
";
    for private_glob in [
        "",
        "use crate::m1::*;",
        "pub(self) use crate::m1::*;",
        "pub(in crate::m3) use crate::m1::*;",
    ] {
        let source = source.replace("PRIVATE_GLOB", private_glob);
        let program = Program::parse(&source).expect("fixture parses");
        let mut variants = vec![source.clone()];
        let mut draw = Draw::seeded(0x0a11_ce07);
        for _ in 0..16 {
            variants.push(program.shuffled(&mut draw).to_text());
        }
        for source in variants {
            let diagnostics = support::rustc::compile(&source);
            assert!(
                !diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.level == "error"),
                "{source}\n{diagnostics:?}"
            );
            let ws = load(&source);
            for (namespace, expected) in [
                (Namespace::Type, "demo::m3::C"),
                (Namespace::Value, "demo::m1::C"),
            ] {
                assert_eq!(
                    ws.resolve_from_in("demo", "self::Probe", namespace)
                        .expect("the import is determinate in both namespaces")
                        .path,
                    expected,
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn a_named_import_reading_back_through_its_module_is_not_a_settled_expression() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/import_cycle_rustc_cannot_determine_through_a_named_import_and_two_globs.rs"
    );
    let baseline = load(source);
    assert_eq!(
        baseline
            .resolve_from_in("demo::m1::s1", "crate::m2::A", Namespace::Value)
            .expect("settled value")
            .path,
        "demo::m1::s0::A"
    );
    let source = source.replace(
        "pub use crate::m0::A;",
        "pub use crate::m0::A;\nuse crate::m2::A as Probe;",
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().any(|d| d
            .text
            .contains("cannot determine resolution for the import")),
        "{diagnostics:?}"
    );
    let ws = load(&source);
    assert!(
        matches!(
            ws.resolve_from_in("demo::m1::s1", "self::Probe", Namespace::Value),
            Err(ResolveError::IndeterminateImport { .. })
        ),
        "{:?}",
        ws.resolve_from_in("demo::m1::s1", "self::Probe", Namespace::Value)
    );
    let valid = source.replace("use crate::m2::A as Probe;", "use crate::m0::A as Probe;");
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        !diagnostics.iter().any(|d| d.level == "error"),
        "{diagnostics:?}"
    );
    assert_eq!(
        load(&valid)
            .resolve_from_in("demo::m1::s1", "self::Probe", Namespace::Value)
            .expect("independent import")
            .path,
        "demo::m1::s0::A"
    );
}

#[test]
fn late_visibility_refusals_and_their_neighbors_are_stable_across_declaration_orders() {
    let named = include_str!("../../../fuzz/corpus/rustc-oracle/import_cycle_rustc_cannot_determine_through_a_named_import_and_two_globs.rs")
        .replace("pub use crate::m0::A;", "pub use crate::m0::A;\nuse crate::m2::A as Probe;");
    let globbed = format!(
        "{}\nuse crate::s0::B as Probe;\n",
        include_str!(
            "../../../fuzz/corpus/rustc-oracle/import_cycle_rustc_cannot_determine_beside_a_glob_that_resolves_it.rs"
        )
    );
    for (source, module, early, expected) in [
        (
            named,
            "demo::m1::s1",
            "\nuse crate::m0::*;\n",
            "demo::m1::s0::A",
        ),
        (globbed, "demo", "\nuse crate::m1::s0::*;\n", "demo::m0::B"),
    ] {
        let program = Program::parse(&source).expect("fixture parses");
        let mut variants = vec![source];
        for seed in [0x0a11_ce01, 0xc0ff_ee01] {
            let mut draw = Draw::seeded(seed);
            for _ in 0..12 {
                variants.push(program.shuffled(&mut draw).to_text());
            }
        }
        for source in variants {
            let diagnostics = support::rustc::compile(&source);
            let ws = load(&source);
            assert!(
                matches!(
                    ws.resolve_from_in(module, "self::Probe", Namespace::Value),
                    Err(ResolveError::IndeterminateImport { .. })
                ),
                "{source}\n{diagnostics:?}\n{:?}",
                ws.resolve_from_in(module, "self::Probe", Namespace::Value)
            );
            assert!(
                diagnostics.iter().any(|d| d
                    .text
                    .contains("cannot determine resolution for the import")),
                "{source}\n{diagnostics:?}"
            );
            let valid = source.replace(early, "\n");
            assert_ne!(
                source, valid,
                "the neighboring fixture removes its early private route"
            );
            let diagnostics = support::rustc::compile(&valid);
            assert!(
                !diagnostics.iter().any(|d| d.level == "error"),
                "{valid}\n{diagnostics:?}"
            );
            assert_eq!(
                load(&valid)
                    .resolve_from_in(module, "self::Probe", Namespace::Value)
                    .expect("accessible neighbor")
                    .path,
                expected,
                "{valid}"
            );
        }
    }
}
