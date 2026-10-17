// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The index against `rustc`: generated library crates of several modules,
//! with private, `pub` and `pub(crate)` globs, named and glob re-exports,
//! shadowing and a struct and a function of one name, are asked the same
//! question by both. A question, a probe, is a `use` of a path as seen from a
//! chosen module. `rustc` accepts or refuses it, and when it accepts, a second
//! compile tells which item it names; the index answers through
//! `Workspace::resolve_from_in`. Every disagreement is classified, and one
//! that matches no known class fails the test.
//!
//! The committed programs under `fuzz/corpus/rustc-oracle` are replayed with
//! a fixed list of seeds. `RAF_ORACLE_PROGRAMS` raises the number of
//! generated programs for the full run.

mod support;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use raf_source::{MemoryTree, Namespace, ResolveError, Workspace};
use support::program::{CRATE, Probe, Program, Space, random_program};
use support::random::Draw;
use support::rustc::{self, Judged, Verdict};

/// The seeds of the default run. The programs of a seed are drawn one after
/// the other from a generator of that seed.
const SEEDS: [u64; 8] = [
    0x0a11_ce01,
    0x0a11_ce02,
    0x0a11_ce03,
    0x0a11_ce04,
    0x0a11_ce05,
    0x0a11_ce06,
    0x0a11_ce07,
    0x0a11_ce08,
];

/// The number of programs of the default run.
const DEFAULT_PROGRAMS: usize = 64;

/// The environment variable that sets the number of programs.
const PROGRAMS_VARIABLE: &str = "RAF_ORACLE_PROGRAMS";

/// The classes of disagreement that are known to occur and are counted and
/// printed, not failed. A class leaves this list in the commit that fixes it,
/// together with its corpus entries.
const KNOWN: [Class; 2] = [Class::Rule2, Class::Documented];

/// How the two sides can disagree about a probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// `rustc` accepts, the index refuses it or finds nothing.
    AcceptedButRefused,
    /// Both accept, and name different items.
    OtherItem,
    /// `rustc` refuses, the index resolves.
    RefusedButResolves,
}

/// What a disagreement is an instance of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    /// A `pub use` of a binding its module reaches only through a private
    /// glob stays no more visible than that binding. The index models it, so
    /// a disagreement of this class is a failure.
    Rule1,
    /// A name the globs of a module bring in ambiguously, one item through a
    /// glob the reading module cannot access, is refused.
    Rule2,
    /// A glob re-export that `rustc` only warns about
    /// (`ambiguous_glob_reexports`) and resolves, which the index refuses.
    Documented,
    /// None of the above.
    Unclassified,
}

impl Class {
    fn name(self) -> &'static str {
        match self {
            Self::Rule1 => "rule1",
            Self::Rule2 => "rule2",
            Self::Documented => "documented",
            Self::Unclassified => "unclassified",
        }
    }

    fn named(name: &str) -> Option<Self> {
        [Self::Rule1, Self::Rule2, Self::Documented]
            .into_iter()
            .find(|class| class.name() == name)
    }
}

/// What the index answers in one name space.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Answer {
    Item {
        tag: u32,
        chain: Vec<String>,
    },
    Ambiguous,
    Missing,
    /// Any other refusal.
    Failed(String),
}

impl Answer {
    fn tag(&self) -> Option<u32> {
        match self {
            Self::Item { tag, .. } => Some(*tag),
            _ => None,
        }
    }
}

/// The answers of the index in both name spaces.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexVerdict {
    ty: Answer,
    value: Answer,
}

impl IndexVerdict {
    fn refuses(&self) -> bool {
        let refusal = |answer: &Answer| matches!(answer, Answer::Ambiguous | Answer::Failed(_));
        refusal(&self.ty)
            || refusal(&self.value)
            || (self.ty == Answer::Missing && self.value == Answer::Missing)
    }
}

/// The index of a program as a workspace.
fn load(source: &str) -> Workspace {
    let manifest = format!(
        "[package]\nname = \"{CRATE}\"\nversion = \"0.1.0\"\nedition = \"{}\"\n",
        rustc::EDITION
    );
    let tree: MemoryTree = [("Cargo.toml", manifest.as_str()), ("src/lib.rs", source)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}\n{source}"))
}

/// The tag of each item of the program, by path and name space.
fn tags(program: &Program) -> BTreeMap<(String, Space), u32> {
    program
        .decls()
        .into_iter()
        .map(|decl| ((decl.path(), decl.space), decl.tag))
        .collect()
}

fn ask(
    ws: &Workspace,
    tags: &BTreeMap<(String, Space), u32>,
    probe: &Probe,
    space: Space,
) -> Answer {
    let namespace = match space {
        Space::Type => Namespace::Type,
        Space::Value => Namespace::Value,
    };
    match ws.resolve_from_in(&probe.module, &probe.path, namespace) {
        Ok(found) => match tags.get(&(found.path.clone(), space)) {
            Some(tag) => Answer::Item {
                tag: *tag,
                chain: found.chain,
            },
            None => Answer::Failed(format!("{} is no generated item", found.path)),
        },
        Err(ResolveError::Ambiguous(_)) => Answer::Ambiguous,
        Err(ResolveError::NotFound(_)) => Answer::Missing,
        Err(other) => Answer::Failed(other.to_string()),
    }
}

/// Whether the two sides disagree about one probe.
fn compare(rustc: &Verdict, index: &IndexVerdict) -> Option<Kind> {
    match rustc {
        Verdict::Rejected(_) => (!index.refuses()).then_some(Kind::RefusedButResolves),
        Verdict::Accepted { ty, value } => {
            if index.refuses() {
                Some(Kind::AcceptedButRefused)
            } else if index.ty.tag() != *ty || index.value.tag() != *value {
                Some(Kind::OtherItem)
            } else {
                None
            }
        }
    }
}

/// Which class a disagreement is. `when_visible` is whether the two sides
/// agree on the probe once nothing in the program is hidden;
/// `ambiguous_in_a_module` is whether the index finds the probed name
/// ambiguous in some module of the program when that module reads it.
fn classify(
    kind: Kind,
    rustc: &Verdict,
    index: &IndexVerdict,
    when_visible: impl FnOnce() -> bool,
    warned_when_exported: impl FnOnce() -> bool,
    ambiguous_in_a_module: impl FnOnce() -> bool,
) -> Class {
    if kind == Kind::AcceptedButRefused {
        // The index refuses a name that rustc resolves. The documented case
        // is the glob re-export that rustc only warns about.
        let ambiguous = index.ty == Answer::Ambiguous || index.value == Answer::Ambiguous;
        if ambiguous && warned_when_exported() {
            return Class::Documented;
        }
        // Otherwise the index counts an item that rustc does not see as a
        // second candidate: the name is ambiguous in a module whose glob the
        // reader cannot access, and rustc hides the name whole (rule 2).
        // Rule 1 only hides, it never makes a name ambiguous or cyclic, so
        // a refusal that no ambiguity explains is not one of the two rules.
        return if when_visible() && ambiguous_in_a_module() {
            Class::Rule2
        } else {
            Class::Unclassified
        };
    }
    // The index is more permissive than rustc. Both rules are about what a
    // hidden binding lets through a glob or a re-export, so the disagreement
    // must go when nothing is hidden; rustc then refused the name as an
    // ambiguity (rule 2) or did not see the binding at all (rule 1).
    if !when_visible() {
        return Class::Unclassified;
    }
    // A name two globs of one module bring in as different items is
    // ambiguous there. A reader that cannot access one of the globs gets the
    // other item from the index, and from rustc a refusal that is E0659 when
    // the glob of the first binding is accessible, and otherwise a name it
    // does not find (E0432) or finds in fewer name spaces. That is rule 2
    // whatever the error, and a binding capped by a hidden route (rule 1)
    // does not need an ambiguity to show.
    match rustc {
        Verdict::Rejected(codes) if codes.iter().any(|code| code == "E0659" || code == "E0573") => {
            Class::Rule2
        }
        _ if ambiguous_in_a_module() => Class::Rule2,
        _ => Class::Rule1,
    }
}

/// Whether `rustc` warns of an ambiguous glob re-export of the name of
/// `probe` once every module of the program is `pub`. `rustc` raises the
/// lint only for what a crate exports, and resolves the name either way.
fn warns_of_ambiguous_re_export(program: &Program, probe: usize) -> bool {
    let exported = program.with_public_modules(probe);
    let name = format!("`{}`", exported.probes[0].name());
    rustc::judge(&exported).is_some_and(|judged| {
        judged.warnings.iter().any(|warning| {
            warning.lint == "ambiguous_glob_reexports" && warning.text.contains(&name)
        })
    })
}

/// Whether the index finds some name of `program` ambiguous in some module
/// of it that has a `use`, when that module reads the name itself: a name
/// that two globs of the module bring in as different items.
fn ambiguous_in_a_module(ws: &Workspace, program: &Program, probe: usize) -> bool {
    let mut names: std::collections::BTreeSet<String> = program
        .decls()
        .into_iter()
        .map(|decl| decl.name)
        .chain(program.uses().into_iter().filter_map(|decl| decl.binding))
        .collect();
    names.insert(program.probes[probe].name().to_owned());
    let modules: std::collections::BTreeSet<String> =
        program.uses().into_iter().map(|decl| decl.module).collect();
    modules.iter().any(|module| {
        names.iter().any(|name| {
            [Namespace::Type, Namespace::Value].into_iter().any(|ns| {
                matches!(
                    ws.resolve_from_in(module, &format!("self::{name}"), ns),
                    Err(ResolveError::Ambiguous(_))
                )
            })
        })
    })
}

/// Whether both sides agree on `probe` in `reduced`, a program of that probe
/// alone.
fn agree_on_the_only_probe(
    reduced: &Program,
    tamper: &dyn Fn(IndexVerdict) -> IndexVerdict,
) -> bool {
    let Some(judged) = rustc::judge(reduced) else {
        return false;
    };
    let source = judged.program.render(&[0], &BTreeMap::new()).source;
    let ws = load(&source);
    let tags = tags(&judged.program);
    let index = tamper(IndexVerdict {
        ty: ask(&ws, &tags, &judged.program.probes[0], Space::Type),
        value: ask(&ws, &tags, &judged.program.probes[0], Space::Value),
    });
    compare(&judged.verdicts[0], &index).is_none()
}

/// Whether the two sides agree on `probe` once nothing in the program is
/// hidden.
fn agree_when_visible(
    program: &Program,
    probe: usize,
    tamper: &dyn Fn(IndexVerdict) -> IndexVerdict,
) -> bool {
    agree_on_the_only_probe(&program.all_public(probe), tamper)
}

/// One disagreement.
#[derive(Debug, Clone)]
struct Divergence {
    probe: usize,
    kind: Kind,
    class: Class,
    rustc: Verdict,
    index: IndexVerdict,
}

/// Everything learned from one program.
#[derive(Debug)]
struct Evaluation {
    /// The program as judged, without the declarations `rustc` refuses.
    program: Program,
    /// False when `rustc` refuses the crate beyond what could be dropped.
    valid: bool,
    accepted: usize,
    rejected: usize,
    divergences: Vec<Divergence>,
}

/// Judge `program` with both sides, the index answers passed through
/// `tamper`.
fn evaluate_with(program: &Program, tamper: &dyn Fn(IndexVerdict) -> IndexVerdict) -> Evaluation {
    let Some(Judged {
        program, verdicts, ..
    }) = rustc::judge(program)
    else {
        return Evaluation {
            program: program.clone(),
            valid: false,
            accepted: 0,
            rejected: 0,
            divergences: Vec::new(),
        };
    };
    let all: Vec<usize> = (0..program.probes.len()).collect();
    let ws = load(&program.render(&all, &BTreeMap::new()).source);
    let tags = tags(&program);
    let mut evaluation = Evaluation {
        program: program.clone(),
        valid: true,
        accepted: 0,
        rejected: 0,
        divergences: Vec::new(),
    };
    for (at, verdict) in verdicts.iter().enumerate() {
        let probe = &program.probes[at];
        if matches!(verdict, Verdict::Rejected(_)) {
            evaluation.rejected += 1;
        } else {
            evaluation.accepted += 1;
        }
        let index = tamper(IndexVerdict {
            ty: ask(&ws, &tags, probe, Space::Type),
            value: ask(&ws, &tags, probe, Space::Value),
        });
        if let Some(kind) = compare(verdict, &index) {
            evaluation.divergences.push(Divergence {
                probe: at,
                kind,
                class: classify(
                    kind,
                    verdict,
                    &index,
                    || agree_when_visible(&program, at, tamper),
                    || warns_of_ambiguous_re_export(&program, at),
                    || ambiguous_in_a_module(&ws, &program, at),
                ),
                rustc: verdict.clone(),
                index,
            });
        }
    }
    evaluation
}

fn evaluate(program: &Program) -> Evaluation {
    evaluate_with(program, &|answer| answer)
}

/// The program reduced to one probe and as few lines as keep a disagreement
/// that `wanted` accepts on it: lines and modules are removed greedily, each removal
/// kept when the reduced program still shows the class.
fn minimise(program: &Program, probe: usize, wanted: &dyn Fn(&Divergence) -> bool) -> Program {
    let mut current = program.clone();
    current.probes = vec![program.probes[probe].clone()];
    current.expect = None;
    let shows = |candidate: &Program| {
        let evaluation = evaluate(candidate);
        evaluation
            .divergences
            .iter()
            .any(wanted)
            .then_some(evaluation.program)
    };
    loop {
        let reduced = current
            .removable()
            .into_iter()
            .rev()
            .find_map(|(from, to)| shows(&current.without(from, to)));
        match reduced {
            Some(smaller) => current = smaller,
            None => return current,
        }
    }
}

/// The programs of the run: the seeds in turn, each seed's programs drawn
/// from one generator.
fn generated(total: usize) -> Vec<(String, Program)> {
    let mut draws: Vec<Draw> = SEEDS.iter().map(|seed| Draw::seeded(*seed)).collect();
    (0..total)
        .map(|n| {
            let which = n % SEEDS.len();
            let program = random_program(&mut draws[which]);
            (
                format!("seed {:#x} program {}", SEEDS[which], n / SEEDS.len()),
                program,
            )
        })
        .collect()
}

fn program_count() -> usize {
    match std::env::var(PROGRAMS_VARIABLE) {
        Ok(text) => text
            .trim()
            .parse()
            .unwrap_or_else(|e| panic!("{PROGRAMS_VARIABLE}={text:?} is not a number: {e}")),
        Err(_) => DEFAULT_PROGRAMS,
    }
}

/// Evaluate every program, on as many threads as the machine offers.
fn evaluate_all(programs: &[(String, Program)]) -> Vec<Evaluation> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Evaluation>>> =
        Mutex::new(programs.iter().map(|_| None).collect());
    let workers = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(16);
    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let at = next.fetch_add(1, Ordering::Relaxed);
                    let Some((_, program)) = programs.get(at) else {
                        break;
                    };
                    let evaluation = evaluate(program);
                    results.lock().expect("the results are not poisoned")[at] = Some(evaluation);
                }
            });
        }
    });
    results
        .into_inner()
        .expect("the results are not poisoned")
        .into_iter()
        .map(|result| result.expect("every program was evaluated"))
        .collect()
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/rustc-oracle")
}

/// The committed programs, by file name.
fn corpus() -> Vec<(String, Program)> {
    let mut names: Vec<String> = fs::read_dir(corpus_dir())
        .expect("the corpus directory exists")
        .map(|entry| {
            entry
                .expect("directory entry")
                .file_name()
                .into_string()
                .expect("corpus file names are UTF-8")
        })
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let text = fs::read_to_string(corpus_dir().join(&name))
                .unwrap_or_else(|e| panic!("cannot read {name}: {e}"));
            let program = Program::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            (name, program)
        })
        .collect()
}

fn describe(program: &Program, divergence: &Divergence) -> String {
    format!(
        "{:?} {:?}: probe {} `{}` in {}\n  rustc: {:?}\n  index: {:?}\n{}",
        divergence.class,
        divergence.kind,
        divergence.probe,
        program.probes[divergence.probe].path,
        program.probes[divergence.probe].module,
        divergence.rustc,
        divergence.index,
        program.to_text()
    )
}

#[test]
fn the_index_agrees_with_rustc_on_generated_programs_up_to_the_known_classes() {
    let total = program_count();
    assert!(total > 0, "{PROGRAMS_VARIABLE} names no programs");
    let programs = generated(total);
    let results = evaluate_all(&programs);
    let mut by_class: BTreeMap<Class, usize> = BTreeMap::new();
    let (mut valid, mut accepted, mut rejected) = (0, 0, 0);
    let mut failures = String::new();
    for ((label, _), result) in programs.iter().zip(&results) {
        valid += usize::from(result.valid);
        accepted += result.accepted;
        rejected += result.rejected;
        for divergence in &result.divergences {
            *by_class.entry(divergence.class).or_default() += 1;
            if !KNOWN.contains(&divergence.class) {
                writeln!(
                    failures,
                    "{label}: {}",
                    describe(&result.program, divergence)
                )
                .expect("write");
            }
        }
    }
    println!("rustc: {}", rustc::version());
    println!(
        "seeds: {}",
        SEEDS
            .iter()
            .map(|seed| format!("{seed:#x}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("programs: {total} ({valid} judged)");
    println!(
        "probes: {} ({accepted} accepted, {rejected} refused)",
        accepted + rejected
    );
    for class in [
        Class::Rule1,
        Class::Rule2,
        Class::Documented,
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
    let entries = corpus();
    assert!(!entries.is_empty(), "the corpus holds no program");
    let programs: Vec<(String, Program)> = entries;
    let results = evaluate_all(&programs);
    let mut divergent = 0;
    for ((name, program), result) in programs.iter().zip(&results) {
        assert!(result.valid, "{name}: rustc refuses the crate");
        assert!(
            result.accepted + result.rejected > 0,
            "{name}: it holds no probe"
        );
        let expect = program
            .expect
            .as_deref()
            .unwrap_or_else(|| panic!("{name}: no `// oracle:` line"));
        if expect == "agree" {
            assert!(result.accepted > 0, "{name}: no probe is accepted");
            assert!(
                result.divergences.is_empty(),
                "{name}: expected agreement\n{}",
                describe(&result.program, &result.divergences[0])
            );
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
fn a_wrong_index_answer_is_reported_beside_the_right_one_which_is_not() {
    let program = fixture(TWO_STRUCTS);
    let right = evaluate(&program);
    assert!(right.valid);
    assert!(right.divergences.is_empty(), "{:?}", right.divergences);
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
        .find(|(_, program)| program.expect.as_deref().and_then(Class::named).is_some())
        .expect("the corpus holds a divergence");
    let class = Class::named(entry.expect.as_deref().expect("expectation")).expect("a class");
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

/// Finds the first program of each known class among the generated ones,
/// reduces it and writes it under the corpus directory as `<class>.rs`, to be
/// renamed after its shape; the
/// first of each kind of unclassified disagreement is reduced and printed, to
/// be examined. Run with `RAF_ORACLE_PROGRAMS` set to search further.
#[test]
#[ignore = "writes the corpus; run by hand"]
fn write_corpus() {
    let programs = generated(program_count());
    let results = evaluate_all(&programs);
    let first = |wanted: &dyn Fn(&Divergence) -> bool| {
        results.iter().find_map(|result| {
            result
                .divergences
                .iter()
                .find(|d| wanted(d))
                .map(|d| (&result.program, d.probe))
        })
    };
    for class in KNOWN {
        let Some((program, probe)) = first(&|d| d.class == class) else {
            println!("no divergence of class {}", class.name());
            continue;
        };
        let mut small = minimise(program, probe, &|d| d.class == class);
        small.expect = Some(class.name().to_owned());
        let path = corpus_dir().join(format!("{}.rs", class.name()));
        fs::write(&path, small.to_text()).expect("write the corpus program");
        println!("wrote {}", path.display());
    }
    for kind in [
        Kind::AcceptedButRefused,
        Kind::OtherItem,
        Kind::RefusedButResolves,
    ] {
        let unclassified = |d: &Divergence| d.class == Class::Unclassified && d.kind == kind;
        if let Some((program, probe)) = first(&unclassified) {
            let small = minimise(program, probe, &unclassified);
            println!("unclassified {kind:?}, reduced:\n{}", small.to_text());
        }
    }
}
