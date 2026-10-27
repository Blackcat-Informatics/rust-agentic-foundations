// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Glob arrivals retain ambiguity across unrelated namespace decisions.

mod support;

use raf_source::{MemoryTree, Namespace, ResolveError, Workspace};
use support::orders::{ambiguous_between, in_every_order, read_as, resolved_as};

fn load(lib: &str) -> Workspace {
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", lib),
    ]
    .into_iter()
    .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|error| panic!("{lib}\n{error}"))
}

#[test]
fn a_private_glob_cut_retains_the_named_shadow_of_a_returning_namespace() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_glob_cut_preserves_a_named_shadow_on_a_returning_route.rs"
    );
    let observed = format!(
        "{source}\nuse crate::C as Observed;\nconst _: () = assert!(Observed::TAG == 1);\n"
    );
    let diagnostics = support::rustc::compile(&observed);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let diagnostics = support::rustc::compile(&format!("{observed}\nconst _: u32 = Observed();\n"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0423")),
        "{diagnostics:?}"
    );
    let ws = load(&observed);
    assert_eq!(
        read_as(&ws, "demo", "self::Observed", Namespace::Type),
        "demo::C"
    );
    assert!(matches!(
        ws.resolve_from_in("demo", "self::Observed", Namespace::Value),
        Err(ResolveError::NotFound(_))
    ));
}

fn private_cut_source() -> &'static str {
    "struct C {}\n\
        mod m2 { pub use crate::m0::*; use super::C; pub use crate::m3::*;\n\
            pub mod s0 { pub use crate::m0::*; } }\n\
        mod m3 { pub(crate) const fn C() -> u32 { 16 } }\n\
        mod m1 { pub(crate) const fn C() -> u32 { 8 } }\n\
        mod m0 { use crate::m1::*; pub use crate::m2::*; }\n\
        pub use crate::m2::s0::*;\n\
        use crate::C as Observed; fn identity(v: Observed) -> crate::C { v }\n"
}

#[test]
fn an_actual_inaccessible_cut_needs_a_closed_local_candidate() {
    let base = private_cut_source();
    for (imports, accepted) in [
        ("use crate::m1::*;", true),
        ("pub(crate) use crate::m1::*;", true),
        ("", false),
    ] {
        let source = base.replace("use crate::m1::*;", imports);
        let diagnostics = support::rustc::compile(&source);
        let ws = load(&source);
        if accepted {
            assert!(
                diagnostics.iter().all(|d| d.level != "error"),
                "{source}\n{diagnostics:?}"
            );
            assert_eq!(
                read_as(&ws, "demo", "self::Observed", Namespace::Type),
                "demo::C"
            );
        } else {
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.code.as_deref() == Some("E0432")
                        || d.text.contains("cannot determine resolution")),
                "{source}\n{diagnostics:?}"
            );
            assert!(matches!(
                ws.resolve_from_in("demo", "self::Observed", Namespace::Type),
                Err(ResolveError::Cyclic(_))
            ));
        }
    }
    let diagnostics = support::rustc::compile(&format!("{base}\nconst _: u32 = Observed();"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0423")),
        "{diagnostics:?}"
    );
    assert!(matches!(
        load(base).resolve_from_in("demo", "self::Observed", Namespace::Value),
        Err(ResolveError::NotFound(_))
    ));
}

#[test]
fn a_private_cut_does_not_hide_a_later_visible_candidate() {
    let base = private_cut_source().replace("mod m0 {", "mod m0 { pub use crate::late::*;");
    let late = "mod late { pub(crate) use crate::m1::C; }";
    let source = format!("{base}\n{late}\nconst _: () = assert!(Observed() == 8);\n");
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let (answers, loads) = in_every_order(load, &[&base, late], &[], |ws| {
        [Namespace::Type, Namespace::Value]
            .into_iter()
            .map(|ns| read_as(ws, "demo", "self::Observed", ns))
            .collect()
    });
    assert_eq!(loads, 2);
    assert_eq!(answers, ["demo::C", "demo::m1::C"]);
    let visible = private_cut_source().replace("use crate::m1::*;", "pub(crate) use crate::m1::*;");
    let diagnostics = support::rustc::compile(&format!(
        "{visible}\nconst _: () = assert!(Observed() == 8);"
    ));
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    assert_eq!(
        read_as(&load(&visible), "demo", "self::Observed", Namespace::Value),
        "demo::m1::C"
    );
}

#[test]
fn a_private_cut_does_not_waive_an_unclosed_sibling_branch() {
    let base = private_cut_source();
    for other in [
        "pub use missing_dependency::*;",
        "pub use crate::bad::*; mod bad { pub use crate::bad2::C; } mod bad2 { pub use crate::bad::C; }",
    ] {
        let source = format!("{base}\n{other}\n");
        let diagnostics = support::rustc::compile(&source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code.as_deref() == Some("E0432")
                    || d.text.contains("cannot determine resolution")),
            "{diagnostics:?}"
        );
        assert!(
            load(&source)
                .resolve_from_in("demo", "self::Observed", Namespace::Type)
                .is_err()
        );
    }
    let source = format!(
        "{base}\npub use crate::bad::*; mod bad {{ pub use crate::bad2::C; }} mod bad2 {{ pub struct C {{}} }}\n"
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    assert_eq!(
        read_as(&load(&source), "demo", "self::Observed", Namespace::Type),
        "demo::C"
    );
}

#[test]
fn a_private_cut_finalizes_a_module_prefix_without_waiving_its_peer_cycle() {
    let base = private_cut_source()
        .replace("struct C {}", "mod C { pub struct Member {} }")
        .replace(
            "pub mod s0 {",
            "pub mod consumer { pub use super::C::*; } pub mod s0 {",
        )
        .replace(
            "fn identity(v: Observed) -> crate::C { v }",
            "fn identity(v: Observed::Member) -> crate::C::Member { v }",
        );
    let source = format!("{base}\nfn member(v: m2::consumer::Member) -> C::Member {{ v }}\n");
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    assert_eq!(
        read_as(
            &load(&source),
            "demo::m2::consumer",
            "self::Member",
            Namespace::Type
        ),
        "demo::C::Member"
    );
    let source = source.replace("use crate::m1::*;", "");
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.text.contains("cannot determine resolution")
                || d.code.as_deref() == Some("E0432")),
        "{diagnostics:?}"
    );
    assert!(
        load(&source)
            .resolve_from_in("demo", "self::Observed", Namespace::Type)
            .is_err()
    );
}

#[test]
fn a_private_cut_requires_a_closed_glob_module_prefix() {
    let base =
        private_cut_source().replace("mod m0 {", "mod m0 { pub use crate::routes::target::*;");
    let valid = "mod routes { pub(crate) use crate::empty as target; } mod empty {}";
    let source = format!("{base}\n{valid}\n");
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let (answers, loads) = in_every_order(load, &[&base, valid], &[], |ws| {
        vec![read_as(ws, "demo", "self::Observed", Namespace::Type)]
    });
    assert_eq!(loads, 2);
    assert_eq!(answers, ["demo::C"]);
    let unresolved = "mod routes { pub(crate) use crate::again::target; } mod again { pub(crate) use crate::routes::target; }";
    let source = format!("{base}\n{unresolved}\n");
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0432")),
        "{diagnostics:?}"
    );
    assert!(
        load(&source)
            .resolve_from_in("demo", "self::Observed", Namespace::Type)
            .is_err()
    );
}

#[test]
fn a_private_cut_does_not_erase_a_visible_ambiguity() {
    let base = private_cut_source().replace("mod m0 {", "mod m0 { pub use crate::other::*;");
    let source = format!(
        "{base}\nmod other {{ pub const fn C() -> u32 {{ 24 }} }}\nconst _: u32 = Observed();\n"
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    assert!(matches!(
        load(&source).resolve_from_in("demo", "self::Observed", Namespace::Value),
        Err(ResolveError::Ambiguous(_))
    ));
}

#[test]
fn a_grounded_free_namespace_releases_only_proven_empty_namespaces() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_grounded_glob_answer_releases_an_empty_namespace_without_a_private_binding.rs"
    );
    let observer = "pub(crate) use crate::m2::A;\n\
        use s0::A as Observed;\n\
        const _: () = assert!(Observed::TAG == 2);\n\
        const _: () = assert!(Observed() == 3);";
    let queried = source.replace("pub(crate) use crate::m2::A;", observer);
    let diagnostics = support::rustc::compile(&queried);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let direct = format!(
        "{queried}\nuse crate::m2::A as Direct;\nconst _: () = assert!(Direct::TAG == 2);\nconst _: () = assert!(Direct() == 3);\n"
    );
    let diagnostics = support::rustc::compile(&direct);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let ws = load(&queried);
    for ns in [Namespace::Type, Namespace::Value] {
        assert_eq!(
            read_as(&ws, "demo::m0::s0", "self::Observed", ns),
            "demo::A"
        );
        assert_eq!(read_as(&ws, "demo", "crate::m2::A", ns), "demo::A");
    }
    assert!(matches!(
        ws.resolve_from_in("demo::m2", "self::s0::A", Namespace::Value),
        Err(ResolveError::NotFound(_))
    ));
    // The private definition is a real absence barrier for the importer.
    // Removing it or making it accessible changes the original declaration
    // validity; neither neighbor is evidence for an empty namespace.
    for replacement in ["", "pub(crate) const fn A() -> u32 { 8 }"] {
        let neighbor = queried.replace("const fn A() -> u32 { 8 }", replacement);
        let diagnostics = support::rustc::compile(&neighbor);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code.as_deref() == Some("E0603")),
            "{diagnostics:?}"
        );
    }
}

#[test]
fn a_final_glob_fallback_waits_for_an_outside_named_cycle() {
    let original = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_grounded_glob_answer_releases_an_empty_namespace_without_a_private_binding.rs"
    );
    let base = original.replacen("mod m2 {", "mod m2 { pub use crate::late::*;", 1);
    for (candidate, valid) in [
        ("pub(crate) use crate::A;", true),
        ("pub struct A {} impl A { pub const TAG: u32 = 12; }", false),
    ] {
        let late = "mod late { pub(crate) use crate::last::A; }";
        let last = "mod last { pub(crate) use crate::late::A; pub use crate::candidate::*; }";
        let candidate = format!("mod candidate {{ {candidate} }}");
        let source = format!("{base}\n{late}\n{last}\n{candidate}\n");
        let observed = source.replace("pub(crate) use crate::m2::A;",
            "pub(crate) use crate::m2::A; use s0::A as Observed; const _: () = assert!(Observed::TAG == 2); const _: () = assert!(Observed() == 3);");
        let diagnostics = support::rustc::compile(&observed);
        if valid {
            assert!(
                diagnostics.iter().all(|d| d.level != "error"),
                "{diagnostics:?}"
            );
        } else {
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.code.as_deref() == Some("E0659")),
                "{diagnostics:?}"
            );
        }
        let (answers, loads) = in_every_order(load, &[&base, late, last, &candidate], &[], |ws| {
            if !valid {
                assert!(
                    ws.resolve_from_in("demo::m0::s0", "s0::A", Namespace::Type)
                        .is_err()
                );
            }
            [Namespace::Type, Namespace::Value]
                .into_iter()
                .map(|ns| read_as(ws, "demo::m0::s0", "s0::A", ns))
                .collect()
        });
        assert_eq!(loads, 24);
        if valid {
            assert_eq!(answers, ["demo::A", "demo::A"]);
        }
    }
}

#[test]
fn a_returning_export_ignores_only_independent_routes_below_its_visibility() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_independent_glob_does_not_widen_a_returning_export.rs"
    );
    let observer = "const fn C() -> u32 { 11 }\n\
        use crate::m2::C as Observed;\n\
        const _: () = assert!(Observed::TAG == 1);\n\
        const _: () = assert!(Observed() == 11);";
    let diagnostics =
        support::rustc::compile(&source.replace("const fn C() -> u32 { 11 }", observer));
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level != "error"),
        "{diagnostics:?}"
    );
    let ws = load(source);
    assert_eq!(
        read_as(&ws, "demo::m2", "crate::m2::C", Namespace::Type),
        "demo::C"
    );
    assert_eq!(
        read_as(&ws, "demo::m2", "crate::m2::C", Namespace::Value),
        "demo::m2::C"
    );
    let visible = source.replacen("\nuse crate::*;", "\npub(crate) use crate::*;", 1);
    let diagnostics =
        support::rustc::compile(&visible.replace("const fn C() -> u32 { 11 }", observer));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    let ws = load(&visible);
    assert_eq!(
        read_as(&ws, "demo::m2", "crate::m2::C", Namespace::Type),
        ambiguous_between("demo::C", "demo::m1::C")
    );
    assert_eq!(
        read_as(&ws, "demo::m2", "crate::m2::C", Namespace::Value),
        "demo::m2::C"
    );
}

#[test]
fn a_settled_returning_named_peer_does_not_invent_a_namespace_export() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_forward_keeps_a_namespace_released_by_a_named_cycle_hidden.rs"
    );
    for (source, present, absent, valid, invalid, code) in [
        (
            source.to_owned(),
            Namespace::Type,
            Namespace::Value,
            "const _: () = assert!(Observed::TAG == 3);",
            "const _: () = assert!(Observed() == 9);",
            "E0423",
        ),
        (
            source.replace(
                "pub const fn B() -> u32 { 1 }",
                "pub struct B {} impl B { pub const TAG: u32 = 1; }",
            ),
            Namespace::Value,
            Namespace::Type,
            "const _: () = assert!(Observed() == 9);",
            "type Absent = Observed;",
            "E0573",
        ),
    ] {
        let source = format!("{source}\nuse m1::B as Observed;\n");
        let diagnostics = support::rustc::compile(&format!("{source}\n{valid}"));
        assert!(
            diagnostics.iter().all(|d| d.level != "error"),
            "{diagnostics:?}"
        );
        let diagnostics = support::rustc::compile(&format!("{source}\n{invalid}"));
        assert!(
            diagnostics.iter().any(|d| d.code.as_deref() == Some(code)),
            "{diagnostics:?}"
        );
        let ws = load(&source);
        assert_eq!(
            ws.resolve_from_in("demo", "self::Observed", present)
                .expect("exported namespace")
                .path,
            "demo::d::B"
        );
        assert!(matches!(
            ws.resolve_from_in("demo", "self::Observed", absent),
            Err(ResolveError::NotFound(_))
        ));
    }
}

#[test]
fn a_forwarded_private_import_preserves_public_peer_namespace_exports() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_forwarded_private_import_keeps_the_public_namespace_family.rs"
    );
    for (source, local_space) in [
        (source.to_owned(), Namespace::Value),
        (
            source.replace(
                "pub const fn B() -> u32 { 1 }",
                "pub struct B {} impl B { pub const TAG: u32 = 1; }",
            ),
            Namespace::Type,
        ),
    ] {
        let lib = format!("{source}\nuse m1::B as Observed;\n");
        let witness = format!(
            "{lib}\nconst _: () = assert!(Observed::TAG == 3);\nconst _: () = assert!(Observed() == 4);\n"
        );
        let diagnostics = support::rustc::compile(&witness);
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.level != "error"),
            "{diagnostics:?}"
        );
        let ws = load(&lib);
        for (space, expected) in [
            (Namespace::Type, "demo::m0::B"),
            (Namespace::Value, "demo::m0::B"),
        ] {
            assert_eq!(
                ws.resolve_from_in("demo", "self::Observed", space)
                    .expect("exported namespace")
                    .path,
                expected
            );
        }
        assert_eq!(
            ws.resolve_from_in("demo::m1", "self::B", local_space)
                .expect("local private binding")
                .path,
            "demo::B"
        );
    }
}

#[test]
fn a_private_forwarded_binding_hides_the_same_glob_item() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_forwarded_binding_hides_the_same_glob_item.rs"
    );
    let cases = [
        (
            source.replace(
                "pub(crate) use crate::m0::*;",
                "pub(crate) use crate::m0::*; use crate::m1::*;",
            ),
            true,
            "demo::B",
        ),
        (
            source.replace(
                "pub(crate) use crate::m0::*;",
                "pub(crate) use crate::m0::*; pub(crate) use crate::m1::*;",
            ),
            false,
            "demo::B",
        ),
        (
            source.replace("pub use crate::m1::B;", ""),
            false,
            "demo::B",
        ),
        (
            source.replace("use crate::m1::s0::B;", ""),
            false,
            "demo::B",
        ),
        (
            source.replace("pub use crate::m1::B;", "pub const fn B() -> u32 { 3 }"),
            false,
            "demo::m0::B",
        ),
        (
            source
                .replace("pub use crate::m1::B;", "pub const fn B() -> u32 { 3 }")
                .replace("pub use crate::*;", "pub use crate::B;"),
            false,
            "demo::m0::B",
        ),
        (source.to_owned(), true, "demo::B"),
    ];
    for (source, hidden, _) in &cases {
        let valid = format!(
            "{source}\nuse m2::B as Observed;\nconst _: () = assert!(Observed::TAG == 9);\n"
        );
        let diagnostics = support::rustc::compile(&valid);
        assert!(
            diagnostics.iter().all(|d| d.level != "error"),
            "{diagnostics:?}"
        );
        let diagnostics = support::rustc::compile(&format!(
            "{valid}\nconst _: () = assert!(Observed() == 3);\n"
        ));
        if *hidden {
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.code.as_deref() == Some("E0423")),
                "{diagnostics:?}"
            );
        } else {
            assert!(
                diagnostics.iter().all(|d| d.level != "error"),
                "{diagnostics:?}"
            );
        }
    }
    for (source, hidden, expected) in cases {
        let ws = load(&format!("{source}\nuse m2::B as Observed;\n"));
        assert_eq!(
            read_as(&ws, "demo", "self::Observed", Namespace::Type),
            "demo::m1::B"
        );
        let value = ws.resolve_from_in("demo", "self::Observed", Namespace::Value);
        if hidden {
            assert!(value.is_err(), "private value escaped: {value:?}");
        } else {
            assert_eq!(value.expect("public glob value").path, expected);
        }
    }
}

#[test]
fn matching_private_imports_preserve_unaliased_glob_families() {
    let namespace = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_forwarded_namespace_keeps_its_matching_glob_route.rs"
    );
    let source = namespace.replace(
        "pub mod m2 {",
        "pub mod m2 { use crate::m2::s1::C as Observed;\n\
        const _: () = assert!(Observed::TAG == 17);\n\
        const _: () = assert!(Observed() == 18);",
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let ws = load(&source);
    for space in [Namespace::Type, Namespace::Value] {
        assert_eq!(
            read_as(&ws, "demo::m2", "self::Observed", space),
            "demo::m3::C"
        );
    }
}

#[test]
fn a_private_alias_retains_a_returning_public_type() {
    let alias = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_forwarded_alias_keeps_a_returning_type.rs"
    );
    let source = alias.replace(
        "mod m0 {",
        "mod m0 { use crate::m3::s0::B as Observed; const _: () = assert!(Observed::TAG == 1);",
    );
    let diagnostics = support::rustc::compile(&source);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let diagnostics =
        support::rustc::compile(&source.replace("Observed::TAG == 1", "Observed() == 1"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0423")),
        "{diagnostics:?}"
    );
    let ws = load(&source);
    assert_eq!(
        read_as(&ws, "demo::m0", "self::Observed", Namespace::Type),
        "demo::m0::A"
    );
    assert!(
        ws.resolve_from_in("demo::m0", "self::Observed", Namespace::Value)
            .is_err()
    );
}

#[test]
fn private_forwarding_preserves_distinct_glob_ambiguity() {
    let source = "pub const fn B() -> u32 { 1 }\n\
        mod left { pub(crate) use crate::B; }\n\
        mod right {\n\
            pub(crate) struct B {} impl B { pub const TAG: u32 = 3; }\n\
            pub(crate) const fn B() -> u32 { 4 }\n\
        }\n\
        mod m0 { pub(crate) use crate::left::*; pub(crate) use crate::right::*; }\n\
        mod m2 { pub(crate) use super::B; }\n\
        pub mod m1 { use crate::m2::B; pub(crate) use crate::m0::*; }\n\
        use m1::B as Observed;\n";
    let valid = format!("{source}const _: () = assert!(Observed::TAG == 3);\n");
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        diagnostics.iter().all(|d| d.level != "error"),
        "{diagnostics:?}"
    );
    let diagnostics = support::rustc::compile(&format!("{valid}const _: u32 = Observed();\n"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    let ws = load(source);
    assert_eq!(
        read_as(&ws, "demo", "self::Observed", Namespace::Type),
        "demo::right::B"
    );
    assert_eq!(
        read_as(&ws, "demo", "self::Observed", Namespace::Value),
        ambiguous_between("demo::B", "demo::right::B")
    );
}

#[test]
fn direct_private_sources_and_missing_public_peers_do_not_export_glob_values() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_forwarded_private_import_keeps_the_public_namespace_family.rs"
    );
    for (source, code, has_type) in [
        (
            source.replace("use crate::m2::B;", "use crate::B;"),
            "E0423",
            true,
        ),
        (
            source.replace(
                "pub(crate) struct B {} impl B { pub const TAG: u32 = 3; }",
                "",
            ),
            "E0603",
            false,
        ),
        (format!("{source}\npub struct B {{}}\n"), "E0603", false),
    ] {
        let lib = format!("{source}\nuse m1::B as Observed;\n");
        let diagnostics = support::rustc::compile(&format!("{lib}\nconst _: u32 = Observed();\n"));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.level == "error"
                    && diagnostic.code.as_deref() == Some(code)),
            "{diagnostics:?}"
        );
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.level == "error")
                .filter_map(|diagnostic| diagnostic.line)
                .all(|line| line > source.lines().count()),
            "an original declaration failed: {diagnostics:?}"
        );
        let ws = load(&lib);
        assert!(
            ws.resolve_from_in("demo", "self::Observed", Namespace::Value)
                .is_err()
        );
        let ty = ws.resolve_from_in("demo", "self::Observed", Namespace::Type);
        if has_type {
            assert_eq!(ty.expect("public type").path, "demo::m0::B");
        } else {
            assert!(ty.is_err());
        }
    }
}

#[test]
fn named_imports_preserve_companions_instead_of_promoting_them_to_independent_routes() {
    for (source, module, path, ty, value) in [
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/a_returning_companion_is_replaced_by_its_independent_canonical_item.rs"
            ),
            "demo::m1::s0",
            "crate::C",
            "demo::m0::C",
            Some("demo::C"),
        ),
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/a_named_return_keeps_the_hidden_candidate_as_a_companion.rs"
            ),
            "demo::m1",
            "self::A",
            "demo::m0::s1::A",
            None,
        ),
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/a_named_import_does_not_widen_a_hidden_returning_candidate.rs"
            ),
            "demo::m1::s1",
            "crate::m1::s0::B",
            "demo::m2::B",
            Some("demo::m0::s0::B"),
        ),
        (
            include_str!(
                "../../../fuzz/corpus/rustc-oracle/a_named_cycle_preserves_the_public_type_and_private_value.rs"
            ),
            "demo::m0",
            "m2::B",
            "demo::B",
            Some("demo::m1::s0::B"),
        ),
    ] {
        let ws = load(source);
        assert_eq!(
            ws.resolve_from_in(module, path, Namespace::Type)
                .unwrap()
                .path,
            ty
        );
        let actual = ws.resolve_from_in(module, path, Namespace::Value);
        match value {
            Some(expected) => assert_eq!(actual.unwrap().path, expected),
            None => assert!(matches!(actual, Err(ResolveError::NotFound(_)))),
        }
    }
}

#[test]
fn an_ambiguous_original_import_is_refused_before_oracle_sanitization() {
    for source in [
        include_str!(
            "../../../fuzz/corpus/rustc-oracle/a_crate_visible_import_that_supplies_an_item_beside_its_companion.rs"
        ),
        include_str!(
            "../../../fuzz/corpus/rustc-oracle/an_item_an_explicit_import_supplies_beside_a_glob_that_carries_it_with_a_companion.rs"
        ),
        "pub mod m1 { pub mod s1 { pub struct B {} } }\n\
         pub mod m2 { pub struct B {} }\n\
         pub mod m0 { pub use crate::m2::*; use crate::m1::s1::*;\nuse crate::mm::*; }\n\
         pub mod w { pub use crate::m2::B; }\n\
         pub mod mm { pub use crate::m0::*; pub use crate::w::*; use self::B as probe; }\n\
         pub mod rr { pub use crate::mm::*; use self::B as probe2; }\n",
    ] {
        let diagnostics = support::rustc::compile(source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code.as_deref() == Some("E0659")),
            "{diagnostics:?}"
        );
        let ws = load(source);
        assert!(matches!(
            ws.resolve_from_in("demo::mm", "self::B", Namespace::Type),
            Err(ResolveError::Ambiguous(_))
        ));
        assert!(
            ws.resolve_from_in("demo::mm", "self::probe", Namespace::Type)
                .is_err()
        );
        let valid = source.replace("use crate::m1::s1::*;\n", "");
        assert_ne!(valid, source);
        let diagnostics = support::rustc::compile(&valid);
        assert!(
            !diagnostics.iter().any(|d| d.level == "error"),
            "{diagnostics:?}"
        );
        assert_eq!(
            load(&valid)
                .resolve_from_in("demo::mm", "self::probe", Namespace::Type)
                .expect("unambiguous named import")
                .path,
            "demo::m2::B"
        );
    }
}

#[test]
fn a_value_import_does_not_erase_the_type_ambiguity_waiting_behind_it() {
    for explicit in [false, true] {
        let mut imports = vec!["use crate::p::*;", "pub use crate::q::*;"];
        if explicit {
            imports.push("use crate::f::B;");
        }
        let (answers, loads) = in_every_order(
            load,
            &[
                "pub mod p { pub struct B; }",
                "pub mod q { pub struct B; }",
                "pub mod f { pub fn B() {} }",
            ],
            &[
                (
                    "pub mod r",
                    &["pub use crate::m::*;", "pub use self::B as Y;"],
                ),
                ("pub mod m", &imports),
            ],
            |ws| {
                let mut answers = vec![resolved_as(ws, "demo::r::Y")];
                for (module, path) in [
                    ("demo::m", "self::B"),
                    ("demo::r", "self::B"),
                    ("demo::r", "self::Y"),
                    ("demo", "crate::r::Y"),
                ] {
                    answers.push(read_as(ws, module, path, Namespace::Type));
                }
                answers
            },
        );
        assert_eq!(loads, if explicit { 1440 } else { 480 });
        assert_eq!(
            answers,
            vec![ambiguous_between("demo::p::B", "demo::q::B"); 5]
        );
    }
}

#[test]
fn rustc_refuses_the_named_import_of_the_ambiguous_type() {
    let source = "pub mod r { pub use crate::m::*; pub use self::B as Y; }\n\
        pub mod m { use crate::p::*; use crate::f::B; pub use crate::q::*; }\n\
        pub mod p { pub struct B; }\n\
        pub mod q { pub struct B; }\n\
        pub mod f { pub fn B() {} }\n";
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0432")),
        "{diagnostics:?}"
    );
    let valid = source.replace("use crate::p::*;", "");
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        !diagnostics.iter().any(|d| d.level == "error"),
        "{diagnostics:?}"
    );
}

#[test]
fn a_private_glob_chain_does_not_fabricate_a_unique_item() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_private_glob_chain_beside_an_accessible_route.rs"
    );
    let diagnostics = support::rustc::compile(&format!("{source}\nuse self::L as Probe;"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0432")),
        "{diagnostics:?}"
    );
    let ws = load(source);
    assert!(
        matches!(
            ws.resolve_from_in("demo", "self::L", Namespace::Type),
            Err(ResolveError::Ambiguous(_) | ResolveError::NotFound(_))
        ),
        "{}",
        read_as(&ws, "demo", "self::L", Namespace::Type)
    );
    let valid = source.replace("use crate::top0::a::p0::c0::*;", "");
    let ws = load(&valid);
    assert_eq!(
        read_as(&ws, "demo", "self::L", Namespace::Type),
        "demo::top0::u::L"
    );
}

#[test]
fn a_visible_route_waiting_beside_a_hidden_arrival_keeps_the_ambiguity() {
    let source = include_str!(
        "../../../fuzz/corpus/rustc-oracle/a_visible_route_waiting_beside_a_hidden_arrival.rs"
    );
    let diagnostics = support::rustc::compile(&format!("{source}\npub const PROBE: u32 = K::TAG;"));
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    let ws = load(source);
    assert_eq!(
        read_as(&ws, "demo", "self::K", Namespace::Type),
        ambiguous_between("demo::top::t::K", "demo::top::u::K")
    );
}

#[test]
fn an_independent_glob_does_not_shadow_an_inherited_ambiguity() {
    for imports in [
        "pub use crate::mixed::*; pub use crate::a::*;",
        "pub use crate::mixed::*; use crate::a::*;",
        "pub use crate::mixed::*; pub use crate::c0::*; use crate::a::*;",
    ] {
        let source = format!(
            "pub mod a {{ pub struct X; }}\n\
            pub mod b {{ pub struct X; }}\n\
            pub mod mixed {{ pub use crate::a::*; use crate::b::*; }}\n\
            pub mod c0 {{ pub use crate::a::*; }}\n\
            pub mod r0 {{ {imports} pub type Probe = X; }}\n"
        );
        let diagnostics = support::rustc::compile(&source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code.as_deref() == Some("E0659")),
            "{source}\n{diagnostics:?}"
        );
        let ws = load(&source);
        assert_eq!(
            read_as(&ws, "demo::r0", "self::X", Namespace::Type),
            ambiguous_between("demo::a::X", "demo::b::X"),
            "{source}"
        );
        let valid = source.replace("use crate::b::*;", "");
        let diagnostics = support::rustc::compile(&valid);
        assert!(
            !diagnostics.iter().any(|d| d.level == "error"),
            "{diagnostics:?}"
        );
        assert_eq!(
            read_as(&load(&valid), "demo::r0", "self::X", Namespace::Type),
            "demo::a::X"
        );
    }
}

#[test]
fn a_cycle_with_an_independent_carrier_route_keeps_its_ambiguity() {
    let source = "pub mod m1 { pub const fn A() -> u32 { 7 } }\n\
        pub mod m0 { pub use crate::m3::s1::*; use crate::m3::*; }\n\
        mod m3 {\n\
            pub mod s0 { pub const fn A() -> u32 { 15 } }\n\
            pub(crate) use crate::m1::*;\n\
            use crate::m3::s0::*;\n\
            pub(crate) use crate::m0::*;\n\
            pub mod s1 { pub use crate::m1::*; pub const CHECK: () = assert!(crate::m0::A() == 7); }\n\
        }\n";
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    let ws = load(source);
    assert_eq!(
        read_as(&ws, "demo::m3::s1", "crate::m0::A", Namespace::Value),
        ambiguous_between("demo::m1::A", "demo::m3::s0::A")
    );
    // Without the bypass the plain carrier binding must return through
    // m0; its independent s1 route then supports the cycle replacement.
    let valid = source.replace("pub(crate) use crate::m1::*;", "");
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        !diagnostics.iter().any(|d| d.level == "error"),
        "{diagnostics:?}"
    );
    assert_eq!(
        read_as(
            &load(&valid),
            "demo::m3::s1",
            "crate::m0::A",
            Namespace::Value
        ),
        "demo::m1::A"
    );
}

#[test]
fn a_private_independent_supporter_and_a_returning_cycle_keep_the_hidden_route() {
    let source = "pub mod d { pub struct X {} }\n\
        pub mod p { pub struct X {} }\n\
        pub mod a { pub use crate::d::*; use crate::p::*; }\n\
        pub mod o { pub use crate::r::*; }\n\
        pub mod r { pub use crate::a::*; use crate::d::*; pub use self::t::*;\n\
            pub mod t { pub use super::*; } use self::X as Probe; }\n";
    let diagnostics = support::rustc::compile(source);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("E0659")),
        "{diagnostics:?}"
    );
    let ws = load(source);
    for module in ["demo::o", "demo::r", "demo::r::t"] {
        assert_eq!(
            read_as(&ws, module, "self::X", Namespace::Type),
            ambiguous_between("demo::d::X", "demo::p::X")
        );
    }
    let valid = source.replace("use crate::p::*;", "");
    let diagnostics = support::rustc::compile(&valid);
    assert!(
        !diagnostics.iter().any(|d| d.level == "error"),
        "{diagnostics:?}"
    );
    let ws = load(&valid);
    for module in ["demo::o", "demo::r", "demo::r::t"] {
        assert_eq!(
            read_as(&ws, module, "self::X", Namespace::Type),
            "demo::d::X"
        );
    }
}
