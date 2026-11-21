// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Pattern scope and the counted-site ratchet of patterns and rules.

use super::{
    Builder, DebtRecord, FileCtx, Format, Ledger, MemoryTree, Rule, RuleFixtures, RuleHit, Term,
    Triple, declared_pattern, fixture_gate, job, lexical_pattern_graph, put, vocab,
};

pub(super) fn ratcheted_declaration() -> Builder {
    let mut graph = declared_pattern(Some("call"), true);
    put(&mut graph, "#pattern", "ratcheted", Term::boolean(true));
    graph
}

/// `ratcheted` is stated only as true, once, and only for a pattern whose jobs
/// are enforced; a forbidden pattern states nothing and still loads.
pub(super) fn ratcheted_declarations_are_true_only_unique_and_held_by_enforced_jobs() {
    let load = |graph: Builder| {
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
    };
    assert!(load(ratcheted_declaration()).unwrap().patterns()[0].ratcheted);
    assert!(
        !load(declared_pattern(Some("call"), true))
            .unwrap()
            .patterns()[0]
            .ratcheted,
        "a pattern that states nothing is forbidden"
    );
    let mut stated_false = declared_pattern(Some("call"), true);
    put(
        &mut stated_false,
        "#pattern",
        "ratcheted",
        Term::boolean(false),
    );
    let mut not_boolean = declared_pattern(Some("call"), true);
    put(
        &mut not_boolean,
        "#pattern",
        "ratcheted",
        Term::simple_literal("true"),
    );
    for graph in [stated_false, not_boolean] {
        let error = load(graph).unwrap_err().to_string();
        assert!(
            error.contains("ratcheted is stated only as true"),
            "{error}"
        );
    }
    let mut twice = ratcheted_declaration();
    put(&mut twice, "#pattern", "ratcheted", Term::boolean(false));
    let error = load(twice).unwrap_err().to_string();
    assert!(error.contains("ratcheted"), "{error}");
    let unenforced = |graph: &mut Builder| {
        assert!(
            graph
                .remove(Triple {
                    subject: Term::iri("#job"),
                    predicate: Term::iri(format!("{}enforced", vocab::REPORT)),
                    object: Term::boolean(true),
                })
                .unwrap()
        );
        put(graph, "#job", "enforced", Term::boolean(false));
    };
    let mut held = ratcheted_declaration();
    unenforced(&mut held);
    let error = load(held).unwrap_err().to_string();
    assert!(error.contains("not enforced"), "{error}");
    let mut forbidden = declared_pattern(Some("call"), true);
    unenforced(&mut forbidden);
    assert!(load(forbidden).is_ok());
    // Another job that reaches the ratcheted pattern through a forbidden rule
    // fingerprint must be enforced too; the enforced neighbour loads.
    let second = |enforced: bool| {
        let mut graph = ratcheted_declaration();
        put(&mut graph, "", "job", Term::iri("#second"));
        put(
            &mut graph,
            "#second",
            "type",
            Term::iri(format!("{}LedgerJob", vocab::REPORT)),
        );
        for (key, value) in [
            ("jobId", "second"),
            ("summary", "Reaches the pattern by rule"),
            ("homeSymbol", "demo::home"),
            ("forbiddenFingerprint", "rule:approved-call"),
        ] {
            put(&mut graph, "#second", key, Term::simple_literal(value));
        }
        put(&mut graph, "#second", "enforced", Term::boolean(enforced));
        graph
    };
    let error = load(second(false)).unwrap_err().to_string();
    assert!(
        error.contains("not enforced") && error.contains("second"),
        "{error}"
    );
    assert!(load(second(true)).is_ok());
}

pub(super) fn scoped_pattern(scope: &[(&str, &str)], shipping_only: bool) -> Builder {
    let mut graph = lexical_pattern_graph("call", "std::mem::drop");
    for (key, value) in scope {
        put(&mut graph, "#pattern", key, Term::simple_literal(*value));
    }
    if shipping_only {
        put(&mut graph, "#pattern", "shippingOnly", Term::boolean(true));
    }
    graph
}

/// One drop call placed at `site`, beside the approved home, in a package with
/// a library, an integration test, a build script and a legacy module.
fn scoped_tree(site: &str, graph: Builder) -> MemoryTree {
    let call = "std::mem::drop(2);";
    let mut lib = String::from(
        "pub mod legacy;\npub fn home(){std::mem::drop(1);}\npub use home as alias;\n",
    );
    let mut files = vec![
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_owned(),
        ),
        ("src/legacy/mod.rs", "pub fn old(){}\n".to_owned()),
        ("tests/it.rs", "#[test]\nfn clean(){}\n".to_owned()),
        ("build.rs", "fn main(){}\n".to_owned()),
    ];
    match site {
        "lib" => lib = format!("{lib}pub fn alternate(){{{call}}}\n"),
        "cfg-test" => {
            lib = format!("{lib}#[cfg(test)]\nmod checks {{ fn alternate(){{{call}}} }}\n");
        }
        "legacy" => files[1].1 = format!("pub fn old(){{{call}}}\n"),
        "test" => files[2].1 = format!("#[test]\nfn it(){{{call}}}\n"),
        "build" => files[3].1 = format!("fn main(){{{call}}}\n"),
        _ => unreachable!("closed fixture sites"),
    }
    files.push(("src/lib.rs", lib));
    files.push((
        "helpers-ledger.ttl",
        graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
    ));
    files.into_iter().collect()
}

fn scoped_copies(site: &str, scope: &[(&str, &str)], shipping_only: bool) -> usize {
    let tree = scoped_tree(site, scoped_pattern(scope, shipping_only));
    fixture_gate(".").check_tree(&tree).unwrap().jobs()[0].copies
}

const RATCHET_ONE: &str = "pub fn one(){std::mem::drop(1);let _=\"alpha\";}\npub fn two(){std::mem::drop(2);let _=\"beta\";}\n";
const RATCHET_HOME: &str = "pub fn home(){std::mem::drop(0);}";
const RATCHET_THREE: &str = "pub fn three(){std::mem::drop(3);let _=\"gamma\";}\n";

/// Modules `src/<name>.rs` beside the approved home, the ratcheted or forbidden
/// drop-call pattern, and an optional recorded debt text.
fn ratchet_tree(
    modules: &[(&str, &str)],
    home: &str,
    record: Option<&str>,
    ratcheted: bool,
) -> MemoryTree {
    let mut graph = lexical_pattern_graph("call", "std::mem::drop");
    if ratcheted {
        put(&mut graph, "#pattern", "ratcheted", Term::boolean(true));
    }
    ratchet_tree_over(graph, modules, home, record)
}

/// `ratchet_tree` over an already built ledger graph.
fn ratchet_tree_over(
    graph: Builder,
    modules: &[(&str, &str)],
    home: &str,
    record: Option<&str>,
) -> MemoryTree {
    let declared = modules
        .iter()
        .map(|(name, _)| format!("pub mod {name};\n"))
        .collect::<Vec<_>>()
        .concat();
    let mut files = vec![
        (
            "Cargo.toml".to_owned(),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_owned(),
        ),
        (
            "src/lib.rs".to_owned(),
            format!("{declared}{home}\npub use home as alias;\n"),
        ),
        (
            "helpers-ledger.ttl".to_owned(),
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ];
    files.extend(
        modules
            .iter()
            .map(|(name, body)| (format!("src/{name}.rs"), (*body).to_owned())),
    );
    if let Some(record) = record {
        files.push(("census-debt.ttl".to_owned(), record.to_owned()));
    }
    files.into_iter().collect()
}

#[test]
fn ratcheted_patterns_record_counted_sites_and_hold_both_directions() {
    let check = |modules: &[(&str, &str)], record: Option<&str>| {
        fixture_gate(".")
            .check_tree(&ratchet_tree(modules, RATCHET_HOME, record, true))
            .unwrap()
    };
    let both = [("a", RATCHET_ONE), ("b", RATCHET_THREE)];
    // Adoption: three hits in two files are recorded as exactly three sites,
    // and the pattern's hits are debt rather than findings.
    let adopted = check(&both, None);
    let debt = adopted.debt();
    assert_eq!(
        debt.ratcheted().iter().collect::<Vec<_>>(),
        ["approved-call"]
    );
    assert_eq!(debt.sites().len(), 3);
    assert!(debt.sites().values().all(|copies| *copies == 1));
    let files: Vec<_> = debt
        .sites()
        .keys()
        .map(|site| site.file().to_string())
        .collect();
    assert_eq!(files, ["src/a.rs", "src/a.rs", "src/b.rs"]);
    assert!(
        adopted
            .findings()
            .iter()
            .all(|finding| finding.contains("Adopt")),
        "{:?}",
        adopted.findings()
    );
    let record = debt.to_turtle().unwrap();
    assert_eq!(
        DebtRecord::from_turtle(&record)
            .unwrap()
            .to_turtle()
            .unwrap(),
        record
    );
    assert!(check(&both, Some(&record)).is_clean());
    // Inserting lines above every recorded site leaves the record byte-identical.
    let shifted = [
        ("a", format!("// a\n// b\n\n{RATCHET_ONE}")),
        ("b", format!("\n\n{RATCHET_THREE}")),
    ];
    let shifted: Vec<_> = shifted.iter().map(|(n, b)| (*n, b.as_str())).collect();
    let moved = check(&shifted, Some(&record));
    assert!(moved.is_clean(), "{:?}", moved.findings());
    assert_eq!(moved.debt().to_turtle().unwrap(), record);
    // A fourth hit is growth whether it is a new key or repeats a recorded one.
    let new_key = format!("{RATCHET_ONE}pub fn four(){{std::mem::drop(4);let _=\"delta\";}}\n");
    let repeated = "pub fn one(){std::mem::drop(1);std::mem::drop(9);let _=\"alpha\";}\npub fn two(){std::mem::drop(2);let _=\"beta\";}\n";
    for (a, recorded, present) in [(new_key.as_str(), 0, 1), (repeated, 1, 2)] {
        let grown = check(&[("a", a), ("b", RATCHET_THREE)], Some(&record));
        assert_eq!(grown.findings().len(), 1, "{:?}", grown.findings());
        let finding = &grown.findings()[0];
        assert!(finding.contains("growth cannot be recorded"), "{finding}");
        assert!(
            finding.contains(&format!("recorded: {recorded}, present: {present}")),
            "{finding}"
        );
        assert!(
            finding.contains("src/a.rs:"),
            "growth names its hit: {finding}"
        );
        assert!(
            record_may_not_admit(&record, grown.debt()),
            "a record cannot absorb the growth"
        );
    }
    // A falling count fails check until the record is lowered; the lowered
    // record then passes.
    let two_hits = "pub fn one(){std::mem::drop(1);std::mem::drop(9);let _=\"alpha\";}\npub fn two(){std::mem::drop(2);let _=\"beta\";}\n";
    let wide = check(&[("a", two_hits), ("b", RATCHET_THREE)], None)
        .debt()
        .to_turtle()
        .unwrap();
    let fallen = check(&both, Some(&wide));
    assert_eq!(fallen.findings().len(), 1, "{:?}", fallen.findings());
    assert!(
        fallen.findings()[0].contains("lower the record"),
        "{:?}",
        fallen.findings()
    );
    assert!(check(&both, Some(&fallen.debt().to_turtle().unwrap())).is_clean());
    // A recorded site whose hit is gone, or whose file is deleted, is stale.
    let unhit = [
        ("a", RATCHET_ONE),
        ("b", "pub fn three(){let _=\"gamma\";}\n"),
    ];
    for modules in [&unhit[..], &both[..1]] {
        let stale = check(modules, Some(&record));
        assert_eq!(stale.findings().len(), 1, "{:?}", stale.findings());
        assert!(
            stale.findings()[0].contains("stale site"),
            "{:?}",
            stale.findings()
        );
        assert!(check(modules, Some(&stale.debt().to_turtle().unwrap())).is_clean());
    }
    burned_down_ratchet_stays_a_guard(&record);
    forbidden_neighbour_keeps_hard_failure_and_records_no_site(&both);
    ratcheted_hit_does_not_hide_a_forbidden_constant(&both, &record);
}

/// A unit that holds a recorded ratcheted hit and also a forbidden constant of
/// the same job is still a COPY: only the generic finding reports the constant,
/// so the ratcheted hit may not suppress it. The ratcheted-only neighbour is clean.
fn ratcheted_hit_does_not_hide_a_forbidden_constant(modules: &[(&str, &str)], record: &str) {
    let with_constant = |modules: &[(&str, &str)]| {
        let mut graph = lexical_pattern_graph("call", "std::mem::drop");
        put(&mut graph, "#pattern", "ratcheted", Term::boolean(true));
        put(
            &mut graph,
            "#job",
            "forbiddenConstant",
            Term::typed_literal("7", vocab::INTEGER),
        );
        fixture_gate(".")
            .check_tree(&ratchet_tree_over(
                graph,
                modules,
                RATCHET_HOME,
                Some(record),
            ))
            .unwrap()
    };
    assert!(with_constant(modules).is_clean());
    let seven = "pub fn one(){std::mem::drop(1);let _=\"alpha\";let _=7;}\npub fn two(){std::mem::drop(2);let _=\"beta\";}\n";
    let report = with_constant(&[("a", seven), ("b", RATCHET_THREE)]);
    assert_eq!(report.findings().len(), 1, "{:?}", report.findings());
    assert!(
        report.findings()[0].contains("one is a COPY"),
        "{:?}",
        report.findings()
    );
    assert_eq!(report.debt().sites().len(), 3);
}

/// With no hit left anywhere, home included, the recorded sites are stale; the
/// lowered record keeps the rule with no site row, the pattern is not reported as
/// matching no source item, and a new hit is growth.
fn burned_down_ratchet_stays_a_guard(record: &str) {
    let burned = [("a", "pub fn one(){}\n"), ("b", "pub fn three(){}\n")];
    let check_burned = |record: &str| {
        fixture_gate(".")
            .check_tree(&ratchet_tree(
                &burned,
                "pub fn home(){}",
                Some(record),
                true,
            ))
            .unwrap()
    };
    let empty = check_burned(record);
    assert_eq!(empty.findings().len(), 3, "{:?}", empty.findings());
    assert!(
        empty
            .findings()
            .iter()
            .all(|finding| finding.contains("stale site")),
        "{:?}",
        empty.findings()
    );
    let guard = empty.debt().to_turtle().unwrap();
    assert!(guard.contains("ratchetedRule") && !guard.contains("SiteDebt"));
    assert!(empty.debt().sites().is_empty());
    let guarded = check_burned(&guard);
    assert!(guarded.is_clean(), "{:?}", guarded.findings());
    assert!(
        guarded
            .findings()
            .iter()
            .all(|finding| !finding.contains("matches no source item")),
    );
    let regrown = fixture_gate(".")
        .check_tree(&ratchet_tree(
            &[("a", RATCHET_ONE)],
            "pub fn home(){}",
            Some(&guard),
            true,
        ))
        .unwrap();
    assert_eq!(regrown.findings().len(), 2, "{:?}", regrown.findings());
    assert!(
        regrown
            .findings()
            .iter()
            .all(|finding| finding.contains("growth cannot be recorded")),
        "{:?}",
        regrown.findings()
    );
}

/// The same hits under a pattern that is not ratcheted stay findings.
fn forbidden_neighbour_keeps_hard_failure_and_records_no_site(modules: &[(&str, &str)]) {
    let forbidden = fixture_gate(".")
        .check_tree(&ratchet_tree(modules, RATCHET_HOME, None, false))
        .unwrap();
    assert!(forbidden.debt().sites().is_empty() && forbidden.debt().ratcheted().is_empty());
    assert_eq!(
        forbidden
            .findings()
            .iter()
            .filter(|finding| finding.contains("rule approved-call"))
            .count(),
        3,
        "{:?}",
        forbidden.findings()
    );
}

/// True when comparing `present` to `record` reports growth, which a write refuses.
fn record_may_not_admit(record: &str, present: &DebtRecord) -> bool {
    DebtRecord::from_turtle(record)
        .unwrap()
        .compare(present)
        .unwrap()
        .iter()
        .any(raf_census::DebtChange::is_growth)
}

#[test]
fn shipping_scope_hits_a_library_and_stays_silent_in_tests_test_modules_and_build_scripts() {
    for (site, scoped) in [("lib", 1), ("test", 0), ("cfg-test", 0), ("build", 0)] {
        assert_eq!(scoped_copies(site, &[], true), scoped, "scoped: {site}");
        assert_eq!(scoped_copies(site, &[], false), 1, "unscoped: {site}");
    }
}

#[test]
fn target_kind_scope_includes_and_excludes_kinds_beside_unscoped_hits() {
    for (site, scope, copies) in [
        ("test", &[("targetKind", "test")][..], 1),
        ("lib", &[("targetKind", "test")][..], 0),
        ("build", &[("excludeTargetKind", "buildScript")][..], 0),
        ("lib", &[("excludeTargetKind", "buildScript")][..], 1),
        ("test", &[("excludeTargetKind", "test")][..], 0),
    ] {
        assert_eq!(
            scoped_copies(site, scope, false),
            copies,
            "{site}: {scope:?}"
        );
    }
}

#[test]
fn path_scope_is_silent_inside_an_excluded_prefix_and_reports_outside_it() {
    let exclude = &[("excludePath", "src/legacy")][..];
    assert_eq!(scoped_copies("legacy", exclude, false), 0);
    assert_eq!(scoped_copies("lib", exclude, false), 1);
    assert_eq!(scoped_copies("test", exclude, false), 1);
    let include = &[("includePath", "src/legacy")][..];
    assert_eq!(scoped_copies("legacy", include, false), 1);
    assert_eq!(scoped_copies("lib", include, false), 0);
    assert_eq!(scoped_copies("legacy", &[], false), 1);
}

#[test]
fn contradictory_or_unknown_scope_refuses_at_load_beside_a_valid_neighbour() {
    let load = |scope: &[(&str, &str)], shipping: bool| {
        let graph = scoped_pattern(scope, shipping).finish().unwrap();
        Ledger::from_turtle(&graph.to_text(Format::Turtle).unwrap())
    };
    load(
        &[
            ("targetKind", "lib"),
            ("excludeTargetKind", "test"),
            ("includePath", "src"),
            ("excludePath", "src/legacy"),
        ],
        true,
    )
    .unwrap();
    load(
        &[
            ("includePath", "src/legacy/deep"),
            ("excludePath", "src/legacy2"),
        ],
        false,
    )
    .unwrap();
    for (scope, shipping, fix) in [
        (&[("targetKind", "library")][..], false, "not a target kind"),
        (
            &[("excludeTargetKind", "Lib")][..],
            false,
            "not a target kind",
        ),
        (
            &[("targetKind", "lib"), ("excludeTargetKind", "lib")][..],
            false,
            "both included and excluded",
        ),
        (
            &[("excludeTargetKind", "lib"), ("excludeTargetKind", "bin")][..],
            true,
            "admit no target kind",
        ),
        (&[("targetKind", "test")][..], true, "admit no target kind"),
        (
            &[("includePath", "src/legacy"), ("excludePath", "src")][..],
            false,
            "lies inside excludePath",
        ),
        (
            &[("includePath", "src"), ("excludePath", "src")][..],
            false,
            "lies inside excludePath",
        ),
        (
            &[("excludePath", "/etc")][..],
            false,
            "relative to the workspace root",
        ),
        (
            &[("includePath", "../out")][..],
            false,
            "relative to the workspace root",
        ),
        (
            &[("excludePath", ".")][..],
            false,
            "names the workspace root",
        ),
    ] {
        let error = load(scope, shipping).unwrap_err().to_string();
        assert!(error.contains(fix), "{scope:?}: {error}");
    }
    let mut graph = scoped_pattern(&[], false);
    put(&mut graph, "#pattern", "shippingOnly", Term::boolean(false));
    let error = Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("stated only as true"), "{error}");
}

fn macro_scope_tree(lib_tail: &str, extra: &[(&str, &str)], graph: Builder) -> MemoryTree {
    let mut files = vec![
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_owned(),
        ),
        (
            "src/lib.rs",
            format!("pub fn home(){{blocked!();}}\npub use home as alias;\n{lib_tail}"),
        ),
        (
            "helpers-ledger.ttl",
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ];
    files.extend(extra.iter().map(|(path, text)| (*path, (*text).to_owned())));
    files.into_iter().collect()
}

#[test]
fn shipping_scope_follows_the_test_role_of_a_module_level_macro_invocation() {
    for (tail, copies) in [("blocked!();\n", 1), ("#[cfg(test)]\nblocked!();\n", 0)] {
        let mut graph = lexical_pattern_graph("macro", "blocked");
        put(&mut graph, "#pattern", "shippingOnly", Term::boolean(true));
        let report = fixture_gate(".")
            .check_tree(&macro_scope_tree(tail, &[], graph))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{tail}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn a_shared_file_hit_is_scoped_by_the_target_whose_resolution_matched() {
    let shared = [
        ("src/shared.rs", "pub fn used(){ super::blocked(); }\n"),
        (
            "tests/it.rs",
            "#[path = \"../src/shared.rs\"]\nmod shared;\nfn blocked(){}\n#[test]\nfn it(){}\n",
        ),
    ];
    for (kind, copies) in [("lib", 1), ("test", 0)] {
        let mut graph = lexical_pattern_graph("call", "demo::blocked");
        put(
            &mut graph,
            "#pattern",
            "targetKind",
            Term::simple_literal(kind),
        );
        let tail = "pub mod shared;\npub fn blocked(){}\n";
        let report = fixture_gate(".")
            .check_tree(&macro_scope_tree(tail, &shared, graph))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{kind}: {:?}",
            report.findings()
        );
    }
}

/// A registered rule that flags every `flag!()` statement of a top-level
/// function, at the line of the statement, in the function's own module.
struct FlagRule;
impl Rule for FlagRule {
    fn id(&self) -> &'static str {
        "flag-rule"
    }
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        let module = ctx
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&ctx.path)
            .trim_end_matches(".rs");
        for item in &file.items {
            let syn::Item::Fn(item) = item else { continue };
            for statement in &item.block.stmts {
                if let syn::Stmt::Macro(flag) = statement
                    && flag.mac.path.is_ident("flag")
                {
                    out.push(RuleHit {
                        line: flag.mac.path.segments[0].ident.span().start().line,
                        message: "a flagged statement".into(),
                        symbol: if module == "lib" {
                            format!("{}::{}", ctx.package, item.sig.ident)
                        } else {
                            format!("{}::{module}::{}", ctx.package, item.sig.ident)
                        },
                    });
                }
            }
        }
    }
    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn run() { flag!(); }"],
            negative: vec!["fn run() {}"],
        }
    }
}

/// Modules of two files with three flagged sites, `hit` being one statement
/// that the rule under test reports.
fn rule_modules(hit: &str) -> [(&'static str, String); 2] {
    [
        (
            "a",
            format!(
                "pub fn one(){{{hit}let _=\"alpha\";}}\npub fn two(){{{hit}let _=\"beta\";}}\n"
            ),
        ),
        ("b", format!("pub fn three(){{{hit}let _=\"gamma\";}}\n")),
    ]
}

/// Two packages: the modules live in `demo`, the approved home in `approved`,
/// so no detector's home-package exemption hides a hit.
fn rule_tree(mut graph: Builder, modules: &[(&str, &str)], record: Option<&str>) -> MemoryTree {
    assert!(
        graph
            .remove(Triple {
                subject: Term::iri("#job"),
                predicate: Term::iri(format!("{}homeSymbol", vocab::REPORT)),
                object: Term::simple_literal("demo::alias"),
            })
            .unwrap()
    );
    put(
        &mut graph,
        "#job",
        "homeSymbol",
        Term::simple_literal("approved::home"),
    );
    let manifest = |name: &str| {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    };
    let declared = modules
        .iter()
        .map(|(name, _)| format!("pub mod {name};\n"))
        .collect::<Vec<_>>()
        .concat();
    let mut files = vec![
        (
            "Cargo.toml".to_owned(),
            "[workspace]\nmembers = [\"approved\", \"demo\"]\nresolver = \"2\"\n".to_owned(),
        ),
        ("approved/Cargo.toml".to_owned(), manifest("approved")),
        (
            "approved/src/lib.rs".to_owned(),
            "/// The operation.\npub fn home(){}\n".to_owned(),
        ),
        ("demo/Cargo.toml".to_owned(), manifest("demo")),
        ("demo/src/lib.rs".to_owned(), declared),
        (
            "helpers-ledger.ttl".to_owned(),
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ];
    files.extend(
        modules
            .iter()
            .map(|(name, body)| (format!("demo/src/{name}.rs"), (*body).to_owned())),
    );
    if let Some(record) = record {
        files.push(("census-debt.ttl".to_owned(), record.to_owned()));
    }
    files.into_iter().collect()
}

/// Check a tree whose one enforced job forbids `rule`, ratcheted or not.
fn rule_report(
    (rule, ratcheted): (&str, bool),
    modules: &[(&str, String)],
    record: Option<&str>,
) -> raf_census::GateReport {
    let mut graph = job(true);
    put(
        &mut graph,
        "#job",
        "forbiddenFingerprint",
        Term::simple_literal(format!("rule:{rule}")),
    );
    if ratcheted {
        put(
            &mut graph,
            "#job",
            "ratchetedRule",
            Term::simple_literal(rule),
        );
    }
    let modules: Vec<_> = modules.iter().map(|(n, b)| (*n, b.as_str())).collect();
    fixture_gate(".")
        .rules(vec![Box::new(FlagRule) as Box<dyn Rule>])
        .check_tree(&rule_tree(graph, &modules, record))
        .unwrap()
}

/// A job-level `ratchetedRule` holds a built-in detector and a registered rule
/// to the same counted-site contract as a ratcheted pattern: exact adoption,
/// line movement leaves the record byte-identical, a rise fails, a fall fails
/// until the record is lowered, a deleted file is stale, and the same rule left
/// forbidden stays a hard finding that records nothing.
#[test]
fn ratcheted_rules_record_counted_sites_and_hold_both_directions() {
    for (rule, hit) in [
        ("serde-json", "let _=serde_json::to_string(&1);"),
        ("flag-rule", "flag!();"),
        ("json-pointer-escape", "let _=\"~0\";"),
    ] {
        let check = |modules: &[(&str, String)], record: Option<&str>| {
            rule_report((rule, true), modules, record)
        };
        let both = rule_modules(hit);
        let adopted = check(&both, None);
        let debt = adopted.debt();
        assert_eq!(debt.ratcheted().iter().collect::<Vec<_>>(), [rule]);
        assert_eq!(debt.sites().len(), 3, "{rule}: {:?}", adopted.findings());
        assert!(debt.sites().keys().all(|site| site.rule().as_str() == rule));
        assert!(
            adopted
                .findings()
                .iter()
                .all(|finding| finding.contains("Adopt")),
            "{rule}: {:?}",
            adopted.findings()
        );
        let record = debt.to_turtle().unwrap();
        assert!(check(&both, Some(&record)).is_clean(), "{rule}");
        // Inserting lines above every site keeps the record byte-identical.
        let shifted = both
            .clone()
            .map(|(name, body)| (name, format!("// moved\n\n{body}")));
        let moved = check(&shifted, Some(&record));
        assert!(moved.is_clean(), "{rule}: {:?}", moved.findings());
        assert_eq!(moved.debt().to_turtle().unwrap(), record, "{rule}");
        // One more hit is growth, as a new key and as a repeat of a recorded
        // key (two hits on one line count twice).
        let fourth = format!("pub fn four(){{{hit}let _=\"delta\";}}\n");
        let new_key = [
            ("a", format!("{}{fourth}", both[0].1)),
            ("b", both[1].1.clone()),
        ];
        let repeated = [
            ("a", both[0].1.replacen(hit, &format!("{hit}{hit}"), 1)),
            ("b", both[1].1.clone()),
        ];
        for modules in [new_key, repeated] {
            let grown = check(&modules, Some(&record));
            assert_eq!(grown.findings().len(), 1, "{rule}: {:?}", grown.findings());
            assert!(
                grown.findings()[0].contains("growth cannot be recorded")
                    && grown.findings()[0].contains("src/a.rs:"),
                "{rule}: {:?}",
                grown.findings()
            );
            assert!(record_may_not_admit(&record, grown.debt()), "{rule}");
        }
        // A fall fails check until the record is lowered; then it passes.
        let wide = check(&repeated_modules(&both, hit), None)
            .debt()
            .to_turtle()
            .unwrap();
        let fallen = check(&both, Some(&wide));
        assert_eq!(
            fallen.findings().len(),
            1,
            "{rule}: {:?}",
            fallen.findings()
        );
        assert!(fallen.findings()[0].contains("lower the record"), "{rule}");
        assert!(check(&both, Some(&fallen.debt().to_turtle().unwrap())).is_clean());
        // A deleted file and a vanished hit leave stale sites.
        let unhit = [
            both[0].clone(),
            ("b", "pub fn three(){let _=\"gamma\";}\n".to_owned()),
        ];
        for modules in [&unhit[..], &both[..1]] {
            let stale = check(modules, Some(&record));
            assert_eq!(stale.findings().len(), 1, "{rule}: {:?}", stale.findings());
            assert!(stale.findings()[0].contains("stale site"), "{rule}");
            assert!(check(modules, Some(&stale.debt().to_turtle().unwrap())).is_clean());
        }
        // The same hits under a rule that no job ratchets are findings, not debt.
        let forbidden = rule_report((rule, false), &both, None);
        assert!(forbidden.debt().sites().is_empty() && forbidden.debt().ratcheted().is_empty());
        assert_eq!(
            forbidden
                .findings()
                .iter()
                .filter(|finding| finding.contains(&format!("rule {rule}:")))
                .count(),
            3,
            "{rule}: {:?}",
            forbidden.findings()
        );
    }
}

fn repeated_modules(both: &[(&'static str, String); 2], hit: &str) -> [(&'static str, String); 2] {
    [
        ("a", both[0].1.replacen(hit, &format!("{hit}{hit}"), 1)),
        both[1].clone(),
    ]
}

/// `ratchetedRule` names a rule the same enforced job forbids, never a pattern
/// or a job-scoped detector, and every job that forbids the rule agrees. Each
/// refusal sits beside the valid declaration it differs from.
pub(super) fn ratchet_rule_declarations_are_enabled_enforced_scoped_and_consistent() {
    let load = |graph: Builder| {
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
    };
    let forbidding = |enforced: bool, rule: &str, ratcheted: Option<&str>| {
        let mut graph = job(enforced);
        put(
            &mut graph,
            "#job",
            "forbiddenFingerprint",
            Term::simple_literal(format!("rule:{rule}")),
        );
        if let Some(id) = ratcheted {
            put(
                &mut graph,
                "#job",
                "ratchetedRule",
                Term::simple_literal(id),
            );
        }
        graph
    };
    let valid = load(forbidding(true, "flag-rule", Some("flag-rule"))).unwrap();
    assert_eq!(valid.jobs()[0].ratcheted_rules, ["flag-rule"]);
    assert_eq!(
        load(forbidding(true, "flag-rule", None)).unwrap().jobs()[0].ratcheted_rules,
        Vec::<String>::new()
    );
    let mut pattern = ratcheted_declaration();
    put(
        &mut pattern,
        "#job",
        "ratchetedRule",
        Term::simple_literal("approved-call"),
    );
    a_pattern_reached_through_another_jobs_rule_cannot_be_ratcheted();
    for (graph, reason) in [
        (
            forbidding(true, "flag-rule", Some("other-rule")),
            "does not forbid",
        ),
        (
            forbidding(false, "flag-rule", Some("flag-rule")),
            "not enforced",
        ),
        (
            forbidding(true, "home-literal", Some("home-literal")),
            "cannot be ratcheted",
        ),
        (pattern, "state ratcheted on the pattern"),
    ] {
        let error = load(graph).unwrap_err().to_string();
        assert!(error.contains(reason), "{reason}: {error}");
    }
    let second = |ratchets: bool| {
        let mut graph = forbidding(true, "flag-rule", Some("flag-rule"));
        put(&mut graph, "", "job", Term::iri("#second"));
        put(
            &mut graph,
            "#second",
            "type",
            Term::iri(format!("{}LedgerJob", vocab::REPORT)),
        );
        for (key, value) in [
            ("jobId", "second"),
            ("summary", "Forbids the same rule"),
            ("homeSymbol", "demo::home"),
            ("forbiddenFingerprint", "rule:flag-rule"),
        ] {
            put(&mut graph, "#second", key, Term::simple_literal(value));
        }
        put(&mut graph, "#second", "enforced", Term::boolean(true));
        if ratchets {
            put(
                &mut graph,
                "#second",
                "ratchetedRule",
                Term::simple_literal("flag-rule"),
            );
        }
        graph
    };
    let error = load(second(false)).unwrap_err().to_string();
    assert!(
        error.contains("forbids it outright") && error.contains("second"),
        "{error}"
    );
    assert!(load(second(true)).is_ok());
}

/// A rule the Gate requires everywhere cannot also be ratcheted by a job; the
/// same ratchet without the requirement evaluates.
#[test]
fn a_required_rule_cannot_also_be_ratcheted() {
    let modules = rule_modules("flag!();");
    let build = || {
        let mut graph = job(true);
        put(
            &mut graph,
            "#job",
            "forbiddenFingerprint",
            Term::simple_literal("rule:flag-rule"),
        );
        put(
            &mut graph,
            "#job",
            "ratchetedRule",
            Term::simple_literal("flag-rule"),
        );
        let modules: Vec<_> = modules.iter().map(|(n, b)| (*n, b.as_str())).collect();
        ratchet_tree_over(graph, &modules, RATCHET_HOME, None)
    };
    let gate = || fixture_gate(".").rules(vec![Box::new(FlagRule) as Box<dyn Rule>]);
    assert!(gate().check_tree(&build()).is_ok());
    let error = gate()
        .require_rules(["flag-rule"])
        .check_tree(&build())
        .unwrap_err()
        .to_string();
    assert!(error.contains("cannot also be ratcheted"), "{error}");
}

/// Two jobs reach one forbidden pattern, the second by a rule fingerprint.
fn a_pattern_reached_through_another_jobs_rule_cannot_be_ratcheted() {
    let load = |graph: Builder| {
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
    };
    // A pattern reached through another job's rule fingerprint cannot be
    // ratcheted there: that would lift the first job's hard failure. Without the
    // ratchetedRule the same two jobs load.
    let via_rule = |ratchet: bool| {
        let mut graph = declared_pattern(Some("call"), true);
        put(&mut graph, "", "job", Term::iri("#second"));
        put(
            &mut graph,
            "#second",
            "type",
            Term::iri(format!("{}LedgerJob", vocab::REPORT)),
        );
        for (key, value) in [
            ("jobId", "second"),
            ("summary", "Reaches the pattern by rule"),
            ("homeSymbol", "demo::home"),
            ("forbiddenFingerprint", "rule:approved-call"),
        ] {
            put(&mut graph, "#second", key, Term::simple_literal(value));
        }
        put(&mut graph, "#second", "enforced", Term::boolean(true));
        if ratchet {
            put(
                &mut graph,
                "#second",
                "ratchetedRule",
                Term::simple_literal("approved-call"),
            );
        }
        graph
    };
    assert!(load(via_rule(false)).is_ok());
    let error = load(via_rule(true)).unwrap_err().to_string();
    assert!(error.contains("state ratcheted on the pattern"), "{error}");
}
