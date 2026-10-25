// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Production ledger evaluation over the source tree seam.

#[path = "census_gate/configured_provider_targets.rs"]
mod configured_provider_targets;
#[path = "support/vocabulary.rs"]
mod vocabulary;

use std::collections::BTreeMap;

use raf_census::{
    Census, CensusOptions, DebtRecord, FileCtx, Fingerprint, Gate, Ledger, LedgerExtension,
    MAX_LEDGER_BYTES, MatchClass, MigrationInput, NORMAL_FORM, Rule, RuleFixtures, RuleHit,
    migrate_index,
};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::MemoryTree;

const SOURCE: &str = "/// The one implementation.\npub fn home() -> u32 { 7 }\npub use home as alias;\npub fn copy() -> u32 { 7 + 1 }\n";

#[test]
fn locked_gate_uses_complete_ledger_roots_and_the_same_debt_path() {
    use raf_source::{LoadOptions, LockedCatalog, LockedManifest, RelPath};

    let ledger = base().finish().unwrap().to_text(Format::Turtle).unwrap();
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['one','two']\n"),
        (
            "one/Cargo.toml",
            "[package]\nname='one'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("one/src/lib.rs", "pub fn first() -> u32 { 1 }"),
        (
            "two/Cargo.toml",
            "[package]\nname='two'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("two/src/lib.rs", "pub fn second() -> u32 { 2 }"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='one'\nversion='1.0.0'\n[[package]]\nname='two'\nversion='1.0.0'\n",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let local: Vec<_> = ["one", "two"]
        .into_iter()
        .map(|name| {
            let row = catalog
                .packages()
                .find(|row| row.name() == name)
                .unwrap()
                .id();
            LockedManifest::new(
                row,
                RelPath::new(&format!("{name}/Cargo.toml")).unwrap(),
                &[],
            )
        })
        .collect();
    let report = Gate::workspace(".")
        .check_locked(&tree, &catalog, &local, &[])
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    assert_eq!(report.census().units().len(), 2);
    let missing = Gate::workspace(".")
        .check_locked(&tree, &catalog, &local[..1], &[])
        .unwrap_err();
    assert!(missing.to_string().contains("association"), "{missing}");
}

fn put(graph: &mut Builder, node: &str, key: &str, object: Term) {
    graph
        .insert(Triple {
            subject: Term::iri(node),
            predicate: Term::iri(if key == "type" {
                vocab::TYPE.into()
            } else {
                format!("{}{key}", vocab::REPORT)
            }),
            object,
        })
        .unwrap();
}

fn base() -> Builder {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        "type",
        Term::iri(format!("{}Ledger", vocab::REPORT)),
    );
    put(
        &mut graph,
        "",
        "normalForm",
        Term::typed_literal(NORMAL_FORM.to_string(), vocab::INTEGER),
    );
    put(
        &mut graph,
        "",
        "reportForm",
        Term::typed_literal(raf_census::REPORT_FORM.to_string(), vocab::INTEGER),
    );
    graph
}

fn job(enforced: bool) -> Builder {
    let mut graph = base();
    put(&mut graph, "", "job", Term::iri("#job"));
    put(
        &mut graph,
        "#job",
        "type",
        Term::iri(format!("{}LedgerJob", vocab::REPORT)),
    );
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "The approved constant"),
        ("homeSymbol", "demo::alias"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    put(&mut graph, "#job", "enforced", Term::boolean(enforced));
    put(
        &mut graph,
        "#job",
        "forbiddenConstant",
        Term::typed_literal("7", vocab::INTEGER),
    );
    graph
}

fn declared_pattern(matcher: Option<&str>, referenced: bool) -> Builder {
    let mut graph = job(true);
    put(&mut graph, "", "pattern", Term::iri("#pattern"));
    put(
        &mut graph,
        "#pattern",
        "type",
        Term::iri(format!("{}Pattern", vocab::REPORT)),
    );
    put(
        &mut graph,
        "#pattern",
        "patternId",
        Term::simple_literal("approved-call"),
    );
    put(
        &mut graph,
        "#pattern",
        "summary",
        Term::simple_literal("The approved operation"),
    );
    if let Some(matcher) = matcher {
        put(
            &mut graph,
            "#pattern",
            matcher,
            Term::simple_literal("demo::home"),
        );
    }
    if referenced {
        put(
            &mut graph,
            "#job",
            "forbiddenPattern",
            Term::simple_literal("approved-call"),
        );
    }
    graph
}

#[test]
fn ledger_settings_keep_source_limits_and_version_refusal_ahead_of_job_interpretation() {
    let graph = job(true).finish().unwrap();
    let mut old = Builder::from_graph(&graph);
    assert!(
        old.remove(Triple {
            subject: Term::iri(""),
            predicate: Term::iri(format!("{}normalForm", vocab::REPORT)),
            object: Term::typed_literal("1", vocab::INTEGER)
        })
        .unwrap()
    );
    put(
        &mut old,
        "",
        "normalForm",
        Term::typed_literal("0", vocab::INTEGER),
    );
    assert!(
        old.remove(Triple {
            subject: Term::iri("#job"),
            predicate: Term::iri(format!("{}homeSymbol", vocab::REPORT)),
            object: Term::simple_literal("demo::alias")
        })
        .unwrap()
    );
    assert!(matches!(
        Ledger::from_turtle(&old.finish().unwrap().to_text(Format::Turtle).unwrap()),
        Err(raf_census::Error::Version { declared: 0, .. })
    ));
    for (limit, clean) in [(8, false), (4096, true)] {
        let mut graph = job(true);
        put(
            &mut graph,
            "",
            "maxFileBytes",
            Term::typed_literal(limit.to_string(), vocab::INTEGER),
        );
        let result = Gate::workspace(".").check_tree(&tree(
            "/// Home.\npub fn home()->u32{7}\npub use home as alias;",
            graph,
            0,
        ));
        if clean {
            assert!(result.unwrap().is_clean());
        } else {
            assert!(result.unwrap_err().to_string().contains("Cargo.toml"));
        }
    }
    let mut invalid = job(true);
    put(
        &mut invalid,
        "#job",
        "forbiddenName",
        Term::simple_literal("["),
    );
    assert!(
        Ledger::from_turtle(&invalid.finish().unwrap().to_text(Format::Turtle).unwrap()).is_err()
    );
    let mut reference = job(true);
    put(
        &mut reference,
        "",
        "reference",
        Term::simple_literal("native"),
    );
    let tree = tree(SOURCE, reference, 0);
    assert!(
        Gate::workspace(".")
            .check_tree(&tree)
            .unwrap_err()
            .to_string()
            .contains("executable reference scan")
    );
}

#[test]
fn ledger_core_nodes_refuse_unknown_missing_and_orphan_assertions_with_valid_neighbours() {
    ledger_vocabulary_matches_validated_graphs();
    for (builder, node) in [
        (base(), ""),
        (job(true), "#job"),
        (variant("src/lib.rs"), "#variant"),
        (cfg_ledger(2), "#distinct"),
        (declared_pattern(Some("call"), true), "#pattern"),
    ] {
        let graph = builder.finish().unwrap();
        Ledger::from_turtle(&graph.to_text(Format::Turtle).unwrap()).unwrap();
        let mut changed = Builder::from_graph(&graph);
        put(
            &mut changed,
            node,
            "misspelled",
            Term::simple_literal("invalid"),
        );
        assert!(
            Ledger::from_turtle(&changed.finish().unwrap().to_text(Format::Turtle).unwrap())
                .unwrap_err()
                .to_string()
                .contains("misspelled")
        );
    }
    let graph = job(true).finish().unwrap();
    for (key, object) in [
        ("jobId", Term::simple_literal("number")),
        ("summary", Term::simple_literal("The approved constant")),
        ("homeSymbol", Term::simple_literal("demo::alias")),
        ("enforced", Term::boolean(true)),
    ] {
        let mut changed = Builder::from_graph(&graph);
        assert!(
            changed
                .remove(Triple {
                    subject: Term::iri("#job"),
                    predicate: Term::iri(format!("{}{key}", vocab::REPORT)),
                    object
                })
                .unwrap()
        );
        assert!(
            Ledger::from_turtle(&changed.finish().unwrap().to_text(Format::Turtle).unwrap())
                .is_err()
        );
    }
    let mut orphan = job(true);
    put(
        &mut orphan,
        "#unowned",
        "type",
        Term::iri(format!("{}Variant", vocab::REPORT)),
    );
    assert!(
        Ledger::from_turtle(&orphan.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap_err()
            .to_string()
            .contains("orphan")
    );
    let oversized = " ".repeat(MAX_LEDGER_BYTES + 1);
    assert!(
        Ledger::from_turtle(&oversized)
            .unwrap_err()
            .to_string()
            .contains("maximum")
    );
    let original = graph.to_text(Format::Turtle).unwrap();
    let summary = "The approved constant";
    let mut exact = Builder::from_graph(&graph);
    assert!(
        exact
            .remove(Triple {
                subject: Term::iri("#job"),
                predicate: Term::iri(format!("{}summary", vocab::REPORT)),
                object: Term::simple_literal(summary)
            })
            .unwrap()
    );
    put(
        &mut exact,
        "#job",
        "summary",
        Term::simple_literal("x".repeat(summary.len() + MAX_LEDGER_BYTES - original.len())),
    );
    let exact = exact.finish().unwrap().to_text(Format::Turtle).unwrap();
    assert_eq!(exact.len(), MAX_LEDGER_BYTES);
    Ledger::from_turtle(&exact).unwrap();
}

fn ledger_vocabulary_matches_validated_graphs() {
    let mut builders = vec![
        variant("src/lib.rs"),
        cfg_ledger(2),
        provider_policy("native-alias", "native_alias::encode"),
        ledger_vocabulary_settings(),
    ];
    builders.extend(
        [
            "call",
            "path",
            "macro",
            "typePath",
            "literalRegex",
            "tokens",
        ]
        .into_iter()
        .map(|matcher| declared_pattern(Some(matcher), true)),
    );
    let reports = builders.into_iter().flat_map(|builder| {
        let graph = builder.finish().unwrap();
        [Format::Turtle, Format::NTriples].map(|format| {
            let text = graph.to_text(format).unwrap();
            let ledger = match format {
                Format::Turtle => Ledger::from_turtle(&text),
                Format::NTriples => Ledger::from_ntriples(&text),
            }
            .unwrap();
            assert_eq!(
                ledger.graph().snapshot().unwrap(),
                graph.snapshot().unwrap()
            );
            ledger.graph().to_text(Format::NTriples).unwrap()
        })
    });
    vocabulary::assert_inventory(
        "| Census Ledger class | Properties and object types |",
        reports,
    );
}

fn ledger_vocabulary_settings() -> Builder {
    let mut graph = job(true);
    for (key, value) in [("minTokens", "1"), ("maxFileBytes", "8192")] {
        put(
            &mut graph,
            "",
            key,
            Term::typed_literal(value, vocab::INTEGER),
        );
    }
    for (key, value) in [("root", "src"), ("reference", "reference")] {
        put(&mut graph, "", key, Term::simple_literal(value));
    }
    for (key, value) in [("entryPoint", "demo::entry"), ("forbiddenName", "^copy$")] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    put(&mut graph, "#job", "extension", Term::iri("#extension"));
    put(
        &mut graph,
        "#extension",
        "type",
        Term::iri(format!("{}Extension", vocab::REPORT)),
    );
    graph
}

#[test]
fn ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence() {
    for matcher in [
        "call",
        "path",
        "macro",
        "typePath",
        "literalRegex",
        "tokens",
    ] {
        let graph = declared_pattern(Some(matcher), true).finish().unwrap();
        let turtle = graph.to_text(Format::Turtle).unwrap();
        let policy = Ledger::from_turtle(&turtle).unwrap();
        assert_eq!(policy.patterns()[0].id, "approved-call");
        let native = graph.to_text(Format::NTriples).unwrap();
        assert_eq!(Ledger::from_ntriples(&native).unwrap().patterns().len(), 1);
        let tree = tree(SOURCE, Builder::from_graph(&graph), 0);
        let result = Gate::workspace(".").check_tree(&tree);
        let report = result.expect("each declared matcher executes");
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| if matcher == "path" {
                    finding.contains("rule approved-call")
                } else {
                    finding.contains("matches no source item")
                })
        );
    }
    for (mut graph, key) in [
        (declared_pattern(None, true), "matcher"),
        (declared_pattern(Some("call"), false), "unused pattern"),
        (declared_pattern(Some("call"), true), "exactly one"),
    ] {
        if key == "exactly one" {
            put(
                &mut graph,
                "#pattern",
                "path",
                Term::simple_literal("demo::Other"),
            );
        }
        let error = Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap_err();
        assert!(error.to_string().contains(key), "{error}");
    }
    let mut invalid = declared_pattern(Some("call"), true);
    for (node, key) in [("#pattern", "patternId"), ("#job", "forbiddenPattern")] {
        assert!(
            invalid
                .remove(Triple {
                    subject: Term::iri(node),
                    predicate: Term::iri(format!("{}{key}", vocab::REPORT)),
                    object: Term::simple_literal("approved-call")
                })
                .unwrap()
        );
        put(&mut invalid, node, key, Term::simple_literal("invalid id"));
    }
    assert!(
        Ledger::from_turtle(&invalid.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap_err()
            .to_string()
            .contains("rule")
    );
    let mut graph = job(true);
    put(
        &mut graph,
        "#job",
        "forbiddenPattern",
        Term::simple_literal("gone"),
    );
    assert!(
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap_err()
            .to_string()
            .contains("unknown pattern")
    );
}

#[test]
fn provider_declarations_require_complete_owned_evidence_and_never_make_a_clean_gate() {
    for missing in [None, Some("package"), Some("entryPoint"), Some("consumer")] {
        let mut graph = job(true);
        put(
            &mut graph,
            "#job",
            "preferredProvider",
            Term::iri("#provider"),
        );
        put(
            &mut graph,
            "#provider",
            "type",
            Term::iri(format!("{}PreferredProvider", vocab::REPORT)),
        );
        for (key, value) in [
            ("package", "native-alias"),
            ("entryPoint", "native_alias::encode"),
            ("consumer", "demo::home"),
        ] {
            if missing != Some(key) {
                put(&mut graph, "#provider", key, Term::simple_literal(value));
            }
        }
        let graph = graph.finish().unwrap();
        let policy = Ledger::from_turtle(&graph.to_text(Format::Turtle).unwrap());
        if let Some(missing) = missing {
            assert!(policy.unwrap_err().to_string().contains(missing));
        } else {
            let policy = policy.unwrap();
            let provider = policy.jobs()[0].preferred_provider.as_ref().unwrap();
            assert_eq!(provider.package, "native-alias");
            assert_eq!(provider.entry_points, ["native_alias::encode"]);
            let error = Gate::workspace(".")
                .check_tree(&tree(SOURCE, Builder::from_graph(&graph), 0))
                .unwrap_err();
            assert!(error.to_string().contains("grounded locked-source"));
        }
    }
}

fn provider_policy(selector: &str, api: &str) -> Builder {
    let mut graph = job(true);
    put(
        &mut graph,
        "#job",
        "forbiddenFingerprint",
        Term::simple_literal("rule:hex-table"),
    );
    put(
        &mut graph,
        "#job",
        "forbiddenFingerprint",
        Term::simple_literal("rule:std-default-hasher"),
    );
    put(
        &mut graph,
        "#job",
        "preferredProvider",
        Term::iri("#provider"),
    );
    put(
        &mut graph,
        "#provider",
        "type",
        Term::iri(format!("{}PreferredProvider", vocab::REPORT)),
    );
    for (key, value) in [
        ("package", selector),
        ("entryPoint", api),
        ("consumer", "demo::run"),
    ] {
        put(&mut graph, "#provider", key, Term::simple_literal(value));
    }
    graph
}

fn configured_provider_gate(
    source: &str,
    selector: &str,
    api: &str,
) -> Result<raf_census::GateReport, raf_census::Error> {
    configured_provider_library(
        source,
        selector,
        api,
        "provider",
        "pub mod api { pub fn encode(value:u32)->u32 { value + 1 } } pub use api::encode;",
    )
}

fn configured_provider_library(
    source: &str,
    selector: &str,
    api: &str,
    library: &str,
    implementation: &str,
) -> Result<raf_census::GateReport, raf_census::Error> {
    configured_provider_operations(source, selector, &[api], library, implementation)
}

fn configured_provider_operations(
    source: &str,
    selector: &str,
    apis: &[&str],
    library: &str,
    implementation: &str,
) -> Result<raf_census::GateReport, raf_census::Error> {
    configured_provider_layout(
        source,
        selector,
        apis,
        library,
        implementation,
        None,
        "demo::alias",
    )
}

fn configured_provider_layout(
    source: &str,
    selector: &str,
    apis: &[&str],
    library: &str,
    implementation: &str,
    shared: Option<&str>,
    home: &str,
) -> Result<raf_census::GateReport, raf_census::Error> {
    use raf_source::{
        DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge, LockedManifest,
        RelPath, TargetKind,
    };

    let mut policy = provider_policy(selector, apis[0]);
    policy
        .remove(Triple {
            subject: Term::iri("#job"),
            predicate: Term::iri(format!("{}homeSymbol", vocab::REPORT)),
            object: Term::simple_literal("demo::alias"),
        })
        .unwrap();
    put(
        &mut policy,
        "#job",
        "homeSymbol",
        Term::simple_literal(home),
    );
    for api in &apis[1..] {
        put(
            &mut policy,
            "#provider",
            "entryPoint",
            Term::simple_literal(*api),
        );
    }
    let ledger = policy.finish().unwrap().to_text(Format::Turtle).unwrap();
    let source =
        format!("pub use renamed::api::encode as forwarded;\n{source}\npub use run as alias;");
    let manifest = format!(
        "[package]\nname='provider'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='{library}'"
    );
    let tree: MemoryTree = [
        ("Cargo.toml", "[workspace]\nmembers=['consumer','provider']"),
        (
            "consumer/Cargo.toml",
            "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='provider',path='../provider',version='1.0.0'}",
        ),
        ("consumer/src/lib.rs", source.as_str()),
        ("consumer/src/good/marker.rs", ""),
        ("consumer/src/bad/marker.rs", ""),
        ("provider/Cargo.toml", manifest.as_str()),
        ("provider/src/lib.rs", implementation),
        (
            "provider/src/api.rs",
            "pub fn encode(value:u32)->u32 {value+1}",
        ),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['provider']\n[[package]]\nname='provider'\nversion='1.0.0'\n",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ].into_iter().chain(shared.map(|source| ("consumer/src/shared.rs", source))).collect();
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let context = |name| {
        DeclaredConfiguration::new(
            DeclaredCfg::complete("fixture-target", &[]),
            &[],
            TargetKind::Lib,
            name,
        )
    };
    let edges = [LockedEdge::new("dependencies.renamed", None)];
    let manifests = [
        LockedManifest::new(
            row("demo"),
            RelPath::new("consumer/Cargo.toml").unwrap(),
            &edges,
        )
        .with_configuration(context("demo")),
        LockedManifest::new(
            row("provider"),
            RelPath::new("provider/Cargo.toml").unwrap(),
            &[],
        )
        .with_configuration(context(library)),
    ];
    Gate::workspace(".").check_locked(&tree, &catalog, &manifests, &[])
}

#[test]
fn configured_provider_detects_unlisted_competitors_in_the_adapter_package() {
    for (helper, copies) in [
        ("pub fn other()->u32 { renamed::encode(0) }", 0),
        ("pub fn other(){let _=b\"0123456789abcdeg\";}", 0),
        ("pub fn other(){let _=b\"0123456789abcdef\";}", 1),
        (
            "pub struct Other { cache:std::collections::HashMap<u8,u8> }",
            1,
        ),
        (
            "pub fn other(value:u32)->u32 {let _=renamed::encode(value); let _=b\"0123456789abcdef\"; value}",
            1,
        ),
    ] {
        let source = format!("pub fn run(value:u32)->u32 {{renamed::encode(value)}} {helper}");
        let report =
            configured_provider_gate(&source, "provider", "provider::api::encode").unwrap();
        assert_eq!(report.jobs()[0].copies, copies, "{helper}: {report:?}");
        assert_eq!(report.is_clean(), copies == 0, "{helper}: {report:?}");
        if copies > 0 {
            assert!(
                report
                    .findings()
                    .iter()
                    .any(|finding| finding.contains("rule hex-table:")
                        || finding.contains("rule std-default-hasher:"))
            );
        }
    }
}

#[test]
fn configured_provider_checks_actual_returned_definitions_and_refuses_dummy_or_shadowed_calls() {
    for source in [
        "pub fn run(value:u32)->u32 { renamed::api::encode(value) }",
        "pub fn run()->u32 { renamed::api::encode(0) }",
        "pub fn run(value:u32)->u32 { forwarded(value) }",
        "pub fn run(value:u32)->u32 { use renamed::api::encode as emit; emit(value) }",
        "pub fn run(value:u32)->u32 { use renamed::api as first; use first::encode as emit; emit(value) }",
        "pub fn run(value:u32)->u32 { let result=renamed::encode(value); result }",
        "pub fn run(value:u32)->u32 { return renamed::encode(value); }",
        "pub fn run(renamed:u32)->u32 { renamed::encode(renamed) }",
        "pub fn run<renamed>(value:u32)->u32 { ::renamed::encode(value) }",
        "#[inline] pub fn run(value:u32)->u32 { renamed::encode(value) }",
        "#[cfg_attr(any(),allow(dead_code))] pub fn run(value:u32)->u32 { renamed::encode(value) }",
        "#[cfg_attr(any(),)] pub fn run(value:u32)->u32 { renamed::encode(value) }",
        "#[cfg_attr(false,allow(dead_code))] pub fn run(value:u32)->u32 { renamed::encode(value) }",
        "#[cfg_attr(false,rewrite)] pub fn run(value:u32)->u32 { renamed::encode(value) }",
    ] {
        let report = configured_provider_gate(source, "provider", "provider::api::encode")
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert!(report.is_clean(), "{source}: {:?}", report.findings());
    }
    for source in [
        "pub fn run(value:u32)->u32 { use renamed::encode; value }",
        "pub fn run(value:u32)->u32 { let _=renamed::encode(value); value }",
        "pub fn run(value:u32)->u32 { let unused=renamed::encode(value); value+1 }",
        "pub fn run()->u32 { let result=renamed::encode(0); 0 }",
        "pub fn run()->u32 { renamed::encode(#[rewrite] 0) }",
        "pub fn run()->u32 { let input=#[rewrite] 0; renamed::encode(input) }",
        "pub fn run(value:u32)->u32 { renamed::encode(value); value }",
        "pub fn run(value:u32)->u32 { let result=renamed::encode(value); let result=value; result }",
        "pub fn run(forwarded:fn(u32)->u32,value:u32)->u32 { forwarded(value) }",
        "pub trait Encode {fn encode(value:u32)->u32;} pub fn run<renamed:Encode>(value:u32)->u32 { renamed::encode(value) }",
        "pub mod local {pub fn encode(value:u32)->u32{value+1}} pub fn run(value:u32)->u32 { local::encode(value) }",
        "pub async fn run(value:u32)->u32 { renamed::encode(value) }",
        "#[rewrite] pub fn run(value:u32)->u32 { renamed::encode(value) }",
        "pub fn run(value:u32)->u32 { #[allow(dead_code)] renamed::encode(value) }",
        "pub fn run(#[cfg(false)] value:u32)->u32 { renamed::encode(value) }",
    ] {
        let error =
            configured_provider_gate(source, "provider", "provider::api::encode").unwrap_err();
        assert!(
            matches!(&error, raf_census::Error::Source { reason, .. } if reason.contains("preferred provider")),
            "{source}: {error}"
        );
    }
    for selector in ["renamed", "provider.*"] {
        assert!(
            configured_provider_gate(
                "pub fn run(value:u32)->u32 { renamed::encode(value) }",
                selector,
                "renamed::api::encode"
            )
            .unwrap()
            .is_clean()
        );
    }
}

#[test]
fn configured_provider_homes_exempt_only_proved_consumers_and_apis() {
    let native = "pub mod api {pub fn encode(value:u32)->u32{value+1}} pub use api::encode;";
    let source = "pub mod helpers {pub fn run(value:u32)->u32{renamed::encode(value)} pub fn other(){let _=b\"0123456789abcdef\";}} pub use helpers::run;";
    for home in [
        "demo::helpers::other",
        "demo::helpers",
        "demo::alias",
        "provider::api::encode",
    ] {
        let report = configured_provider_layout(
            source,
            "provider",
            &["provider::api::encode"],
            "provider",
            native,
            None,
            home,
        )
        .unwrap();
        assert!(!report.is_clean(), "{home}: {report:?}");
        if home == "demo::helpers::other" {
            assert!(
                report
                    .findings()
                    .iter()
                    .any(|finding| finding.contains("no proved consumer or provider API")),
                "{report:?}"
            );
        } else {
            assert_eq!(report.jobs()[0].copies, 1, "{home}: {report:?}");
        }
    }
}

#[test]
fn configured_provider_checks_every_context_of_a_shared_consumer_body() {
    let native = "pub mod api {pub fn encode(value:u32)->u32{value+1}} pub use api::encode;";
    for (other, accepted) in [
        ("pub use renamed::encode;", true),
        ("pub fn encode(value:u32)->u32{value+1}", false),
    ] {
        let source = format!(
            "pub mod good {{pub use renamed::encode; #[path=\"../shared.rs\"] pub mod inner;}} pub mod bad {{{other} #[path=\"../shared.rs\"] pub mod inner;}} pub use good::inner::run;"
        );
        let result = configured_provider_layout(
            &source,
            "provider",
            &["provider::api::encode"],
            "provider",
            native,
            Some("pub fn run(value:u32)->u32{super::encode(value)}"),
            "demo::alias",
        );
        if accepted {
            let report = result.unwrap();
            assert!(report.is_clean(), "{report:?}");
        } else {
            let error = result.unwrap_err();
            assert!(error.to_string().contains("preferred provider"), "{error}");
        }
    }
}

fn provider_home_identity_policy() -> String {
    let mut policy = provider_policy("provider.*", "native::api::encode");
    for (node, key, value) in [
        ("#job", "homeSymbol", "demo::alias"),
        ("#provider", "consumer", "demo::run"),
    ] {
        assert!(
            policy
                .remove(Triple {
                    subject: Term::iri(node),
                    predicate: Term::iri(format!("{}{key}", vocab::REPORT)),
                    object: Term::simple_literal(value)
                })
                .unwrap()
        );
    }
    put(
        &mut policy,
        "#job",
        "homeSymbol",
        Term::simple_literal("native::api::encode"),
    );
    for consumer in ["one::run", "two::run"] {
        put(
            &mut policy,
            "#provider",
            "consumer",
            Term::simple_literal(consumer),
        );
    }
    policy.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn provider_home_identity_fixture(same: bool) -> Result<raf_census::GateReport, raf_census::Error> {
    use raf_source::{
        DeclaredCfg, DeclaredConfiguration, LoadOptions, LockedCatalog, LockedEdge, LockedManifest,
        RelPath, TargetKind,
    };

    let ledger = provider_home_identity_policy();
    let second = if same { "provider-one" } else { "provider-two" };
    let manifest = format!(
        "[package]\nname='two'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nnative={{package='{second}',path='../{second}'}}"
    );
    let lock = format!(
        "version=4\n[[package]]\nname='one'\nversion='1.0.0'\ndependencies=['provider-one']\n[[package]]\nname='two'\nversion='1.0.0'\ndependencies=['{second}']\n[[package]]\nname='provider-one'\nversion='1.0.0'\n[[package]]\nname='provider-two'\nversion='1.0.0'\n"
    );
    let tree = MemoryTree::from([
        (
            "Cargo.toml",
            "[workspace]\nmembers=['one','two','provider-one','provider-two']",
        ),
        (
            "one/Cargo.toml",
            "[package]\nname='one'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nnative={package='provider-one',path='../provider-one'}",
        ),
        ("two/Cargo.toml", manifest.as_str()),
        (
            "one/src/lib.rs",
            "pub fn run(value:u32)->u32{native::api::encode(value)}",
        ),
        (
            "two/src/lib.rs",
            "pub fn run()->u32{native::api::encode(0)}",
        ),
        (
            "provider-one/Cargo.toml",
            "[package]\nname='provider-one'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='provider_one'",
        ),
        (
            "provider-two/Cargo.toml",
            "[package]\nname='provider-two'\nversion='1.0.0'\nedition='2024'\n[lib]\nname='provider_two'",
        ),
        (
            "provider-one/src/lib.rs",
            "pub mod api{pub fn encode(value:u32)->u32{value+1}}",
        ),
        (
            "provider-two/src/lib.rs",
            "pub mod api{pub fn encode(value:u32)->u32{value+2}}",
        ),
        ("Cargo.lock", lock.as_str()),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let catalog = LockedCatalog::load(
        &tree,
        &RelPath::new("Cargo.lock").unwrap(),
        &LoadOptions::default(),
    )
    .unwrap();
    let edges = [LockedEdge::new("dependencies.native", None)];
    let manifests: Vec<_> = [
        ("one", "one"),
        ("two", "two"),
        ("provider-one", "provider_one"),
        ("provider-two", "provider_two"),
    ]
    .into_iter()
    .map(|(name, library)| {
        let id = catalog
            .packages()
            .find(|package| package.name() == name)
            .unwrap()
            .id();
        LockedManifest::new(
            id,
            RelPath::new(&format!("{name}/Cargo.toml")).unwrap(),
            if name == "one" || name == "two" {
                &edges
            } else {
                &[]
            },
        )
        .with_configuration(DeclaredConfiguration::new(
            DeclaredCfg::complete("fixture-target", &[]),
            &[],
            TargetKind::Lib,
            library,
        ))
    })
    .collect();
    Gate::workspace(".").check_locked(&tree, &catalog, &manifests, &[])
}

#[test]
fn configured_provider_api_home_requires_one_identity_across_consumers() {
    let report = provider_home_identity_fixture(true).unwrap();
    assert!(report.is_clean(), "{report:?}");
    let error = provider_home_identity_fixture(false).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("different API definitions across consumer contexts"),
        "{error}"
    );
}

#[test]
fn configured_provider_audits_library_attributes_and_generic_computation() {
    let source = "pub fn run(value:u32)->u32 { renamed::encode(value) }";
    for attribute in ["", "#[inline]", "#[cfg_attr(any(),)]"] {
        let implementation = format!(
            "pub mod api {{ {attribute} pub fn encode(value:u32)->u32 {{value+1}} }} pub use api::encode;"
        );
        assert!(
            configured_provider_library(
                source,
                "provider",
                "native_library::api::encode",
                "native_library",
                &implementation
            )
            .unwrap()
            .is_clean()
        );
    }
    let error = configured_provider_library(
        source,
        "provider",
        "native_library::api::encode",
        "native_library",
        "pub mod api { #[rewrite] pub fn encode(value:u32)->u32 {value+1} } pub use api::encode;",
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unsupported rewriting attribute"),
        "{error}"
    );
    let implementation = "pub mod api { pub fn encode<const N:usize>(value:u32)->u32 {value+1} } pub use api::encode;";
    for argument in ["3", "{3}", "N"] {
        let source = format!(
            "pub fn run<const N:usize>(value:u32)->u32 {{renamed::encode::<{argument}>(value)}}"
        );
        assert!(
            configured_provider_library(
                &source,
                "provider",
                "provider::api::encode",
                "provider",
                implementation
            )
            .unwrap()
            .is_clean()
        );
    }
    let source = "const fn competing()->usize{3} pub fn run(value:u32)->u32 {renamed::encode::<{competing()}>(value)}";
    let error = configured_provider_library(
        source,
        "provider",
        "provider::api::encode",
        "provider",
        implementation,
    )
    .unwrap_err();
    assert!(error.to_string().contains("generic arguments"), "{error}");
}

#[test]
fn configured_provider_audits_inherited_inline_and_file_module_attributes() {
    let source = "pub fn run(value:u32)->u32 {renamed::api::encode(value)}";
    for module in [
        "pub mod api {pub fn encode(value:u32)->u32{value+1}} pub use api::encode;",
        "pub mod api; pub use api::encode;",
    ] {
        for (attribute, accepted) in [
            ("#[allow(dead_code)]", true),
            ("#[cfg_attr(false,rewrite)]", true),
            ("#[cfg_attr(true,rewrite)]", false),
            ("#[rewrite]", false),
        ] {
            let implementation = format!("{attribute} {module}");
            let result = configured_provider_library(
                source,
                "provider",
                "provider::api::encode",
                "provider",
                &implementation,
            );
            if accepted {
                assert!(result.unwrap().is_clean());
            } else {
                let error = result.unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("unsupported rewriting attribute"),
                    "{error}"
                );
            }
        }
    }
}

#[test]
fn configured_provider_tracks_native_state_updates_and_refuses_parallel_mutation() {
    let implementation = "pub mod api {pub fn encode(value:u32)->u32{value+1}} pub struct State(u32); impl State {pub fn new()->Self{Self(0)} pub fn update(&mut self,value:u32){self.0=value;} pub fn finish(&self)->u32{self.0}}";
    let apis = [
        "provider::State::new",
        "provider::State::update",
        "provider::State::finish",
        "provider::api::encode",
    ];
    let prefix = "let mut state=renamed::State::new();";
    let suffix = "renamed::State::finish(&state)";
    for (middle, accepted) in [
        ("renamed::State::update(&mut state,value);", true),
        (
            "use renamed::State as Native; Native::update(&mut state,value);",
            true,
        ),
        ("renamed::api::encode(value);", false),
        ("state=renamed::State::new();", false),
        ("state.0=value;", false),
        ("state.update(value);", false),
    ] {
        let source = format!("pub fn run(value:u32)->u32{{{prefix}{middle}{suffix}}}");
        let result =
            configured_provider_operations(&source, "provider", &apis, "provider", implementation);
        if accepted {
            assert!(
                result
                    .unwrap_or_else(|error| panic!("{middle}: {error}"))
                    .is_clean()
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("preferred provider"),
                "{middle}: {error}"
            );
        }
    }
    let source = "pub fn run(value:u32)->u32{let mut state=value; renamed::State::update(&mut state,value); renamed::State::finish(&state)}";
    assert!(
        configured_provider_operations(source, "provider", &apis, "provider", implementation)
            .unwrap_err()
            .to_string()
            .contains("mutable adapter state")
    );
    for body in [
        "let mut dead=renamed::State::new(); let live=renamed::State::new(); renamed::State::update(&mut dead,value); renamed::State::finish(&live)",
        "let unused=renamed::State::new(); let state=renamed::State::new(); renamed::State::finish(&state)",
        "let unused=renamed::api::encode(value); let state=renamed::State::new(); renamed::State::finish(&state)",
        "let mut state=renamed::State::new(); let unused=renamed::State::finish(&state); renamed::State::update(&mut state,value); renamed::State::finish(&state)",
        "let mut state=renamed::State::new(); let result=renamed::State::finish(&state); renamed::State::update(&mut state,value); result",
    ] {
        let source = format!("pub fn run(value:u32)->u32{{{body}}}");
        let error =
            configured_provider_operations(&source, "provider", &apis, "provider", implementation)
                .unwrap_err();
        assert!(
            error.to_string().contains("does not contribute"),
            "{body}: {error}"
        );
    }
    for body in [
        "let mut state=renamed::State::new(); let result=renamed::State::finish(&state); let reference=&mut state; let unused=renamed::State::update(reference,value); result",
        "let mut state=renamed::State::new(); let reference=(&mut state); renamed::State::update(reference,value); renamed::State::finish(&state)",
        "let mut state=renamed::State::new(); let reference=&(&mut state); let unused=renamed::State::update(reference,value); renamed::State::finish(&state)",
    ] {
        let source = format!("pub fn run(value:u32)->u32{{{body}}}");
        let error =
            configured_provider_operations(&source, "provider", &apis, "provider", implementation)
                .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("reference forwarding is not modeled"),
            "{body}: {error}"
        );
    }
    let implementation = "pub struct State(u32); impl State {pub fn new()->Self{Self(0)} pub fn update(&mut self,value:u32)->&mut Self{self.0=value;self} pub fn finish(&self)->u32{self.0}}";
    let apis = &apis[..3];
    for body in [
        "let mut state=renamed::State::new(); let reference=renamed::State::update(&mut state,value); let result=renamed::State::finish(&reference); let unused=renamed::State::update(reference,value); result",
        "let mut state=renamed::State::new(); let result=renamed::State::update(&mut state,value); result",
    ] {
        let source = format!("pub fn run(value:u32)->u32{{{body}}}");
        let error =
            configured_provider_operations(&source, "provider", apis, "provider", implementation)
                .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("possibly borrowed result is not modeled"),
            "{body}: {error}"
        );
    }
    let source = "pub fn run(value:u32)->u32{let mut state=renamed::State::new();renamed::State::update(&mut state,value);let result=renamed::State::finish(&state);let forwarded=result;forwarded}";
    assert!(
        configured_provider_operations(source, "provider", apis, "provider", implementation)
            .unwrap()
            .is_clean()
    );
    configured_provider_hidden_reference_receipts(implementation, apis);
}

fn configured_provider_hidden_reference_receipts(implementation: &str, apis: &[&str]) {
    for source in [
        "pub fn run(state:&mut renamed::State,value:u32)->u32{let result=renamed::State::finish(&state);let unused=renamed::State::update(state,value);result}",
        "type StateRef<'a>=&'a mut renamed::State; pub fn run(state:StateRef<'_>,value:u32)->u32{let result=renamed::State::finish(&state);let unused=renamed::State::update(state,value);result}",
    ] {
        let error =
            configured_provider_operations(source, "provider", apis, "provider", implementation)
                .unwrap_err();
        assert!(error.to_string().contains("does not contribute"), "{error}");
    }
    let implementation = "pub struct State(u32); impl State{pub fn update(&mut self,value:u32){self.0=value;}pub fn finish(&self)->u32{self.0}} pub fn borrow()->&'static mut State{Box::leak(Box::new(State(0)))}";
    let apis = [
        "provider::borrow",
        "provider::State::update",
        "provider::State::finish",
    ];
    let source = "pub fn run(value:u32)->u32{let reference=renamed::borrow();let result=renamed::State::finish(&reference);let unused=renamed::State::update(reference,value);result}";
    let error =
        configured_provider_operations(source, "provider", &apis, "provider", implementation)
            .unwrap_err();
    assert!(error.to_string().contains("does not contribute"), "{error}");
    for implementation in [
        "pub struct State(u32); impl State{pub fn update(&mut self,value:u32){self.0=value;}pub fn finish(&self)->u32{self.0}} pub type StateRef=&'static mut State; pub fn borrow()->StateRef{Box::leak(Box::new(State(0)))}",
        "pub struct State(u32); impl State{pub fn update(state:StateRef,value:u32){state.0=value;}pub fn finish(&self)->u32{self.0}} pub type StateRef=&'static mut State; pub fn borrow()->&'static mut State{Box::leak(Box::new(State(0)))}",
    ] {
        let error =
            configured_provider_operations(source, "provider", &apis, "provider", implementation)
                .unwrap_err();
        assert!(error.to_string().contains("does not contribute"), "{error}");
    }
    let implementation = "pub struct State(pub u32); pub struct Holder<'a>(pub &'a mut State); pub fn finish(h:&Holder)->u32{h.0.0} pub fn update(h:Holder,value:u32){h.0.0=value;}";
    let apis = ["provider::finish", "provider::update"];
    let source = "pub fn run(h:renamed::Holder<'_>,value:u32)->u32{let result=renamed::finish(&h);let unused=renamed::update(h,value);result}";
    let error =
        configured_provider_operations(source, "provider", &apis, "provider", implementation)
            .unwrap_err();
    assert!(error.to_string().contains("does not contribute"), "{error}");
    let source = "pub fn run(h:renamed::Holder<'_>,value:u32)->u32{renamed::finish(&h)}";
    assert!(
        configured_provider_operations(source, "provider", &apis, "provider", implementation)
            .unwrap()
            .is_clean()
    );
}

#[test]
fn configured_provider_audits_selected_nominal_headers_before_method_proof() {
    let source = "pub fn run(value:u32)->u32{let mut state=renamed::State::new();renamed::State::update(&mut state,value);renamed::State::finish(&state)}";
    let apis = [
        "provider::State::new",
        "provider::State::update",
        "provider::State::finish",
    ];
    for (declaration, implementation) in [
        (
            "pub struct State(u32);",
            "impl State{pub fn new()->Self{Self(0)}pub fn update(&mut self,value:u32){self.0=value;}pub fn finish(&self)->u32{self.0}}",
        ),
        (
            "pub enum State{Value(u32)}",
            "impl State{pub fn new()->Self{Self::Value(0)}pub fn update(&mut self,value:u32){*self=Self::Value(value);}pub fn finish(&self)->u32{match self{Self::Value(value)=>*value}}}",
        ),
        (
            "pub union State{value:u32}",
            "impl State{pub fn new()->Self{Self{value:0}}pub fn update(&mut self,value:u32){self.value=value;}pub fn finish(&self)->u32{unsafe{self.value}}}",
        ),
    ] {
        for (attribute, accepted) in [
            ("#[derive(Copy,Clone)]", true),
            ("#[cfg_attr(false,rewrite)]", true),
            ("#[rewrite]", false),
            ("#[cfg_attr(true,rewrite)]", false),
        ] {
            let library = format!(
                "pub mod api{{pub fn encode(value:u32)->u32{{value+1}}}} {attribute} {declaration} {implementation}"
            );
            let result =
                configured_provider_operations(source, "provider", &apis, "provider", &library);
            if accepted {
                assert!(
                    result
                        .unwrap_or_else(|error| panic!("{attribute} {declaration}: {error}"))
                        .is_clean()
                );
            } else {
                let error = result.unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("unsupported rewriting attribute"),
                    "{attribute} {declaration}: {error}"
                );
            }
        }
    }
}

#[test]
fn configured_provider_audits_declared_and_actual_call_alias_headers() {
    let apis = [
        "provider::State::new",
        "provider::State::update",
        "provider::State::finish",
    ];
    let native = "pub mod api{pub fn encode(value:u32)->u32{value+1}} pub struct State(u32); impl State{pub fn new()->Self{Self(0)}pub fn update(&mut self,value:u32){self.0=value;}pub fn finish(&self)->u32{self.0}}";
    for (attribute, accepted) in [
        ("#[allow(dead_code)]", true),
        ("#[cfg_attr(false,rewrite)]", true),
        ("#[rewrite]", false),
    ] {
        let source = format!(
            "{attribute} pub mod local{{pub type Adapter=renamed::State;}} pub fn run(value:u32)->u32{{let mut state=local::Adapter::new();local::Adapter::update(&mut state,value);local::Adapter::finish(&state)}}"
        );
        let result = configured_provider_operations(&source, "provider", &apis, "provider", native);
        if accepted {
            assert!(
                result
                    .unwrap_or_else(|error| panic!("{source}: {error}"))
                    .is_clean()
            );
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("unsupported rewriting attribute")
            );
        }
        let source = format!(
            "{attribute} pub type Adapter=renamed::State; pub fn run(value:u32)->u32{{let mut state=Adapter::new();Adapter::update(&mut state,value);Adapter::finish(&state)}}"
        );
        let result = configured_provider_operations(&source, "provider", &apis, "provider", native);
        if accepted {
            assert!(
                result
                    .unwrap_or_else(|error| panic!("{source}: {error}"))
                    .is_clean()
            );
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("unsupported rewriting attribute")
            );
        }
        let library = format!("{native} {attribute} pub type Adapter=State;");
        let source = "pub fn run(value:u32)->u32{let mut state=renamed::Adapter::new();renamed::Adapter::update(&mut state,value);renamed::Adapter::finish(&state)}";
        let declared = [
            "provider::Adapter::new",
            "provider::Adapter::update",
            "provider::Adapter::finish",
        ];
        let result =
            configured_provider_operations(source, "provider", &declared, "provider", &library);
        if accepted {
            assert!(
                result
                    .unwrap_or_else(|error| panic!("{attribute}: {error}"))
                    .is_clean()
            );
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("unsupported rewriting attribute")
            );
        }
    }
}

#[test]
fn configured_provider_revisits_ancestors_of_aliases_discovered_before_their_headers() {
    let apis = [
        "provider::State::new",
        "provider::State::update",
        "provider::State::finish",
    ];
    let native = "pub mod api{pub fn encode(value:u32)->u32{value+1}} pub struct State(u32); impl State{pub fn new()->Self{Self(0)}pub fn update(&mut self,value:u32){self.0=value;}pub fn finish(&self)->u32{self.0}}";
    let shared = "pub fn run(value:u32)->u32{let mut state=crate::alias_path::Adapter::new();crate::alias_path::Adapter::update(&mut state,value);crate::alias_path::Adapter::finish(&state)} pub type Adapter=renamed::State;";
    for (attribute, accepted) in [
        ("", true),
        ("#[cfg_attr(false,rewrite)]", true),
        ("#[rewrite]", false),
    ] {
        let source = format!(
            "{attribute} #[path=\"shared.rs\"] pub mod alias_path; #[path=\"shared.rs\"] pub mod consumer_path; pub use consumer_path::run;"
        );
        let result = configured_provider_layout(
            &source,
            "provider",
            &apis,
            "provider",
            native,
            Some(shared),
            "demo::alias",
        );
        if accepted {
            assert!(
                result
                    .unwrap_or_else(|error| panic!("{attribute}: {error}"))
                    .is_clean()
            );
        } else {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("unsupported rewriting attribute")
            );
        }
    }
}

#[test]
fn configured_provider_missing_paths_keep_the_original_typed_source_failure() {
    use std::error::Error as _;

    let source = "pub fn run(value:u32)->u32 { renamed::encode(value) }";
    let missing = configured_provider_gate(source, "provider", "provider::api::gone").unwrap_err();
    assert!(
        matches!(&missing, raf_census::Error::ProviderPath { path, source, .. }
        if path == "provider::api::gone" && matches!(source.as_ref(), raf_source::CodeResolveError::Resolve(raf_source::ResolveError::NotFound(_))))
    );
    assert!(
        missing
            .source()
            .unwrap()
            .downcast_ref::<raf_source::CodeResolveError>()
            .is_some()
    );
    let unknown = configured_provider_gate(source, "missing", "provider::api::encode").unwrap_err();
    assert!(unknown.to_string().contains("no admitted dependency route"));
    assert!(
        configured_provider_gate(source, "provider", "provider::api::encode")
            .unwrap()
            .is_clean()
    );
}

fn tree(source: &str, graph: Builder, copies: usize) -> MemoryTree {
    let ledger = graph.finish().unwrap().to_text(Format::Turtle).unwrap();
    let mut files = vec![
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_owned(),
        ),
        ("src/lib.rs", source.to_owned()),
        ("helpers-ledger.ttl", ledger),
    ];
    if copies > 0 {
        let record =
            DebtRecord::new(NORMAL_FORM, [], BTreeMap::from([("number".into(), copies)])).unwrap();
        files.push(("census-debt.ttl", record.to_turtle().unwrap()));
    }
    files.into_iter().collect()
}

fn lexical_pattern_tree(source: &str, matcher: &str, expression: &str) -> MemoryTree {
    tree(source, lexical_pattern_graph(matcher, expression), 0)
}

fn lexical_pattern_graph(matcher: &str, expression: &str) -> Builder {
    let mut graph = declared_pattern(Some(matcher), true);
    for (node, key, value) in [
        ("#pattern", matcher, Term::simple_literal("demo::home")),
        (
            "#job",
            "forbiddenConstant",
            Term::typed_literal("7", vocab::INTEGER),
        ),
    ] {
        assert!(
            graph
                .remove(Triple {
                    subject: Term::iri(node),
                    predicate: Term::iri(format!("{}{key}", vocab::REPORT)),
                    object: value,
                })
                .unwrap()
        );
    }
    put(
        &mut graph,
        "#pattern",
        matcher,
        Term::simple_literal(expression),
    );
    graph
}

#[test]
fn path_pattern_kinds_execute_with_positive_and_negative_syntax_neighbours() {
    for (kind, expression, home, positive, negative) in [
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(1);",
            "std::mem::drop(2);",
            "std::mem::forget(2);",
        ),
        (
            "path",
            "std::mem::drop",
            "let _=std::mem::drop::<u32>;",
            "let _=std::mem::drop::<u32>;",
            "let _=std::mem::forget::<u32>;",
        ),
        (
            "macro",
            "println",
            "println!(\"home\");",
            "std::println!(\"copy\");",
            "eprintln!(\"other\");",
        ),
        (
            "typePath",
            "std::sync::Mutex",
            "let _:Option<std::sync::Mutex<u32>>=None;",
            "let _:Option<std::sync::Mutex<u32>>=None;",
            "let _:Option<std::sync::RwLock<u32>>=None;",
        ),
    ] {
        for (body, copies) in [(positive, 1), (negative, 0)] {
            let source = format!(
                "pub fn home(){{{home}}}\npub use home as alias;\npub fn alternate(){{{body}}}\n"
            );
            let report = Gate::workspace(".")
                .check_tree(&lexical_pattern_tree(&source, kind, expression))
                .unwrap();
            assert_eq!(
                report.jobs()[0].copies,
                copies,
                "{kind}: {body}: {:?}",
                report.findings()
            );
            assert_eq!(
                report.is_clean(),
                copies == 0,
                "{kind}: {body}: {:?}",
                report.findings()
            );
        }
    }
}

#[test]
fn call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures() {
    for (other, copies) in [
        ("pub fn other(){release(2);}", 1),
        ("pub fn other(){use std::mem::drop as inner; inner(2);}", 1),
        ("pub fn other(release:fn(u32)){release(2);}", 0),
        ("pub fn other(){let release=|_:u32|{}; release(2);}", 0),
        ("pub fn other(){let _=|release:fn(u32)| release(2);}", 0),
        ("pub fn other(){let release=release::<u32>; release(2);}", 0),
        ("pub fn other(){let _=std::mem::drop::<u32>;}", 0),
    ] {
        let source = format!(
            "use std::mem::drop as release;\npub fn home(){{release(1);}}\npub use home as alias;\n{other}\n"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn type_patterns_follow_aliases_without_capturing_generic_or_module_shadows() {
    for (other, copies) in [
        ("pub fn other(_:Lock<u32>){}", 1),
        ("pub fn other<Lock>(_:Lock){}", 0),
        ("pub fn other<std>(){let _:std::sync::Mutex<u32>;}", 0),
        (
            "pub mod nested {pub mod std {pub mod sync {pub struct Mutex<T>(T);}} pub fn other(_:std::sync::Mutex<u32>){}}",
            0,
        ),
        (
            "pub mod nested {pub mod std {} pub fn other(_: ::std::sync::Mutex<u32>){}}",
            1,
        ),
    ] {
        let source = format!(
            "use std::sync::Mutex as Lock;\npub fn home(_:Lock<u32>){{}}\npub use home as alias;\n{other}\n"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(
                &source,
                "typePath",
                "std::sync::Mutex",
            ))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites() {
    let graph = lexical_pattern_graph("call", "demo::work")
        .finish()
        .unwrap();
    for (body, copies) in [("through();", 1), ("let through=||{}; through();", 0)] {
        let files:MemoryTree = [
            ("Cargo.toml", "[workspace]\nmembers=['home','consumer']\nresolver='2'\n".to_owned()),
            ("home/Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".into()),
            ("consumer/Cargo.toml", "[package]\nname='consumer'\nversion='0.1.0'\nedition='2021'\n[dependencies]\ndemo={path='../home'}\n".into()),
            ("home/src/lib.rs", "pub fn work(){} pub fn home(){work();} pub use home as alias;".into()),
            ("consumer/src/lib.rs", format!("mod reexport {{pub use demo::work as renamed;}} use reexport::renamed as through; pub fn other(){{{body}}}")),
            ("helpers-ledger.ttl", graph.to_text(Format::Turtle).unwrap()),
        ].into_iter().collect();
        let report = Gate::workspace(".").check_tree(&files).unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{body}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{body}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn macro_name_patterns_follow_aliases_but_not_a_differently_named_local_macro() {
    for (other, copies) in [
        ("pub fn other(){say!(\"copy\");}", 1),
        (
            "pub fn other(){say!(\"copy\"); macro_rules! say {($x:expr)=>{}}}",
            1,
        ),
        (
            "macro_rules! say {($x:expr)=>{}} pub fn other(){say!(\"local\");}",
            0,
        ),
        (
            "pub fn other(){macro_rules! say {($x:expr)=>{}} say!(\"local\");}",
            0,
        ),
    ] {
        let source = format!(
            "use std::println as say; pub fn home(){{say!(\"home\");}} pub use home as alias; {other}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "macro", "println"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function() {
    for (pattern, home, copies) in [
        ("demo::drop_it", "drop_it();", 0),
        ("std::mem::drop", "std::mem::drop(0);", 1),
    ] {
        let source = format!(
            "pub fn drop_it(){{}} pub fn home(){{{home}}} pub use home as alias; pub fn other(){{use std::mem::drop as drop_it; drop_it(1);}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", pattern))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{pattern}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{pattern}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn constant_pattern_identifiers_keep_their_actual_reference_in_match_arms() {
    for (arm, copies) in [
        ("BANNED => {let _=BANNED;}, _=>{}", 1),
        ("binding => {let _=binding;}", 0),
    ] {
        let source = format!(
            "pub const BANNED:u8=1; pub fn home(){{let _=BANNED;}} pub use home as alias; pub fn other(value:u8){{match value {{{arm}}}}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "path", "demo::BANNED"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{arm}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{arm}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn call_patterns_observe_macro_repetition_and_statement_arguments_with_clean_neighbours() {
    for (body, copies) in [
        ("let _=vec![std::mem::drop(1); 2];", 1),
        ("let _=vec![std::mem::forget(1); 2];", 0),
        ("statements!{let _=std::mem::drop(1);}", 1),
        ("statements!{let _=std::mem::forget(1);}", 0),
    ] {
        let source = format!(
            "macro_rules! statements {{($($body:tt)*)=>{{$($body)*}}}} pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{{body}}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{body}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{body}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn general_path_patterns_include_grouped_renamed_imports_in_their_actual_module_scope() {
    for (import, copies) in [
        ("use std::mem::{self, drop as outside};", 1),
        ("use std::mem::{self, forget as outside};", 0),
    ] {
        let source = format!(
            "pub mod home {{pub use std::mem::drop as approved; pub fn run(){{approved(0);}}}} pub use home as alias; {import}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "path", "std::mem::drop"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{import}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{import}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn call_patterns_include_qualified_trait_and_inherent_paths_without_receiver_guessing() {
    for (target, home, other, copies) in [
        (
            "demo::Tr::blocked",
            "<S as Tr>::blocked();",
            "<S as Tr>::blocked();",
            1,
        ),
        (
            "demo::Tr::blocked",
            "<S as Tr>::blocked();",
            "<S as Other>::blocked();",
            0,
        ),
        (
            "demo::S::inherent",
            "<S>::inherent();",
            "<S>::inherent();",
            1,
        ),
        ("demo::S::inherent", "<S>::inherent();", "<S>::other();", 0),
    ] {
        let source = format!(
            "pub struct S; pub trait Tr {{fn blocked(){{}}}} pub trait Other {{fn blocked(){{}}}} impl Tr for S{{}} impl Other for S{{}} impl S{{pub fn inherent(){{}} pub fn other(){{}}}} pub fn home(){{{home}}} pub use home as alias; pub fn copy(){{{other}}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", target))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{target}: {other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{target}: {other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains() {
    let source = "pub fn home(){std::mem::drop(0);} pub use home as alias; pub fn other(){use b as a; use a as b; a(1);}";
    let error = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "call", "std::mem::drop"))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cyclic written import namespace dependency"),
        "{error}"
    );
    let mut imports = "use std::mem as a0;".to_owned();
    for index in 1..=64 {
        use std::fmt::Write as _;
        write!(imports, "use a{} as a{index};", index - 1).unwrap();
    }
    let source = format!(
        "pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{{imports} use a64::drop as d; d(1);}}"
    );
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1);
    assert!(!report.is_clean());
}

#[test]
fn import_targets_keep_their_declaration_frame() {
    for (body, copies) in [
        (
            "use std::mem as x; use x::drop as d; {struct x{field:u8} d(1);}",
            1,
        ),
        (
            "use std::mem as x; {mod x{pub fn drop(_:u8){}} x::drop(1);}",
            0,
        ),
        (
            "use std::mem as x; use x::drop as d; {use std::hint::black_box as d; d(1);}",
            0,
        ),
    ] {
        let source = format!(
            "pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{{body}}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{body}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{body}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn transitive_block_imports_classify_their_expanded_namespace() {
    for (prefix, copies) in [("std::mem", 1), ("crate::y", 0)] {
        let source = format!(
            "pub mod x{{pub struct drop{{field:u8}}}} pub mod y{{pub fn drop(_:u8){{}}}} pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{use {prefix} as x; use x::drop as d; d(1);}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{prefix}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{prefix}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn qualified_pattern_heads_use_types_beside_value_and_macro_bindings() {
    for (kind, target, home, declarations, other, copies) in [
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "",
            "let std=(); std::mem::drop(std);",
            1,
        ),
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "pub fn local_fn(){}",
            "use crate::local_fn as std; std::mem::drop(0);",
            1,
        ),
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "",
            "use std::mem::drop as std; std(0);",
            1,
        ),
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "",
            "use std::mem::drop as local; let local=||{}; local();",
            0,
        ),
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "pub mod types{pub struct Chosen{}}",
            "let chosen=||{}; {use crate::types::Chosen as chosen; chosen();}",
            0,
        ),
        (
            "call",
            "std::mem::drop",
            "std::mem::drop(0);",
            "",
            "mod std{pub mod mem{pub fn drop(_:u8){}}} std::mem::drop(0);",
            0,
        ),
        (
            "macro",
            "std::println",
            "std::println!(\"home\");",
            "",
            "macro_rules! std{()=>{}} std::println!(\"copy\"); std!();",
            1,
        ),
        (
            "macro",
            "std::println",
            "std::println!(\"home\");",
            "",
            "macro_rules! println{($($arg:tt)*)=>{}} println!(\"local\");",
            0,
        ),
    ] {
        let source = format!(
            "{declarations} pub fn home(){{{home}}} pub use home as alias; pub fn other(){{{other}}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, kind, target))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{kind}: {other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{kind}: {other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn qualified_dependency_heads_ignore_value_imports_and_keep_type_imports() {
    for (other, copies) in [
        ("use crate::local_fn as renamed; renamed::encode(1);", 1),
        ("use crate::local as renamed; renamed::encode(1);", 0),
    ] {
        let graph = lexical_pattern_graph("call", "provider::encode")
            .finish()
            .unwrap();
        let files: MemoryTree = [
            ("Cargo.toml", "[workspace]\nmembers=['provider']\nresolver='2'\n[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[dependencies]\nrenamed={package='provider',path='provider'}\n".to_owned()),
            ("src/lib.rs", format!("pub fn local_fn(){{}} pub mod local{{pub fn encode(_:u8){{}}}} pub fn home(){{renamed::encode(0);}} pub use home as alias; pub fn other(){{{other}}}")),
            ("provider/Cargo.toml", "[package]\nname='provider'\nversion='0.1.0'\nedition='2021'\n".into()),
            ("provider/src/lib.rs", "pub fn encode(_:u8){}".into()),
            ("helpers-ledger.ttl", graph.to_text(Format::Turtle).unwrap()),
        ].into_iter().collect();
        let report = Gate::workspace(".").check_tree(&files).unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{other}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{other}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn same_head_imports_keep_distinct_type_and_value_bindings() {
    for (target, home) in [
        (
            "demo::types::chosen::encode",
            "crate::types::chosen::encode(0);",
        ),
        ("demo::values::chosen", "crate::values::chosen();"),
    ] {
        let source = format!(
            "pub mod types{{pub mod chosen{{pub fn encode(_:u8){{}}}}}} pub mod values{{pub fn chosen(){{}}}} pub fn home(){{{home}}} pub use home as alias; pub fn other(){{use crate::types::chosen; use crate::values::chosen; chosen::encode(0); chosen();}}"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "call", target))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            1,
            "{target}: {:?}",
            report.findings()
        );
        assert!(!report.is_clean());
    }
}

#[test]
fn competing_import_namespaces_refuse_beside_distinct_namespace_imports() {
    for (first, refused) in [("pub fn chosen(){}", true), ("pub mod chosen{}", false)] {
        let source = format!(
            "pub mod first{{{first}}} pub mod second{{pub fn chosen(){{}}}} pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{use crate::first::chosen; use crate::second::chosen; chosen();}}"
        );
        let result = Gate::workspace(".").check_tree(&lexical_pattern_tree(
            &source,
            "call",
            "std::mem::drop",
        ));
        if refused {
            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("competing Value bindings"),
                "{error}"
            );
        } else {
            let report = result.unwrap();
            assert!(report.is_clean(), "{:?}", report.findings());
        }
    }
}

#[test]
fn anonymous_qualified_receivers_refuse_beside_supported_named_receivers() {
    for (call, refused) in [("<[u8]>::len(&[])", true), ("<Named>::len()", false)] {
        let source = format!(
            "pub fn len()->usize{{0}} pub struct Named; impl Named{{pub fn len()->usize{{0}}}} pub fn home(){{let _=len();}} pub use home as alias; pub fn other(){{let _={call};}}"
        );
        let result = Gate::workspace(".").check_tree(&lexical_pattern_tree(&source, "call", "len"));
        if refused {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("qualified self lookup requires a named type or trait path")
            );
        } else {
            assert!(result.unwrap().is_clean());
        }
    }
}

#[test]
fn mixed_constant_binding_reaches_refuse_beside_consistent_constant_reaches() {
    let graph = lexical_pattern_graph("call", "std::mem::drop")
        .finish()
        .unwrap();
    for (provider, refused) in [
        ("pub fn BANNED(){}", true),
        ("pub const BANNED:u8=1;", false),
    ] {
        let files: MemoryTree = [
            ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".to_owned()),
            ("src/lib.rs", "pub fn home(){std::mem::drop(0);} pub use home as alias; mod a; mod b;".into()),
            ("src/a.rs", "pub const BANNED:u8=1; #[path=\"shared.rs\"] mod shared;".into()),
            ("src/b.rs", format!("{provider} #[path=\"shared.rs\"] mod shared;")),
            ("src/shared.rs", "use super::BANNED; pub fn other(value:u8){match value{BANNED=>{let _=BANNED;},_=>{}}}".into()),
            ("helpers-ledger.ttl", graph.to_text(Format::Turtle).unwrap()),
        ].into_iter().collect();
        let result = Gate::workspace(".").check_tree(&files);
        if refused {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("changes binding meaning across source reaches"),
                "{error}"
            );
        } else {
            let report = result.unwrap();
            assert!(report.is_clean(), "{:?}", report.findings());
        }
    }
}

#[test]
fn lexical_pattern_variants_require_the_actual_item_when_a_whole_file_span_ties_its_root() {
    for (symbol, copies) in [("consumer", 1), ("consumer::Banned", 0)] {
        let mut graph = lexical_pattern_graph("literalRegex", "^blocked$");
        put(&mut graph, "#job", "variant", Term::iri("#variant"));
        put(
            &mut graph,
            "#variant",
            "type",
            Term::iri(format!("{}Variant", vocab::REPORT)),
        );
        for (key, value) in [
            ("symbol", symbol),
            ("file", "consumer/src/lib.rs"),
            ("detector", "approved-call"),
            ("criterion", "layering"),
            ("reason", "The documented type is an approved exception"),
        ] {
            put(&mut graph, "#variant", key, Term::simple_literal(value));
        }
        put(
            &mut graph,
            "#variant",
            "anchor",
            Term::iri("urn:raf:code:demo::home"),
        );
        let report = Gate::workspace(".")
            .check_tree(&detector_fixture(
                graph,
                "/// Home.\npub fn home()->&'static str {\"blocked\"}",
                "#[doc=\"blocked\"] pub struct Banned;",
            ))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{symbol}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{symbol}: {:?}",
            report.findings()
        );
        assert!(
            report.jobs()[0]
                .matches
                .iter()
                .any(|found| found.symbol == "consumer::Banned")
        );
    }
}

#[test]
fn lexical_patterns_match_one_token_or_group_but_never_an_unbalanced_run() {
    for (body, copies) in [
        ("consume(2);", 1),
        ("consume({2 + 3});", 1),
        ("consume((2, 3));", 1),
        ("consume(2 + 3);", 0),
        ("let _ = 2 + 3;", 0),
    ] {
        let source = format!(
            "/// Home.\npub fn home() {{ consume(1); }}\npub use home as alias;\nfn consume<T>(value:T)->T {{ value }}\npub fn alternate() {{ {body} }}\n"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "tokens", "consume($_)"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{body}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{body}: {:?}",
            report.findings()
        );
        if copies > 0 {
            assert!(
                report
                    .findings()
                    .iter()
                    .any(|finding| finding.contains("rule approved-call"))
            );
        }
    }
}

#[test]
fn lexical_patterns_reuse_local_normalization_and_keep_parameter_context() {
    let source = "/// Home.\npub fn home() { let original = 1; let _ = original; }\npub use home as alias;\npub fn copy() { let renamed = 2; let _ = renamed; }\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(
            source,
            "tokens",
            "let local = $_; let _ = local;",
        ))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo::copy")
    );
    let source = "/// Home.\npub fn home(first:u32) { consume(first); }\npub use home as alias;\nfn consume<T>(value:T)->T {value}\npub fn other(second:u32) { consume(second); }\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "tokens", "consume($_)"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1);
}

#[test]
fn lexical_pattern_module_homes_own_source_subjects_without_sanctioning_the_file_root() {
    let source = "/// Home.\npub mod home { #[doc=\"blocked\"] pub struct Approved; }\npub use home as alias;\n#[doc=\"blocked\"] pub struct Copy;\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "literalRegex", "^blocked$"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(report.jobs()[0].matches.iter().any(|found| {
        found.symbol == "demo::home::Approved" && found.class == MatchClass::Home
    }));
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| { found.symbol == "demo::Copy" && found.class == MatchClass::Copy })
    );
}

#[test]
fn lexical_pattern_module_homes_include_macros_exported_from_their_actual_source_scope() {
    for (other, copies) in [("blocked", 1), ("allowed", 0)] {
        let source = format!(
            "pub mod home {{ #[macro_export] macro_rules! approved {{()=>{{\"blocked\"}}}} }}\npub use home as alias;\npub fn other()->&'static str {{\"{other}\"}}\n"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "literalRegex", "^blocked$"))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, copies, "{:?}", report.findings());
        assert_eq!(report.is_clean(), copies == 0, "{:?}", report.findings());
        assert!(
            report.jobs()[0]
                .matches
                .iter()
                .any(|found| found.symbol == "demo::approved" && found.class == MatchClass::Home)
        );
    }
}

#[test]
fn many_constant_sites_keep_indexed_matching_within_the_default_grant() {
    struct Source<'a>(&'a str);
    impl std::fmt::Display for Source<'_> {
        fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            out.write_str(
                "pub fn home(){std::mem::drop(0);} pub use home as alias; pub fn other(){",
            )?;
            for index in 0..4000 {
                write!(out, "let _value_{index}={index};")?;
            }
            write!(out, "{}(0);}}", self.0)
        }
    }
    for (callee, copies) in [("std::mem::drop", 1), ("std::mem::forget", 0)] {
        let mut admission = purrdf_lex::allocation::Resident;
        let source = purrdf_lex::allocation::Memory::new(&mut admission)
            .format(&Source(callee))
            .unwrap();
        let tree = lexical_pattern_tree(&source, "call", "std::mem::drop");
        let report = Gate::workspace(".").check_tree(&tree).unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{callee}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{callee}: {:?}",
            report.findings()
        );
        let error = Gate::workspace(".")
            .matching_limits(purrdf_jsonschema::ecma::MatchLimits {
                steps: 200_000,
                states: 65_536,
            })
            .check_tree(&tree)
            .unwrap_err();
        assert!(error.to_string().contains("step limit"), "{error}");
    }
}

#[test]
fn literal_patterns_exclude_doc_comments_but_keep_written_doc_attributes() {
    for (documentation, copies) in [
        ("/// blocked\npub fn other(){}", 0),
        ("//! blocked\npub fn other(){}", 0),
        ("/** blocked */ pub fn other(){}", 0),
        ("/*! blocked */ pub fn other(){}", 0),
        ("#[doc=\"blocked\"] pub fn other(){}", 1),
        ("pub fn other(){let _=\"blocked\";}", 1),
    ] {
        let source = format!(
            "{documentation}\npub fn home()->&'static str{{\"blocked\"}} pub use home as alias;"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "literalRegex", "blocked"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{documentation}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{documentation}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn lexical_patterns_match_decoded_strings_in_macros_and_attributes_but_not_bytes() {
    for (body, copies) in [
        ("let _ = r#\"blocked\"#;", 1),
        ("check!(\"blocked\");", 1),
        ("let _ = b\"blocked\";", 0),
        ("let _ = c\"blocked\";", 0),
        ("let _ = \"allowed\";", 0),
    ] {
        let source = format!(
            "/// Home.\npub fn home()->&'static str {{ \"blocked\" }}\npub use home as alias;\nmacro_rules! check {{ ($value:expr)=>{{}} }}\npub fn alternate() {{ {body} }}\n"
        );
        let report = Gate::workspace(".")
            .check_tree(&lexical_pattern_tree(&source, "literalRegex", "^blocked$"))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{body}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.is_clean(),
            copies == 0,
            "{body}: {:?}",
            report.findings()
        );
    }
    let source = "/// Home.\npub fn home()->&'static str {\"blocked\"}\npub use home as alias;\n#[doc=\"blocked\"] pub struct Attribute;\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "literalRegex", "^blocked$"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo::Attribute")
    );
}

#[test]
fn lexical_patterns_do_not_mistake_a_file_root_for_its_first_written_unit() {
    let source = "pub fn home()->&'static str {\"blocked\"}\npub use home as alias;\nmacro_rules! check { ($value:expr)=>{} }\ncheck!(\"blocked\");\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "literalRegex", "^blocked$"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo" && found.class == MatchClass::Copy)
    );
}

#[test]
fn token_patterns_include_file_level_macro_arguments_without_renaming_item_names() {
    let source = "pub fn home() { check!(1); }\npub use home as alias;\nmacro_rules! check { ($value:expr)=>{} }\ncheck!({2 + 3});\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "tokens", "check!($_)"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo" && found.class == MatchClass::Copy)
    );
}

#[test]
fn token_patterns_normalize_body_local_declarations_without_renaming_module_items() {
    let source = "pub fn home(){fn local(){}}\npub use home as alias;\npub fn copy(){fn renamed(){}}\npub fn banned(){}\n";
    let report = Gate::workspace(".")
        .check_tree(&lexical_pattern_tree(source, "tokens", "fn banned() {}"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo::copy")
    );
    assert!(
        !report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo::banned")
    );
}

#[test]
fn lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans() {
    let retained: syn::File = syn::parse_str("fn retained() {}").unwrap();
    let source = "/// Home.\npub fn home() { consume(1); }\npub use home as alias;\nfn consume<T>(value:T)->T { value }\n";
    let tree = lexical_pattern_tree(source, "tokens", "consume($_)");
    let refused = Gate::workspace(".")
        .matching_limits(purrdf_jsonschema::ecma::MatchLimits {
            steps: 0,
            states: 64,
        })
        .check_tree(&tree)
        .unwrap_err();
    assert!(refused.to_string().contains("step limit"), "{refused}");
    assert!(Gate::workspace(".").check_tree(&tree).unwrap().is_clean());
    let syn::Item::Fn(item) = &retained.items[0] else {
        panic!("retained function")
    };
    assert_eq!(item.sig.ident.to_string(), "retained");
    let empty = lexical_pattern_tree(source, "tokens", "/* empty */");
    assert!(
        Gate::workspace(".")
            .check_tree(&empty)
            .unwrap_err()
            .to_string()
            .contains("must not be empty")
    );
}

#[test]
fn aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once() {
    let mut ledger = job(false);
    put(
        &mut ledger,
        "#job",
        "forbiddenName",
        Term::simple_literal("^copy$"),
    );
    let report = Gate::workspace(".")
        .check_tree(&tree(SOURCE, ledger, 1))
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    assert_eq!(report.jobs()[0].home, "demo::home");
    assert_eq!(report.jobs()[0].copies, 1);
    assert_eq!(report.jobs()[0].matches.len(), 2);
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "demo::home" && found.class == MatchClass::Home)
    );
}

#[test]
fn enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours() {
    let report = Gate::workspace(".")
        .check_tree(&tree(SOURCE, job(true), 0))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("is a COPY"))
    );
    let home = "/// The implementation.\npub fn home() -> u32 { 7 }\npub use home as alias;";
    let stale = Gate::workspace(".")
        .check_tree(&tree(home, job(false), 0))
        .unwrap();
    assert!(
        stale
            .findings()
            .iter()
            .any(|finding| finding.contains("set enforced to true"))
    );
    assert!(
        Gate::workspace(".")
            .check_tree(&tree(home, job(true), 0))
            .unwrap()
            .is_clean()
    );
}

#[test]
fn stale_members_fingerprints_and_variants_are_findings_beside_a_clean_policy() {
    let source = "/// Implementation.\npub fn home()->u32 { 7 }\npub use home as alias;";
    let mut missing_entry = job(true);
    put(
        &mut missing_entry,
        "#job",
        "entryPoint",
        Term::simple_literal("demo::gone"),
    );
    let mut missing_fingerprint = job(true);
    put(
        &mut missing_fingerprint,
        "#job",
        "forbiddenFingerprint",
        Term::simple_literal(Fingerprint::body(&[9; 32]).to_string()),
    );
    for (graph, expected) in [
        (missing_entry, "stale ledger path \"demo::gone\""),
        (missing_fingerprint, "matches no unit"),
        (
            variant_at("src/lib.rs", "demo::gone"),
            "stale ledger path \"demo::gone\"",
        ),
    ] {
        let report = Gate::workspace(".")
            .rules(rules())
            .check_tree(&tree(source, graph, 0))
            .unwrap();
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| finding.contains(expected)),
            "{:?}",
            report.findings()
        );
    }
    assert!(
        Gate::workspace(".")
            .check_tree(&tree(source, job(true), 0))
            .unwrap()
            .is_clean()
    );
}

fn detector_ledger(id: &str) -> Builder {
    let mut graph = job(true);
    assert!(
        graph
            .remove(Triple {
                subject: Term::iri("#job"),
                predicate: Term::iri(format!("{}forbiddenConstant", vocab::REPORT)),
                object: Term::typed_literal("7", vocab::INTEGER),
            })
            .unwrap()
    );
    put(
        &mut graph,
        "#job",
        "forbiddenFingerprint",
        Term::simple_literal(format!("rule:{id}")),
    );
    graph
}

fn detector_tree(id: &str, home: &str, consumer: &str) -> MemoryTree {
    detector_fixture(detector_ledger(id), home, consumer)
}

fn detector_fixture(graph: Builder, home: &str, consumer: &str) -> MemoryTree {
    [
        (
            "Cargo.toml",
            "[workspace]\nmembers=['home','consumer']\nresolver='2'\n".into(),
        ),
        (
            "home/Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".into(),
        ),
        (
            "consumer/Cargo.toml",
            "[package]\nname='consumer'\nversion='0.1.0'\nedition='2021'\n".into(),
        ),
        ("home/src/lib.rs", format!("{home}\npub use home as alias;")),
        ("consumer/src/lib.rs", consumer.into()),
        (
            "helpers-ledger.ttl",
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ]
    .into_iter()
    .collect()
}

#[test]
fn default_hasher_detector_uses_visible_std_paths_and_lexical_aliases() {
    for source in [
        "use std::collections::HashMap as Map; fn copy(){let _=Map::<u8,u8>::new();}",
        "use std::collections::HashMap as r#Map; fn copy(){let _=r#Map::<u8,u8>::r#new();}",
        "mod std {} use ::std::collections::HashMap as Map; fn copy(){let _=Map::<u8,u8>::new();}",
        "use std::collections as c; fn copy(_:c::HashSet<u8>){}",
        "use std::collections as c; use c::HashMap as Map; fn copy(_:Map<u8,u8>){}",
        "struct Cache { map: std::collections::HashMap<u8,u8> }",
        "type Cache = std::collections::HashMap<u8,u8>;",
        "fn copy(){assert_eq!(1,std::collections::HashSet::from([1]));}",
        "#[cfg(test)] fn copy(){let _=std::collections::HashMap::<u8,u8>::default();}",
        "mod a {use foreign::HashMap;} mod b {use std::collections::HashMap; fn copy(_:HashMap<u8,u8>) {}}",
        "fn copy(){let _: std::collections::HashMap<u8,u8,Fixed> = wrap(std::collections::HashMap::default());}",
    ] {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(
                "std-default-hasher",
                "/// Home.\npub fn home(){}",
                source,
            ))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, 1, "{source}: {report:?}");
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| finding.contains("rule std-default-hasher:"))
        );
    }
    for source in [
        "use foreign::HashMap; fn copy(_:HashMap<u8,u8>){}",
        "use std::collections::HashMap; fn copy<HashMap>(_:HashMap){}",
        "use std::collections::HashMap; struct Cache<HashMap>{map:HashMap}",
        "fn copy(_:HashMap<u8,u8>){}",
        "use std::collections::HashMap; fn copy(){let _:HashMap<u8,u8,Fixed>=HashMap::default();}",
        "fn copy(){let _=std::collections::HashSet::<u8,Fixed>::default();}",
        "use std::collections::HashMap; mod a {struct HashMap<K,V>(K,V); fn copy(_:HashMap<u8,u8>) {}}",
        "fn copy(){let _=std::collections::HashMap::with_hasher(Fixed);}",
    ] {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(
                "std-default-hasher",
                "/// Home.\npub fn home(){}",
                source,
            ))
            .unwrap();
        assert!(report.is_clean(), "{source}: {report:?}");
    }
}

#[test]
fn byte_layout_detectors_execute_with_shipping_and_home_package_boundaries() {
    let cases = [
        (
            "length-prefix-frame",
            "fn copy(out:&mut Vec<u8>,b:&[u8]) {out.extend_from_slice(&(b.len() as u64).to_le_bytes());out.extend_from_slice(b);}",
            "fn copy(out:&mut Vec<u8>,b:&[u8]) {write_varint(out,b.len() as u64);out.extend_from_slice(b);}",
        ),
        (
            "le-slice-int",
            "fn copy(b:&[u8])->u32 {u32::from_le_bytes(b[0..4].try_into().unwrap())}",
            "fn copy(b:&[u8])->u32 {u32::from_le_bytes(*b.first_chunk().unwrap())}",
        ),
        (
            "align-up-mask",
            "fn copy(n:usize)->usize {(n+7)&!7}",
            "fn copy(n:usize)->usize {n.next_multiple_of(8)}",
        ),
        (
            "div-ceil-by-hand",
            "fn copy(n:usize,d:usize)->usize {(n+d-1)/d}",
            "fn copy(n:usize,d:usize)->usize {n.div_ceil(d)}",
        ),
        (
            "xor-first-mismatch",
            "fn copy(a:&[u8;8],b:&[u8;8])->u32 {(u64::from_le_bytes(*a)^u64::from_le_bytes(*b)).trailing_zeros()}",
            "fn copy(a:&[u8;8],b:&[u8;8])->u64 {u64::from_le_bytes(*a)^u64::from_le_bytes(*b)}",
        ),
    ];
    let home = "/// Approved implementation.\npub fn home() {}";
    for (id, positive, negative) in cases {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(id, home, positive))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, 1, "{id}: {:?}", report.findings());
        assert_eq!(report.findings().len(), 1, "{id}: {:?}", report.findings());
        assert!(report.findings()[0].contains(&format!("rule {id}:")));
        // Keep the ledger's chosen public entry point in the diagnostic;
        // resolution classifies its physical definition separately.
        assert!(report.findings()[0].ends_with("use demo::alias"));
        for source in [
            negative.to_owned(),
            format!("#[cfg(test)] mod oracle {{{positive}}}"),
        ] {
            let clean = Gate::workspace(".")
                .check_tree(&detector_tree(id, home, &source))
                .unwrap();
            assert!(clean.is_clean(), "{id}: {:?}", clean.findings());
        }
        let own = format!(
            "/// Approved implementation.\n{}",
            positive.replace("fn copy(", "pub fn home(")
        );
        let clean = Gate::workspace(".")
            .check_tree(&detector_tree(id, &own, ""))
            .unwrap();
        assert!(clean.is_clean(), "{id}: {:?}", clean.findings());
    }
}

#[test]
fn layout_findings_require_related_operands_and_rendered_prefix_order() {
    let cases = [
        ("align-up-mask", "fn copy(n:usize)->usize {n&!7}", false),
        (
            "align-up-mask",
            "fn copy(n:usize)->usize {(n+15)&!7}",
            false,
        ),
        (
            "align-up-mask",
            "fn copy(n:u128)->u128 {(n+1)&!0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffffu128}",
            false,
        ),
        (
            "align-up-mask",
            "fn copy(n:usize,a:usize)->usize {(n+(a-1))&!(a-1)}",
            true,
        ),
        (
            "align-up-mask",
            "fn copy(n:usize,a:usize)->usize {(n+a-1)&!(a-1)}",
            true,
        ),
        (
            "xor-first-mismatch",
            "fn copy(a:&[u8;8],b:&[u8;8])->u64 {let unrelated=1u64.trailing_zeros();u64::from_le_bytes(*a)^u64::from_le_bytes(*b)}",
            false,
        ),
        (
            "xor-first-mismatch",
            "fn copy(a:&[u8;8],b:&[u8;8])->u32 {let x=u64::from_le_bytes(*a);let y=u64::from_le_bytes(*b);let diff=x^y;diff.trailing_zeros()}",
            true,
        ),
        (
            "xor-first-mismatch",
            "fn copy(a:&[u8;8],b:&[u8;8])->u32 {let mut diff=u64::from_le_bytes(*a)^u64::from_le_bytes(*b);diff=0;diff.trailing_zeros()}",
            false,
        ),
        (
            "xor-first-mismatch",
            "fn copy(a:&[u8;8],b:&[u8;8])->u32 {let mut diff=u64::from_le_bytes(*a)^u64::from_le_bytes(*b);diff&=0;diff.trailing_zeros()}",
            false,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{:.*}{}\",b.len(),b,b);}",
            false,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{}:{}\",b,b.len());}",
            false,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{}:{}\",b.len(),b);}",
            true,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{1}:{0}\",b,b.len());}",
            true,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{1}:{0}\",b.len(),b);}",
            false,
        ),
        (
            "length-prefix-frame",
            "fn copy(b:&str) {let _=format!(\"{{data}}{count}:{data}\",data=b,count=b.len());}",
            true,
        ),
        (
            "length-prefix-frame",
            "fn copy(out:&mut String,b:&str) {let _=write!(out,\"{}:{}\",b.len(),b);}",
            true,
        ),
    ];
    for (id, source, found) in cases {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(id, "/// Home.\npub fn home() {}", source))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            usize::from(found),
            "{id}, {source}: {:?}",
            report.findings()
        );
        assert_eq!(
            report.findings().len(),
            usize::from(found),
            "{id}, {source}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn fact_detectors_reuse_collected_values_and_keep_their_target_scope() {
    for (id, positive, negative, tests_apply) in [
        (
            "hex-table",
            "fn copy(){let _=b\"0123456789abcdef\";}",
            "fn copy(){let _=b\"0123456789abcdeg\";}",
            true,
        ),
        (
            "json-pointer-escape",
            "fn copy(){let _=\"~0\";}",
            "fn copy(){let _=b\"~0\";}",
            false,
        ),
    ] {
        let home = "/// Home.\npub fn home() {}";
        for (source, expected) in [
            (positive.to_owned(), true),
            (negative.to_owned(), false),
            (
                format!("#[cfg(test)] mod oracle {{{positive}}}"),
                tests_apply,
            ),
        ] {
            let report = Gate::workspace(".")
                .check_tree(&detector_tree(id, home, &source))
                .unwrap();
            assert_eq!(
                report.jobs()[0].copies,
                usize::from(expected),
                "{id}: {:?}",
                report.findings()
            );
            assert_eq!(
                report.findings().len(),
                usize::from(expected),
                "{id}: {:?}",
                report.findings()
            );
        }
        let own = format!(
            "/// Home.\n{}",
            positive.replace("fn copy(", "pub fn home(")
        );
        assert!(
            Gate::workspace(".")
                .check_tree(&detector_tree(id, &own, ""))
                .unwrap()
                .is_clean()
        );
    }
}

#[test]
fn hex_detectors_apply_to_their_declared_targets_and_actual_format_fields() {
    for (id, positive, negative, tests_apply) in [
        (
            "hex-pair-radix",
            "fn copy(s:&str){let _=<u8>::from_str_radix(s,0x10);}",
            "fn copy(s:&str){let _=u8::from_str_radix(s,10);}",
            true,
        ),
        (
            "hex-digit-radix",
            "fn copy(c:char){let _=c.to_digit(0x10);}",
            "fn copy(c:char){let _=c.to_digit(10);}",
            false,
        ),
        (
            "hex-format-loop",
            "fn copy(b:u8){let _=||format!(\"{b:02x}\");}",
            "fn copy(b:u8){let _=format!(\"{b:02x}\");}",
            true,
        ),
    ] {
        let home = "/// Home.\npub fn home() {}";
        for (source, expected) in [
            (positive.to_owned(), true),
            (negative.to_owned(), false),
            (
                format!("#[cfg(test)] mod oracle {{{positive}}}"),
                tests_apply,
            ),
        ] {
            let report = Gate::workspace(".")
                .check_tree(&detector_tree(id, home, &source))
                .unwrap();
            assert_eq!(
                report.jobs()[0].copies,
                usize::from(expected),
                "{id}, {source}: {:?}",
                report.findings()
            );
            assert_eq!(
                report.findings().len(),
                usize::from(expected),
                "{id}, {source}: {:?}",
                report.findings()
            );
        }
        let own = format!(
            "/// Home.\n{}",
            positive.replace("fn copy(", "pub fn home(")
        );
        assert!(
            Gate::workspace(".")
                .check_tree(&detector_tree(id, &own, ""))
                .unwrap()
                .is_clean()
        );
    }
    for (source, expected) in [
        ("fn copy(s:&str){let _=u32::from_str_radix(s,16);}", true),
        ("fn copy(s:&str){let _=<u8>::from_str_radix(s,16);}", false),
    ] {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(
                "hex-digit-radix",
                "/// Home.\npub fn home() {}",
                source,
            ))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, usize::from(expected));
    }
    for (source, expected) in [
        (
            "fn copy(bytes:&[u8]){for byte in bytes {let _=format!(\"{byte:02X}\");}}",
            true,
        ),
        (
            "fn copy(out:&mut String,b:u8){let _=||write!(out,\"{b:02x}\");}",
            true,
        ),
        ("fn copy(){let _=||format!(\"{{:02x}}\");}", false),
        ("fn copy(){let _=||format!(\"{}\",\"{:02x}\");}", false),
        (
            "fn copy(){for _ in 0..1 {fn nested(b:u8){let _=format!(\"{b:02x}\");}}}",
            false,
        ),
    ] {
        let report = Gate::workspace(".")
            .check_tree(&detector_tree(
                "hex-format-loop",
                "/// Home.\npub fn home() {}",
                source,
            ))
            .unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            usize::from(expected),
            "{source}: {:?}",
            report.findings()
        );
    }
}

#[test]
fn contextual_literal_detectors_use_only_their_jobs_home_evidence() {
    for (id, home, positive, negative) in [
        (
            "home-literal",
            "/// Home.\npub fn home()->&'static str {\"token\"}",
            "fn copy(){let _=\"token\";}",
            "fn copy(){let _=\"different\";}",
        ),
        (
            "vocabulary-literal",
            "/// Vocabulary.\npub mod home {pub const NS:&str={ (\"urn:demo:\") };}",
            "fn copy(){let _=\"urn:demo:term\";}",
            "fn copy(){let _=\"urn:other:term\";}",
        ),
    ] {
        for (source, expected) in [
            (positive.to_owned(), true),
            (negative.to_owned(), false),
            (format!("#[cfg(test)] mod oracle {{{positive}}}"), false),
        ] {
            let report = Gate::workspace(".")
                .check_tree(&detector_tree(id, home, &source))
                .unwrap();
            assert_eq!(
                report.jobs()[0].copies,
                usize::from(expected),
                "{id}: {:?}",
                report.findings()
            );
            assert_eq!(
                report.findings().len(),
                usize::from(expected),
                "{id}: {:?}",
                report.findings()
            );
        }
    }
    let empty = Gate::workspace(".")
        .check_tree(&detector_tree(
            "home-literal",
            "/// Home.\npub fn home()->&'static str {\"\"}",
            "fn copy(){let _=\"\";}",
        ))
        .unwrap();
    assert_eq!(empty.jobs()[0].copies, 1);
    let home = "/// Home.\npub fn home()->&'static str {\"first\"}\npub fn second()->&'static str {\"second\"}";
    let mut graph = detector_ledger("home-literal");
    put(&mut graph, "", "job", Term::iri("#second"));
    put(
        &mut graph,
        "#second",
        "type",
        Term::iri(format!("{}LedgerJob", vocab::REPORT)),
    );
    for (key, value) in [
        ("jobId", "second"),
        ("summary", "A different token"),
        ("homeSymbol", "demo::second"),
        ("forbiddenFingerprint", "rule:home-literal"),
    ] {
        put(&mut graph, "#second", key, Term::simple_literal(value));
    }
    put(&mut graph, "#second", "enforced", Term::boolean(true));
    let report = Gate::workspace(".")
        .check_tree(&detector_fixture(
            graph,
            home,
            "fn copy(){let _=\"first\";}",
        ))
        .unwrap();
    assert_eq!(
        report
            .jobs()
            .iter()
            .find(|job| job.id == "number")
            .unwrap()
            .copies,
        1
    );
    assert_eq!(
        report
            .jobs()
            .iter()
            .find(|job| job.id == "second")
            .unwrap()
            .copies,
        0
    );
    assert_eq!(report.findings().len(), 1);
}

#[test]
fn contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope() {
    for home in [
        "/// Home.\npub mod home {pub const NS:&str=concat!(\"urn:\",\"demo:\");}",
        "/// Home.\npub mod home {pub const NS:&str=\"\";}",
        "/// Home.\npub mod home {pub const NS:&str=\"urn:demo:\";pub const OTHER_NS:&str=prefix(\"computed\");}",
    ] {
        let error = Gate::workspace(".")
            .check_tree(&detector_tree("vocabulary-literal", home, ""))
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("nonempty literal string initializer")
        );
    }
    assert!(
        Gate::workspace(".")
            .check_tree(&detector_tree(
                "home-literal",
                "/// Home.\npub fn home(){}",
                ""
            ))
            .unwrap_err()
            .to_string()
            .contains("home containing string literals")
    );
    for test_only in [false, true] {
        let mut graph = detector_ledger("sole-name");
        put(
            &mut graph,
            "#job",
            "forbiddenName",
            Term::simple_literal("^copy$"),
        );
        let copy = if test_only {
            "#[cfg(test)] mod oracle {fn copy(){}}"
        } else {
            "fn copy(){}"
        };
        let home = format!("/// Home.\npub fn home(){{}}\n{copy}");
        let report = Gate::workspace(".")
            .check_tree(&detector_fixture(graph, &home, ""))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, 1);
        assert!(report.findings()[0].contains("rule sole-name:"));
    }
    let mut graph = detector_ledger("sole-name");
    put(
        &mut graph,
        "#job",
        "forbiddenName",
        Term::simple_literal("^copy$"),
    );
    put(
        &mut graph,
        "#job",
        "entryPoint",
        Term::simple_literal("demo::copy"),
    );
    let report = Gate::workspace(".")
        .check_tree(&detector_fixture(
            graph,
            "/// Home.\npub fn home(){} pub fn copy(){}",
            "",
        ))
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
}

struct CopyRule;
impl Rule for CopyRule {
    fn id(&self) -> &'static str {
        "copy-rule"
    }
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        for item in &file.items {
            if let syn::Item::Fn(item) = item
                && item.sig.ident == "copy"
            {
                out.push(RuleHit {
                    line: item.sig.ident.span().start().line,
                    message: "use home".into(),
                    symbol: format!("{}::copy", ctx.package),
                });
            }
        }
    }
    fn fixtures(&self) -> RuleFixtures {
        RuleFixtures {
            positive: vec!["fn copy() {}"],
            negative: vec!["fn home() {}"],
        }
    }
}

fn rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(CopyRule)]
}

struct RootOwnerRule;
impl Rule for RootOwnerRule {
    fn id(&self) -> &'static str {
        "root-owner"
    }
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        CopyRule.check(ctx, file, out);
        for hit in out {
            hit.symbol.clone_from(&ctx.package);
        }
    }
    fn fixtures(&self) -> RuleFixtures {
        CopyRule.fixtures()
    }
}

#[test]
fn registered_rule_hits_cannot_claim_the_root_as_a_first_unit_owner() {
    let source = "pub fn home(){}\npub use home as alias;\nfn copy(){}";
    let error = Gate::workspace(".")
        .rules(vec![Box::new(RootOwnerRule) as Box<dyn Rule>])
        .check_tree(&tree(source, detector_ledger("root-owner"), 0))
        .unwrap_err();
    assert!(error.to_string().contains("one containing census unit"));
    let neighbour = Gate::workspace(".")
        .rules(rules())
        .check_tree(&tree(source, detector_ledger("copy-rule"), 0))
        .unwrap();
    assert_eq!(neighbour.jobs()[0].copies, 1);
}

#[test]
fn an_unknown_rule_is_hard_and_registered_hits_cannot_be_adopted_as_debt() {
    let ledger = || {
        let mut graph = job(false);
        put(
            &mut graph,
            "#job",
            "forbiddenFingerprint",
            Term::simple_literal("rule:copy-rule"),
        );
        graph
    };
    let missing = Gate::workspace(".")
        .check_tree(&tree(SOURCE, ledger(), 1))
        .unwrap_err();
    assert!(missing.to_string().contains("unknown rule id"));
    let report = Gate::workspace(".")
        .rules(rules())
        .check_tree(&tree(SOURCE, ledger(), 1))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("rule copy-rule: use home"))
    );
    assert_eq!(report.jobs()[0].copies, 1);
}

fn variant(file: &str) -> Builder {
    variant_at(file, "demo::copy")
}

fn variant_at(file: &str, symbol: &str) -> Builder {
    let mut graph = job(true);
    put(&mut graph, "#job", "variant", Term::iri("#variant"));
    put(
        &mut graph,
        "#variant",
        "type",
        Term::iri(format!("{}Variant", vocab::REPORT)),
    );
    for (key, value) in [
        ("symbol", symbol),
        ("file", file),
        ("detector", "copy-rule"),
        ("criterion", "layering"),
        ("reason", "The other target has a separate implementation"),
    ] {
        put(&mut graph, "#variant", key, Term::simple_literal(value));
    }
    put(
        &mut graph,
        "#variant",
        "anchor",
        Term::iri("urn:raf:code:demo::home"),
    );
    graph
}

fn shared_tree(graph: Builder) -> MemoryTree {
    [
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".into()),
        ("src/lib.rs", "#[path=\"shared.rs\"] pub mod a;\n#[path=\"shared.rs\"] pub mod b;\npub use b::home;\npub use b::home as alias;".into()),
        ("src/shared.rs", "/// Implementation.\npub fn home()->u32 { 7 }\npub fn copy()->u32 { 7+1 }".into()),
        ("helpers-ledger.ttl", graph.finish().unwrap().to_text(Format::Turtle).unwrap()),
    ].into_iter().collect()
}

#[test]
fn a_home_reached_through_two_modules_uses_one_physical_owner() {
    let report = Gate::workspace(".")
        .check_tree(&shared_tree(job(true)))
        .unwrap();
    let matches = &report.jobs()[0].matches;
    assert_eq!(matches.len(), 2);
    assert!(
        matches
            .iter()
            .any(|found| found.symbol == "demo::a::home" && found.class == MatchClass::Home)
    );
    assert_eq!(report.jobs()[0].copies, 1);
}

#[test]
fn reexported_module_homes_own_their_units_and_actual_home_package() {
    let source = "/// Implementation module.\npub mod implementation {pub fn home()->u32 {7}}\npub use implementation as alias;\npub fn copy()->u32 {7+1}";
    let report = Gate::workspace(".")
        .check_tree(&tree(source, job(true), 0))
        .unwrap();
    assert_eq!(report.jobs()[0].home, "demo::implementation");
    assert_eq!(report.jobs()[0].copies, 1);
    assert!(report.jobs()[0].matches.iter().any(|found| found.symbol
        == "demo::implementation::home"
        && found.class == MatchClass::Home));
    assert_eq!(report.findings().len(), 1);

    let home = "/// Implementation module.\npub mod home {pub fn implementation() {}}\npub fn local_copy(n:usize,d:usize)->usize {(n+d-1)/d}";
    let consumer = "fn copy(n:usize,d:usize)->usize {(n+d-1)/d}";
    let report = Gate::workspace(".")
        .check_tree(&detector_tree("div-ceil-by-hand", home, consumer))
        .unwrap();
    assert_eq!(report.jobs()[0].home, "demo::home");
    assert_eq!(report.jobs()[0].copies, 1);
    assert_eq!(report.findings().len(), 1);
    assert!(report.findings()[0].contains("consumer/src/lib.rs"));
}

struct SharedCopyRule;
impl Rule for SharedCopyRule {
    fn id(&self) -> &'static str {
        "copy-rule"
    }
    fn check(&self, ctx: &FileCtx, file: &syn::File, out: &mut Vec<RuleHit>) {
        CopyRule.check(ctx, file, out);
        if ctx.package != "fixture" {
            for hit in out {
                hit.symbol = "demo::b::copy".into();
            }
        }
    }
    fn fixtures(&self) -> RuleFixtures {
        CopyRule.fixtures()
    }
}

#[test]
fn rules_and_variants_share_physical_ownership_without_widening_file_sanctions() {
    for (file, clean) in [("src/shared.rs", true), ("src/other.rs", false)] {
        let report = Gate::workspace(".")
            .rules(vec![Box::new(SharedCopyRule) as Box<dyn Rule>])
            .check_tree(&shared_tree(variant_at(file, "demo::b::copy")))
            .unwrap();
        assert_eq!(report.is_clean(), clean, "{:?}", report.findings());
        assert_eq!(report.jobs()[0].copies, usize::from(!clean));
    }
}

#[test]
fn a_variant_must_match_its_declared_file_and_documented_anchor() {
    let wrong = Gate::workspace(".")
        .rules(rules())
        .check_tree(&tree(SOURCE, variant("src/other.rs"), 0))
        .unwrap();
    assert!(!wrong.is_clean());
    assert!(
        wrong
            .findings()
            .iter()
            .any(|finding| finding.contains("does not exist in src/other.rs"))
    );
    let correct = Gate::workspace(".")
        .rules(rules())
        .check_tree(&tree(SOURCE, variant("src/lib.rs"), 0))
        .unwrap();
    assert!(correct.is_clean(), "{:?}", correct.findings());
    assert!(
        correct.jobs()[0]
            .matches
            .iter()
            .any(|found| found.class == MatchClass::Variant)
    );
}

struct Owner;
impl LedgerExtension for Owner {
    fn validate(
        &self,
        _: &str,
        graph: raf_census::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>> {
        let node = graph.root();
        Some(
            if node
                .values("urn:example:owner")
                .any(|value| value.literal().is_some_and(|(text, _)| text == "demo"))
            {
                Ok(())
            } else {
                Err("owner must be demo".into())
            },
        )
    }
}

// Nested evidence keeps graph identity and same-owner cycles. This
// validator reads the data through the public membership-restricted view.
struct Nested;
impl LedgerExtension for Nested {
    fn validate(
        &self,
        _: &str,
        graph: raf_census::LedgerExtensionGraph<'_>,
    ) -> Option<Result<(), String>> {
        assert_eq!(graph.root_id(), "#ext");
        assert_eq!(
            graph.nodes().map(|(id, _)| id).collect::<Vec<_>>(),
            ["#a", "#b", "#ext"]
        );
        for unowned in [
            "",
            "#job",
            "#missing",
            "urn:external",
            "urn:raf:construction#ext",
        ] {
            assert!(graph.get(unowned).is_none());
        }
        let root = graph.root();
        assert_eq!(
            root.values("urn:example:owner").next().unwrap().literal(),
            Some(("demo", vocab::STRING))
        );
        assert_eq!(
            root.values("urn:example:row").next().unwrap().iri(),
            Some("#a")
        );
        assert!(
            root.properties()
                .all(|key| [vocab::TYPE, "urn:example:owner", "urn:example:row"].contains(&key))
        );
        let a = graph.get("#a").unwrap();
        let b = graph.get("#b").unwrap();
        assert!(
            a.properties()
                .all(|key| ["urn:example:next", "urn:example:value"].contains(&key))
        );
        assert!(
            b.properties()
                .all(|key| ["urn:example:next", "urn:example:quoted"].contains(&key))
        );
        assert_eq!(
            a.values("urn:example:next").next().unwrap().iri(),
            Some("#b")
        );
        assert_eq!(
            b.values("urn:example:next").next().unwrap().iri(),
            Some("#a")
        );
        let quoted = b
            .values("urn:example:quoted")
            .next()
            .unwrap()
            .quoted()
            .unwrap();
        assert_eq!(
            (quoted.0.iri(), quoted.1.iri(), quoted.2.iri()),
            (Some("#a"), Some("urn:example:claim"), Some("#b"))
        );
        Some(
            if a.values("urn:example:value")
                .next()
                .and_then(raf_rdf::Value::literal)
                == Some(("preserved", vocab::STRING))
            {
                Ok(())
            } else {
                Err("nested evidence changed".into())
            },
        )
    }
}

#[test]
fn extensions_require_one_real_validator_without_loosening_core_fields() {
    let ledger = |owner: &str| {
        let mut graph = job(true);
        put(&mut graph, "#job", "extension", Term::iri("#ext"));
        put(
            &mut graph,
            "#ext",
            "type",
            Term::iri(format!("{}Extension", vocab::REPORT)),
        );
        graph
            .insert(Triple {
                subject: Term::iri("#ext"),
                predicate: Term::iri("urn:example:owner"),
                object: Term::simple_literal(owner),
            })
            .unwrap();
        graph
    };
    let source = "/// Home.\npub fn home()->u32 { 7 }\npub use home as alias;";
    assert!(
        Gate::workspace(".")
            .check_tree(&tree(source, ledger("demo"), 0))
            .unwrap_err()
            .to_string()
            .contains("unclaimed extension")
    );
    let gate =
        || Gate::workspace(".").extensions(vec![Box::new(Owner) as Box<dyn LedgerExtension>]);
    assert!(
        gate()
            .check_tree(&tree(source, ledger("other"), 0))
            .unwrap_err()
            .to_string()
            .contains("owner must be demo")
    );
    assert!(
        gate()
            .check_tree(&tree(source, ledger("demo"), 0))
            .unwrap()
            .is_clean()
    );
    assert!(
        Gate::workspace(".")
            .extensions(vec![
                Box::new(Owner) as Box<dyn LedgerExtension>,
                Box::new(Owner)
            ])
            .check_tree(&tree(source, ledger("demo"), 0))
            .unwrap_err()
            .to_string()
            .contains("more than one LedgerExtension")
    );

    nested_extension_validation(&ledger, source);
    let mut unknown = ledger("demo");
    put(
        &mut unknown,
        "#job",
        "unknownCore",
        Term::simple_literal("not an extension"),
    );
    assert!(gate().check_tree(&tree(source, unknown, 0)).is_err());
}

fn nested_extension_validation(ledger: &impl Fn(&str) -> Builder, source: &str) {
    let nested = |value: &str| {
        let mut graph = ledger("demo");
        for (subject, predicate, object) in [
            ("#ext", "urn:example:row", Term::iri("#a")),
            ("#a", "urn:example:next", Term::iri("#b")),
            ("#a", "urn:example:value", Term::simple_literal(value)),
            ("#b", "urn:example:next", Term::iri("#a")),
            (
                "#b",
                "urn:example:quoted",
                Term::Triple {
                    s: raf_rdf::TermBox::new(Term::iri("#a")),
                    p: raf_rdf::TermBox::new(Term::iri("urn:example:claim")),
                    o: raf_rdf::TermBox::new(Term::iri("#b")),
                },
            ),
        ] {
            graph
                .insert(Triple {
                    subject: Term::iri(subject),
                    predicate: Term::iri(predicate),
                    object,
                })
                .unwrap();
        }
        graph
    };
    let nested_gate =
        || Gate::workspace(".").extensions(vec![Box::new(Nested) as Box<dyn LedgerExtension>]);
    assert!(
        nested_gate()
            .check_tree(&tree(source, nested("preserved"), 0))
            .unwrap()
            .is_clean()
    );
    assert!(
        nested_gate()
            .check_tree(&tree(source, nested("changed"), 0))
            .unwrap_err()
            .to_string()
            .contains("nested evidence changed")
    );

    let graph = nested("preserved").finish().unwrap();
    for format in [Format::Turtle, Format::NTriples] {
        let text = graph.to_text(format).unwrap();
        let policy = match format {
            Format::Turtle => Ledger::from_turtle(&text),
            Format::NTriples => Ledger::from_ntriples(&text),
        }
        .unwrap();
        assert_eq!(
            policy.graph().snapshot().unwrap(),
            graph.snapshot().unwrap()
        );
        let extension = policy.extension("#ext").unwrap();
        assert!(
            extension
                .get(&format!("{}#a", graph.snapshot().unwrap()))
                .is_none()
        );
        assert!(Nested.validate("number", extension).unwrap().is_ok());
    }
    extension_core_capture_refusals(&graph);
    extension_shared_ownership_refusals(&graph);
}

fn extension_core_capture_refusals(graph: &raf_rdf::Graph) {
    // A core/root edge is never an ownership edge, including either owned
    // position of an RDF 1.2 triple term.
    for object in [
        Term::iri(""),
        Term::iri("#job"),
        Term::Triple {
            s: raf_rdf::TermBox::new(Term::iri("#job")),
            p: raf_rdf::TermBox::new(Term::iri("urn:example:claim")),
            o: raf_rdf::TermBox::new(Term::iri("#a")),
        },
        Term::Triple {
            s: raf_rdf::TermBox::new(Term::iri("#a")),
            p: raf_rdf::TermBox::new(Term::iri("urn:example:claim")),
            o: raf_rdf::TermBox::new(Term::iri("#job")),
        },
    ] {
        let mut changed = Builder::from_graph(graph);
        changed
            .insert(Triple {
                subject: Term::iri("#b"),
                predicate: Term::iri("urn:example:invalid"),
                object,
            })
            .unwrap();
        assert!(
            Ledger::from_turtle(&changed.finish().unwrap().to_text(Format::Turtle).unwrap())
                .unwrap_err()
                .to_string()
                .contains("core subject")
        );
    }
}

fn extension_shared_ownership_refusals(graph: &raf_rdf::Graph) {
    // Reserve the second root first; neither root nor descendant ownership
    // depends on the order in which the jobs are traversed.
    for target in ["#ext", "#a", "#ext2"] {
        let mut changed = Builder::from_graph(graph);
        put(&mut changed, "", "job", Term::iri("#job2"));
        for mut row in graph
            .assertions()
            .filter(|row| row.subject == Term::iri("#job"))
        {
            row.subject = Term::iri("#job2");
            if row.predicate == Term::iri(format!("{}jobId", vocab::REPORT)) {
                row.object = Term::simple_literal("second");
            } else if row.predicate == Term::iri(format!("{}extension", vocab::REPORT)) {
                row.object = Term::iri(if target == "#ext" { "#ext" } else { "#ext2" });
            }
            changed.insert(row).unwrap();
        }
        if target != "#ext" {
            put(
                &mut changed,
                "#ext2",
                "type",
                Term::iri(format!("{}Extension", vocab::REPORT)),
            );
            let (from, to) = if target == "#ext2" {
                ("#a", "#ext2")
            } else {
                ("#ext2", "#a")
            };
            changed
                .insert(Triple {
                    subject: Term::iri(from),
                    predicate: Term::iri("urn:example:shared"),
                    object: Term::iri(to),
                })
                .unwrap();
        }
        let error =
            Ledger::from_turtle(&changed.finish().unwrap().to_text(Format::Turtle).unwrap())
                .unwrap_err()
                .to_string();
        assert!(
            error.contains("shared by jobs") || error.contains("owned by extension"),
            "{error}"
        );
    }
    let mut orphan = Builder::from_graph(graph);
    orphan
        .insert(Triple {
            subject: Term::iri("#orphan"),
            predicate: Term::iri("urn:example:value"),
            object: Term::simple_literal("unowned"),
        })
        .unwrap();
    assert!(
        Ledger::from_turtle(&orphan.finish().unwrap().to_text(Format::Turtle).unwrap())
            .unwrap_err()
            .to_string()
            .contains("orphan")
    );
}

#[test]
fn distinct_sanctions_an_exact_set_and_a_new_clone_breaks_it() {
    let ledger = || {
        let mut graph = base();
        put(
            &mut graph,
            "",
            "minTokens",
            Term::typed_literal("1", vocab::INTEGER),
        );
        put(&mut graph, "", "distinct", Term::iri("#distinct"));
        put(
            &mut graph,
            "#distinct",
            "type",
            Term::iri(format!("{}Distinct", vocab::REPORT)),
        );
        for (key, value) in [
            ("kind", "body"),
            ("memberSymbol", "demo::home"),
            ("memberSymbol", "demo::copy"),
            ("criterion", "layering"),
            ("reason", "Separate target implementations"),
        ] {
            put(&mut graph, "#distinct", key, Term::simple_literal(value));
        }
        put(
            &mut graph,
            "#distinct",
            "anchor",
            Term::iri("urn:raf:code:demo::home"),
        );
        graph
    };
    let source =
        "/// Home.\npub fn home(a:u32)->u32 { a+a+a+a }\npub fn copy(b:u32)->u32 { b+b+b+b }";
    let correct = Gate::workspace(".")
        .check_tree(&tree(source, ledger(), 0))
        .unwrap();
    assert!(correct.is_clean(), "{:?}", correct.findings());
    let mut root = ledger();
    assert!(
        root.remove(Triple {
            subject: Term::iri("#distinct"),
            predicate: Term::iri(format!("{}memberSymbol", vocab::REPORT)),
            object: Term::simple_literal("demo::home"),
        })
        .unwrap()
    );
    put(
        &mut root,
        "#distinct",
        "memberSymbol",
        Term::simple_literal("demo"),
    );
    let report = Gate::workspace(".")
        .check_tree(&tree(source, root, 0))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("does not name a census implementation unit"))
    );
    let changed = format!("{source}\npub fn third(c:u32)->u32 {{ c+c+c+c }}");
    let report = Gate::workspace(".")
        .check_tree(&tree(&changed, ledger(), 0))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("equals no current group"))
    );
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("growth cannot be recorded"))
    );
}

fn cfg_ledger(count: usize) -> Builder {
    let mut graph = base();
    put(
        &mut graph,
        "",
        "minTokens",
        Term::typed_literal("1", vocab::INTEGER),
    );
    if count == 0 {
        return graph;
    }
    put(&mut graph, "", "distinct", Term::iri("#distinct"));
    put(
        &mut graph,
        "#distinct",
        "type",
        Term::iri(format!("{}Distinct", vocab::REPORT)),
    );
    for (key, value) in [
        ("kind", "body"),
        ("memberSymbol", "demo::f"),
        ("criterion", "platform"),
        ("reason", "Separate platform implementations"),
    ] {
        put(&mut graph, "#distinct", key, Term::simple_literal(value));
    }
    put(
        &mut graph,
        "#distinct",
        "anchor",
        Term::iri("urn:raf:code:demo::f"),
    );
    put(&mut graph, "#distinct", "multiplicity", Term::iri("#count"));
    put(
        &mut graph,
        "#count",
        "type",
        Term::iri(format!("{}MemberMultiplicity", vocab::REPORT)),
    );
    put(
        &mut graph,
        "#count",
        "symbol",
        Term::simple_literal("demo::f"),
    );
    put(
        &mut graph,
        "#count",
        "copies",
        Term::typed_literal(count.to_string(), vocab::INTEGER),
    );
    graph
}

#[test]
fn cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction() {
    let source = "/// Unix.\n#[cfg(unix)] pub fn f(a:u32)->u32 { a+a+a+a }\n/// Windows.\n#[cfg(windows)] pub fn f(b:u32)->u32 { b+b+b+b }";
    let gate = Gate::workspace(".");
    let report = gate.check_tree(&tree(source, cfg_ledger(0), 0)).unwrap();
    assert_eq!(report.debt().groups().len(), 1);
    assert_eq!(
        report.debt().groups().first().unwrap().members(),
        &BTreeMap::from([("demo::f".into(), 2)])
    );
    let clean = gate.check_tree(&tree(source, cfg_ledger(2), 0)).unwrap();
    assert!(clean.is_clean(), "{:?}", clean.findings());
    let extra = format!(
        "{source}\n/// Other.\n#[cfg(target_os=\"freebsd\")] pub fn f(c:u32)->u32 {{ c+c+c+c }}"
    );
    let changed = gate.check_tree(&tree(&extra, cfg_ledger(2), 0)).unwrap();
    assert!(
        changed
            .findings()
            .iter()
            .any(|finding| finding.contains("declared 2 source units, found 3"))
    );
    assert_eq!(
        changed.debt().groups().first().unwrap().members()["demo::f"],
        3
    );
}

fn migration_ledger(fingerprints: &[Fingerprint], normal_form: u32) -> String {
    let mut graph = job(true);
    let version = |normal_form: u32| Triple {
        subject: Term::iri(""),
        predicate: Term::iri(format!("{}normalForm", vocab::REPORT)),
        object: Term::typed_literal(normal_form.to_string(), vocab::INTEGER),
    };
    assert!(graph.remove(version(NORMAL_FORM)).unwrap());
    graph.insert(version(normal_form)).unwrap();
    put(
        &mut graph,
        "",
        "minTokens",
        Term::typed_literal("1", vocab::INTEGER),
    );
    for fingerprint in fingerprints {
        put(
            &mut graph,
            "#job",
            "forbiddenFingerprint",
            Term::simple_literal(fingerprint.to_string()),
        );
    }
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn migration_source() -> (MemoryTree, Census) {
    let tree = tree(SOURCE, job(true), 0);
    let census = Census::load(
        &tree,
        &["."],
        &CensusOptions::default().min_tokens(1).unwrap(),
    )
    .unwrap();
    (tree, census)
}

fn migrated(
    tree: &MemoryTree,
    ledger: &str,
    index: &str,
) -> Result<raf_census::Migration, raf_census::Error> {
    migrate_index(
        tree,
        MigrationInput {
            path: "older-ledger.ttl",
            text: ledger,
        },
        MigrationInput {
            path: "older-index.ttl",
            text: index,
        },
    )
}

#[test]
fn index_migration_splits_changed_members_and_preserves_nonhash_assertions() {
    let (tree, census) = migration_source();
    let old = Fingerprint::body(&[9; 32]);
    let mut previous = census.index(&[]).unwrap();
    previous.normal_form = NORMAL_FORM - 1;
    previous.groups.clear();
    for unit in &mut previous.units {
        unit.body = Some(old.clone());
        unit.shim = None;
    }
    let ledger = migration_ledger(
        &[
            old,
            "table:hex-lower".parse().unwrap(),
            "rule:copy-rule".parse().unwrap(),
        ],
        previous.normal_form,
    );
    let result = migrated(&tree, &ledger, &previous.to_turtle().unwrap()).unwrap();
    assert_eq!(result.unmapped, []);
    let policy = Ledger::from_turtle(&result.ledger).unwrap();
    let fingerprints = &policy.jobs()[0].job.forbidden.fingerprints;
    let current: Vec<_> = census
        .units()
        .iter()
        .filter_map(|unit| unit.print.as_ref().and_then(|print| print.body.clone()))
        .collect();
    assert_eq!(current.len(), 2);
    assert!(
        current
            .iter()
            .all(|fingerprint| fingerprints.contains(fingerprint))
    );
    assert!(fingerprints.contains(&"table:hex-lower".parse().unwrap()));
    assert!(fingerprints.contains(&"rule:copy-rule".parse().unwrap()));
    assert_eq!(policy.jobs()[0].job.home, "demo::alias");
    assert_eq!(policy.jobs()[0].job.forbidden.constants, [7]);
    assert!(policy.jobs()[0].enforced);
}

#[test]
fn index_migration_keeps_a_new_hash_that_equals_another_old_row() {
    let (tree, census) = migration_source();
    let zero = Fingerprint::body(&[0; 32]);
    let home = census
        .unit("demo::home")
        .unwrap()
        .print
        .as_ref()
        .unwrap()
        .body
        .clone()
        .unwrap();
    assert_ne!(zero, home);
    let mut previous = census.index(&[]).unwrap();
    previous.normal_form = NORMAL_FORM - 1;
    previous.groups.clear();
    for unit in &mut previous.units {
        unit.body = Some(if unit.symbol == "demo::home" {
            zero.clone()
        } else {
            home.clone()
        });
        unit.shim = None;
    }
    let ledger = migration_ledger(&[zero, home], previous.normal_form);
    let result = migrated(&tree, &ledger, &previous.to_turtle().unwrap()).unwrap();
    let policy = Ledger::from_turtle(&result.ledger).unwrap();
    let expected: std::collections::BTreeSet<_> = census
        .units()
        .iter()
        .filter_map(|unit| unit.print.as_ref().and_then(|print| print.body.clone()))
        .collect();
    assert_eq!(
        policy.jobs()[0]
            .job
            .forbidden
            .fingerprints
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
}

#[test]
fn index_migration_retains_unmapped_rows_with_reasons_and_comments() {
    let (tree, census) = migration_source();
    let old = Fingerprint::body(&[9; 32]);
    let mut previous = census.index(&[]).unwrap();
    previous.normal_form = NORMAL_FORM - 1;
    let ledger = migration_ledger(std::slice::from_ref(&old), previous.normal_form);
    let result = migrated(&tree, &ledger, &previous.to_turtle().unwrap()).unwrap();
    assert_eq!(result.unmapped.len(), 1);
    assert_eq!(result.unmapped[0].original, old.to_string());
    assert!(result.unmapped[0].reason.contains("no member symbol"));
    assert!(
        result
            .ledger
            .contains("# unmapped: job number fingerprint body:")
    );
    let canonical = result.ledger.split("# unmapped:").next().unwrap();
    assert!(
        Ledger::from_turtle(canonical).unwrap().jobs()[0]
            .job
            .forbidden
            .fingerprints
            .contains(&old)
    );
    for unit in &mut previous.units {
        unit.body = Some(old.clone());
        unit.symbol = "demo::gone".into();
    }
    // The native index still owns distinct physical sites, even when the old
    // path has disappeared from the current source.
    let result = migrated(&tree, &ledger, &previous.to_turtle().unwrap()).unwrap();
    assert!(
        result.unmapped[0]
            .reason
            .contains("no member symbol resolves")
    );
    previous.units[0].symbol = "demo::home".into();
    let partial = migrated(&tree, &ledger, &previous.to_turtle().unwrap()).unwrap();
    assert_eq!(partial.unmapped.len(), 1);
    assert!(
        partial.unmapped[0]
            .reason
            .contains("incomplete member mapping")
    );
    assert!(partial.unmapped[0].reason.contains("demo::gone"));
    assert_eq!(partial.unmapped[0].original, old.to_string());
    let canonical = partial.ledger.split("# unmapped:").next().unwrap();
    assert!(
        Ledger::from_turtle(canonical).unwrap().jobs()[0]
            .job
            .forbidden
            .fingerprints
            .contains(&old)
    );
}

#[test]
fn index_migration_refuses_current_versions_wrong_reports_and_mismatched_versions() {
    let (tree, census) = migration_source();
    let old = migration_ledger(&[], NORMAL_FORM - 1);
    let current = migration_ledger(&[], NORMAL_FORM);
    let index = census.index(&[]).unwrap().to_turtle().unwrap();
    for (ledger, report, expected) in [
        (current.as_str(), index.clone(), "nothing to migrate"),
        (old.as_str(), index, "does not match ledger normal form"),
        (
            old.as_str(),
            census.baseline().to_turtle().unwrap(),
            "older-index.ttl",
        ),
        (old.as_str(), "not RDF".into(), "older-index.ttl"),
    ] {
        let error = migrated(&tree, ledger, &report).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn index_migration_maps_every_cfg_shim_site_of_a_shared_symbol() {
    let source = "#[cfg(unix)] pub fn home(x:u32,y:u32)->u32 { helper(x,y) }\n#[cfg(windows)] pub fn home(x:u32,y:u32)->u32 { helper(y,x) }\npub use home as alias;\nfn helper(x:u32,y:u32)->u32 { x+y }";
    let tree = tree(source, job(true), 0);
    let census = Census::load(
        &tree,
        &["."],
        &CensusOptions::default().min_tokens(1).unwrap(),
    )
    .unwrap();
    let expected: std::collections::BTreeSet<_> = census
        .units_of("demo::home")
        .filter_map(|unit| unit.shim_key.clone())
        .collect();
    assert_eq!(expected.len(), 2);
    let old = Fingerprint::shim(&[3; 32]);
    let mut index = census.index(&[]).unwrap();
    index.normal_form = NORMAL_FORM - 1;
    index.groups.clear();
    for unit in &mut index.units {
        unit.shim = (unit.symbol == "demo::home").then(|| old.clone());
    }
    let ledger = migration_ledger(&[old], index.normal_form);
    let result = migrated(&tree, &ledger, &index.to_turtle().unwrap()).unwrap();
    assert_eq!(result.unmapped, []);
    let policy = Ledger::from_turtle(&result.ledger).unwrap();
    assert_eq!(
        policy.jobs()[0]
            .job
            .forbidden
            .fingerprints
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
}

#[test]
fn index_migration_refuses_oversized_split_output_beside_a_same_size_mapping() {
    let (tree, census) = migration_source();
    let old = Fingerprint::body(&[9; 32]);
    let ledger = migration_ledger(std::slice::from_ref(&old), NORMAL_FORM - 1);
    let graph = raf_rdf::Graph::from_text(&ledger, Format::Turtle).unwrap();
    let mut padded = Builder::from_graph(&graph);
    let summary = |text: &str| Triple {
        subject: Term::iri("#job"),
        predicate: Term::iri(format!("{}summary", vocab::REPORT)),
        object: Term::simple_literal(text),
    };
    let original = "The approved constant";
    assert!(padded.remove(summary(original)).unwrap());
    padded
        .insert(summary(
            &"p".repeat(original.len() + MAX_LEDGER_BYTES - 1 - ledger.len()),
        ))
        .unwrap();
    let ledger = padded.finish().unwrap().to_text(Format::Turtle).unwrap();
    assert_eq!(ledger.len(), MAX_LEDGER_BYTES - 1);
    let mut index = census.index(&[]).unwrap();
    index.normal_form = NORMAL_FORM - 1;
    index.groups.clear();
    for unit in &mut index.units {
        unit.body = Some(old.clone());
        unit.shim = None;
    }
    let error = migrated(&tree, &ledger, &index.to_turtle().unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("migrated ledger is"), "{error}");
    assert!(error.contains("maximum is"), "{error}");
    for unit in &mut index.units {
        if unit.symbol != "demo::home" {
            unit.body = None;
        }
    }
    let result = migrated(&tree, &ledger, &index.to_turtle().unwrap()).unwrap();
    assert_eq!(result.ledger.len(), ledger.len());
    assert_eq!(result.unmapped, []);
}
