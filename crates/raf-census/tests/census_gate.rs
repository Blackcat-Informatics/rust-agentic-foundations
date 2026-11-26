// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Production ledger evaluation over the source tree seam.

#[path = "support/structural.rs"]
mod structural;
use structural::fixture_gate;

#[path = "census_gate/configured_provider_targets.rs"]
mod configured_provider_targets;
#[path = "census_gate/enum_arms.rs"]
mod enum_arms;
use raf_census::detector_fixtures;
use raf_fixtures::layout;
#[path = "census_gate/native_provider.rs"]
mod native_provider;
#[path = "census_gate/numeric_bounds.rs"]
mod numeric_bounds;
#[path = "census_gate/ratchet.rs"]
mod ratchet;
#[path = "support/vocabulary.rs"]
mod vocabulary;

use std::collections::BTreeMap;

use raf_census::{
    Census, CensusOptions, DebtRecord, Error as CensusError, FileCtx, Fingerprint, Ledger,
    LedgerExtension, MAX_LEDGER_BYTES, MatchClass, MigrationInput, NORMAL_FORM, ReportError, Rule,
    RuleFixtures, RuleHit, migrate_index,
};
use raf_rdf::{Builder, Error as RdfError, Format, TermValue as Term, Triple, vocab};
use raf_source::MemoryTree;

const SOURCE: &str = "/// The one implementation.\npub fn home() -> u32 { 7 }\npub use home as alias;\npub fn copy() -> u32 { 7 + 1 }\n";

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let report = fixture_gate(".")
        .check_locked(&tree, &catalog, &local, &[])
        .unwrap();
    assert!(report.is_clean(), "{:?}", report.findings());
    assert_eq!(report.census().units().len(), 2);
    let missing = fixture_gate(".")
        .check_locked(&tree, &catalog, &local[..1], &[])
        .unwrap_err();
    assert!(missing.to_string().contains("association"), "{missing}");
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
            Term::simple_literal(pattern_fixture_selector(matcher)),
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let result = fixture_gate(".").check_tree(&tree(
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
    let reference = reference.finish().unwrap().to_text(Format::Turtle).unwrap();
    let reference =
        layout::assert_upgrade(raf_census::ReportKind::Ledger, &reference, Format::Turtle);
    let tree = tree(
        SOURCE,
        Builder::from_graph(&raf_rdf::Graph::from_text(&reference, Format::Turtle).unwrap()),
        0,
    );
    assert!(
        fixture_gate(".")
            .check_tree(&tree)
            .unwrap_err()
            .to_string()
            .contains("Cargo.lock")
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let set = Ledger::from_turtle(&exact).unwrap();
    let load_base = |graph: Builder| {
        Ledger::from_turtle(&graph.finish().unwrap().to_text(Format::Turtle).unwrap()).unwrap()
    };
    assert_eq!(load_base(base()).compose_jobs(&[&set]).unwrap(), exact);
    let mut with_scope = base();
    put(&mut with_scope, "", "excludeTestItems", Term::boolean(true));
    assert!(matches!(
        load_base(with_scope).compose_jobs(&[&set]),
        Err(CensusError::Report(ReportError::Ground(RdfError::OutputLimit { max_bytes })))
            if max_bytes == MAX_LEDGER_BYTES
    ));
}

fn pattern_fixture_selector(matcher: &str) -> &'static str {
    if matcher == "methodOn" {
        "demo::Receiver::run"
    } else {
        "demo::home"
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn ledger_vocabulary_matches_validated_graphs() {
    let mut builders = vec![
        variant("src/lib.rs"),
        cfg_ledger(2),
        provider_policy("native-alias", "native_alias::encode"),
        ledger_vocabulary_settings(),
    ];
    builders.push(ratchet::scoped_pattern(
        &[
            ("targetKind", "lib"),
            ("targetKind", "bin"),
            ("excludeTargetKind", "test"),
            ("includePath", "src"),
            ("excludePath", "src/generated"),
        ],
        true,
    ));
    builders.extend(
        [
            "call",
            "path",
            "macro",
            "typePath",
            "methodOn",
            "literalRegex",
            "tokens",
        ]
        .into_iter()
        .map(|matcher| declared_pattern(Some(matcher), true)),
    );
    builders.push(ratchet::ratcheted_declaration());
    builders.push(numeric_bounds::vocabulary());
    builders.push(enum_arms::enum_policy(&[(
        "Idle",
        "Reserved for the decoder",
    )]));
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
    put(&mut graph, "", "excludeTestItems", Term::boolean(true));
    put(
        &mut graph,
        "",
        "excludeDir",
        Term::simple_literal("generated"),
    );
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
    for (key, value) in [
        ("vectorPath", "vectors/add.json"),
        ("benchTarget", "demo:speed"),
        ("specRef", "RFC 0000"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    for (key, value) in [
        ("entryPoint", "demo::entry"),
        ("forbiddenName", "^copy$"),
        ("forbiddenFingerprint", "rule:copy-rule"),
        ("ratchetedRule", "copy-rule"),
    ] {
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
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
fn ledger_pattern_declarations_are_closed_referenced_and_not_executable_by_presence() {
    for matcher in [
        "call",
        "path",
        "macro",
        "typePath",
        "methodOn",
        "literalRegex",
        "tokens",
    ] {
        let graph = declared_pattern(Some(matcher), true).finish().unwrap();
        let turtle = graph.to_text(Format::Turtle).unwrap();
        let policy = Ledger::from_turtle(&turtle).unwrap();
        assert_eq!(policy.patterns()[0].id, "approved-call");
        let native = graph.to_text(Format::NTriples).unwrap();
        layout::assert_upgrade(raf_census::ReportKind::Ledger, &turtle, Format::Turtle);
        layout::assert_upgrade(raf_census::ReportKind::Ledger, &native, Format::NTriples);
        assert_eq!(Ledger::from_ntriples(&native).unwrap().patterns().len(), 1);
        let receiver_source;
        let source = if matcher == "methodOn" {
            receiver_source = format!("{SOURCE}\npub struct Receiver;");
            &receiver_source
        } else {
            SOURCE
        };
        let tree = tree(source, Builder::from_graph(&graph), 0);
        let result = fixture_gate(".").check_tree(&tree);
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
    ratchet::ratcheted_declarations_are_true_only_unique_and_held_by_enforced_jobs();
    ratchet::ratchet_rule_declarations_are_enabled_enforced_scoped_and_consistent();
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
            layout::assert_upgrade(
                raf_census::ReportKind::Ledger,
                &graph.to_text(Format::Turtle).unwrap(),
                Format::Turtle,
            );
            let policy = policy.unwrap();
            let provider = policy.jobs()[0].preferred_provider.as_ref().unwrap();
            assert_eq!(provider.package, "native-alias");
            assert_eq!(provider.entry_points, ["native_alias::encode"]);
            let error = fixture_gate(".")
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

/// Reusable jobs keep their complete owned policy. The base alone supplies
/// workspace options, and one declared identity has exactly one owner.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "the authored fixture must produce its planned canonical input"
)]
fn reusable_jobs_compose_into_the_native_ledger_without_overriding_policy() {
    let read = |graph: Builder| {
        let text = graph.finish().unwrap().to_text(Format::Turtle).unwrap();
        Ledger::from_turtle(&text).unwrap()
    };
    let mut workspace = base();
    put(
        &mut workspace,
        "",
        "minTokens",
        Term::typed_literal("30", vocab::INTEGER),
    );
    let workspace = read(workspace);
    let old_snapshot = workspace.graph().snapshot().unwrap();
    let literal_iri = "urn:raf:report:construction#foreign";
    let authored = read(provider_policy("purrdf-hash", "purrdf_hash::hex::encode"));
    let mut shared = Builder::from_graph(authored.graph());
    put(&mut shared, "#job", "extension", Term::iri("#ext"));
    put(
        &mut shared,
        "#ext",
        "type",
        Term::iri(format!("{}Extension", vocab::REPORT)),
    );
    let evidence = [
        ("urn:example:base", old_snapshot.as_str()),
        ("urn:example:foreign", literal_iri),
    ];
    for (predicate, object) in evidence {
        shared
            .insert(Triple {
                subject: Term::iri("#ext"),
                predicate: Term::iri(predicate),
                object: Term::iri(object),
            })
            .unwrap();
    }
    let shared = read(shared);
    let joined = workspace.compose_jobs(&[&shared]).unwrap();
    let joined = Ledger::from_turtle(&joined).unwrap();
    assert_eq!(joined.options().min_tokens, 30);
    assert_eq!(joined.jobs().len(), 1);
    let job = joined.jobs().first().unwrap();
    let provider = job.preferred_provider.as_ref().unwrap();
    assert_eq!(provider.package, "purrdf-hash");
    assert_eq!(provider.entry_points, ["purrdf_hash::hex::encode"]);
    assert!(joined.graph().get("#provider").is_some());
    let extension = joined.extension("#ext").unwrap().get("#ext").unwrap();
    for (predicate, expected) in evidence {
        assert_eq!(
            extension.values(predicate).next().unwrap().iri(),
            Some(expected)
        );
    }

    let overlay = workspace.compose_jobs(&[&workspace]).unwrap_err();
    assert!(
        overlay
            .to_string()
            .contains("declare workspace policy in the base ledger")
    );
    let collision = workspace.compose_jobs(&[&shared, &shared]).unwrap_err();
    assert!(collision.to_string().contains("ledger job set node"));

    let mut repeated = base();
    put(&mut repeated, "", "job", Term::iri("#other"));
    put(
        &mut repeated,
        "#other",
        "type",
        Term::iri(format!("{}LedgerJob", vocab::REPORT)),
    );
    for (key, value) in [
        ("jobId", "number"),
        ("summary", "A competing declaration"),
        ("homeSymbol", "demo::other"),
    ] {
        put(&mut repeated, "#other", key, Term::simple_literal(value));
    }
    put(&mut repeated, "#other", "enforced", Term::boolean(true));
    let repeated = read(repeated);
    assert!(matches!(
        workspace.compose_jobs(&[&shared, &repeated]),
        Err(CensusError::Report(ReportError::DuplicateJob(id)))
            if id == "number"
    ));
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

#[expect(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
    fixture_gate(".").check_locked(&tree, &catalog, &manifests, &[])
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
    fixture_gate(".").check_locked(&tree, &catalog, &manifests, &[])
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    native_provider::actual_registry_hasher();
    configured_provider_state_transitions();
}

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
fn configured_provider_state_transitions() {
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn lexical_pattern_graph(matcher: &str, expression: &str) -> Builder {
    let mut graph = declared_pattern(Some(matcher), true);
    for (node, key, value) in [
        (
            "#pattern",
            matcher,
            Term::simple_literal(pattern_fixture_selector(matcher)),
        ),
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
            let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn call_patterns_follow_written_imports_and_respect_parameters_locals_and_closures() {
    for (other, copies) in [
        ("pub fn other(){release(2);}", 1),
        ("pub fn other(){use std::mem::drop as inner; inner(2);}", 1),
        ("pub fn other(release:fn(u32)){release(2);}", 0),
        ("pub fn other(){let release=|_:u32|{}; release(2);}", 0),
        ("pub fn other(){let _=|release:fn(u32)| release(2);}", 0),
        ("pub fn other(){let release=release::<u32>; release(2);}", 0),
        (
            "pub fn other(){fn inner(){release(2);} inner(); let release=|_:u32|{}; let _=||release(2);}",
            1,
        ),
        (
            "pub fn other(){let release=|_:u32|{}; fn inner(release:fn(u32)){release(2);} let _=||release(2);}",
            0,
        ),
        (
            "pub fn other(){use std::mem::drop as imported; fn inner(){imported(2);} inner(); let imported=|_:u32|{}; let _=||imported(2);}",
            1,
        ),
        ("pub fn other(){let _=std::mem::drop::<u32>;}", 0),
    ] {
        let source = format!(
            "use std::mem::drop as release;\npub fn home(){{release(1);}}\npub use home as alias;\n{other}\n"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn type_patterns_follow_aliases_without_capturing_generic_or_module_shadows() {
    for (other, copies) in [
        ("pub fn other(_:Lock<u32>){}", 1),
        ("pub fn other<Lock>(_:Lock){}", 0),
        ("pub fn other<Other>(){fn inner(_:Lock<u32>){} }", 1),
        ("pub fn other<Lock>(){fn inner<Lock>(_:Lock){} }", 0),
        ("pub union Other<Lock: Copy> { field: Lock }", 0),
        (
            "pub union Other { field: std::mem::ManuallyDrop<Lock<u32>> }",
            1,
        ),
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn indexed_type_patterns_follow_written_aliases_at_their_uses() {
    let home = "
pub mod home {
    pub struct Banned;
    pub struct Other;
    pub type First = Banned;
    pub type Last = First;
    pub type Unrelated = Other;
    pub type Primitive = u8;
    pub type External = std::time::Duration;
    pub type Applied<T> = (T,);
    pub type Tuple = (u8,);
    #[cfg(unix)] pub type ConditionalOutside = Tuple;
}
pub use home as alias;
pub use home::Last as Exported;
use Exported as Through;
";
    for (outside, copies) in [
        ("pub fn outside(_: Through) {}", 1),
        ("pub fn outside<Through>(_: Through) {}", 0),
        (
            "pub fn outside(_: home::Unrelated, _: home::Primitive, _: home::External, _: home::Applied<u8>, _: home::ConditionalOutside) {}",
            0,
        ),
    ] {
        let source = format!("{outside}\n{home}");
        let report = fixture_gate(".")
            .check_tree(&lexical_pattern_tree(
                &source,
                "typePath",
                "demo::home::Banned",
            ))
            .unwrap();
        let job = &report.jobs()[0];
        assert_eq!(job.copies, copies, "{outside}: {:?}", report.findings());
        assert_eq!(report.is_clean(), copies == 0);
        if copies != 0 {
            assert!(job.matches.iter().any(|found| {
                found.symbol == "demo::outside" && found.class == MatchClass::Copy
            }));
        }
    }
    // The use precedes both alias declarations in the file, so a refusal at
    // line one proves the use queried the chain, rather than scanning its RHS.
    let ambiguous = "pub fn outside(_: home::End) {}
pub mod home {
    pub struct Banned;
    pub type End = Link;
    pub type Link = Choice;
    #[cfg(unix)] pub struct Choice;
    #[cfg(windows)] pub struct Choice;
}
pub use home as alias;
";
    let error = fixture_gate(".")
        .check_tree(&lexical_pattern_tree(
            ambiguous,
            "typePath",
            "demo::home::Banned",
        ))
        .unwrap_err();
    assert!(
        matches!(
            &error,
            raf_census::Error::CodePath { line: 1, source, .. }
                if matches!(source.as_ref(), raf_source::CodeResolveError::Resolve(raf_source::ResolveError::Ambiguous(_)))
        ),
        "{error}"
    );
}

const WRITTEN_RECEIVER_SOURCE: &str = r#"
pub mod home {
    pub struct Banned;
    pub struct Other;
    pub struct TypeOnly { field: u8 }
    pub type First = Banned;
    pub type Last = First;
    impl Banned { pub fn run(&mut self) -> bool { true } }
    impl Other { pub fn run(&self) -> bool { true } }
    pub fn exported_value() -> u8 { 0 }
    pub fn sanctioned(mut value: Banned) { value.run(); }
}
pub use home as alias;
pub use home::Last as Selected;
pub use home::Other as Choice;
mod imported { pub fn value() -> u8 { 0 } }
mod type_glob {
    pub use crate::home::{Banned as Choice, Banned as FromGlob, TypeOnly as value};
}
mod wildcard_routes {
    pub mod empty {}
    pub mod active {
        pub mod incoming { pub use crate::home::Banned as Through; }
    }
    pub mod optional {
        pub mod incoming { pub use crate::home::Other as Through; }
    }
    pub mod unrelated { pub mod empty_provider {} }
}
pub use wildcard_routes::empty as route;
#[allow(dead_code)]
pub fn outside(mut value: Selected, other: home::Other) {
    value
        .run(); // hit
    other.run();
    { use home::Banned as Local; let mut local: Local = home::Banned; local.run(); } // hit
    {
        use { ::std::fmt::Debug as D, crate::home::Banned as Grouped };
        let mut local: Grouped = home::Banned;
        local.run(); // hit
    }
    { mod home { pub struct Banned; } use home::Banned as Local; let local: Local = home::Banned; local.run(); }
    { use opaque::*; use home::Banned as Local; let local: Local = home::Banned; local.run(); }
    { use home::Other as Selected; value.run(); } // hit
    { use crate::home::exported_value as value; value.run(); }
    { use crate::home::TypeOnly as value; value.run(); } // hit
    { #[cfg(any())] use crate::home::Banned as Choice; let local: Choice = home::Other; local.run(); }
    { #[cfg(any())] use { ::std::fmt::Debug as D, crate::home::Banned as Choice }; let local: Choice = home::Other; local.run(); }
    { #[cfg(any())] use crate::type_glob::*; let local: Choice = home::Other; local.run(); }
    { #[allow(unused_imports)] use crate::home::Banned as Choice; let mut local: Choice = home::Banned; local.run(); } // hit
    { use crate::type_glob::*; let mut local: FromGlob = home::Banned; local.run(); } // hit
    { #[cfg(feature = "alternate")] use crate::home::Other as Selected; let local: Selected = home::Banned; local.run(); }
    { #[cfg(feature = "alternate")] use crate::home::exported_value as Selected; let mut local: Selected = home::Banned; local.run(); } // hit
    { #[cfg(feature = "alternate")] use crate::type_glob::*; let mut local: FromGlob = home::Banned; local.run(); }
    {
        use crate::wildcard_routes::active::*;
        { use incoming::*; let mut local: Through = home::Banned; local.run(); } // hit
    }
    {
        #[cfg(feature = "alternate")] use crate::wildcard_routes::optional as route;
        use route::*;
        use crate::wildcard_routes::active::*;
        { use incoming::*; let mut local: Through = home::Banned; local.run(); }
    }
    {
        use crate::wildcard_routes::active::*;
        use incoming::*;
        #[cfg(feature = "alternate")] use empty_provider::*;
        use crate::wildcard_routes::unrelated::*;
        let mut local: Through = home::Banned;
        local.run(); // hit
    }
    {
        use crate::wildcard_routes::unrelated::*;
        #[cfg(feature = "alternate")] use empty_provider::*;
        use crate::wildcard_routes::active::*;
        use incoming::*;
        let mut local: Through = home::Banned;
        local.run(); // hit
    }
    { #[cfg(any())] use crate::home::exported_value as value; value.run(); } // hit
    { #[cfg(any())] use { ::std::fmt::Debug as D, crate::home::exported_value as value }; value.run(); } // hit
    { #[cfg(feature = "alternate")] use crate::home::exported_value as value; value.run(); }
    { #[cfg(feature = "alternate")] use { ::std::fmt::Debug as D, crate::home::exported_value as value }; value.run(); }
    { #[cfg(feature = "alternate")] use crate::home::TypeOnly as value; value.run(); } // hit
    { use crate::imported::*; value.run(); }
    { use crate::home::*; value.run(); } // hit
    { #[cfg(any())] use crate::imported::*; value.run(); } // hit
    { #[cfg(any())] use { ::std::fmt::Debug as D, crate::imported::* }; value.run(); } // hit
    { #[cfg(feature = "alternate")] use crate::imported::*; value.run(); }
    { #[cfg(feature = "alternate")] use crate::type_glob::*; value.run(); } // hit
    { use opaque::*; value.run(); }
    { use opaque::*; let mut value: crate::home::Banned = home::Banned; value.run(); } // hit
    let mut capture = || value.run(); // hit
    let _ = capture();
    let _ = |mut value: Selected| value.run(); // hit
    { let ref value: Selected = home::Banned; value.run(); }
    { let (value,): (home::Other,) = (home::Other,); value.run(); }
    { let value = home::Other; value.run(); }
    { #[cfg(any())] let value = home::Other; value.run(); } // hit
    { let mut value: Selected = home::Banned; #[cfg(any())] let value = home::Other; value.run(); } // hit
    #[cfg(any())] let disabled = value.run();
    { use opaque::*; let mut value: Selected = home::Banned; value.run(); }
    const { value.run(); }
    fn nested() { value.run(); }
    match () { () if value.run() => (), _ => () } // hit
    match Some(home::Other) { Some(value) if value.run() => (), _ => () }
    {
        let seed: Selected = home::Banned;
        let mut moved = seed;
        moved.run(); // hit
        let mut chained = ((moved));
        chained.run(); // hit
        let mut chained = (chained);
        (chained).run(); // hit
    }
    { let moved = (other); moved.run(); }
    { let mut moved: _ = value; moved.run(); }
    { let value = &mut value; let moved = (value); moved.run(); }
    { use crate::home::exported_value as value; let moved = (value); moved.run(); }
    { use opaque::*; let moved = (value); moved.run(); }
    { let mut moved = (#[cfg(feature = "alternate")] (value)); moved.run(); }
    let value: home::Other = { value.run(); home::Other }; // hit
    value.run();
}
"#;

const CONSTRUCTOR_RECEIVER_SOURCE: &str = r#"
pub mod home {
    pub struct Banned {}
    pub type Alias = Banned;
    impl Banned { pub fn run(&mut self) {} }
    pub fn sanctioned(mut value: Banned) { value.run(); }
}
pub mod other {
    pub struct Banned {}
    impl Banned { pub fn run(&mut self) {} }
}
pub use home as alias;
pub use home::Alias as Reexport;
pub use other::Banned as Choice;
fn factory() -> home::Banned { home::Banned {} }

pub fn outside() {
    use crate::Reexport as Constructor;
    #[allow(non_snake_case)]
    fn Constructor() {}
    let mut annotated: Constructor = factory();
    annotated.run(); // hit
    ((annotated))
        .run(); // hit
    let mut value = Constructor {};
    value.run(); // hit
    let mut value = Constructor { ..{ value.run(); Constructor {} } }; // hit
    value.run(); // hit
    { let mut value = other::Banned {}; value.run(); }
    { let mut value = ((Constructor {})); (value).run(); } // hit
    { let mut value = (other::Banned {}); (value).run(); }
    { let mut value = (#[cfg(feature = "alternate")] (Constructor {})); (value).run(); }
    { let mut value = Constructor {}; (#[cfg(feature = "alternate")] (value)).run(); }
    { let mut value = factory(); value.run(); }
    { let mut value: _ = Constructor {}; value.run(); }
    { let ref mut value = Constructor {}; value.run(); }
    { let value = &mut Constructor {}; value.run(); }
    {
        #[cfg(any())] use crate::home::Banned as Choice;
        let mut value = Choice {};
        value.run();
    }
    {
        #[cfg(any())] use crate::other::Banned as Constructor;
        let mut value = Constructor {};
        value.run(); // hit
    }
    {
        #[cfg(feature = "alternate")] use crate::other::Banned as Constructor;
        let mut value = Constructor {};
        value.run();
    }
    { #[cfg(any())] let mut value = other::Banned {}; value.run(); } // hit
    { let mut value = other::Banned {}; #[cfg(feature = "alternate")] let mut value = Constructor {}; value.run(); }
    let _ = || value.run(); // hit
}
"#;

const TUPLE_RECEIVER_SOURCE: &str = r"
pub mod home {
    pub struct Banned(pub u8);
    pub struct Other(pub u8);
    impl Banned { pub fn run(&self) {} }
    impl Other { pub fn run(&self) {} }
    pub fn sanctioned(value: Banned) { value.run(); }
}
pub use home as alias;
pub use home::Banned as Reexport;

pub fn outside() {
    use crate::Reexport as make;
    let typed: home::Banned = home::Banned { 0: 1 };
    typed.run(); // hit
    let braced = home::Banned { 0: 2 };
    braced.run();
    let inferred = make(3);
    inferred.run(); // hit
    let inferred = (((make))(3));
    ((inferred)).run(); // hit
    let other = home::Other(4);
    other.run();
    {
        fn make(value: u8) -> home::Other { home::Other(value) }
        let shadowed = ((make)(5));
        (shadowed).run();
    }
    let callable = make;
    let indirect = ((callable)(6));
    (indirect).run();
}
";

const UNIT_RECEIVER_SOURCE: &str = r"
pub mod home {
    pub struct Banned;
    pub struct Other;
    impl Banned { pub fn run(&self) {} }
    impl Other { pub fn run(&self) {} }
    pub fn sanctioned(value: Banned) { value.run(); }
}
pub use home as alias;
pub use home::Banned as Reexport;

pub fn outside() {
    use crate::Reexport as BUILD;
    let typed: BUILD = BUILD;
    typed.run(); // hit
    let inferred = BUILD;
    inferred.run(); // hit
    let inferred = ((BUILD));
    (inferred).run(); // hit
    let other = home::Other;
    other.run();
    {
        const BUILD: home::Other = home::Other;
        let shadowed = (BUILD);
        (shadowed).run();
    }
    {
        let build = home::Other;
        {
            use crate::Reexport as build;
            let from_import = (build);
            from_import.run(); // hit
        }
    }
}
";

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn written_receiver_methods_keep_declaration_identity_and_runtime_shadowing() {
    use std::collections::BTreeSet;

    // An alias retains its source's nominal identity, subject to the same
    // shadowing rules as written types and constructor initializers.
    for source in [
        WRITTEN_RECEIVER_SOURCE,
        CONSTRUCTOR_RECEIVER_SOURCE,
        TUPLE_RECEIVER_SOURCE,
        UNIT_RECEIVER_SOURCE,
    ] {
        let report = fixture_gate(".")
            .check_tree(&lexical_pattern_tree(
                source,
                "methodOn",
                "demo::home::Banned::run",
            ))
            .unwrap();
        let expected: BTreeSet<_> = source
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains("// hit"))
            .map(|(line, _)| line + 1)
            .collect();
        let mixed = mixed_policy_keeps_type_path_findings(source);
        for (report, rule) in [(&report, "approved-call"), (&mixed, "written-receiver")] {
            let rule = format!(", rule {rule}:");
            let actual: BTreeSet<_> = report
                .findings()
                .iter()
                .filter(|finding| finding.contains(&rule))
                .filter_map(|finding| finding.strip_prefix("src/lib.rs:"))
                .map(|finding| finding.split(':').next().unwrap().parse::<usize>().unwrap())
                .collect();
            assert_eq!(actual, expected, "{:?}", report.findings());
        }
        assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn mixed_policy_keeps_type_path_findings(source: &str) -> raf_census::GateReport {
    let graph = lexical_pattern_graph("typePath", "demo::home::Banned")
        .finish()
        .unwrap();
    let gate = fixture_gate(".");
    let ordinary = gate
        .check_tree(&tree(source, Builder::from_graph(&graph), 0))
        .unwrap();
    let mut mixed = Builder::from_graph(&graph);
    for (node, key, value) in [
        ("", "pattern", Term::iri("#method")),
        (
            "#job",
            "forbiddenPattern",
            Term::simple_literal("written-receiver"),
        ),
        (
            "#method",
            "type",
            Term::iri(format!("{}Pattern", vocab::REPORT)),
        ),
        (
            "#method",
            "patternId",
            Term::simple_literal("written-receiver"),
        ),
        (
            "#method",
            "summary",
            Term::simple_literal("Written receiver call"),
        ),
        (
            "#method",
            "methodOn",
            Term::simple_literal("demo::home::Banned::run"),
        ),
    ] {
        put(&mut mixed, node, key, value);
    }
    let mixed = gate.check_tree(&tree(source, mixed, 0)).unwrap();
    let ordinary_findings = ordinary
        .findings()
        .iter()
        .filter(|finding| finding.contains("rule approved-call:"));
    assert!(
        ordinary_findings.clone().next().is_some(),
        "{:?}",
        ordinary.findings()
    );
    assert!(
        ordinary_findings.eq(mixed
            .findings()
            .iter()
            .filter(|finding| finding.contains("rule approved-call:"))),
        "ordinary {:?}, mixed {:?}",
        ordinary.findings(),
        mixed.findings()
    );
    mixed
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn written_receiver_method_scope_keeps_the_matching_reach_of_a_shared_file() {
    // A non-shipping reach cannot contribute its receiver proof to a
    // shipping reach, and two proofs still count one physical location.
    for (shipping_only, shipping, tests, copies) in [
        (true, "Other", "Banned", 0),
        (true, "Banned", "Other", 1),
        (true, "Banned", "Banned", 1),
        (false, "Banned", "Banned", 1),
    ] {
        let mut ledger = lexical_pattern_graph("methodOn", "demo::home::Banned::run");
        if shipping_only {
            put(&mut ledger, "#pattern", "shippingOnly", Term::boolean(true));
        }
        let graph = ledger.finish().unwrap().to_text(Format::Turtle).unwrap();
        let tree: MemoryTree = [
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n".to_owned(),
            ),
            ("helpers-ledger.ttl", graph),
            (
                "src/lib.rs",
                format!(
                    r#"
pub mod home {{
    pub struct Banned {{}}
    pub struct Other {{}}
    pub fn sanctioned(value: Banned) {{ value.run(); }}
}}
pub use home as alias;
pub mod shipping {{
    pub use crate::home::{shipping} as Choice;
    #[path = "../shared.rs"] pub mod body;
}}
#[cfg(test)] pub mod tests {{
    pub use crate::home::{tests} as Choice;
    #[path = "../shared.rs"] pub mod body;
}}
"#
                ),
            ),
            (
                "src/shared.rs",
                "pub fn copy(value: super::Choice) {
    value.run();
    let inferred = (super::Choice {});
    (inferred).run();
    let inferred = ((inferred));
    inferred.run();
}"
                .to_owned(),
            ),
        ]
        .into_iter()
        .collect();
        let report = fixture_gate(".").check_tree(&tree).unwrap();
        assert_eq!(
            report.jobs()[0].copies,
            copies,
            "{shipping} {tests}: {:?}",
            report.findings()
        );
        for position in ["shared.rs:2: job", "shared.rs:4: job", "shared.rs:6: job"] {
            assert_eq!(
                report
                    .findings()
                    .iter()
                    .filter(|line| line.contains(position))
                    .count(),
                copies,
                "{shipping} {tests}: {:?}",
                report.findings()
            );
        }
    }
}

#[test]
#[expect(
    clippy::expect_used,
    reason = "fixture prerequisites and required observations carry their assertion messages"
)]
fn written_receiver_alias_keeps_a_native_refusal_at_the_first_matching_method() {
    // Moving a receiver never reports its deferred Source refusal. The
    // first matching method on the new binding reports the original cause.
    let source = r"
pub mod home {
    pub struct Banned;
    #[cfg(unix)] pub type Conditional = Banned;
    pub fn sanctioned(value: Banned) { value.run(); }
}
pub use home as alias;
pub fn outside(value: home::Conditional) {
    let moved = ((value));
    let moved = moved;
    (moved)
        .run(); // selected
    value.run(); // selected
}
";
    let first_method = source
        .lines()
        .position(|line| line.contains("// selected"))
        .expect("source has a matching method")
        + 1;
    let gate = fixture_gate(".");
    let check = |source| {
        gate.check_tree(&lexical_pattern_tree(
            source,
            "methodOn",
            "demo::home::Banned::run",
        ))
    };
    let quiet = source.replace("run(); // selected", "observe();");
    assert!(
        check(&quiet)
            .expect("no matching outside method")
            .is_clean()
    );
    let error = check(source).expect_err("the moved receiver's method reports its source refusal");
    assert!(
        matches!(
            &error,
            raf_census::Error::CodePath { line, source, .. }
                if *line == first_method
                    && matches!(source.as_ref(), raf_source::CodeResolveError::AliasMember { .. })
        ),
        "{error}"
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn written_receiver_method_selector_refuses_non_nominal_and_malformed_paths() {
    // The ledger needs a specific indexed nominal receiver before it can
    // select the method. Written receiver aliases are proved at their uses.
    let source = "pub mod home { pub struct Banned; pub type Alias = Banned; pub fn clean(value: Banned) { value.run(); } } pub use home as alias;";
    let gate = fixture_gate(".");
    assert!(
        gate.check_tree(&lexical_pattern_tree(
            source,
            "methodOn",
            "::demo::home::Banned::r#run"
        ))
        .unwrap()
        .is_clean()
    );
    for (selector, message) in [
        (
            "demo::home::Alias::run",
            "must resolve to one indexed first-party struct, enum or union",
        ),
        (
            "demo::home::Banned::run()",
            "methodOn requires an indexed type path and method",
        ),
        (
            "demo::home::Banned::run/**/",
            "methodOn requires an indexed type path and method",
        ),
    ] {
        let error = gate
            .check_tree(&lexical_pattern_tree(source, "methodOn", selector))
            .unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn written_type_alias_hits_share_physical_sites_and_respect_shipping_scope() {
    let mut ledger = lexical_pattern_graph("typePath", "demo::home::Banned");
    put(&mut ledger, "#pattern", "shippingOnly", Term::boolean(true));
    let graph = ledger.finish().unwrap().to_text(Format::Turtle).unwrap();
    let mut tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n".into(),
        ),
        ("helpers-ledger.ttl", graph),
        (
            "src/lib.rs",
            "
pub mod home {
    pub struct Banned;
    pub type Through = Banned;
    #[cfg(unix)] pub type Conditional = Banned;
}
pub use home as alias;
macro_rules! observe { ($e:expr) => {} }
#[cfg(test)] observe!({ let _: home::Conditional; });
#[path=\"shared.rs\"] mod first;
#[path=\"shared.rs\"] mod second;
"
            .into(),
        ),
        (
            "src/shared.rs",
            "use crate::home::Through; pub fn copy(_: Through) {}".into(),
        ),
    ]
    .into_iter()
    .collect();
    let report = fixture_gate(".").check_tree(&tree).unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{:?}", report.findings());

    // The same parsed macro argument is a refused conditional alias when the
    // invocation is in scope, even though its enclosing item is the crate.
    let unscoped = lexical_pattern_graph("typePath", "demo::home::Banned")
        .finish()
        .unwrap()
        .to_text(Format::Turtle)
        .unwrap();
    tree.replace(
        &raf_source::RelPath::new("helpers-ledger.ttl").unwrap(),
        unscoped,
    )
    .unwrap();
    let error = fixture_gate(".").check_tree(&tree).unwrap_err();
    assert!(
        matches!(
            &error,
            raf_census::Error::CodePath { line: 9, source, .. }
                if matches!(source.as_ref(), raf_source::CodeResolveError::AliasMember { .. })
        ),
        "{error}"
    );
}

#[test]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
fn macro_type_hits_use_the_scope_of_the_reach_that_resolves_the_type() {
    // One invocation can resolve differently under distinct targets or under
    // distinct modules in a single target. A proof must use its own reach.
    for (scope, separate_target) in [("targetKind", true), ("shippingOnly", false)] {
        let mut ledger = lexical_pattern_graph("typePath", "demo::home::Banned");
        let value = if separate_target {
            Term::simple_literal("lib")
        } else {
            Term::boolean(true)
        };
        put(&mut ledger, "#pattern", scope, value);
        let graph = ledger.finish().unwrap().to_text(Format::Turtle).unwrap();
        for (shipping, test, copies) in [
            ("Other", "Through", 0),
            ("Through", "Other", 1),
            ("Other", "Banned", 0),
            ("Banned", "Other", 1),
            ("Other", "Conditional", 0),
        ] {
            let other_mount = if separate_target {
                String::new()
            } else {
                format!(
                    "pub mod test {{ use crate::home::{test} as Choice; #[cfg(test)] #[path = \"../shared.rs\"] mod shared; }}"
                )
            };
            let entries = [
                (
                    "Cargo.toml",
                    "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n".to_owned(),
                ),
                ("helpers-ledger.ttl", graph.clone()),
                (
                    "src/lib.rs",
                    format!(
                        "
pub mod home {{
    pub struct Banned;
    pub struct Other;
    pub type Through = Banned;
    #[cfg(unix)] pub type Conditional = Banned;
}}
pub use home as alias;
pub mod shipping {{
    use crate::home::{shipping} as Choice;
    #[path = \"../shared.rs\"] mod shared;
}}
{other_mount}
"
                    ),
                ),
                (
                    "src/shared.rs",
                    "pub mod nested { observe!({ let _: super::super::Choice; }); }".to_owned(),
                ),
            ];
            let integration = separate_target.then(|| {
                (
                    "tests/it.rs",
                    format!(
                        "
use demo::home::{test} as Choice;
#[path = \"../src/shared.rs\"] mod shared;
"
                    ),
                )
            });
            let tree: MemoryTree = entries.into_iter().chain(integration).collect();
            let report = fixture_gate(".")
                .check_tree(&tree)
                .unwrap_or_else(|error| panic!("{scope} {shipping} {test}: {error}"));
            assert_eq!(
                report.jobs()[0].copies,
                copies,
                "{scope} {shipping} {test}: {:?}",
                report.findings()
            );
        }
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn workspace_call_patterns_follow_cross_crate_reexports_through_actual_source_sites() {
    selected_reference_patterns_follow_actual_exports();
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
        let report = fixture_gate(".").check_tree(&files).unwrap();
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn selected_reference_patterns_follow_actual_exports() {
    let imported = "mod facade {pub use renamed::exported as alias;} use facade::alias as through;";
    for (matcher, imports, body, matched) in [
        ("call", imported, "through();", true),
        ("call", imported, "let through=||{}; through();", false),
        ("path", imported, "let _=through as fn();", true),
        (
            "path",
            "mod facade {pub fn alias(){}} use facade::alias as through;",
            "let _=through as fn();",
            false,
        ),
    ] {
        let tree =
            selected_reference_pattern_tree(matcher, "native_helper::provided", imports, body);
        let report = fixture_gate(".").check_tree(&tree).unwrap();
        assert_eq!(
            report.jobs()[0]
                .matches
                .iter()
                .any(|hit| hit.symbol == "demo::other"),
            matched,
            "{matcher}: {body}: {:?}",
            report.findings()
        );
        assert_eq!(report.is_clean(), !matched, "{:?}", report.findings());
        assert!(
            report
                .census()
                .units()
                .iter()
                .all(|unit| unit.package == "demo")
        );
        assert!(report.debt().groups().is_empty());
    }
    let missing =
        selected_reference_pattern_tree("call", "native_helper::missing", imported, "through();");
    let error = fixture_gate(".").check_tree(&missing).unwrap_err();
    assert!(
        error.to_string().contains("selected reference target"),
        "{error}"
    );
    selected_reference_alias_admission();
}

#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
fn selected_reference_alias_admission() {
    for visibility in ["", "pub "] {
        for name in ["provided", "missing"] {
            let imports = format!("{visibility}use renamed::{name} as blocked;");
            let tree =
                selected_reference_pattern_tree("call", "demo::blocked", &imports, "blocked();");
            let result = fixture_gate(".").check_tree(&tree);
            if name == "missing" {
                let Err(error) = result else {
                    panic!("an unresolved selected-reference alias must refuse");
                };
                assert!(
                    error.to_string().contains("selected reference target"),
                    "{error}"
                );
            } else {
                let report = result.unwrap();
                assert!(
                    report.jobs()[0]
                        .matches
                        .iter()
                        .any(|hit| hit.symbol == "demo::other")
                );
            }
        }
    }
    let external = selected_reference_pattern_tree(
        "call",
        "std::mem::drop",
        "use std::mem::drop as blocked;",
        "blocked(7);",
    );
    let report = fixture_gate(".").check_tree(&external).unwrap();
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|hit| hit.symbol == "demo::other")
    );
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn selected_reference_pattern_tree(
    matcher: &str,
    expression: &str,
    imports: &str,
    body: &str,
) -> MemoryTree {
    let mut graph = lexical_pattern_graph(matcher, expression);
    put(&mut graph, "", "reference", Term::simple_literal("helper"));
    [
        ("Cargo.toml", "[workspace]\nmembers=['app']\nexclude=['helper']\nresolver='2'\n".to_owned()),
        ("app/Cargo.toml", "[package]\nname='demo'\nversion='1.0.0'\nedition='2021'\n[dependencies]\nrenamed={package='helper',path='../helper',version='1'}\n".into()),
        ("helper/Cargo.toml", "[package]\nname='helper'\nversion='1.0.0'\nedition='2021'\n[lib]\nname='native_helper'\n".into()),
        ("helper/src/lib.rs", "pub fn provided(){} pub use provided as exported;".into()),
        ("app/src/lib.rs", format!("{imports} pub fn home()->u32{{renamed::provided(); let _=renamed::provided as fn(); 0}} pub use home as alias; pub fn other(){{{body}}}")),
        ("Cargo.lock", "version=4\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['helper']\n[[package]]\nname='helper'\nversion='1.0.0'\n".into()),
        ("helpers-ledger.ttl", graph.finish().unwrap().to_text(Format::Turtle).unwrap()),
    ].into_iter().collect()
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn block_import_patterns_use_the_import_binding_instead_of_a_shadowed_module_function() {
    for (pattern, home, copies) in [
        ("demo::drop_it", "drop_it();", 0),
        ("std::mem::drop", "std::mem::drop(0);", 1),
    ] {
        let source = format!(
            "pub fn drop_it(){{}} pub fn home(){{{home}}} pub use home as alias; pub fn other(){{use std::mem::drop as drop_it; drop_it(1);}}"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn constant_pattern_identifiers_keep_their_actual_reference_in_match_arms() {
    for (arm, copies) in [
        ("BANNED => {let _=BANNED;}, _=>{}", 1),
        ("binding => {let _=binding;}", 0),
    ] {
        let source = format!(
            "pub const BANNED:u8=1; pub fn home(){{let _=BANNED;}} pub use home as alias; pub fn other(value:u8){{match value {{{arm}}}}}"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn general_path_patterns_include_grouped_renamed_imports_in_their_actual_module_scope() {
    for (import, copies) in [
        ("use std::mem::{self, drop as outside};", 1),
        ("use std::mem::{self, forget as outside};", 0),
    ] {
        let source = format!(
            "pub mod home {{pub use std::mem::drop as approved; pub fn run(){{approved(0);}}}} pub use home as alias; {import}"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn namespace_dependencies_refuse_cycles_beside_bounded_transitive_chains() {
    let source = "pub fn home(){std::mem::drop(0);} pub use home as alias; pub fn other(){use b as a; use a as b; a(1);}";
    let error = fixture_gate(".")
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
    let report = fixture_gate(".")
        .check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1);
    assert!(!report.is_clean());
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn transitive_block_imports_classify_their_expanded_namespace() {
    for (prefix, copies) in [("std::mem", 1), ("crate::y", 0)] {
        let source = format!(
            "pub mod x{{pub struct drop{{field:u8}}}} pub mod y{{pub fn drop(_:u8){{}}}} pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{use {prefix} as x; use x::drop as d; d(1);}}"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".").check_tree(&files).unwrap();
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn competing_import_namespaces_refuse_beside_distinct_namespace_imports() {
    for (first, refused) in [("pub fn chosen(){}", true), ("pub mod chosen{}", false)] {
        let source = format!(
            "pub mod first{{{first}}} pub mod second{{pub fn chosen(){{}}}} pub fn home(){{std::mem::drop(0);}} pub use home as alias; pub fn other(){{use crate::first::chosen; use crate::second::chosen; chosen();}}"
        );
        let result =
            fixture_gate(".").check_tree(&lexical_pattern_tree(&source, "call", "std::mem::drop"));
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn anonymous_qualified_receivers_refuse_beside_supported_named_receivers() {
    for (call, refused) in [("<[u8]>::len(&[])", true), ("<Named>::len()", false)] {
        let source = format!(
            "pub fn len()->usize{{0}} pub struct Named; impl Named{{pub fn len()->usize{{0}}}} pub fn home(){{let _=len();}} pub use home as alias; pub fn other(){{let _={call};}}"
        );
        let result = fixture_gate(".").check_tree(&lexical_pattern_tree(&source, "call", "len"));
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let result = fixture_gate(".").check_tree(&files);
        let visibility = raf_census::visibility(&files, &raf_source::RelPath::root());
        if refused {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("changes binding meaning across source reaches"),
                "{error}"
            );
            assert!(
                visibility
                    .unwrap_err()
                    .to_string()
                    .contains("changes binding meaning across source reaches")
            );
        } else {
            let report = result.unwrap();
            assert!(report.is_clean(), "{:?}", report.findings());
            assert!(visibility.is_ok(), "{visibility:?}");
        }
    }
    unit_variant_reaches_are_classified_as_path_patterns();
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn unit_variant_reaches_are_classified_as_path_patterns() {
    for (variant, refused) in [("Ready", false), ("Replaced", true)] {
        let enum_source = format!(
            "pub enum Provider {{ {variant}, Busy }} pub fn sample() -> Provider {{ Provider::Busy }} #[path=\"shared.rs\"] mod shared;"
        );
        let files = MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "mod a; mod b;"),
            ("src/a.rs", enum_source.as_str()),
            (
                "src/b.rs",
                "pub mod Provider { pub const Ready: u8 = 1; } pub fn sample() -> u8 { 1 } #[path=\"shared.rs\"] mod shared;",
            ),
            (
                "src/shared.rs",
                "fn check() -> bool { use super::Provider::*; match super::sample() { Ready => true, _ => false } }",
            ),
        ]);
        let visibility = raf_census::visibility(&files, &raf_source::RelPath::root());
        if refused {
            assert!(
                visibility
                    .unwrap_err()
                    .to_string()
                    .contains("changes binding meaning across source reaches")
            );
        } else {
            assert!(visibility.is_ok(), "{visibility:?}");
        }
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn lexical_patterns_reuse_local_normalization_and_keep_parameter_context() {
    let source = "/// Home.\npub fn home() { let original = 1; let _ = original; }\npub use home as alias;\npub fn copy() { let renamed = 2; let _ = renamed; }\n";
    let report = fixture_gate(".")
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
    let report = fixture_gate(".")
        .check_tree(&lexical_pattern_tree(source, "tokens", "consume($_)"))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 1);
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn lexical_pattern_module_homes_own_source_subjects_without_sanctioning_the_file_root() {
    let source = "/// Home.\npub mod home { #[doc=\"blocked\"] pub struct Approved; }\npub use home as alias;\n#[doc=\"blocked\"] pub struct Copy;\n";
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn lexical_pattern_module_homes_include_macros_exported_from_their_actual_source_scope() {
    for (other, copies) in [("blocked", 1), ("allowed", 0)] {
        let source = format!(
            "pub mod home {{ #[macro_export] macro_rules! approved {{()=>{{\"blocked\"}}}} }}\npub use home as alias;\npub fn other()->&'static str {{\"{other}\"}}\n"
        );
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".").check_tree(&tree).unwrap();
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
        let error = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn lexical_patterns_do_not_mistake_a_file_root_for_its_first_written_unit() {
    let source = "pub fn home()->&'static str {\"blocked\"}\npub use home as alias;\nmacro_rules! check { ($value:expr)=>{} }\ncheck!(\"blocked\");\n";
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn token_patterns_include_file_level_macro_arguments_without_renaming_item_names() {
    let source = "pub fn home() { check!(1); }\npub use home as alias;\nmacro_rules! check { ($value:expr)=>{} }\ncheck!({2 + 3});\n";
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn token_patterns_normalize_body_local_declarations_without_renaming_module_items() {
    let source = "pub fn home(){fn local(){}}\npub use home as alias;\npub fn copy(){fn renamed(){}}\npub fn banned(){}\n";
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn token_patterns_match_only_the_files_their_scope_admits() {
    // A file outside includePath is neither normalized nor charged to the matching
    // grant. With a grant that fits only the admitted file, the scoped rule
    // succeeds and its unscoped neighbour is refused at the padded file.
    let padding = "pub fn pad(){let _=(1,2,3,4,5,6,7,8);}\n".repeat(200);
    let run = |graph: Builder| -> Result<Vec<String>, String> {
        let ledger = graph.finish().unwrap().to_text(Format::Turtle).unwrap();
        let tree: MemoryTree = [
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n".to_owned(),
            ),
            (
                "src/lib.rs",
                "pub mod admitted;\npub mod other;\npub fn home(){}\npub use home as alias;\n"
                    .to_owned(),
            ),
            ("src/admitted.rs", "pub fn a(){check!(1);}\n".to_owned()),
            (
                "src/other.rs",
                format!("pub fn b(){{check!(2);}}\n{padding}"),
            ),
            ("helpers-ledger.ttl", ledger),
        ]
        .into_iter()
        .collect();
        let report = fixture_gate(".")
            .matching_limits(purrdf_jsonschema::ecma::MatchLimits {
                steps: 3_000,
                states: 64,
            })
            .check_tree(&tree)
            .map_err(|error| error.to_string())?;
        Ok(report.jobs()[0]
            .matches
            .iter()
            .map(|found| found.symbol.clone())
            .collect())
    };
    let mut scoped = lexical_pattern_graph("tokens", "check!($_)");
    put(
        &mut scoped,
        "#pattern",
        "includePath",
        Term::simple_literal("src/admitted.rs"),
    );
    assert_eq!(run(scoped), Ok(vec!["demo::admitted::a".to_owned()]));
    let refused = run(lexical_pattern_graph("tokens", "check!($_)")).unwrap_err();
    assert!(refused.contains("rule:approved-call"), "{refused}");
    assert!(
        refused.contains("matching stopped at src/other.rs"),
        "{refused}"
    );
    assert!(
        refused.contains("unfinished admitted files: src/other.rs"),
        "{refused}"
    );
}

#[test]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
)]
fn lexical_patterns_share_an_explicit_native_budget_and_release_only_their_own_spans() {
    let retained: syn::File = syn::parse_str("fn retained() {}").unwrap();
    let source = "/// Home.\npub fn home() { consume(1); }\npub use home as alias;\nfn consume<T>(value:T)->T { value }\n";
    let tree = lexical_pattern_tree(source, "tokens", "consume($_)");
    let refused = fixture_gate(".")
        .matching_limits(purrdf_jsonschema::ecma::MatchLimits {
            steps: 0,
            states: 64,
        })
        .check_tree(&tree)
        .unwrap_err();
    assert!(refused.to_string().contains("step limit"), "{refused}");
    assert!(
        refused
            .to_string()
            .contains("matching stopped at src/lib.rs"),
        "{refused}"
    );
    assert!(
        refused.to_string().contains("rule:approved-call"),
        "{refused}"
    );
    assert!(fixture_gate(".").check_tree(&tree).unwrap().is_clean());
    let syn::Item::Fn(item) = &retained.items[0] else {
        panic!("retained function")
    };
    assert_eq!(item.sig.ident.to_string(), "retained");
    let empty = lexical_pattern_tree(source, "tokens", "/* empty */");
    assert!(
        fixture_gate(".")
            .check_tree(&empty)
            .unwrap_err()
            .to_string()
            .contains("must not be empty")
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn aliases_resolve_and_a_unit_selected_by_two_forbidden_fields_counts_once() {
    let mut ledger = job(false);
    put(
        &mut ledger,
        "#job",
        "forbiddenName",
        Term::simple_literal("^copy$"),
    );
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn enforced_copies_and_unenforced_zero_counts_fail_with_clean_neighbours() {
    let report = fixture_gate(".")
        .check_tree(&tree(SOURCE, job(true), 0))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("is a COPY"))
    );
    let home = "/// The implementation.\npub fn home() -> u32 { 7 }\npub use home as alias;";
    let stale = fixture_gate(".")
        .check_tree(&tree(home, job(false), 0))
        .unwrap();
    assert!(
        stale
            .findings()
            .iter()
            .any(|finding| finding.contains("set enforced to true"))
    );
    assert!(
        fixture_gate(".")
            .check_tree(&tree(home, job(true), 0))
            .unwrap()
            .is_clean()
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
        fixture_gate(".")
            .check_tree(&tree(source, job(true), 0))
            .unwrap()
            .is_clean()
    );
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    detector_fixture_with(graph, home, consumer, &[])
}

/// The detector workspace with extra files, such as a consumer test target.
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn detector_fixture_with(
    graph: Builder,
    home: &str,
    consumer: &str,
    extra: &[(&str, &str)],
) -> MemoryTree {
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
            "[package]\nname='consumer'\nversion='0.1.0'\nedition='2021'\n[dependencies]\ndemo={path='../home'}\n".into(),
        ),
        ("home/src/lib.rs", format!("{home}\npub use home as alias;")),
        ("consumer/src/lib.rs", consumer.into()),
        (
            "helpers-ledger.ttl",
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ]
    .into_iter()
    .chain(extra.iter().map(|(path, text)| (*path, (*text).into())))
    .collect()
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn default_hasher_detector_uses_visible_std_paths_and_lexical_aliases() {
    for source in detector_fixtures::HASHER_POSITIVE {
        let report = fixture_gate(".")
            .check_tree(&detector_tree(
                detector_fixtures::HASHER_ID,
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
    for source in detector_fixtures::HASHER_NEGATIVE {
        let report = fixture_gate(".")
            .check_tree(&detector_tree(
                detector_fixtures::HASHER_ID,
                "/// Home.\npub fn home(){}",
                source,
            ))
            .unwrap();
        assert!(report.is_clean(), "{source}: {report:?}");
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn byte_layout_detectors_execute_with_shipping_and_home_package_boundaries() {
    required_rdf_rules_have_no_home_or_test_exemption();
    required_rdf_macro_paths_follow_lexical_context();
    let cases = detector_fixtures::LAYOUT;
    let home = "/// Approved implementation.\npub fn home() {}";
    for (id, positive, negative) in cases {
        let report = fixture_gate(".")
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
            let clean = fixture_gate(".")
                .check_tree(&detector_tree(id, home, &source))
                .unwrap();
            assert!(clean.is_clean(), "{id}: {:?}", clean.findings());
        }
        let own = format!(
            "/// Approved implementation.\n{}",
            positive.replace("fn copy(", "pub fn home(")
        );
        let clean = fixture_gate(".")
            .check_tree(&detector_tree(id, &own, ""))
            .unwrap();
        assert!(clean.is_clean(), "{id}: {:?}", clean.findings());
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn required_rdf_macro_paths_follow_lexical_context() {
    for (imports, body, forbidden) in [
        ("", "fn load(value: serde_json::Value){}", true),
        (
            "use serde_json as json;",
            "fn load(value: json::Value){}",
            true,
        ),
        (
            "mod serde_json {}",
            "fn load(value: ::serde_json::Value){}",
            true,
        ),
        ("", "use serde_json::Value;", true),
        ("", "use ::serde_json::Value;", true),
        (
            "extern crate purrdf_lex as serde_json;",
            "fn load(value: ::serde_json::allocation::Resident){}",
            false,
        ),
        (
            "use purrdf_lex as json;",
            "fn load(value: json::allocation::Resident){}",
            false,
        ),
        (
            "mod wrapper {pub mod serde_json {pub struct Value;}}",
            "fn load(value: wrapper::serde_json::Value){}",
            false,
        ),
        (
            "mod wrapper {pub mod serde_json {pub struct Value;}}",
            "use wrapper::serde_json::Value;",
            false,
        ),
        (
            "mod serde_json {pub struct Value;}",
            "use crate::serde_json::Value;",
            false,
        ),
    ] {
        let source = format!("{imports}\nmacro_rules! typed {{()=>{{\n{body}\n}}}}");
        let report = fixture_gate(".")
            .require_rules(["serde-json"])
            .check_tree(&detector_fixture(base(), "pub fn home(){}", &source))
            .unwrap();
        let annotation_hit = report.findings().iter().any(|finding| {
            finding.starts_with("required rule serde-json:")
                && finding.contains("consumer/src/lib.rs:3:")
        });
        assert_eq!(annotation_hit, forbidden, "{body}: {:?}", report.findings());
        if !forbidden {
            assert!(report.is_clean(), "{body}: {:?}", report.findings());
        }
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn required_rdf_rules_have_no_home_or_test_exemption() {
    let home = "/// Home.\npub fn home() {}";
    for (id, positive, negative) in detector_fixtures::RDF
        .into_iter()
        .chain(detector_fixtures::RDF_NESTED)
    {
        let gate = fixture_gate(".").require_rules([id]);
        for source in [
            positive.to_owned(),
            format!("#[cfg(test)] mod oracle {{{positive}}}"),
        ] {
            let report = gate
                .check_tree(&detector_fixture(base(), home, &source))
                .unwrap();
            assert_eq!(report.jobs(), []);
            assert!(
                report
                    .findings()
                    .iter()
                    .any(|finding| finding.contains(&format!("required rule {id}:"))),
                "{id}: {:?}",
                report.findings()
            );
        }
        let clean = gate
            .check_tree(&detector_fixture(base(), home, negative))
            .unwrap();
        assert!(clean.is_clean(), "{id}: {:?}", clean.findings());
        let report = gate
            .check_tree(&detector_fixture(detector_ledger(id), home, positive))
            .unwrap();
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| finding.contains(&format!("required rule {id}:")))
        );
    }
    let tree = detector_fixture(base(), home, "");
    let refused: &[&[&str]] = &[
        &["missing-rule"],
        &["home-literal"],
        &["vocabulary-literal"],
        &["sole-name"],
        &["foreign-host-import"],
        &["serde-json", "serde-json"],
    ];
    for ids in refused {
        assert!(
            fixture_gate(".")
                .require_rules(ids.iter().copied())
                .check_tree(&tree)
                .is_err()
        );
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn fact_detectors_reuse_collected_values_and_keep_their_target_scope() {
    for (id, positive, negative, tests_apply) in detector_fixtures::FACTS {
        let home = "/// Home.\npub fn home() {}";
        for (source, expected) in [
            (positive.to_owned(), true),
            (negative.to_owned(), false),
            (
                format!("#[cfg(test)] mod oracle {{{positive}}}"),
                tests_apply,
            ),
        ] {
            let report = fixture_gate(".")
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
            fixture_gate(".")
                .check_tree(&detector_tree(id, &own, ""))
                .unwrap()
                .is_clean()
        );
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn hex_detectors_apply_to_their_declared_targets_and_actual_format_fields() {
    for (id, positive, negative, tests_apply) in detector_fixtures::HEX {
        let home = "/// Home.\npub fn home() {}";
        for (source, expected) in [
            (positive.to_owned(), true),
            (negative.to_owned(), false),
            (
                format!("#[cfg(test)] mod oracle {{{positive}}}"),
                tests_apply,
            ),
        ] {
            let report = fixture_gate(".")
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
            fixture_gate(".")
                .check_tree(&detector_tree(id, &own, ""))
                .unwrap()
                .is_clean()
        );
    }
    for (source, expected) in [
        ("fn copy(s:&str){let _=u32::from_str_radix(s,16);}", true),
        ("fn copy(s:&str){let _=<u8>::from_str_radix(s,16);}", false),
    ] {
        let report = fixture_gate(".")
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
        let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn contextual_literal_detectors_use_only_their_jobs_home_evidence() {
    for (id, home, positives, negatives) in detector_fixtures::CONTEXTUAL {
        let mut sources = Vec::new();
        for positive in positives {
            let inactive = fixture_gate(".")
                .check_tree(&detector_fixture(base(), home, positive))
                .unwrap();
            assert!(inactive.is_clean(), "{id}: {:?}", inactive.findings());
            sources.push(((*positive).to_owned(), true));
            // Only `foreign-host-import` reads test targets; the others read shipping code.
            sources.push((
                format!("#[cfg(test)] mod oracle {{{positive}}}"),
                id == "foreign-host-import",
            ));
        }
        sources.extend(
            negatives
                .iter()
                .map(|negative| ((*negative).to_owned(), false)),
        );
        for (source, expected) in sources {
            let report = fixture_gate(".")
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
    let empty = fixture_gate(".")
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
    let report = fixture_gate(".")
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

fn structure_ledger(id: &str, entries: &[&str]) -> Builder {
    let mut graph = detector_ledger(id);
    for entry in entries {
        put(
            &mut graph,
            "#job",
            "entryPoint",
            Term::simple_literal(*entry),
        );
    }
    graph
}

/// The copies a structure detector finds in a consumer when the job declares `entries`.
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn structure_copies(id: &str, home: &str, entries: &[&str], consumer: &str) -> usize {
    let report = fixture_gate(".")
        .check_tree(&detector_fixture(
            structure_ledger(id, entries),
            home,
            consumer,
        ))
        .unwrap_or_else(|error| panic!("{id}: {consumer}: {error}"));
    let copies = report
        .jobs()
        .first()
        .unwrap_or_else(|| panic!("{id}: {consumer}: {:?}", report.findings()))
        .copies;
    assert_eq!(
        report.findings().len(),
        copies,
        "{id}: {:?}",
        report.findings()
    );
    if consumer == detector_fixtures::EXPECT_ARGUMENT_ROW {
        let [finding] = report.findings() else {
            panic!("expected one nested row: {:?}", report.findings());
        };
        assert!(finding.contains("consumer/src/lib.rs:4:"), "{finding}");
    }
    copies
}

/// A structure detector judges a function against the entry points its own job
/// declares, never a fixed type name, and reads shipping code only.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn structure_detectors_judge_calls_against_the_jobs_declared_entry_points() {
    for (id, home, entries, positives, negatives) in detector_fixtures::STRUCTURE {
        for positive in positives {
            assert_eq!(
                structure_copies(id, home, entries, positive),
                1,
                "{id}: {positive}"
            );
        }
        for negative in negatives {
            assert_eq!(
                structure_copies(id, home, entries, negative),
                0,
                "{id}: {negative}"
            );
        }
        let test_only = detector_fixture_with(
            structure_ledger(id, entries),
            home,
            "",
            &[("consumer/tests/oracle.rs", positives[0])],
        );
        let report = fixture_gate(".").check_tree(&test_only).unwrap();
        assert_eq!(report.jobs()[0].copies, 0, "{id}: {report:?}");
    }
    let [
        (_, home, entries, positives, negatives),
        (_, walk_home, walk_entries, walkers, delegators),
    ] = detector_fixtures::STRUCTURE;
    // The reifier constructor is a hit only when the job declares it.
    assert_eq!(
        structure_copies("reifier-default-graph", home, &[], positives[0]),
        0
    );
    assert_eq!(
        structure_copies(
            "reifier-default-graph",
            home,
            &["demo"],
            "trait Insert {fn insert(&mut self,s:u8,p:u8,o:u8);} impl Insert for demo::Dataset {fn insert(&mut self,s:u8,p:u8,o:u8){}} fn copy(d:&mut demo::Dataset){d.insert(1,rdf::REIFIES,2);}",
        ),
        1
    );
    assert_eq!(
        structure_copies("reifier-default-graph", home, entries, negatives[0]),
        0
    );
    // A traversal is delegated only when the job declares it.
    let free_walk = &walk_entries[..1];
    assert_eq!(
        structure_copies("rdf-list-walk", walk_home, &[], delegators[0]),
        1
    );
    assert_eq!(
        structure_copies("rdf-list-walk", walk_home, free_walk, delegators[0]),
        0
    );
    assert_eq!(
        structure_copies("rdf-list-walk", walk_home, free_walk, walkers[0]),
        1
    );

    // One physical function can reach different callees through two shipping
    // modules. A delegated reach must not hide a manual reach of that source.
    let tree = detector_fixture_with(
        structure_ledger("rdf-list-walk", walk_entries),
        walk_home,
        "mod delegated; mod manual;",
        &[
            (
                "consumer/src/delegated.rs",
                "use demo::walk_list as walk; #[path=\"shared.rs\"] mod shared;",
            ),
            (
                "consumer/src/manual.rs",
                "fn walk(n:u8)->u8{n} #[path=\"shared.rs\"] mod shared;",
            ),
            (
                "consumer/src/shared.rs",
                "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=super::walk(n);}}",
            ),
        ],
    );
    let report = fixture_gate(".").check_tree(&tree).unwrap();
    assert_eq!(report.jobs()[0].copies, 1, "{report:?}");
}

/// The copies a `foreign-host-import` job finds in a detector workspace.
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn host_copies(home: &str, consumer: &str, extra: &[(&str, &str)]) -> usize {
    let tree = detector_fixture_with(
        detector_ledger("foreign-host-import"),
        home,
        consumer,
        extra,
    );
    fixture_gate(".").check_tree(&tree).unwrap().jobs()[0].copies
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn foreign_host_import_reports_only_repeats_of_the_homes_host_function() {
    const HOME: &str = "/// Home.\npub mod home {#[wasm_bindgen(module = \"clock\")] extern \"C\" {pub fn now() -> f64;}}";
    const SAME: &str = "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn now() -> f64;}";
    const OTHER: &str = "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn later() -> f64;}";
    const DATE: &str = "/// Home.\npub mod home {#[wasm_bindgen(js_namespace = Date, js_name = now)] extern \"C\" {pub fn read() -> f64;}}";
    let copies = host_copies;
    // A second package, a second module of the home package and a test target
    // each repeat the home's host function; a different host function does not.
    assert_eq!(
        copies(&format!("{HOME}\npub mod second {{{SAME}}}"), "", &[]),
        1
    );
    assert_eq!(
        copies(&format!("{HOME}\npub mod second {{{OTHER}}}"), "", &[]),
        0
    );
    assert_eq!(copies(HOME, "", &[("consumer/tests/host.rs", SAME)]), 1);
    assert_eq!(copies(HOME, "", &[("consumer/tests/host.rs", OTHER)]), 0);
    // A static method and a namespace argument name one host function; a
    // receiver method of the same name is another.
    assert_eq!(
        copies(
            DATE,
            "#[wasm_bindgen(static_method_of = Date)] extern \"C\" {fn now() -> f64;}",
            &[]
        ),
        1
    );
    assert_eq!(
        copies(
            DATE,
            "#[wasm_bindgen(method)] extern \"C\" {fn now(this: &Date) -> f64;}",
            &[]
        ),
        0
    );
    // `link` and `cfg_attr` are read by their bare names only: a path-qualified one is
    // another crate's attribute, neither read as the built-in nor refused.
    let linked =
        "/// Home.\npub mod home {#[link(name = \"m\")] extern \"C\" {pub fn now() -> f64;}}";
    assert_eq!(
        copies(
            linked,
            "#[link(name = \"m\")] extern \"C\" {fn now() -> f64;}",
            &[]
        ),
        1
    );
    assert_eq!(
        copies(
            linked,
            "#[shim::link(name = \"m\")] extern \"C\" {fn now() -> f64;}",
            &[]
        ),
        0
    );
    // An argument the detector cannot place is refused beside a valid neighbour.
    for refused in [
        "#[wasm_bindgen(module = \"clock\", js_name)] extern \"C\" {fn now() -> f64;}",
        "#[wasm_bindgen(module = 1)] extern \"C\" {fn now() -> f64;}",
        "#[wasm_bindgen(module = \"clock\", unheard_of)] extern \"C\" {fn now() -> f64;}",
    ] {
        let tree = detector_fixture(detector_ledger("foreign-host-import"), HOME, refused);
        let error = fixture_gate(".").check_tree(&tree).unwrap_err();
        assert!(
            error.to_string().contains("host binding"),
            "{refused}: {error}"
        );
    }
    assert_eq!(
        copies(
            HOME,
            "#[wasm_bindgen(module = \"clock\", catch)] extern \"C\" {fn now() -> f64;}",
            &[]
        ),
        1
    );
    // The home alone is not a finding.
    assert_eq!(copies(HOME, "", &[]), 0);
    // A home that imports nothing leaves the job without evidence.
    let error = fixture_gate(".")
        .check_tree(&detector_tree(
            "foreign-host-import",
            "/// Home.\npub fn home() {}",
            SAME,
        ))
        .unwrap_err();
    assert!(error.to_string().contains("requires a home that declares"));
}

const HOST_HOME: &str = "/// Home.\npub mod home {#[wasm_bindgen(module = \"clock\")] extern \"C\" {pub fn now() -> f64;}}";
const HOST_LINKED: &str =
    "/// Home.\npub mod home {#[link(name = \"m\")] extern \"C\" {pub fn now() -> f64;}}";

/// Each case is a home, a source the detector refuses, the text of its refusal, and a
/// neighbouring source with the one fault corrected, which it accepts.
#[expect(
    clippy::panic,
    reason = "the assertion reports the unexpected outcome and terminates the test"
)]
fn refused_beside_neighbour(cases: &[(&str, &str, &str, &str)]) {
    for &(home, refused, expected, accepted) in cases {
        let tree = detector_fixture(detector_ledger("foreign-host-import"), home, refused);
        let error = fixture_gate(".")
            .check_tree(&tree)
            .err()
            .unwrap_or_else(|| panic!("accepted: {refused}"));
        assert!(error.to_string().contains(expected), "{refused}: {error}");
        host_copies(home, accepted, &[]);
    }
}

#[test]
fn foreign_host_import_refuses_method_constructor_and_property_shapes_beside_a_valid_neighbour() {
    const HOME: &str = HOST_HOME;
    // Each case is as in `refused_beside_neighbour`.
    let cases = [
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(getter, setter)] fn now() -> f64;}",
            "both a getter and a setter",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(getter)] fn now() -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(getter, getter)] fn now() -> f64;}",
            "`getter` is written twice",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(getter)] fn now() -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(method)] fn now() -> f64;}",
            "typed first parameter",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(method)] fn now(this: &Date) -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(method)] fn now(this: &[u8]) -> f64;}",
            "receiver must be a named type",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(method)] fn now(this: &Date) -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(constructor)] fn new() -> ();}",
            "return type must be a named type",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(constructor)] fn new() -> Date;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(constructor)] fn new();}",
            "needs a return type",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(constructor)] fn new() -> Date;}",
        ),
    ];
    refused_beside_neighbour(&cases);
}

#[test]
fn foreign_host_import_refuses_arguments_it_cannot_place_beside_a_valid_neighbour() {
    const HOME: &str = HOST_HOME;
    const LINKED: &str = HOST_LINKED;
    // Each case is as in `refused_beside_neighbour`.
    let cases = [
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\", wasm_bindgen::catch)] extern \"C\" {fn now() -> f64;}",
            "must be a plain name",
            "#[wasm_bindgen(module = \"clock\", catch)] extern \"C\" {fn now() -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(js_namespace = Date)] extern \"C\" {#[wasm_bindgen(js_namespace = Time)] fn now() -> f64;}",
            "both an extern block and an item",
            "#[wasm_bindgen(js_namespace = Date)] extern \"C\" {fn now() -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn now() -> f64; declared!();}",
            "macro invocation inside an extern block",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn now() -> f64;}",
        ),
        (
            LINKED,
            "#[link(name = \"m\", kind(static))] extern \"C\" {fn now() -> f64;}",
            "does not take a parenthesized list",
            "#[link(name = \"m\", kind = \"static\")] extern \"C\" {fn now() -> f64;}",
        ),
        (
            LINKED,
            "#[link(name = \"m\", cfg = true)] extern \"C\" {fn now() -> f64;}",
            "value must be a string, a path or a list of paths",
            "#[link(name = \"m\", cfg(target_os = \"linux\"))] extern \"C\" {fn now() -> f64;}",
        ),
        (
            HOME,
            "#[wasm_bindgen(module = \"clock\", catch(x))] extern \"C\" {fn now() -> f64;}",
            "does not take a parenthesized list",
            "#[wasm_bindgen(module = \"clock\", catch)] extern \"C\" {fn now() -> f64;}",
        ),
    ];
    refused_beside_neighbour(&cases);
    // The `cfg` predicate of `link` does not change which host function is imported.
    assert_eq!(
        host_copies(
            LINKED,
            "#[link(name = \"m\", cfg(target_os = \"linux\"))] extern \"C\" {fn now() -> f64;}",
            &[]
        ),
        1
    );
    // A namespace on the block alone and on the item alone each place the import.
    assert_eq!(
        host_copies(
            "/// Home.\npub mod home {#[wasm_bindgen(js_namespace = Date)] extern \"C\" {pub fn now() -> f64;}}",
            "extern \"C\" {#[wasm_bindgen(js_namespace = Date)] fn now() -> f64;}",
            &[]
        ),
        1
    );
}

#[test]
fn foreign_host_import_names_a_property_by_the_property_it_reads() {
    let copies = host_copies;
    // A property import is named by its own value, then `js_name`, then the Rust
    // name: two spellings of one property match, two properties on one Rust name do not.
    let width = |attributes: &str, function: &str| {
        format!(
            "#[wasm_bindgen(module = \"window\")] extern \"C\" {{#[wasm_bindgen({attributes})] fn {function}(this: &Window) -> f64;}}"
        )
    };
    let home_width = format!(
        "/// Home.\npub mod home {{{}}}",
        width("method, getter, js_name = innerWidth", "width")
    );
    assert_eq!(
        copies(
            &home_width,
            &width("method, getter, js_name = innerWidth", "inner_width"),
            &[]
        ),
        1
    );
    assert_eq!(
        copies(
            &home_width,
            &width("method, getter = innerWidth", "other"),
            &[]
        ),
        1
    );
    assert_eq!(
        copies(
            &home_width,
            &width("method, getter, js_name = innerHeight", "width"),
            &[]
        ),
        0
    );
    // A setter is named the same way: its own value, then `js_name`, then the Rust name
    // without `set_`.
    let set_width = |attributes: &str, function: &str| {
        format!(
            "#[wasm_bindgen(module = \"window\")] extern \"C\" {{#[wasm_bindgen({attributes})] fn {function}(this: &Window, value: f64);}}"
        )
    };
    let home_set = format!(
        "/// Home.\npub mod home {{{}}}",
        set_width("method, setter, js_name = innerWidth", "set_inner_width")
    );
    assert_eq!(
        copies(
            &home_set,
            &set_width("method, setter", "set_innerWidth"),
            &[]
        ),
        1
    );
    assert_eq!(
        copies(
            &home_set,
            &set_width("method, setter", "set_innerHeight"),
            &[]
        ),
        0
    );
}

#[test]
fn foreign_host_import_names_a_class_by_its_js_class_or_constructed_type() {
    let copies = host_copies;
    let bound = |attributes: &str, signature: &str| {
        format!(
            "#[wasm_bindgen(module = \"dates\")] extern \"C\" {{#[wasm_bindgen({attributes})] {signature};}}"
        )
    };
    let in_home = |import: String| format!("/// Home.\npub mod home {{{import}}}");
    // A constructor is named by the class it builds, not by its Rust name: an overload
    // under another name builds the same class and matches.
    let home = in_home(bound("constructor", "pub fn new() -> Date"));
    assert_eq!(
        copies(&home, &bound("constructor", "fn new() -> Date"), &[]),
        1
    );
    assert_eq!(
        copies(
            &home,
            &bound("constructor", "fn new_with_millis(ms: f64) -> Date"),
            &[]
        ),
        1
    );
    assert_eq!(
        copies(&home, &bound("constructor", "fn new() -> Map"), &[]),
        0
    );
    // A stated `js_class` names a method's class, whatever the Rust receiver is called.
    let method = |class: &str, receiver: &str| {
        bound(
            &format!("method, js_class = \"{class}\", js_name = getTime"),
            &format!("fn get_time(this: &{receiver}) -> f64"),
        )
    };
    let home = in_home(method("Date", "HomeDate"));
    assert_eq!(copies(&home, &method("Date", "OtherDate"), &[]), 1);
    assert_eq!(copies(&home, &method("Map", "OtherDate"), &[]), 0);
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn contextual_rules_refuse_unavailable_namespace_evidence_and_keep_sole_name_scope() {
    for home in [
        "/// Home.\npub mod home {pub const NS:&str=concat!(\"urn:\",\"demo:\");}",
        "/// Home.\npub mod home {pub const NS:&str=\"\";}",
        "/// Home.\npub mod home {pub const NS:&str=\"urn:demo:\";pub const OTHER_NS:&str=prefix(\"computed\");}",
    ] {
        let error = fixture_gate(".")
            .check_tree(&detector_tree("vocabulary-literal", home, ""))
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("nonempty literal string initializer")
        );
    }
    assert!(
        fixture_gate(".")
            .check_tree(&detector_tree(
                "home-literal",
                "/// Home.\npub fn home(){}",
                ""
            ))
            .unwrap_err()
            .to_string()
            .contains("home containing string literals")
    );
    for copy in detector_fixtures::SOLE_NAME_POSITIVE {
        let mut graph = detector_ledger(detector_fixtures::SOLE_NAME_ID);
        put(
            &mut graph,
            "#job",
            "forbiddenName",
            Term::simple_literal("^copy$"),
        );
        let home = format!("/// Home.\npub fn home(){{}}\n{copy}");
        let inactive = fixture_gate(".")
            .check_tree(&detector_fixture(base(), &home, ""))
            .unwrap();
        assert!(inactive.is_clean(), "{:?}", inactive.findings());
        let report = fixture_gate(".")
            .check_tree(&detector_fixture(graph, &home, ""))
            .unwrap();
        assert_eq!(report.jobs()[0].copies, 1);
        assert!(report.findings()[0].contains("rule sole-name:"));
    }
    for (home, entry_point) in detector_fixtures::SOLE_NAME_NEGATIVE {
        let mut graph = detector_ledger(detector_fixtures::SOLE_NAME_ID);
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
            Term::simple_literal(entry_point),
        );
        let report = fixture_gate(".")
            .check_tree(&detector_fixture(graph, home, ""))
            .unwrap();
        assert!(report.is_clean(), "{:?}", report.findings());
    }
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn registered_rule_hits_cannot_claim_the_root_as_a_first_unit_owner() {
    let source = "pub fn home(){}\npub use home as alias;\nfn copy(){}";
    let error = fixture_gate(".")
        .rules(vec![Box::new(RootOwnerRule) as Box<dyn Rule>])
        .check_tree(&tree(source, detector_ledger("root-owner"), 0))
        .unwrap_err();
    assert!(error.to_string().contains("one containing census unit"));
    let neighbour = fixture_gate(".")
        .rules(rules())
        .check_tree(&tree(source, detector_ledger("copy-rule"), 0))
        .unwrap();
    assert_eq!(neighbour.jobs()[0].copies, 1);
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let missing = fixture_gate(".")
        .check_tree(&tree(SOURCE, ledger(), 1))
        .unwrap_err();
    assert!(missing.to_string().contains("unknown rule id"));
    let report = fixture_gate(".")
        .rules(rules())
        .check_tree(&tree(SOURCE, ledger(), 1))
        .unwrap();
    let duplicate = fixture_gate(".")
        .rules(rules())
        .rules(rules())
        .check_tree(&tree(SOURCE, ledger(), 1))
        .unwrap_err();
    assert!(
        duplicate
            .to_string()
            .contains("unique lowercase kebab-case"),
        "{duplicate}"
    );
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
    variant_for(file, symbol, "copy-rule")
}

fn variant_for(file: &str, symbol: &str, detector: &str) -> Builder {
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
        ("detector", detector),
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

/// A structure variant has to match in its job's context, including when the
/// ledger names the detector only in the variant.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn structure_variants_require_the_job_scoped_detector() {
    let (_, home, ..) = detector_fixtures::STRUCTURE[1];
    for (iteration, matches) in [("", false), ("while n>0 {n-=1;}", true)] {
        let source = format!("fn copy(mut n:u8){{let _=rdf::FIRST;let _=rdf::REST;{iteration}}}");
        let policy = variant_for("consumer/src/lib.rs", "consumer::copy", "rdf-list-walk");
        let report = fixture_gate(".")
            .check_tree(&detector_fixture(policy, home, &source))
            .unwrap();
        assert_eq!(report.is_clean(), matches, "{:?}", report.findings());
        if !matches {
            assert!(report.findings().iter().any(|finding| {
                finding.contains("no longer matches detector \"rdf-list-walk\"")
            }));
        }
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn shared_tree(graph: Builder) -> MemoryTree {
    [
        ("Cargo.toml", "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".into()),
        ("src/lib.rs", "#[path=\"shared.rs\"] pub mod a;\n#[path=\"shared.rs\"] pub mod b;\npub use b::home;\npub use b::home as alias;".into()),
        ("src/shared.rs", "/// Implementation.\npub fn home()->u32 { 7 }\npub fn copy()->u32 { 7+1 }".into()),
        ("helpers-ledger.ttl", graph.finish().unwrap().to_text(Format::Turtle).unwrap()),
    ].into_iter().collect()
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_home_reached_through_two_modules_uses_one_physical_owner() {
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn reexported_module_homes_own_their_units_and_actual_home_package() {
    let source = "/// Implementation module.\npub mod implementation {pub fn home()->u32 {7}}\npub use implementation as alias;\npub fn copy()->u32 {7+1}";
    let report = fixture_gate(".")
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
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn rules_and_variants_share_physical_ownership_without_widening_file_sanctions() {
    for (file, clean) in [("src/shared.rs", true), ("src/other.rs", false)] {
        let report = fixture_gate(".")
            .rules(vec![Box::new(SharedCopyRule) as Box<dyn Rule>])
            .check_tree(&shared_tree(variant_at(file, "demo::b::copy")))
            .unwrap();
        assert_eq!(report.is_clean(), clean, "{:?}", report.findings());
        assert_eq!(report.jobs()[0].copies, usize::from(!clean));
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_variant_must_match_its_declared_file_and_documented_anchor() {
    let wrong = fixture_gate(".")
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
    let correct = fixture_gate(".")
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
    #[expect(
        clippy::unwrap_in_result,
        clippy::unwrap_used,
        reason = "a failed prerequisite or unexpected observation must fail this test at the point of use"
    )]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        fixture_gate(".")
            .check_tree(&tree(source, ledger("demo"), 0))
            .unwrap_err()
            .to_string()
            .contains("unclaimed extension")
    );
    let gate = || fixture_gate(".").extensions(vec![Box::new(Owner) as Box<dyn LedgerExtension>]);
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
        fixture_gate(".")
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
        || fixture_gate(".").extensions(vec![Box::new(Nested) as Box<dyn LedgerExtension>]);
    let mut retained = nested("preserved");
    retained
        .insert(Triple {
            subject: Term::iri("#b"),
            predicate: Term::iri("urn:example:source-pin"),
            object: Term::simple_literal("source:immutable-original-pin"),
        })
        .unwrap();
    let retained = retained.finish().unwrap();
    let mut retained = Builder::from_graph(&retained);
    for property in ["referenceSource", "referenceWitness"] {
        put(
            &mut retained,
            "#b",
            property,
            Term::simple_literal("opaque original annotation"),
        );
    }
    let retained = retained.finish().unwrap();
    for format in [Format::Turtle, Format::NTriples] {
        layout::assert_upgrade(
            raf_census::ReportKind::Ledger,
            &retained.to_text(format).unwrap(),
            format,
        );
    }
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let correct = fixture_gate(".")
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
    let report = fixture_gate(".")
        .check_tree(&tree(source, root, 0))
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("does not name a census implementation unit"))
    );
    let changed = format!("{source}\npub fn third(c:u32)->u32 {{ c+c+c+c }}");
    let report = fixture_gate(".")
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn cfg_alternatives_retain_debt_multiplicity_and_require_an_explicit_exact_sanction() {
    let source = "/// Unix.\n#[cfg(unix)] pub fn f(a:u32)->u32 { a+a+a+a }\n/// Windows.\n#[cfg(windows)] pub fn f(b:u32)->u32 { b+b+b+b }";
    let gate = fixture_gate(".");
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
    let ledger = layout::assert_upgrade(raf_census::ReportKind::Ledger, &ledger, Format::Turtle);
    let previous = layout::assert_upgrade(
        raf_census::ReportKind::Index,
        &previous.to_turtle().unwrap(),
        Format::Turtle,
    );
    let result = migrated(&tree, &ledger, &previous).unwrap();
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
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

fn scope_ledger(settings: &[(&str, Term)]) -> Builder {
    let mut graph = base();
    put(
        &mut graph,
        "",
        "minTokens",
        Term::typed_literal("1", vocab::INTEGER),
    );
    for (key, value) in settings {
        put(&mut graph, "", key, value.clone());
    }
    graph
}

/// The scope ledger with a distinct sanction over the test helpers `f` and `g`.
fn sanctioned_scope_ledger(settings: &[(&str, Term)]) -> Builder {
    let mut graph = scope_ledger(settings);
    put(&mut graph, "", "distinct", Term::iri("#distinct"));
    put(
        &mut graph,
        "#distinct",
        "type",
        Term::iri(format!("{}Distinct", vocab::REPORT)),
    );
    for (key, value) in [
        ("kind", "body"),
        ("memberSymbol", "demo::tests::f"),
        ("memberSymbol", "demo::tests::g"),
        ("criterion", "test helpers"),
        ("reason", "Kept apart on purpose"),
    ] {
        put(&mut graph, "#distinct", key, Term::simple_literal(value));
    }
    put(
        &mut graph,
        "#distinct",
        "anchor",
        Term::iri("urn:raf:code:demo::tests::f"),
    );
    graph
}

/// The ledger's grouping scope reaches the census, and the index and the exact
/// debt record state it; a record of another scope is a difference that is
/// not growth, and a sanction over a group the scope removes is stale. Only a
/// true flag and plain directory names are accepted.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn ledger_grouping_scope_is_stated_by_the_index_and_the_debt_record() {
    let source = "#[cfg(test)]\nmod tests {\n/// One.\npub fn f(a:u32)->u32 { a+a+a+a }\n/// Two.\npub fn g(b:u32)->u32 { b+b+b+b }\n}";
    let gate = fixture_gate(".");
    let complete = gate
        .check_tree(&tree(source, scope_ledger(&[]), 0))
        .unwrap();
    assert_eq!(complete.debt().groups().len(), 1);
    assert!(complete.debt().scope().is_complete());
    assert!(!complete.debt().to_turtle().unwrap().contains("exclude"));

    let settings = [
        ("excludeTestItems", Term::boolean(true)),
        ("excludeDir", Term::simple_literal("generated")),
    ];
    let scoped = gate
        .check_tree(&tree(source, scope_ledger(&settings), 0))
        .unwrap();
    assert!(scoped.is_clean(), "{:?}", scoped.findings());
    assert!(scoped.debt().groups().is_empty());
    let scope = scoped.census().options().scope.clone();
    assert!(scope.excludes_test_items());
    assert_eq!(scope.excluded_dirs().collect::<Vec<_>>(), ["generated"]);
    assert_eq!(scoped.debt().scope(), &scope);
    let record = scoped.debt().to_turtle().unwrap();
    assert!(record.contains("excludeTestItems") && record.contains("\"generated\""));
    assert_eq!(DebtRecord::from_turtle(&record).unwrap().scope(), &scope);
    let index = scoped.census().index(scoped.jobs()).unwrap();
    assert_eq!(index.scope, scope);
    let text = index.to_turtle().unwrap();
    assert_eq!(raf_census::Index::from_turtle(&text).unwrap().scope, scope);

    // A sanction over the group that the exclusion removes holds under the
    // complete scope and is stale once the scope is on.
    let sanctioned = |settings: &[(&str, Term)]| {
        gate.check_tree(&tree(source, sanctioned_scope_ledger(settings), 0))
            .unwrap()
    };
    let held = sanctioned(&[]);
    assert!(held.is_clean(), "{:?}", held.findings());
    let removed = sanctioned(&settings);
    assert!(
        removed
            .findings()
            .iter()
            .any(|finding| finding.contains("equals no current group")),
        "{:?}",
        removed.findings()
    );

    let unscoped = DebtRecord::new(NORMAL_FORM, [], BTreeMap::new()).unwrap();
    let changes = unscoped.compare(scoped.debt()).unwrap();
    assert!(matches!(
        changes.as_slice(),
        [raf_census::DebtChange::Scope { .. }]
    ));
    assert!(!changes[0].is_growth());
    let ledger = scope_ledger(&settings)
        .finish()
        .unwrap()
        .to_text(Format::Turtle)
        .unwrap();
    let stale: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", source),
        ("helpers-ledger.ttl", ledger.as_str()),
        ("census-debt.ttl", unscoped.to_turtle().unwrap().as_str()),
    ]
    .into();
    let stale = gate.check_tree(&stale).unwrap();
    assert!(
        stale
            .findings()
            .iter()
            .any(|finding| finding.contains("record the present grouping scope")),
        "{:?}",
        stale.findings()
    );

    for (key, value, needle) in [
        ("excludeTestItems", Term::boolean(false), "only as true"),
        ("excludeDir", Term::simple_literal("a/b"), "plain directory"),
        ("excludeDir", Term::simple_literal(".."), "plain directory"),
    ] {
        let text = scope_ledger(&[(key, value)])
            .finish()
            .unwrap()
            .to_text(Format::Turtle)
            .unwrap();
        let error = Ledger::from_turtle(&text).unwrap_err().to_string();
        assert!(error.contains(needle), "{error}");
    }
}

/// A required module path containment check reports each `path` attribute,
/// plain or inside `cfg_attr`, that leads a censused package into a sibling
/// package, through both the structural and the locked gate; one that stays
/// in its own package is not reported, and without the requirement the
/// verdict is unchanged.
#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn module_paths_outside_their_package_are_required_findings_beside_contained_ones() {
    use raf_source::{LoadOptions, LockedCatalog, LockedManifest, RelPath};

    let ledger = base().finish().unwrap().to_text(Format::Turtle).unwrap();
    let tree = MemoryTree::from([
        ("Cargo.toml", "[workspace]\nmembers=['one','two']\n"),
        (
            "one/Cargo.toml",
            "[package]\nname='one'\nversion='1.0.0'\nedition='2024'\n",
        ),
        (
            "one/src/lib.rs",
            "#[path = \"inside.rs\"]\nmod inside;\n\
             #[path = \"../../two/src/shared.rs\"]\nmod shared;\n\
             #[cfg_attr(feature = \"extra\", path = \"../../two/src/extra.rs\")]\nmod extra;\n",
        ),
        ("one/src/inside.rs", ""),
        (
            "two/Cargo.toml",
            "[package]\nname='two'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("two/src/lib.rs", "#[path = \"shared.rs\"]\nmod own;\n"),
        ("two/src/shared.rs", ""),
        ("two/src/extra.rs", ""),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='one'\nversion='1.0.0'\n[[package]]\nname='two'\nversion='1.0.0'\n",
        ),
        ("helpers-ledger.ttl", ledger.as_str()),
    ]);
    let plain = fixture_gate(".").check_tree(&tree).unwrap();
    assert!(plain.is_clean(), "{:?}", plain.findings());
    let required = fixture_gate(".").require_module_paths_in_package();
    let report = required.check_tree(&tree).unwrap();
    assert_eq!(
        report.findings(),
        [
            "module path attribute: one/src/lib.rs:4: module `one::shared` of package `one` resolves to two/src/shared.rs, in package `two`; move the file into the directory of `one` or use it through a dependency",
            "module path attribute: one/src/lib.rs:6: module `one::extra` of package `one` resolves to two/src/extra.rs, in package `two`; move the file into the directory of `one` or use it through a dependency",
        ]
    );
    // The locked gate reports the same sites through its own load.
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
    let locked = required.check_locked(&tree, &catalog, &local, &[]).unwrap();
    assert_eq!(locked.findings(), report.findings());
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_top_level_hit_in_an_out_of_line_module_is_attributed_to_its_declaring_module() {
    // INVARIANT: top-level code of an out-of-line module file lies in no item of
    // its own; the `mod` item that declares the file owns the hit, and the finding
    // names the file and line of the hit. An inline module with the same use is the
    // neighbour that must keep attributing to its own module.
    let outline = detector_fixture_with(
        base(),
        "pub mod yamlld;\npub fn home(){}",
        "",
        &[("home/src/yamlld.rs", "use serde_json::Value;\n")],
    );
    let report = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&outline)
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("yamlld.rs:1:") && finding.contains("demo::yamlld")),
        "{:?}",
        report.findings()
    );
    let inline = detector_fixture_with(
        base(),
        "pub mod yamlld { use serde_json::Value; }\npub fn home(){}",
        "",
        &[],
    );
    let report = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&inline)
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("demo::yamlld")),
        "{:?}",
        report.findings()
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_top_level_hit_is_reported_and_varied_at_the_declaring_module_in_its_own_file() {
    // INVARIANT: the `mod` item declared in another file owns the whole module
    // file, as a crate root owns its own: a job match names the module and the
    // module file at its first byte and line, and a ledger variant naming that
    // module and file accepts the hit. The declaration and the hit sit on other
    // lines of other files, so a location mixed from the two points nowhere.
    let consumer = "pub fn a() {}\n\n\npub mod yamlld;\n";
    let extra = [("consumer/src/yamlld.rs", "\n\nuse serde_json::Value;\n")];
    let report = fixture_gate(".")
        .check_tree(&detector_fixture_with(
            detector_ledger("serde-json"),
            "/// Home.\npub fn home(){}",
            consumer,
            &extra,
        ))
        .unwrap();
    let job = &report.jobs()[0];
    assert_eq!(job.copies, 1, "{report:?}");
    let found = job
        .matches
        .iter()
        .find(|found| found.symbol == "consumer::yamlld")
        .unwrap();
    assert_eq!(
        (found.file.as_str(), found.line, found.start_byte),
        ("consumer/src/yamlld.rs", 1, 0),
        "{report:?}"
    );
    let mut varied = detector_ledger("serde-json");
    put(&mut varied, "#job", "variant", Term::iri("#variant"));
    put(
        &mut varied,
        "#variant",
        "type",
        Term::iri(format!("{}Variant", vocab::REPORT)),
    );
    for (key, value) in [
        ("symbol", "consumer::yamlld"),
        ("file", "consumer/src/yamlld.rs"),
        ("detector", "serde-json"),
        ("criterion", "layering"),
        ("reason", "The module reads a separate format"),
    ] {
        put(&mut varied, "#variant", key, Term::simple_literal(value));
    }
    put(
        &mut varied,
        "#variant",
        "anchor",
        Term::iri("urn:raf:code:demo::home"),
    );
    let report = fixture_gate(".")
        .check_tree(&detector_fixture_with(
            varied,
            "/// Home.\npub fn home(){}",
            consumer,
            &extra,
        ))
        .unwrap();
    assert_eq!(report.jobs()[0].copies, 0, "{report:?}");
    assert!(
        report.jobs()[0]
            .matches
            .iter()
            .any(|found| found.symbol == "consumer::yamlld" && found.class == MatchClass::Variant),
        "{report:?}"
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn configured_path_alternatives_each_own_only_the_file_they_name() {
    // INVARIANT: a file belongs to the declaration that reaches it, not to every
    // declaration of the same module path. Two `mod imp;` items under exclusive
    // configurations name two files through `path`, so each file has one owner and
    // a top-level hit in either is attributed rather than refused as ambiguous.
    let tree = detector_fixture_with(
        base(),
        "#[cfg(unix)]\n#[path = \"unix.rs\"]\npub mod imp;\n#[cfg(windows)]\n#[path = \"windows.rs\"]\npub mod imp;\npub fn home(){}",
        "",
        &[
            ("home/src/unix.rs", "use serde_json::Value;\n"),
            ("home/src/windows.rs", "use serde_json::Value;\n"),
        ],
    );
    let report = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&tree)
        .unwrap();
    for file in ["unix.rs:1:", "windows.rs:1:"] {
        assert!(
            report
                .findings()
                .iter()
                .any(|finding| finding.contains(file) && finding.contains("demo::imp")),
            "{file}: {:?}",
            report.findings()
        );
    }
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_file_declared_by_two_module_items_is_refused_as_ambiguous() {
    // INVARIANT: a hit in the top-level code of a file that two `mod` items declare
    // has two owners and no single physical source item, so it is refused rather
    // than attributed to whichever declaration was recorded last.
    let ambiguous = detector_fixture_with(
        base(),
        "#[cfg(feature = \"a\")]\npub mod yamlld;\n#[cfg(not(feature = \"a\"))]\npub mod yamlld;\npub fn home(){}",
        "",
        &[("home/src/yamlld.rs", "use serde_json::Value;\n")],
    );
    let error = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&ambiguous)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("detector hit requires one physical source item"),
        "{error}"
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_file_declared_at_one_offset_by_two_parent_files_is_refused_as_ambiguous() {
    // INVARIANT: the physical site of a declaration is its declaring file and its
    // offset together. Two parent files that declare the same file at the same
    // offset are two sites, so the hit is refused rather than collapsed to one owner.
    let ambiguous = detector_fixture_with(
        base(),
        "pub mod a;\npub mod b;\npub fn home(){}",
        "",
        &[
            (
                "home/src/a.rs",
                "#[path = \"yamlld.rs\"]\npub mod yamlld;\n",
            ),
            (
                "home/src/b.rs",
                "#[path = \"yamlld.rs\"]\npub mod yamlld;\n",
            ),
            ("home/src/yamlld.rs", "use serde_json::Value;\n"),
        ],
    );
    let error = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&ambiguous)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("detector hit requires one physical source item"),
        "{error}"
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn an_undeclared_loose_file_is_never_visited_and_a_root_top_level_hit_belongs_to_the_root() {
    // INVARIANT: the loader visits only files that a `mod` item or a crate root
    // reaches, so a loose file that nothing declares produces no finding and no
    // refusal. A top-level hit in the crate root belongs to the root module.
    let tree = detector_fixture_with(
        base(),
        "use serde_json::Value;\npub fn home(){}",
        "",
        &[("home/src/loose.rs", "use serde_json::Value;\n")],
    );
    let report = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&tree)
        .unwrap();
    assert!(
        report
            .findings()
            .iter()
            .all(|finding| !finding.contains("loose.rs")),
        "{:?}",
        report.findings()
    );
    assert!(
        report
            .findings()
            .iter()
            .any(|finding| finding.contains("lib.rs:1:") && finding.contains("demo")),
        "{:?}",
        report.findings()
    );
}

#[test]
#[expect(
    clippy::unwrap_used,
    reason = "a failed fixture operation is a test failure at the point of use"
)]
fn a_declaration_reached_through_two_target_paths_is_one_physical_owner() {
    // INVARIANT: `leaf` is one declaration in `shared.rs`, which the library and the
    // binary both load under their own module paths. The top-level hit of its file
    // has one physical declaring site, so it is reported once rather than refused
    // as two owners.
    let tree: MemoryTree = [
        (
            "Cargo.toml",
            "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n".into(),
        ),
        ("src/lib.rs", "pub mod shared;\n".into()),
        ("src/main.rs", "mod shared;\nfn main() {}\n".into()),
        ("src/shared.rs", "pub mod leaf;\n".into()),
        ("src/shared/leaf.rs", "use serde_json::Value;\n".into()),
        (
            "helpers-ledger.ttl",
            base().finish().unwrap().to_text(Format::Turtle).unwrap(),
        ),
    ]
    .into_iter()
    .collect();
    let report = fixture_gate(".")
        .require_rules(["serde-json"])
        .check_tree(&tree)
        .unwrap();
    let findings: Vec<_> = report
        .findings()
        .iter()
        .filter(|finding| finding.contains("leaf.rs:1:"))
        .collect();
    assert_eq!(findings.len(), 1, "{:?}", report.findings());
}
