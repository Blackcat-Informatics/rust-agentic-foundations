// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Counted source observations and normative RDF are inert fixture data.

use super::*;
mod fixture;
use crate::embedded::{DebtRefusal, OutputFailureKind, RefusalKind, pairing, policy, workspace};
use fixture::{read, render, text};
use raf_rdf::{Builder, Format, Graph, TermValue as Term};
use raf_source::{LoadOptions, MemoryTree, RelPath, Workspace};

const MANIFEST: &str = "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n";
const CORE: &str = "let x = input + 1; let y = x * 3; sink(y, x, input, 7);";

fn load(source: &str) -> Workspace {
    Workspace::load(
        &MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source)]),
        &["."],
    )
    .unwrap()
}

fn analyze(workspace: &Workspace) -> Matches<'_> {
    pairing::find(
        workspace::collect(workspace, 100_000_000, 10_000_000).unwrap(),
        policy::Limits::default(),
    )
    .unwrap()
}

fn in_run<'m>(record: &Record<'_>, matches: &'m Matches<'_>) -> Record<'m> {
    let text = text(record, Format::Turtle);
    read(&text, Format::Turtle, matches.analysis.meter())
}

fn repeated(count: usize) -> String {
    format!(
        "#![no_implicit_prelude]\nfn repeated(input:i32) {{ {} }}",
        format!("{{ {CORE} }}").repeat(count)
    )
}

#[test]
fn adoption_preserves_line_shifts_counts_repetition_and_refuses_growth_after_exact_shrink() {
    public_run_preserves_verified_pairs_and_explicit_shrink();
    let original = load(&repeated(3));
    let observed = analyze(&original);
    let record = Record::adopt(&observed).unwrap();
    assert_eq!(
        record
            .entries
            .iter()
            .map(|(_, count)| count.get())
            .sum::<usize>(),
        3
    );
    assert_eq!(record.compare(&observed).unwrap(), Comparison::Unchanged);
    let shifted = load(&format!(
        "// Unrelated source heading.\n\n{}",
        repeated(3).replace(
            "fn repeated(input:i32) {",
            "fn repeated(input:i32) { let unrelated=9; emit(unrelated);"
        )
    ));
    let shifted = analyze(&shifted);
    assert_eq!(
        in_run(&record, &shifted).compare(&shifted).unwrap(),
        Comparison::Unchanged
    );
    let smaller = load(&repeated(2));
    let smaller = analyze(&smaller);
    assert_eq!(
        in_run(&record, &smaller).compare(&smaller).unwrap(),
        Comparison::Shrunk { removed: 2 }
    );
    let shrunk = in_run(&record, &smaller)
        .rewrite_existing(&smaller)
        .unwrap();
    assert_eq!(shrunk.compare(&smaller).unwrap(), Comparison::Unchanged);
    assert!(matches!(
        in_run(&shrunk, &observed).compare(&observed),
        Err(Failure::Growth {
            recorded: 1,
            present: 3,
            ..
        })
    ));
    for changed in [repeated(4), repeated(3).replace("+ 1", "- 1")] {
        let changed = load(&changed);
        let changed = analyze(&changed);
        assert!(matches!(
            in_run(&record, &changed).compare(&changed),
            Err(Failure::Growth { .. })
        ));
        assert!(matches!(
            in_run(&record, &changed).rewrite_existing(&changed),
            Err(Failure::Growth { .. })
        ));
    }
    let empty = load("fn empty() {}");
    let empty = analyze(&empty);
    let zero = in_run(&shrunk, &empty).rewrite_existing(&empty).unwrap();
    assert_eq!(zero.compare(&empty).unwrap(), Comparison::Unchanged);
    assert!(matches!(
        in_run(&zero, &observed).compare(&observed),
        Err(Failure::Growth { recorded: 0, .. })
    ));
    for format in [Format::Turtle, Format::NTriples] {
        let text = text(&record, format);
        let imported = read(&text, format, observed.analysis.meter());
        assert_eq!(imported.compare(&observed).unwrap(), Comparison::Unchanged);
        assert_eq!(fixture::text(&imported, format), text);
        Record::from_text(
            &fixture::text(&zero, format),
            format,
            empty.analysis.meter(),
            |outcome| assert!(outcome.is_ok()),
        );
    }
}

// The shipped facade must reach the same source proof, independent RDF and
// shrink-only allowance path as the controlled production fixture above.
fn public_run_preserves_verified_pairs_and_explicit_shrink() {
    use crate::embedded::{Comparison as PublicComparison, Error, Options, Run};
    let workspace = load(&repeated(3));
    let options = Options::default();
    let run = Run::analyze(&workspace, &options).unwrap();
    assert_eq!(run.witnesses().count(), 3);
    for witness in run.witnesses() {
        assert_eq!(witness.left().text(), witness.right().text());
        assert_eq!(witness.left().function().owners().len(), 1);
        assert!(std::ptr::eq(
            witness.left().function().workspace(),
            &raw const workspace
        ));
    }
    let evidence = run.report(|outcome| outcome.unwrap());
    let text = evidence.to_text(Format::Turtle, |outcome| {
        outcome.unwrap().as_str().to_owned()
    });
    let imported_evidence = run.import_evidence(&text, Format::Turtle, |outcome| outcome.unwrap());
    imported_evidence.to_text(Format::Turtle, |outcome| {
        assert_eq!(outcome.unwrap().as_str(), text);
    });
    let adopted = run.adopt_debt(|outcome| outcome.unwrap());
    run.compare_debt(&adopted, |outcome| {
        assert_eq!(outcome.unwrap(), PublicComparison::Unchanged);
    });
    let record = adopted.to_text(Format::Turtle, |outcome| outcome.unwrap());
    let smaller_workspace = load(&repeated(2));
    let smaller = Run::analyze(&smaller_workspace, &options).unwrap();
    smaller.compare_debt(&adopted, |outcome| {
        let Err(cause) = outcome else {
            panic!("another account must refuse")
        };
        assert_eq!(cause.debt_refusal(), Some(DebtRefusal::AccountMismatch));
        assert!(cause.refusal().is_none());
    });
    let imported = smaller.import_debt(record.as_str(), Format::Turtle, |outcome| outcome.unwrap());
    smaller.compare_debt(&imported, |outcome| {
        assert_eq!(outcome.unwrap(), PublicComparison::Shrunk { removed: 2 });
    });
    let shrunk = smaller.shrink_debt(&imported, |outcome| outcome.unwrap());
    let shrunk_text = shrunk.to_text(Format::Turtle, |outcome| outcome.unwrap());
    smaller.compare_debt(&shrunk, |outcome| {
        assert_eq!(outcome.unwrap(), PublicComparison::Unchanged);
    });
    let allowance = run.import_debt(shrunk_text.as_str(), Format::Turtle, |outcome| {
        outcome.unwrap()
    });
    run.shrink_debt(&allowance, |outcome| {
        let Err(cause) = outcome else {
            panic!("an existing allowance cannot grow")
        };
        assert!(matches!(
            cause.debt_refusal(),
            Some(DebtRefusal::Growth {
                recorded: 1,
                present: 3,
                ..
            })
        ));
        assert!(cause.refusal().is_none());
    });
    run.import_debt(&text, Format::Turtle, |outcome| {
        let Err(cause) = outcome else {
            panic!("evidence cannot supply normative debt")
        };
        assert_eq!(cause.kind(), OutputFailureKind::Domain, "{cause}");
        assert!(cause.refusal().is_none(), "{cause}");
    });
    let mut refused = options;
    refused.max_verified_pairs = 0;
    assert!(matches!(
        Run::analyze(&workspace, &refused),
        Err(Error::Refused(_))
    ));
}

#[test]
fn complete_owner_descriptors_separate_traits_generic_self_types_and_physical_aliases() {
    for (declarations, first, second) in [
        (
            "struct S; trait A {fn helper(input:i32);} trait B {fn helper(input:i32);}",
            "A for S",
            "B for S",
        ),
        ("struct S<const N:usize>;", "S<1>", "S<2>"),
    ] {
        let make = |implementation| {
            load(&format!(
                "#![no_implicit_prelude]\n{declarations}\nfn other(input:i32) {{ {CORE} }} impl {implementation} {{ fn helper(input:i32) {{ {CORE} }} }}"
            ))
        };
        let before = make(first);
        let after = make(second);
        let method = |workspace: &Workspace| {
            workspace
                .items()
                .find(|item| item.name.as_str() == "helper" && item.body.is_some())
                .unwrap()
                .path
                .to_string()
        };
        assert_eq!(
            method(&before),
            method(&after),
            "displayed paths alone cannot distinguish these owners"
        );
        let before = analyze(&before);
        let after = analyze(&after);
        let record = Record::adopt(&before).unwrap();
        assert!(matches!(
            in_run(&record, &after).compare(&after),
            Err(Failure::Growth { .. })
        ));
    }
    let source = format!("fn one(input:i32) {{ {CORE} }} fn two(input:i32) {{ {CORE} }}");
    let tree = MemoryTree::from([
        ("Cargo.toml", MANIFEST),
        (
            "src/lib.rs",
            "#![no_implicit_prelude]\n#[path=\"shared.rs\"] mod shared;",
        ),
        (
            "src/main.rs",
            "#![no_implicit_prelude]\n#[path=\"shared.rs\"] mod shared; fn main() {}",
        ),
        ("src/shared.rs", source.as_str()),
    ]);
    let workspace = Workspace::load(&tree, &["."]).unwrap();
    let observed = analyze(&workspace);
    let record = Record::adopt(&observed).unwrap();
    assert_eq!(
        record
            .entries
            .iter()
            .map(|(_, count)| count.get())
            .sum::<usize>(),
        1
    );
    let mut analysis = workspace::collect(&workspace, 100_000_000, 10_000_000).unwrap();
    analysis.functions.reverse();
    for function in &mut analysis.functions {
        if let crate::embedded::analysis::Origin::Source { owners, .. } = &mut function.origin {
            owners.reverse();
        }
    }
    let reordered = pairing::find(analysis, policy::Limits::default()).unwrap();
    assert_eq!(
        in_run(&record, &reordered).compare(&reordered).unwrap(),
        Comparison::Unchanged
    );
    assert_eq!(
        text(&record, Format::Turtle),
        text(&Record::adopt(&reordered).unwrap(), Format::Turtle)
    );
}

fn external_workspace(reverse: bool) -> Workspace {
    use raf_source::{LockedCatalog, LockedManifest, ReferencePackageRoots, ReferenceTree};
    let source = format!(
        "#![no_implicit_prelude]\nfn one(input:i32) {{ {CORE} }} fn two(input:i32) {{ {CORE} }}"
    );
    let first = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='first'\nversion='0.1.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "fn first() {}"),
        (
            "Cargo.lock",
            "version=4\n[[package]]\nname='first'\nversion='0.1.0'\n[[package]]\nname='dependency'\nversion='1.0.0'\nsource='registry+https://registry.example.invalid/index'\n[[package]]\nname='other'\nversion='1.0.0'\nsource='registry+https://registry.example.invalid/index'\n",
        ),
    ]);
    let dependency = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='dependency'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", source.as_str()),
    ]);
    let other = MemoryTree::from([
        (
            "Cargo.toml",
            "[package]\nname='other'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", "fn other() {}"),
    ]);
    let options = LoadOptions::default();
    let catalog =
        LockedCatalog::load(&first, &RelPath::new("Cargo.lock").unwrap(), &options).unwrap();
    let row = |name| {
        catalog
            .packages()
            .find(|row| row.name() == name)
            .unwrap()
            .id()
    };
    let local = [LockedManifest::new(
        row("first"),
        RelPath::new("Cargo.toml").unwrap(),
        &[],
    )];
    let dependency_rows = [LockedManifest::new(
        row("dependency"),
        RelPath::new("Cargo.toml").unwrap(),
        &[],
    )];
    let other_rows = [LockedManifest::new(
        row("other"),
        RelPath::new("Cargo.toml").unwrap(),
        &[],
    )];
    let mut sources = [
        ReferencePackageRoots::new(ReferenceTree::Opaque(&dependency), &dependency_rows),
        ReferencePackageRoots::new(ReferenceTree::Opaque(&other), &other_rows),
    ];
    if reverse {
        sources.reverse();
    }
    Workspace::load_reference_sources_under(
        &catalog,
        ReferenceTree::Opaque(&first),
        &[RelPath::root()],
        &local,
        &sources,
        &options,
    )
    .unwrap()
}

#[test]
fn checked_external_sources_survive_catalog_reordering_while_unbound_roots_refuse() {
    let before = external_workspace(false);
    let after = external_workspace(true);
    let authority = |workspace: &Workspace| {
        workspace
            .packages()
            .iter()
            .find(|package| package.name == "dependency")
            .unwrap()
            .manifest
            .authority()
    };
    assert_ne!(authority(&before), authority(&after));
    let before = analyze(&before);
    let after = analyze(&after);
    let record = Record::adopt(&before).unwrap();
    assert_eq!(
        in_run(&record, &after).compare(&after).unwrap(),
        Comparison::Unchanged
    );
    assert_eq!(
        text(&record, Format::NTriples),
        text(&Record::adopt(&after).unwrap(), Format::NTriples)
    );

    let source = repeated(2);
    let data = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", source.as_str())]);
    let first = MemoryTree::default();
    let dependencies: [&dyn raf_source::Tree; 1] = [&data];
    let sources = raf_source::SourceTrees::new(&first, &dependencies);
    let external = sources.authorities().nth(1).unwrap();
    let unbound = Workspace::load_from(
        &sources,
        &[sources.root(external).unwrap()],
        &LoadOptions::default(),
    )
    .unwrap();
    assert!(
        matches!(Record::adopt(&analyze(&unbound)), Err(cause) if crate::embedded::OutputFailure::failure(&cause).kind() == OutputFailureKind::Domain && cause.to_string().contains("load-local external authority"))
    );
    assert!(Record::adopt(&analyze(&Workspace::load(&data, &["."]).unwrap())).is_ok());
}

#[test]
fn imported_allowances_refuse_unknown_policy_zero_counts_orphans_and_total_overflow() {
    let original = load(&repeated(4));
    let observed = analyze(&original);
    let meter = observed.analysis.meter();
    let record = Record::adopt(&observed).unwrap();
    assert!(
        record.entries.iter().count() >= 2,
        "different maximal repeated regions retain independent allowances"
    );
    let text = text(&record, Format::Turtle);
    let graph = Graph::from_text(&text, Format::Turtle).unwrap();
    for refusal in ["policy", "zero", "overflow", "orphan", "unknown"] {
        let mut builder = Builder::from_graph(&graph);
        for mut triple in graph.assertions() {
            let property = triple.predicate.as_iri().unwrap();
            let changed = match refusal {
                "policy" if property.ends_with("#matchingProfile") => {
                    Some(Term::simple_literal("unknown"))
                }
                "zero" if property.ends_with("#pairs") => Some(crate::report::shape::integer(0)),
                "overflow" if property.ends_with("#pairs") => {
                    Some(crate::report::shape::integer(usize::MAX as u128))
                }
                _ => None,
            };
            if let Some(value) = changed {
                assert!(builder.remove(triple.clone()).unwrap());
                triple.object = value;
                builder.insert(triple).unwrap();
            }
        }
        if matches!(refusal, "orphan" | "unknown") {
            builder
                .insert(raf_rdf::Triple {
                    subject: Term::iri(if refusal == "orphan" { "#unused" } else { "" }),
                    predicate: Term::iri("urn:unrecognized:allowance"),
                    object: Term::boolean(true),
                })
                .unwrap();
        }
        let changed = builder.finish().unwrap();
        Record::from_text(
            &changed.to_text(Format::Turtle).unwrap(),
            Format::Turtle,
            meter,
            |outcome| {
                let Err(cause) = outcome else {
                    panic!("{refusal} must refuse")
                };
                assert!(cause.refusal().is_none(), "{cause}");
                if refusal == "overflow" {
                    assert_eq!(cause.debt_refusal(), Some(DebtRefusal::Overflow));
                } else {
                    assert_eq!(cause.kind(), OutputFailureKind::Domain, "{cause}");
                }
            },
        );
        Record::from_text(&text, Format::Turtle, meter, |outcome| {
            assert!(outcome.is_ok());
        });
    }
    refuse_nested_descriptors(&record.entries.iter().next().unwrap().0.left, meter);
    let report = crate::embedded::report::tests::graph(&analyze(&original))
        .to_text(Format::Turtle)
        .unwrap();
    Record::from_text(&report, Format::Turtle, meter, |outcome| {
        let Err(cause) = outcome else {
            panic!("evidence cannot supply normative debt")
        };
        assert_eq!(cause.kind(), OutputFailureKind::Domain, "{cause}");
        assert!(cause.refusal().is_none(), "{cause}");
    });
    Record::from_text(
        &" ".repeat(crate::MAX_DEBT_BYTES + 1),
        Format::Turtle,
        meter,
        |outcome| {
            let Err(cause) = outcome else {
                panic!("oversized debt must refuse")
            };
            assert_eq!(cause.kind(), OutputFailureKind::Domain, "{cause}");
            assert!(cause.refusal().is_none(), "{cause}");
            assert!(cause.to_string().contains("byte limit"));
        },
    );
}

fn refuse_nested_descriptors(descriptor: &str, meter: &Meter) {
    // Outer framing stays canonical while nested records become malformed.
    // Exercise the production descriptor boundary without a stale RDF name.
    let decode = |text: &str| {
        text.split('/')
            .skip(1)
            .map(|part| purrdf_lex::percent::decode(part).unwrap().into_owned())
            .collect::<Vec<_>>()
    };
    let mut accepted = Vec::new();
    for refusal in [
        "owner",
        "target",
        "source",
        "escaping",
        "empty-trait",
        "duplicate-trait",
        "repository",
        "git-query",
        "git-fragment",
        "registry-fragment",
    ] {
        let mut endpoint = decode(descriptor);
        let mut owner = decode(&endpoint[4]);
        let mut target = decode(&owner[0]);
        match refusal {
            "owner" => endpoint[4] = String::from("#owner-v1/"),
            "target" => {
                owner[0] = String::from("#target-v1/");
                endpoint[4] = crate::embedded::report::tests::key("owner-v1", &owner);
            }
            "source" => {
                target[0] = String::from("#source-v1/");
                owner[0] = crate::embedded::report::tests::key("target-v1", &target);
                endpoint[4] = crate::embedded::report::tests::key("owner-v1", &owner);
            }
            "escaping" => {
                let altered = owner[0].replace("%2F", "%2f");
                assert_ne!(altered, owner[0]);
                owner[0] = altered;
                endpoint[4] = crate::embedded::report::tests::key("owner-v1", &owner);
            }
            "empty-trait" | "duplicate-trait" => {
                owner[1] = String::from("trait-impl-method");
                owner[3] = crate::embedded::report::tests::key("present", &[String::from("S")]);
                owner[4] =
                    crate::embedded::report::tests::key("present", &[String::from("WrittenTrait")]);
                let mut candidates = vec![String::from("WrittenTrait")];
                if refusal == "duplicate-trait" {
                    candidates.extend([String::from("some::Trait"), String::from("some::Trait")]);
                }
                owner[5] = crate::embedded::report::tests::key("unresolved", &candidates);
                endpoint[4] = crate::embedded::report::tests::key("owner-v1", &owner);
                let mut valid = owner.clone();
                valid[5] = crate::embedded::report::tests::key(
                    "unresolved",
                    &[String::from("WrittenTrait"), String::from("some::Trait")],
                );
                let mut neighbor = endpoint.clone();
                neighbor[4] = crate::embedded::report::tests::key("owner-v1", &valid);
                assert!(
                    identity::validate_endpoint(
                        &crate::embedded::report::tests::key("endpoint-v1", &neighbor),
                        meter
                    )
                    .is_ok()
                );
            }
            "repository" | "git-query" | "git-fragment" | "registry-fragment" => {
                corrupt_repository_descriptor(
                    refusal,
                    &mut target,
                    &mut owner,
                    &mut endpoint,
                    meter,
                );
            }
            _ => unreachable!(),
        }
        let malformed = crate::embedded::report::tests::key("endpoint-v1", &endpoint);
        if identity::validate_endpoint(&malformed, meter).is_ok() {
            accepted.push(refusal);
        }
        assert!(identity::validate_endpoint(descriptor, meter).is_ok());
    }
    assert!(
        accepted.is_empty(),
        "malformed nested policy keys were accepted: {accepted:?}"
    );
}

fn corrupt_repository_descriptor(
    refusal: &str,
    target: &mut [String],
    owner: &mut [String],
    endpoint: &mut [String],
    meter: &Meter,
) {
    let git = refusal.starts_with("git-");
    let repository = match refusal {
        "repository" => "relative/source",
        "git-query" => "https://example.invalid/repo?branch=other",
        _ => "https://example.invalid/repo#fragment",
    };
    let mut source = vec![
        String::from("structural-observation"),
        target[2].clone(),
        String::from("1.0.0"),
        String::from(if git { "git" } else { "registry" }),
        repository.into(),
        String::from(if git { "default-branch" } else { "absent" }),
        String::new(),
        String::from("absent"),
        String::new(),
    ];
    target[0] = crate::embedded::report::tests::key("source-v1", &source);
    owner[0] = crate::embedded::report::tests::key("target-v1", target);
    endpoint[4] = crate::embedded::report::tests::key("owner-v1", owner);
    source[4] = String::from("https://example.invalid/repo");
    let mut valid = target.to_vec();
    valid[0] = crate::embedded::report::tests::key("source-v1", &source);
    let mut valid_owner = owner.to_vec();
    valid_owner[0] = crate::embedded::report::tests::key("target-v1", &valid);
    let mut neighbor = endpoint.to_vec();
    neighbor[4] = crate::embedded::report::tests::key("owner-v1", &valid_owner);
    assert!(
        identity::validate_endpoint(
            &crate::embedded::report::tests::key("endpoint-v1", &neighbor),
            meter
        )
        .is_ok()
    );
}

#[test]
fn descriptors_and_debt_tables_share_exact_grants_and_release_failed_projection_storage() {
    fn project<'m>(
        meter: &'m Meter,
        lexical: &[&str],
    ) -> Result<(Entries<'m>, Text<'m>), RefusalKind> {
        let descriptor =
            Text::new(meter, "#endpoint-v1/component%2Fowner").map_err(RefusalKind::from)?;
        let mut entries = Entries::new(meter);
        assert!(
            entries
                .insert([1; 32], &descriptor, &descriptor, 2)
                .map_err(|cause| fixture::resource(&cause))?
        );
        assert!(
            !entries
                .insert([1; 32], &descriptor, &descriptor, 9)
                .map_err(|cause| fixture::resource(&cause))?
        );
        assert_eq!(
            entries
                .get(&[1; 32], &descriptor, &descriptor)?
                .unwrap()
                .get(),
            2
        );
        // The retained table stays live while native render scratch and the
        // canonical output borrow the same original run's aggregate account.
        let text = render(meter, lexical, Format::NTriples)?;
        Ok((entries, text))
    }
    const LEXICAL: &[&str] = &["shared prefix a", "shared prefix z", "é\n\0\"\\"];
    let mut builder = Builder::new();
    for &lexical in LEXICAL {
        builder
            .insert(raf_rdf::Triple {
                subject: Term::iri(""),
                predicate: Term::iri("urn:predicate"),
                object: Term::simple_literal(lexical),
            })
            .unwrap();
    }
    let graph = builder.finish().unwrap();
    let probe = Meter::new(usize::MAX, usize::MAX);
    let (entries, text) = project(&probe, LEXICAL).unwrap();
    assert_eq!(&*text, graph.to_text(Format::NTriples).unwrap());
    drop((entries, text));
    let (work, peak) = probe.usage();
    let exact = Meter::new(work, peak);
    drop(project(&exact, LEXICAL).unwrap());
    assert_eq!(exact.usage(), (work, peak));
    let short_work = Meter::new(work - 1, peak);
    assert!(matches!(
        project(&short_work, LEXICAL),
        Err(RefusalKind::Work { .. })
    ));
    let short_storage = Meter::new(usize::MAX, peak - 1);
    assert!(matches!(
        project(&short_storage, LEXICAL),
        Err(RefusalKind::Memory { .. })
    ));
    // Failure destroyed every partial descriptor/table before releasing grants.
    // The same original account can subsequently retain its small valid neighbour.
    let neighbour = Text::new(&short_storage, "valid").unwrap();
    drop(neighbour);
    drop(Text::new(&short_storage, "valid").unwrap());
    for _ in 0..2 {
        drop(render(&short_storage, &["valid"], Format::NTriples).unwrap());
    }
    // A surviving output must consume its original account during another
    // render; calibrating only whole-operation peaks cannot prove that lifetime.
    for format in [Format::Turtle, Format::NTriples] {
        let output_probe = Meter::new(usize::MAX, usize::MAX);
        drop(render(&output_probe, LEXICAL, format).unwrap());
        let (_, output_peak) = output_probe.usage();
        assert!(output_peak > 0);
        let output_account = Meter::new(usize::MAX, output_peak);
        let held = render(&output_account, LEXICAL, format).unwrap();
        assert!(matches!(
            render(&output_account, LEXICAL, format),
            Err(RefusalKind::Memory { .. })
        ));
        drop(held);
        drop(render(&output_account, LEXICAL, format).unwrap());
    }
}

#[test]
fn debt_requires_the_selected_run_and_import_restores_exact_comparison() {
    let workspace = load(&repeated(2));
    let first = analyze(&workspace);
    let second = analyze(&workspace);
    let record = Record::adopt(&first).unwrap();
    assert!(matches!(
        record.compare(&second),
        Err(Failure::AccountMismatch)
    ));
    assert!(matches!(
        record.rewrite_existing(&second),
        Err(Failure::AccountMismatch)
    ));
    let imported = in_run(&record, &second);
    assert_eq!(imported.compare(&second).unwrap(), Comparison::Unchanged);
}
