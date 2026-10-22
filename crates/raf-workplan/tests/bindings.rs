// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The binding rules over a source workspace in memory: what resolves, what
//! does not, and the role each list requires.

mod support;

use raf_source::{MemoryTree, Workspace};
use raf_workplan::{BindingKind, ExampleKind, Finding, LinkContext, Workplan};
use support::fixtures::{GAMMA, bound, bound_findings};
use support::{assert_clean, demo_workspace, without_sync_findings};

const CASE: &str = "[demo::test_crate::it::case]";

/// The single binding refusal, as `(kind, message)`. Other complete-policy
/// findings remain in the producer result and are not claimed absent here.
fn only_binding_refusal(findings: &[Finding]) -> (&'static str, &str) {
    let binding: Vec<&Finding> = findings
        .iter()
        .filter(|finding| finding.kind.starts_with("binding-"))
        .collect();
    assert_eq!(binding.len(), 1, "{findings:#?}");
    assert_eq!(binding[0].file, GAMMA);
    (binding[0].kind, binding[0].message.as_str())
}

#[test]
fn bindings_that_name_real_symbols_resolve() {
    assert_clean(&bound_findings("[demo::real]", CASE, "[demo::real]"));
}

#[test]
fn a_binding_through_a_re_export_resolves_to_its_definition() {
    assert_clean(&bound_findings("[demo::shown]", CASE, "[demo::shown]"));
    let repo = bound("[demo::shown]", CASE, "[demo::example_crate::show]");
    let tree: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    let binding = plan
        .graph()
        .bindings_of("cap:gamma")
        .find(|b| b.path() == "demo::shown")
        .expect("the binding is in the graph");
    let sites = binding.resolution().expect("it resolves");
    assert_eq!(sites.len(), 1);
    let item = workspace.item(sites[0]).expect("an indexed item");
    assert_eq!(&*item.path, "demo::inner::go");
}

#[test]
fn the_role_is_judged_on_the_definition_a_re_export_leads_to() {
    let findings = bound_findings("[demo::unit_helper]", CASE, "[demo::example_crate::show]");
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-wrong-role");
    assert_eq!(
        message,
        "cap:gamma code binding `demo::unit_helper` names an item that is test code or is not in a shipping target"
    );
}

#[test]
fn a_binding_to_a_missing_path_is_reported_with_its_capability() {
    let findings = bound_findings("[demo::nope]", CASE, "[demo::example_crate::show]");
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-unresolved");
    assert_eq!(
        message,
        "cap:gamma code binding `demo::nope` does not resolve: `demo::nope` names nothing in the workspace"
    );
}

#[test]
fn a_binding_to_a_macro_generated_function_is_not_found() {
    let findings = bound_findings("[demo::generated]", CASE, "[demo::example_crate::show]");
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-macro-generated");
    assert_eq!(
        message,
        "cap:gamma code binding `demo::generated` does not resolve: `demo::generated` names nothing in the workspace; the name appears in the macro invocation at src/lib.rs:6, which may generate it; a generated symbol is not in the index and cannot be bound"
    );
}

#[test]
fn a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found() {
    let findings = bound_findings(
        "[demo::body::FenceBlock]",
        CASE,
        "[demo::example_crate::show]",
    );
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-unresolved");
    assert_eq!(
        message,
        "cap:gamma code binding `demo::body::FenceBlock` does not resolve: `demo::body::FenceBlock` names nothing in the workspace"
    );
    assert_clean(&bound_findings(
        "[demo::body::blocks::FenceBlock, demo::body::in_body]",
        CASE,
        "[demo::example_crate::show]",
    ));
}

#[test]
fn every_unresolved_binding_of_every_list_is_reported() {
    let findings = bound_findings(
        "[demo::real, demo::nope, demo::generated]",
        "[demo::test_crate::it::missing]",
        "[demo::real, demo::absent]",
    );
    assert_eq!(findings.len(), 4, "{findings:#?}");
    for needle in [
        "code binding `demo::nope`",
        "code binding `demo::generated`",
        "tests binding `demo::test_crate::it::missing`",
        "examples binding `demo::absent`",
    ] {
        assert!(
            findings.iter().any(|f| f.message.contains(needle)),
            "no finding for {needle}: {findings:#?}"
        );
    }
}

#[test]
fn a_path_two_globs_both_bring_in_is_ambiguous_and_names_both_definitions() {
    let findings = bound_findings("[demo::dup]", CASE, "[demo::example_crate::show]");
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-ambiguous");
    assert!(
        message.contains("`demo::left::dup` (src/lib.rs:")
            && message.contains("`demo::right::dup` (src/lib.rs:"),
        "{message}"
    );
    assert!(
        findings[0]
            .message
            .starts_with("cap:gamma code binding `demo::dup` names more than one symbol")
    );
}

#[test]
fn a_tests_binding_must_name_a_function_marked_test() {
    let findings = bound_findings(
        "[demo::real]",
        "[demo::test_crate::it::plain]",
        "[demo::example_crate::show]",
    );
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-wrong-role");
    assert_eq!(
        message,
        "cap:gamma tests binding `demo::test_crate::it::plain` names an item that is not a function marked `#[test]`"
    );
    let findings = bound_findings(
        "[demo::real]",
        "[demo::real]",
        "[demo::example_crate::show]",
    );
    let (kind, message) = only_binding_refusal(&findings);
    assert_eq!(kind, "binding-wrong-role");
    assert!(message.contains("tests binding `demo::real` names an item"));
}

#[test]
fn a_code_binding_must_name_a_shipping_item() {
    for binding in ["demo::unit::helper", "demo::test_crate::it::case"] {
        let findings = bound_findings(&format!("[{binding}]"), CASE, "[demo::example_crate::show]");
        let (kind, message) = only_binding_refusal(&findings);
        assert_eq!(kind, "binding-wrong-role");
        assert_eq!(
            message,
            format!(
                "cap:gamma code binding `{binding}` names an item that is test code or is not in a shipping target"
            )
        );
    }
}

#[test]
fn an_examples_binding_is_an_example_target_or_a_documented_library_item() {
    assert_clean(&bound_findings(
        "[demo::real]",
        CASE,
        "[demo::example_crate::show, demo::real, demo::shown]",
    ));
    let wrong = [
        // Documented, but the example target runs no documentation tests.
        "demo::example_crate::show::helper",
        // Documented, and the manifest asks for doctests, which cargo honours
        // for the library alone.
        "demo::example_crate::flagged::flagged_helper",
        // In the library and without documentation.
        "demo::bare",
        // A target that is not an example.
        "demo::test_crate::it",
        // A module inside an example crate is not its crate root.
        "demo::example_crate::show::inner",
        // A function of that target, whatever its attributes.
        "demo::test_crate::it::case",
        // Documented, in test code: rustdoc does not collect it.
        "demo::unit::helper",
    ];
    for binding in wrong {
        let findings = bound_findings("[demo::real]", CASE, &format!("[{binding}]"));
        let (kind, message) = only_binding_refusal(&findings);
        assert_eq!(kind, "binding-wrong-role", "{binding}");
        assert!(
            message.starts_with(&format!(
                "cap:gamma examples binding `{binding}` names an item that is neither the crate root of an example target nor a documented item of a library whose documentation tests run"
            )),
            "{message}"
        );
    }
}

#[test]
fn every_finding_of_a_wrong_role_names_the_fix_for_its_list() {
    for (code, tests, examples, needle) in [
        (
            "[demo::unit::helper]",
            CASE,
            "[demo::example_crate::show]",
            "not test code",
        ),
        (
            "[demo::real]",
            "[demo::real]",
            "[demo::example_crate::show]",
            "marked `#[test]`",
        ),
        ("[demo::real]", CASE, "[demo::bare]", "example_crate"),
    ] {
        let findings = bound_findings(code, tests, examples);
        assert!(findings[0].fix.contains(needle), "{:#?}", findings[0]);
    }
}

#[test]
fn the_graph_keeps_each_resolution_for_the_rules_and_the_views_that_read_it() {
    let repo = bound(
        "[demo::real, demo::nope]",
        CASE,
        "[demo::example_crate::show, demo::real]",
    );
    let tree: MemoryTree = repo.iter().collect();
    let workspace = demo_workspace();
    let plan = Workplan::load(&tree, &workspace);
    let bindings: Vec<_> = plan.graph().bindings_of("cap:gamma").collect();
    let summary: Vec<(BindingKind, &str, bool)> = bindings
        .iter()
        .map(|b| (b.kind(), b.path(), b.resolution().is_ok()))
        .collect();
    assert_eq!(
        summary,
        [
            (BindingKind::Code, "demo::nope", false),
            (BindingKind::Code, "demo::real", true),
            (BindingKind::Test, "demo::test_crate::it::case", true),
            (BindingKind::Example, "demo::example_crate::show", true),
            (BindingKind::Example, "demo::real", true),
        ]
    );
    assert!(bindings[0].resolution().is_err());
    let kinds: Vec<Option<ExampleKind>> = bindings[3..]
        .iter()
        .map(|b| b.example_kind(b.items()[0]))
        .collect();
    assert_eq!(
        kinds,
        [Some(ExampleKind::Target), Some(ExampleKind::Doctest)]
    );
    // A binding of another list carries no example kind.
    assert_eq!(bindings[2].example_kind(bindings[2].items()[0]), None);
    assert_eq!(plan.graph().bindings_of("cap:absent").count(), 0);
}

#[test]
fn a_capability_that_binds_nothing_has_no_binding_findings() {
    // The designed and considered capabilities of the fixture carry empty
    // lists; an implementing one may too.
    assert_clean(&support::check(&support::fixtures::valid()));
    let findings = support::fixtures::mutated(GAMMA, "state: delivered", "state: implementing");
    assert!(
        findings.iter().all(|f| !f.kind.starts_with("binding-")),
        "{findings:#?}"
    );
}

/// The findings of the valid repository against a source workspace with the
/// library `lib`, the capability gamma binding `code`.
fn findings_with_lib(lib: &str, code: &str) -> Vec<Finding> {
    findings_with_lists(
        lib,
        &format!("[{code}]"),
        CASE,
        "[demo::example_crate::show]",
    )
}

/// As [`findings_with_lib`], with all three lists given.
fn findings_with_lists(lib: &str, code: &str, tests: &str, examples: &str) -> Vec<Finding> {
    let mut files = vec![
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        ),
        ("src/lib.rs", lib),
        ("src/side.rs", "make!(side_made);\n"),
        ("tests/it.rs", "#[test]\nfn case() {}\n"),
    ];
    if examples.contains("demo::example_crate::show") {
        files.push(("examples/show.rs", "fn main() {}\n"));
    }
    let source = Workspace::load(&files.into_iter().collect::<MemoryTree>(), &["."])
        .expect("the workspace loads");
    let mut repo = bound(code, tests, examples);
    repo.remove(support::AUXILIARY);
    let tree: MemoryTree = repo.iter().collect();
    let context = LinkContext::of_tree(&tree);
    without_sync_findings(Workplan::load_with(&tree, &source, &context).check())
}

#[test]
fn the_macro_hint_is_given_only_for_a_macro_at_the_item_level_of_the_parent_module() {
    let lib = "\
macro_rules! make { ($n:ident) => { pub fn $n() {} }; }
mod side;
pub mod inner {
    make!(inner_made);
    pub fn body() { make!(in_a_body); }
}
pub mod other {}
make!(root_made);
";
    // The invocation is in the module that is the parent of the path.
    for (path, line) in [
        ("demo::root_made", "src/lib.rs:8"),
        ("demo::inner::inner_made", "src/lib.rs:4"),
    ] {
        let found = findings_with_lib(lib, path);
        let hit = found
            .iter()
            .find(|f| f.kind == "binding-macro-generated")
            .unwrap_or_else(|| panic!("{path}: {found:#?}"));
        assert!(
            hit.message.contains(&format!("invocation at {line}")),
            "{hit:#?}"
        );
    }
    // A name no invocation of its module mentions is a plain missing path,
    // whichever other module has such a call.
    for path in [
        "demo::inner_made",
        "demo::inner::root_made",
        "demo::other::inner_made",
        "demo::inner::in_a_body",
        "demo::side_made",
        "demo::nowhere::root_made",
    ] {
        let found = findings_with_lib(lib, path);
        assert!(
            found.iter().any(|f| f.kind == "binding-unresolved"),
            "{path}: {found:#?}"
        );
        assert!(
            found.iter().all(|f| f.kind != "binding-macro-generated"),
            "{path}: {found:#?}"
        );
    }
}

#[test]
fn a_macro_in_the_file_of_a_module_is_read_for_the_path_of_that_module() {
    let lib = "mod side;\nmacro_rules! make { ($n:ident) => { pub fn $n() {} }; }\n";
    let found = findings_with_lib(lib, "demo::side::side_made");
    assert!(
        found.iter().any(|f| f.kind == "binding-macro-generated"
            && f.message.contains("invocation at src/side.rs:1")),
        "{found:#?}"
    );
}

#[test]
fn a_library_that_runs_no_documentation_tests_has_no_doctest_examples() {
    let findings = |enabled: bool| {
        let manifest = format!(
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[lib]\ndoctest = {enabled}\n"
        );
        let source = Workspace::load(
            &MemoryTree::from([
                ("Cargo.toml", manifest.as_str()),
                ("src/lib.rs", "/// Documented.\npub fn real() {}\n"),
                ("tests/it.rs", "#[test]\nfn case() {}\n"),
            ]),
            &["."],
        )
        .expect("the workspace loads");
        let mut repo = bound("[demo::real]", CASE, "[demo::real]");
        repo.remove(support::AUXILIARY);
        let tree: MemoryTree = repo.iter().collect();
        let context = LinkContext::of_tree(&tree);
        without_sync_findings(Workplan::load_with(&tree, &source, &context).check())
    };
    let found = findings(false);
    let (kind, message) = only_binding_refusal(&found);
    assert_eq!(kind, "binding-wrong-role");
    assert!(
        message.contains("examples binding `demo::real`"),
        "{message}"
    );
    assert_clean(&findings(true));
}

const MAKE: &str = "macro_rules! make { ($n:ident) => { pub fn $n() {} }; }\n";

#[test]
fn a_name_held_only_inside_a_nested_group_of_the_arguments_is_still_named() {
    for (invocation, path) in [
        ("make!(wrap(generated));\n", "demo::generated"),
        ("make! { fn generated() {} }\n", "demo::generated"),
        ("make!([a, {b, deep}]);\n", "demo::deep"),
    ] {
        let found = findings_with_lib(&format!("{MAKE}{invocation}"), path);
        assert!(
            found.iter().any(|f| f.kind == "binding-macro-generated"),
            "{invocation}: {found:#?}"
        );
    }
}

#[test]
fn only_a_path_before_a_bang_and_a_group_is_an_invocation() {
    for lib in [
        // An inner attribute, whose `!` follows `#`.
        "#![allow(generated)]\npub fn real() {}\n",
        // An expression negation.
        "const A: bool = !(generated);\n",
        // A macro definition: `macro_rules ! name { .. }`.
        "macro_rules! wrapper { () => { pub fn generated() {} } }\n",
        // A negative impl.
        "pub struct S;\nimpl !Send for S {}\npub fn generated_not() {}\n",
    ] {
        let found = findings_with_lib(lib, "demo::generated");
        assert!(
            found.iter().any(|f| f.kind == "binding-unresolved"),
            "{lib}: {found:#?}"
        );
        assert!(
            found.iter().all(|f| f.kind != "binding-macro-generated"),
            "{lib}: {found:#?}"
        );
    }
}

#[test]
fn the_macro_hint_is_only_for_a_parent_that_is_a_module() {
    // `made` appears in a call inside the body of the function `f`, which is
    // not a module and defines no item.
    let lib = format!("{MAKE}pub fn f() {{ make!(made); }}\n");
    let found = findings_with_lib(&lib, "demo::f::made");
    assert!(
        found.iter().any(|f| f.kind == "binding-unresolved"),
        "{found:#?}"
    );
    assert!(
        found.iter().all(|f| f.kind != "binding-macro-generated"),
        "{found:#?}"
    );
}

#[test]
fn one_failing_site_fails_a_binding_that_resolves_to_several() {
    // `a` is defined twice under `cfg`; the rule is that every definition
    // must fit the list, as for the precedent: one failing site fails.
    let code_lib = "#[cfg(test)]\npub fn a() {}\n#[cfg(not(test))]\npub fn a() {}\n";
    let found = findings_with_lists(code_lib, "[demo::a]", CASE, "[demo::example_crate::show]");
    let (kind, message) = only_binding_refusal(&found);
    assert_eq!(kind, "binding-wrong-role");
    assert!(message.contains("code binding `demo::a`"), "{message}");

    let tests_lib =
        "pub fn real() {}\n#[cfg(test)]\n#[test]\nfn a() {}\n#[cfg(not(test))]\nfn a() {}\n";
    let found = findings_with_lists(
        tests_lib,
        "[demo::real]",
        "[demo::a]",
        "[demo::example_crate::show]",
    );
    let (kind, message) = only_binding_refusal(&found);
    assert_eq!(kind, "binding-wrong-role");
    assert!(message.contains("tests binding `demo::a`"), "{message}");

    let docs_lib = "/// Documented.\n#[cfg(test)]\npub fn a() {}\n/// Documented.\n#[cfg(not(test))]\npub fn a() {}\n";
    let found = findings_with_lists(docs_lib, "[demo::real]", CASE, "[demo::a]");
    assert!(
        found
            .iter()
            .any(|f| f.kind == "binding-wrong-role"
                && f.message.contains("examples binding `demo::a`")),
        "{found:#?}"
    );
    // The same twins that both fit the list are accepted.
    let both =
        "/// One.\n#[cfg(unix)]\npub fn a() {}\n/// Two.\n#[cfg(not(unix))]\npub fn a() {}\n";
    assert_clean(&findings_with_lists(both, "[demo::a]", CASE, "[demo::a]"));
}

#[test]
fn a_missing_example_target_is_reported_with_its_capability_list_and_binding() {
    let found = bound_findings("[demo::real]", CASE, "[demo::example_crate::absent]");
    let (kind, message) = only_binding_refusal(&found);
    assert_eq!(kind, "binding-unresolved");
    assert_eq!(
        message,
        "cap:gamma examples binding `demo::example_crate::absent` does not resolve: `demo::example_crate::absent` names nothing in the workspace"
    );
}

#[test]
fn a_failure_with_no_name_to_find_says_a_macro_or_derive_may_be_behind_it() {
    let found = bound_findings("[demo::nope]", CASE, "[demo::example_crate::show]");
    assert!(
        found[0].fix.contains("macro, derive or attribute"),
        "{:#?}",
        found[0]
    );
}
