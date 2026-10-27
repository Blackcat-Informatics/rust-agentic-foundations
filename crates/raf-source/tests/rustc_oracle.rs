// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The index against `rustc`: generated library crates of several modules,
//! with private, `pub` and `pub(crate)` globs, named and glob re-exports,
//! shadowing and a struct and a function of one name, are asked the same
//! question by both. A question, a probe, is a `use` of a path as seen from a
//! chosen module; a path starts with `crate`, `self`, `super`, or a bare name,
//! the name of a module the reading module holds or its globs bring in. `rustc` accepts or refuses it, and when it accepts, a second
//! compile tells which item it names; the index answers through
//! `Workspace::resolve_from_in`, through the actual generated import alias.
//! Every disagreement is classified, and one
//! that matches no known class fails the test.
//!
//! Cyclic or indeterminate imports are compared without a tolerated cycle
//! class. A fresh lookup of the original source path can succeed after an
//! actual import of that path failed to become determinate; comparing that
//! fresh lookup would ask a different question from the compiler's probe.
//!
//! Whether `rustc` answers a probe differently when the declarations come in
//! another order, in a name space where the two sides disagree, is asked for
//! every kind of disagreement, whichever way the index errs, after the
//! documented class. Order classification requires independently observed
//! compiler variation and an index answer represented by an observed outcome.
//! An unobserved target, budget failure or index-order instability remains a
//! failure. Compiler observations ask one namespace at a time, using a type
//! alias or function expression, and include ambiguity of an original import
//! only when its source or bound local name belongs to that probe. Unrelated
//! source errors give no observable outcome.
//!
//! The committed programs under `fuzz/corpus/rustc-oracle` are replayed. The
//! generated programs come from eight seeds, `base + 1` to `base + 8`. The
//! default run draws [`oracle::DEFAULT_PROGRAMS`] programs from each of two bases,
//! `0xa11ce00` and `0xc0ffee00`. `RAF_ORACLE_SEED_BASE` (hexadecimal with a
//! `0x` prefix, or decimal) selects one other base and the run then draws
//! only from it, and `RAF_ORACLE_PROGRAMS` sets the number of programs of the
//! full run: of the base selected, or of `0xa11ce00` when none is.
//!
//! To compare another resolver by program: copy `tests/` of this crate and
//! the corpus under `fuzz/corpus/rustc-oracle` into a clone of the older
//! commit, which must have the API the tests call, and run
//! `RAF_ORACLE_LIST=1 RAF_ORACLE_PROGRAMS=n cargo test -p raf-source --release
//! --test rustc_oracle -- --nocapture` in both trees. Every divergence,
//! those of the known classes included, is then printed as
//! `seed 0x.. program N: <class> probe <i> <path> in <module>`. The programs
//! at one seed and number are the same text on both sides, so the sorted
//! lines can be compared with `diff`; the run of the older tree may fail on
//! classes that tree does not model, and its output is still complete.

mod support;

#[allow(
    dead_code,
    reason = "the shared maintenance implementation also supplies the explicit generator example"
)]
#[path = "support/oracle.rs"]
mod oracle;

use oracle::{
    Answer, Class, KNOWN, Kind, LIST_VARIABLE, MOST_REDUCED, ORDERS_TRIED, PROGRAMS_VARIABLE,
    SEED_BASE_VARIABLE, assert_corpus_writer, corpus, describe, describe_unstable, evaluate,
    evaluate_all, evaluate_in, evaluate_with, families, generated, minimise, minimise_unstable,
    parse_seed_base, seeds_of,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use support::program::Program;
use support::rustc;

/// Programs written out, for the comparator tests.
fn fixture(text: &str) -> Program {
    Program::parse(text).expect("a fixture parses")
}

const TWO_STRUCTS: &str = "\
// probe demo crate::m0::A
// probe demo crate::m1::A
// probe demo crate::m0::Hidden
mod m0 {
pub struct A {} impl A { pub const TAG: u32 = 1; }
struct Hidden {} impl Hidden { pub const TAG: u32 = 3; }
}
mod m1 {
pub struct A {} impl A { pub const TAG: u32 = 2; }
}
";

#[test]
fn the_index_agrees_with_rustc_on_generated_programs_up_to_the_known_classes() {
    let programs = generated();
    let total = programs.len();
    assert!(total > 0, "{PROGRAMS_VARIABLE} names no programs");
    let results = evaluate_all(&programs);
    let mut by_class: BTreeMap<Class, usize> = BTreeMap::new();
    let (mut valid, mut accepted, mut rejected) = (0, 0, 0);
    let mut failures = String::new();
    let listing = std::env::var(LIST_VARIABLE).is_ok_and(|value| value == "1");
    for ((label, _), result) in programs.iter().zip(&results) {
        valid += usize::from(result.valid);
        accepted += result.accepted;
        rejected += result.rejected;
        for divergence in &result.divergences {
            *by_class.entry(divergence.class).or_default() += 1;
            if listing {
                let probe = &result.program.probes[divergence.probe];
                println!(
                    "{label}: {} probe {} {} in {}",
                    divergence.class.name(),
                    divergence.probe,
                    probe.path,
                    probe.module
                );
            }
            if !KNOWN.contains(&divergence.class) {
                writeln!(
                    failures,
                    "{label}: {}",
                    describe(&result.program, divergence)
                )
                .expect("write");
            }
        }
        for unstable in &result.unstable {
            *by_class.entry(Class::IndexOrder).or_default() += 1;
            if listing {
                match unstable.probe {
                    Some(at) => println!(
                        "{label}: index-order probe {at} {} in {}",
                        result.program.probes[at].path, result.program.probes[at].module
                    ),
                    None => println!("{label}: index-order surface"),
                }
            }
            // Only the first few are reduced, which costs a compile for each
            // step.
            let reduced = if by_class[&Class::IndexOrder] <= MOST_REDUCED {
                format!(
                    "reduced:\n{}",
                    minimise_unstable(&result.program, unstable.probe).to_text()
                )
            } else {
                String::new()
            };
            writeln!(
                failures,
                "{label}: {}{}\n{reduced}",
                describe_unstable(&result.program, unstable),
                result.program.to_text()
            )
            .expect("write");
        }
    }
    println!("rustc: {}", rustc::version());
    for (base, count) in families() {
        println!(
            "seeds: {} ({count} programs)",
            seeds_of(base)
                .iter()
                .map(|seed| format!("{seed:#x}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    println!("programs: {total} ({valid} judged)");
    println!(
        "probes: {} ({accepted} accepted, {rejected} refused)",
        accepted + rejected
    );
    for class in [
        Class::Rule1,
        Class::Rule2,
        Class::Documented,
        Class::Order,
        Class::Cycle,
        Class::IndexOrder,
        Class::Unclassified,
    ] {
        println!(
            "divergences {}: {}",
            class.name(),
            by_class.get(&class).copied().unwrap_or(0)
        );
    }
    assert!(valid > 0, "no program was judged");
    assert!(accepted > 0, "no probe was accepted by rustc");
    assert!(rejected > 0, "no probe was refused by rustc");
    assert!(failures.is_empty(), "unlisted divergences:\n{failures}");
}

#[test]
fn every_corpus_program_shows_the_divergence_it_records_and_no_other() {
    oracle::assert_namespace_cycles_have_no_tolerated_determinacy_disagreement();
    oracle::assert_the_oracle_compares_the_import_it_rendered_instead_of_a_fresh_path_read();
    let entries = corpus();
    assert!(!entries.is_empty(), "the corpus holds no program");
    let programs: Vec<(String, Program)> = entries;
    let results = evaluate_all(&programs);
    let mut divergent = 0;
    let (mut agreeing_accepts, mut agreeing_refusals) = (0, 0);
    for ((name, program), result) in programs.iter().zip(&results) {
        assert!(result.valid, "{name}: rustc refuses the crate");
        assert!(
            result.unstable.is_empty(),
            "{name}: the index answers differently in another order\n{}",
            describe_unstable(&result.program, &result.unstable[0])
        );
        assert!(
            result.accepted + result.rejected > 0,
            "{name}: it holds no probe"
        );
        let expect = program
            .expect
            .as_deref()
            .unwrap_or_else(|| panic!("{name}: no `// oracle:` line"));
        if expect == "agree" {
            assert!(
                result.divergences.is_empty(),
                "{name}: expected agreement\n{}",
                describe(&result.program, &result.divergences[0])
            );
            agreeing_accepts += result.accepted;
            agreeing_refusals += result.rejected;
            continue;
        }
        let class =
            Class::named(expect).unwrap_or_else(|| panic!("{name}: unknown class {expect}"));
        assert!(
            KNOWN.contains(&class),
            "{name}: the class {expect} is no longer known; remove the entry or record it as agreeing"
        );
        divergent += 1;
        assert!(
            result.divergences.iter().any(|d| d.class == class),
            "{name}: it no longer diverges as {expect}\n{}",
            program.to_text()
        );
        for divergence in &result.divergences {
            assert_eq!(
                divergence.class,
                class,
                "{name}: an unlisted divergence\n{}",
                describe(&result.program, divergence)
            );
        }
    }
    assert!(divergent > 0, "the corpus records no divergence");
    assert!(
        agreeing_accepts > 0 && agreeing_refusals > 0,
        "the agreeing entries hold probes that rustc accepts and probes that it refuses"
    );
    for class in KNOWN {
        assert!(
            programs
                .iter()
                .any(|(_, program)| program.expect.as_deref() == Some(class.name())),
            "the corpus holds no program of the class {}",
            class.name()
        );
    }
}

#[test]
fn a_wrong_index_answer_is_reported_beside_the_right_one_which_is_not() {
    oracle::assert_resource_failures_cannot_agree_with_compiler_refusal_or_hide_beside_documented_ambiguity();
    let program = fixture(TWO_STRUCTS);
    let right = evaluate(&program);
    assert!(right.valid);
    assert!(right.divergences.is_empty(), "{:?}", right.divergences);
    assert!(right.unstable.is_empty(), "{:?}", right.unstable);
    assert_eq!((right.accepted, right.rejected), (2, 1));

    // The index names another item.
    let other = evaluate_with(&program, &|mut answer| {
        if let Answer::Item { tag, .. } = &mut answer.ty {
            *tag = if *tag == 1 { 2 } else { 1 };
        }
        answer
    });
    let kinds: Vec<(usize, Kind)> = other
        .divergences
        .iter()
        .map(|d| (d.probe, d.kind))
        .collect();
    assert_eq!(kinds, [(0, Kind::OtherItem), (1, Kind::OtherItem)]);

    // The index refuses what rustc accepts.
    let refusing = evaluate_with(&program, &|mut answer| {
        if answer.ty.tag().is_some() {
            answer.ty = Answer::Ambiguous;
        }
        answer
    });
    assert_eq!(refusing.divergences.len(), 2);
    assert!(
        refusing
            .divergences
            .iter()
            .all(|d| d.kind == Kind::AcceptedButRefused)
    );

    // The index resolves what rustc refuses.
    let resolving = evaluate_with(&program, &|mut answer| {
        if answer.ty == Answer::Missing {
            answer.ty = Answer::Item {
                tag: 3,
                chain: Vec::new(),
            };
        }
        answer
    });
    let kinds: Vec<(usize, Kind)> = resolving
        .divergences
        .iter()
        .map(|d| (d.probe, d.kind))
        .collect();
    assert_eq!(kinds, [(2, Kind::RefusedButResolves)]);
}

#[test]
fn a_divergence_that_matches_no_class_is_unclassified() {
    let program = fixture(TWO_STRUCTS);
    let wrong = evaluate_with(&program, &|mut answer| {
        if let Answer::Item { tag, .. } = &mut answer.ty {
            *tag += 10;
        }
        answer
    });
    assert!(!wrong.divergences.is_empty());
    assert!(
        wrong
            .divergences
            .iter()
            .all(|d| d.class == Class::Unclassified && !KNOWN.contains(&d.class))
    );
}

#[test]
fn the_minimiser_drops_what_a_divergence_does_not_need_and_keeps_its_class() {
    let entries = corpus();
    let (name, entry) = entries
        .iter()
        .find(|(name, _)| name == "glob_reexport_ambiguity_warned.rs")
        .expect("the explicit glob re-export ambiguity fixture exists");
    let class = Class::named(entry.expect.as_deref().expect("expectation")).expect("a class");
    assert_eq!(class, Class::Documented);
    assert!(
        evaluate(entry)
            .divergences
            .iter()
            .any(|divergence| divergence.class == class),
        "the original input reproduces its divergence"
    );
    let mut padded = entry.clone();
    padded.lines.push("mod spare {".to_owned());
    padded
        .lines
        .push("pub struct Z {} impl Z { pub const TAG: u32 = 9000; }".to_owned());
    padded.lines.push("}".to_owned());
    padded
        .lines
        .insert(0, "const fn Spare() -> u32 { 9001 }".to_owned());
    let evaluation = evaluate(&padded);
    let at = evaluation
        .divergences
        .iter()
        .find(|d| d.class == class)
        .unwrap_or_else(|| panic!("{name}: the padded program no longer diverges"))
        .probe;
    let smaller = minimise(&evaluation.program, at, &|d| d.class == class);
    assert!(
        smaller.lines.len() <= entry.lines.len(),
        "{name}: not reduced to the stored size\n{}",
        smaller.to_text()
    );
    assert!(
        !smaller
            .lines
            .iter()
            .any(|line| line.contains("Spare") || line.contains("spare"))
    );
    assert!(
        evaluate(&smaller)
            .divergences
            .iter()
            .any(|d| d.class == class)
    );
}

#[test]
fn write_corpus() {
    assert_corpus_writer();
}

#[test]
fn an_index_answer_that_changes_with_the_order_is_reported_as_index_order() {
    let program = fixture(TWO_STRUCTS);
    // Neither a sampled shuffle nor deterministic import reversal can hide
    // a different index answer behind a compiler-order exception.
    for changed in [3, ORDERS_TRIED + 1] {
        let changing = evaluate_in(&program, &|answer| answer, &|order, mut answer| {
            if order == changed
                && let Answer::Item { tag, .. } = &mut answer.ty
            {
                *tag = if *tag == 1 { 2 } else { 1 };
            }
            answer
        });
        assert!(
            changing.divergences.is_empty(),
            "{:?}",
            changing.divergences
        );
        let probes: Vec<Option<usize>> = changing.unstable.iter().map(|u| u.probe).collect();
        assert_eq!(probes, [Some(0), Some(1)]);
        let orders: Vec<usize> = changing.unstable[0]
            .answers
            .iter()
            .map(|(order, _)| *order)
            .collect();
        assert_eq!(orders, [0, changed]);
        let report = describe_unstable(&changing.program, &changing.unstable[0]);
        assert!(
            report.starts_with("index-order: probe 0 `crate::m0::A` in demo\n"),
            "{report}"
        );
        assert!(
            report.contains("as written")
                && report.contains(if changed == 3 {
                    "shuffle 3"
                } else {
                    "reversed imports"
                }),
            "{report}"
        );
    }
    assert!(!KNOWN.contains(&Class::IndexOrder));
    assert_eq!(Class::named("index-order"), None);
}

#[test]
fn a_probe_rustc_answers_by_the_order_is_not_order_when_the_index_does_too() {
    oracle::assert_an_ambiguous_original_import_is_an_order_witness_only_for_its_own_name();
    oracle::assert_compiler_order_dependence_does_not_hide_an_unobserved_target_or_resource_refusal(
    );
    let entries = corpus();
    let (name, entry) = entries
        .iter()
        .find(|(name, _)| name == "glob_order_changes_what_rustc_accepts.rs")
        .expect("the corpus holds the compiler-order witness");
    let stable = evaluate(entry);
    assert!(stable.unstable.is_empty(), "{name}");
    assert!(
        stable.divergences.iter().any(|d| d.class == Class::Order),
        "{name}: it no longer diverges as order"
    );
    // The index answers every probe otherwise in the first shuffle.
    let varying = evaluate_in(entry, &|answer| answer, &|order, mut answer| {
        if order == 1 {
            answer.ty = Answer::Cyclic;
        }
        answer
    });
    assert!(!varying.unstable.is_empty(), "{name}");
    assert!(!varying.divergences.is_empty(), "{name}");
    for divergence in &varying.divergences {
        assert_ne!(divergence.class, Class::Order, "{name}");
    }
}

#[test]
fn the_seed_base_is_read_in_hexadecimal_or_decimal_and_refuses_anything_else() {
    support::program::assert_item_tags_preserve_unsigned_spelling_and_u32_bounds();
    assert_eq!(parse_seed_base("0xa11ce00"), Ok(0x0a11_ce00));
    assert_eq!(parse_seed_base(" 0XC0FF_EE00 "), Ok(0xc0ff_ee00));
    assert_eq!(parse_seed_base("1000"), Ok(1000));
    for (text, value) in [
        ("+001", 1),
        ("0x+1", 1),
        ("0X+01", 1),
        ("0_x_+_1", 1),
        ("0x00000000000000000000001", 1),
        ("0x00000000000000000000000", 0),
        ("0xFFFFFFFFFFFFFFF7", u64::MAX - 8),
        ("18446744073709551607", u64::MAX - 8),
    ] {
        assert_eq!(parse_seed_base(text), Ok(value), "{text}");
    }
    assert_eq!(seeds_of(0x0a11_ce00)[0], 0x0a11_ce01);
    assert_eq!(seeds_of(0x0a11_ce00)[7], 0x0a11_ce08);
    for bad in [
        "",
        "0x",
        "0x+",
        "0x++1",
        "0x-1",
        "0xzz",
        "0xé",
        "seed",
        "-1",
        "12ab",
        "0xffffffffffffffff",
        "0x10000000000000000",
        "18446744073709551616",
        "18446744073709551608",
    ] {
        let error = parse_seed_base(bad).expect_err(bad);
        assert!(
            error.contains(SEED_BASE_VARIABLE) && error.contains("is not a number")
                || error.contains("no room"),
            "{error}"
        );
    }
}

#[test]
fn diagnostic_levels_are_required_beside_a_valid_null_code() {
    use purrdf_lex::json::{self, Value};
    let valid = json::read(r#"{"level":"error","code":null}"#).unwrap();
    assert_eq!(rustc::diagnostic_level(&valid), "error");
    assert_eq!(valid.get("code"), Some(&Value::Null));
    for text in [r#"{"code":null}"#, r#"{"level":null}"#, r#"{"level":2}"#] {
        let value = json::read(text).unwrap();
        assert!(std::panic::catch_unwind(|| rustc::diagnostic_level(&value)).is_err());
    }
}
