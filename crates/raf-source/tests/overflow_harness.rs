// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shapes that overflowed the parse stack before the recursion budget counted
//! them are judged in a child process. Each must end in a typed refusal
//! naming the file and the budget; an abort, a wrong index or any other
//! failure fails the fixture. A child can still overflow on purpose, through a
//! visitor that recurses on the parse thread, which shows what an overflow
//! looks like.

mod support;

use std::panic;
use std::thread;

use raf_source::{Budget, PARSE_STACK_BYTES, RelPath, parse_source};
use support::child::{Outcome, classify, problem, run_child, run_child_if_requested, selector};
use support::shapes;

/// The measured stack cost of one nesting level of the `return` chain, in
/// bytes, taken from an optimised build (the cheaper of the two profiles):
/// the parse of `return return ...` first overflowed 64 MiB between 18143 and
/// 18218 levels, which is about 3700 bytes a level. A debug build costs about
/// 8800.
const RETURN_LEVEL_BYTES: usize = 3_700;

/// Levels that need twice the parse stack even at the cheaper cost.
const OVERFLOW_LEVELS: usize = 2 * PARSE_STACK_BYTES / RETURN_LEVEL_BYTES;

/// Levels that fit in a small part of the stack in either profile.
const SAFE_LEVELS: usize = 1_000;

/// The child that overflows on purpose: its visitor recurses on the parse
/// thread until the stack is gone.
const OVERFLOW_VISITOR: &str = "overflow_visitor";

/// Recurse without end; each frame keeps a kilobyte the optimiser cannot drop.
fn descend(depth: usize) -> usize {
    let pad = std::hint::black_box([0_u8; 1024]);
    if depth == usize::MAX {
        return 0;
    }
    descend(depth + 1).wrapping_add(usize::from(pad[depth % 1024]))
}

fn return_chain(levels: usize) -> String {
    format!("fn f() {{ {}; }}\n", "return ".repeat(levels))
}

fn summary(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Aborted { .. } => "aborted".to_owned(),
        other => format!("{other:?}"),
    }
}

fn text_of(selector: &str) -> String {
    match selector {
        "overflow_return" => return_chain(OVERFLOW_LEVELS),
        "neighbour_return" => return_chain(SAFE_LEVELS),
        // Children that misbehave on purpose, to test the judge.
        "fixture_panic" => panic!("deliberate panic in the child"),
        "fixture_exit_three" => std::process::exit(3),
        name => {
            if let Some(text) = family_text(name) {
                return text;
            }
            let Some(shape) = shapes::all().into_iter().find(|shape| shape.name == name) else {
                panic!("no shape named {name}");
            };
            shape.text
        }
    }
}

/// The text a `family_<name>_at` or `family_<name>_over` selector names.
fn family_text(selector: &str) -> Option<String> {
    let rest = selector.strip_prefix("family_")?;
    let (name, over) = match rest.strip_suffix("_over") {
        Some(name) => (name, true),
        None => (rest.strip_suffix("_at")?, false),
    };
    let family = shapes::families()
        .into_iter()
        .find(|family| family.name == name)?;
    Some((family.text)(family.at + usize::from(over)))
}

/// Selected by the parent process; an ordinary run of it does nothing.
#[test]
fn child_main() {
    if selector().as_deref() == Some(OVERFLOW_VISITOR) {
        let file = RelPath::new(&format!("{OVERFLOW_VISITOR}.rs")).expect("path");
        let depth = parse_source(&file, "fn f() {}", |_| descend(0));
        panic!("the visitor returned {depth:?} instead of overflowing");
    }
    run_child_if_requested(text_of);
}

/// The budget that refuses each overflow shape: the recursion budget, except
/// for a chain of closure heads with `move` between them, which the closure
/// bar budget meets first.
fn refusing_budget(name: &str) -> Budget {
    if name == "uncounted_move_closure" {
        Budget::ClosureBars
    } else {
        Budget::Recursion
    }
}

#[test]
fn every_committed_overflow_shape_is_refused_by_a_budget_and_none_aborts() {
    let judged: Vec<_> = shapes::composites()
        .into_iter()
        .chain(shapes::uncounted())
        .chain(shapes::chains())
        .collect();
    assert_eq!(judged.len(), 35);
    for shape in judged {
        let outcome = run_child("child_main", shape.name);
        eprintln!("{}: {}", shape.name, summary(&outcome));
        let file = format!("{}.rs", shape.name);
        assert_eq!(
            problem(&outcome, &file, shape.symbols),
            None,
            "{}",
            shape.name
        );
        assert_eq!(
            outcome,
            Outcome::Refused {
                file,
                budget: refusing_budget(shape.name).name().to_owned(),
            },
            "{}",
            shape.name
        );
    }
}

#[test]
fn every_shape_family_loads_at_its_largest_admitted_size_and_one_size_more_is_refused() {
    let families = shapes::families();
    assert_eq!(families.len(), 39);
    for family in families {
        let at = format!("family_{}_at", family.name);
        let symbols: Vec<String> = family.symbols.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(
            run_child("child_main", &at),
            Outcome::Index(symbols),
            "{at}"
        );
        let over = format!("family_{}_over", family.name);
        assert_eq!(
            run_child("child_main", &over),
            Outcome::Refused {
                file: format!("{over}.rs"),
                budget: family.over.name().to_owned(),
            },
            "{over}"
        );
    }
}

#[test]
fn a_shape_just_inside_its_budgets_exits_zero_with_the_exact_symbols() {
    let outcome = run_child("child_main", "budget_closure_bars_at");
    assert_eq!(outcome, Outcome::Index(vec!["f".to_owned()]));
    let outcome = run_child("child_main", "budget_closure_bars_over");
    assert!(
        matches!(&outcome, Outcome::Refused { file, budget }
            if file == "budget_closure_bars_over.rs" && budget == "closure bar chain"),
        "{outcome:?}"
    );
}

#[test]
fn a_deliberate_overflow_aborts_and_the_runtime_names_the_parse_thread() {
    let outcome = run_child("child_main", OVERFLOW_VISITOR);
    let Outcome::Aborted { detail } = &outcome else {
        panic!("expected an abort, got {outcome:?}");
    };
    // The runtime prints the thread name, then its id, then the verdict.
    assert!(
        detail.lines().any(|line| {
            line.starts_with(&format!("thread '{OVERFLOW_VISITOR}.rs'"))
                && line.ends_with("has overflowed its stack")
        }),
        "{detail}"
    );
    // An abort fails the judge.
    assert!(problem(&outcome, &format!("{OVERFLOW_VISITOR}.rs"), &["f"]).is_some());
}

#[test]
fn a_return_chain_that_needs_twice_the_stack_is_refused_beside_one_at_a_safe_depth() {
    assert_eq!(
        run_child("child_main", "overflow_return"),
        Outcome::Refused {
            file: "overflow_return.rs".to_owned(),
            budget: Budget::Recursion.name().to_owned(),
        }
    );
    assert_eq!(
        run_child("child_main", "neighbour_return"),
        Outcome::Index(vec!["f".to_owned()])
    );
}

#[test]
fn the_judge_rejects_a_wrong_index_a_misnamed_refusal_an_abort_and_a_stray_failure() {
    let wrong = Outcome::Index(vec!["g".to_owned()]);
    assert!(problem(&wrong, "x.rs", &["f"]).is_some());
    let partial = Outcome::Index(Vec::new());
    assert!(problem(&partial, "x.rs", &["f"]).is_some());
    let misnamed = Outcome::Refused {
        file: "y.rs".to_owned(),
        budget: "nested `<`".to_owned(),
    };
    assert!(problem(&misnamed, "x.rs", &[]).is_some());
    let unnamed = Outcome::Refused {
        file: "x.rs".to_owned(),
        budget: String::new(),
    };
    assert!(problem(&unnamed, "x.rs", &[]).is_some());
    let failed = Outcome::Failed {
        message: "syntax".to_owned(),
    };
    assert!(problem(&failed, "x.rs", &[]).is_some());

    let abort = Outcome::Aborted {
        detail: String::new(),
    };
    assert!(problem(&abort, "x.rs", &["f"]).is_some());

    // Each allowed outcome passes beside them.
    assert_eq!(
        problem(&Outcome::Index(vec!["f".to_owned()]), "x.rs", &["f"]),
        None
    );
    let refusal = Outcome::Refused {
        file: "x.rs".to_owned(),
        budget: "nested `<`".to_owned(),
    };
    assert_eq!(problem(&refusal, "x.rs", &["f"]), None);
}

#[test]
fn the_parse_thread_is_named_after_the_file_and_has_the_stated_stack() {
    assert_eq!(PARSE_STACK_BYTES, 64 * 1024 * 1024);
    let file = RelPath::new("src/deep/named.rs").expect("path");
    let name = parse_source(&file, "fn f() {}", |_| {
        thread::current().name().map(str::to_owned)
    })
    .expect("parses");
    assert_eq!(name.as_deref(), Some("src/deep/named.rs"));
    assert_ne!(thread::current().name().map(str::to_owned), name);
}

#[test]
fn a_panic_in_the_visitor_is_raised_again_with_its_payload() {
    let file = RelPath::new("src/lib.rs").expect("path");
    let caught = panic::catch_unwind(|| {
        parse_source(&file, "fn f() {}", |_| -> () { panic!("visitor failed") })
    });
    let payload = caught.expect_err("the panic crosses the thread");
    assert_eq!(
        payload.downcast_ref::<&str>().copied(),
        Some("visitor failed")
    );
    // The same call without the panic returns normally.
    assert_eq!(
        parse_source(&file, "fn f() {}", |tree| tree.items.len()).expect("parses"),
        1
    );
}

#[test]
fn deep_but_admitted_source_parses_on_the_dedicated_stack_not_the_callers() {
    // 1000 nested returns cost more than half of the caller's 2 MiB test
    // stack in a debug build, so this only passes because the parse runs on
    // the dedicated thread.
    let file = RelPath::new("src/deep.rs").expect("path");
    let items = parse_source(&file, &return_chain(SAFE_LEVELS), |tree| tree.items.len());
    assert_eq!(items.expect("parses"), 1);
}

#[test]
fn a_child_that_panics_is_a_failure_and_not_an_abort() {
    let outcome = run_child("child_main", "fixture_panic");
    assert!(matches!(outcome, Outcome::Failed { .. }), "{outcome:?}");
    assert!(problem(&outcome, "x.rs", &[]).is_some());
}

#[test]
fn a_child_with_an_unknown_selector_is_a_failure() {
    let outcome = run_child("child_main", "no_such_shape");
    assert!(matches!(outcome, Outcome::Failed { .. }), "{outcome:?}");
    assert!(problem(&outcome, "x.rs", &[]).is_some());
}

#[test]
fn a_child_that_exits_three_without_a_result_line_is_a_failure() {
    let outcome = run_child("child_main", "fixture_exit_three");
    assert!(matches!(outcome, Outcome::Failed { .. }), "{outcome:?}");
    assert!(problem(&outcome, "x.rs", &[]).is_some());
}

#[test]
fn the_classifier_accepts_only_a_signal_a_stack_overflow_status_or_the_runtime_message_as_an_abort()
{
    let abort = |code, signaled, stderr: &str| classify(code, signaled, "", stderr);
    // Abnormal ends that are aborts.
    assert!(matches!(abort(None, true, ""), Outcome::Aborted { .. }));
    assert!(matches!(
        abort(
            Some(134),
            false,
            "thread 'a.rs' (1) has overflowed its stack\n"
        ),
        Outcome::Aborted { .. }
    ));
    assert!(matches!(
        abort(Some(0xC000_00FD_u32.cast_signed()), false, ""),
        Outcome::Aborted { .. }
    ));
    // Ends that are not.
    assert!(matches!(
        abort(Some(101), false, "panicked"),
        Outcome::Failed { .. }
    ));
    assert!(matches!(abort(Some(1), false, ""), Outcome::Failed { .. }));
    assert!(matches!(abort(Some(3), false, ""), Outcome::Failed { .. }));
    assert!(matches!(abort(Some(4), false, ""), Outcome::Failed { .. }));
    assert!(matches!(abort(Some(0), false, ""), Outcome::Failed { .. }));
    // A signal after a result line is not an abort of this parse either.
    assert!(matches!(
        classify(None, true, "RAF_CHILD index f\n", ""),
        Outcome::Failed { .. }
    ));
    // The allowed non-abort outcomes still classify.
    assert_eq!(
        classify(Some(0), false, "test x ... RAF_CHILD index a,b\n", ""),
        Outcome::Index(vec!["a".to_owned(), "b".to_owned()])
    );
    assert_eq!(
        classify(Some(3), false, "RAF_CHILD refused x.rs nested `<`\n", ""),
        Outcome::Refused {
            file: "x.rs".to_owned(),
            budget: "nested `<`".to_owned()
        }
    );
}
