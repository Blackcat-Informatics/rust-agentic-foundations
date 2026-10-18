// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The index against `rustc`: generated library crates of several modules,
//! with private, `pub` and `pub(crate)` globs, named and glob re-exports,
//! shadowing and a struct and a function of one name, are asked the same
//! question by both. A question, a probe, is a `use` of a path as seen from a
//! chosen module; a path starts with `crate`, `self`, `super`, or a bare name,
//! the name of a module the reading module holds or its globs bring in. `rustc` accepts or refuses it, and when it accepts, a second
//! compile tells which item it names; the index answers through
//! `Workspace::resolve_from_in`. Every disagreement is classified, and one
//! that matches no known class fails the test.
//!
//! A known gap of the index: in some programs where globs and imports of one
//! name lead back to each other, the two sides settle the cycle differently.
//! Either the index finds that the imports form a cycle in a name space
//! (`ResolveError::Cyclic`) and refuses the probe where `rustc` accepts it, or
//! `rustc` gives up on the probe with "cannot determine resolution for the
//! import" where the index resolves it. These divergences are counted as
//! `cycle` and are not failures; the index does not settle such a cycle as
//! `rustc` does, and the corpus records each known shape. The first direction
//! counts only when the other name space agrees. The second is told by the
//! diagnostic, which has no error code, and by the import graph of the
//! program having a cycle among the modules the probe's module reaches. The
//! trade-off: a cyclic program whose probe `rustc` refuses with that
//! diagnostic is counted as this gap, so a fault of the index about a rule
//! on such a program would show only under another diagnostic.
//!
//! Whether `rustc` answers a probe differently when the declarations come in
//! another order, in a name space where the two sides disagree, is asked for
//! every kind of disagreement, whichever way the index errs, after
//! `documented` (and, where `rustc` accepts a probe the index refuses as a
//! cycle, after `cycle`). A rule bug on a probe that `rustc` answers order-dependently
//! cannot be told from the order dependence. It is asked one name space at a
//! time: the path as a type alias
//! and as the expression of a function body, so that a `use`, which `rustc`
//! accepts when either name space binds, does not hide a name space whose
//! answer changes.
//!
//! The committed programs under `fuzz/corpus/rustc-oracle` are replayed. The
//! generated programs come from eight seeds, `base + 1` to `base + 8`. The
//! default run draws [`DEFAULT_PROGRAMS`] programs from each of two bases,
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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use raf_source::{MemoryTree, Namespace, ResolveError, TargetKind, Workspace};
use support::program::{CRATE, Probe, Program, Space, random_program};
use support::random::Draw;
use support::rustc::{self, Judged, SpaceVerdict, UNDETERMINABLE, Verdict};

/// The base of the seeds of the first family of the default run; its seeds
/// are `base + 1` to `base + SEEDS_PER_BASE`. The programs of a seed are
/// drawn one after the other from a generator of that seed.
const BASE: u64 = 0x0a11_ce00;

/// The base of the second family of the default run.
const SECOND_BASE: u64 = 0xc0ff_ee00;

/// The number of seeds of a base.
const SEEDS_PER_BASE: u64 = 8;

/// The environment variable that selects the base of the seeds.
const SEED_BASE_VARIABLE: &str = "RAF_ORACLE_SEED_BASE";

/// The environment variable that, set to `1`, lists every divergence.
const LIST_VARIABLE: &str = "RAF_ORACLE_LIST";

/// The seeds of a base.
fn seeds_of(base: u64) -> Vec<u64> {
    (1..=SEEDS_PER_BASE).map(|n| base + n).collect()
}

/// A number written in hexadecimal with a `0x` prefix, or in decimal;
/// underscores are ignored.
fn parse_seed_base(text: &str) -> Result<u64, String> {
    let clean: String = text.trim().chars().filter(|c| *c != '_').collect();
    let parsed = match clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        Some(digits) => u64::from_str_radix(digits, 16),
        None => clean.parse(),
    };
    let base = parsed.map_err(|e| format!("{SEED_BASE_VARIABLE}={text:?} is not a number: {e}"))?;
    base.checked_add(SEEDS_PER_BASE)
        .map(|_| base)
        .ok_or_else(|| format!("{SEED_BASE_VARIABLE}={text:?} leaves no room for the seeds"))
}

/// The base the environment selects, if it does.
fn selected_base() -> Option<u64> {
    std::env::var(SEED_BASE_VARIABLE)
        .ok()
        .map(|text| parse_seed_base(&text).unwrap_or_else(|e| panic!("{e}")))
}

/// The bases of the run and the number of programs drawn from each.
fn families() -> Vec<(u64, usize)> {
    let set = std::env::var(PROGRAMS_VARIABLE).is_ok();
    match (selected_base(), set) {
        (Some(base), _) => vec![(base, program_count())],
        (None, true) => vec![(BASE, program_count())],
        (None, false) => vec![(BASE, DEFAULT_PROGRAMS), (SECOND_BASE, DEFAULT_PROGRAMS)],
    }
}

/// The number of programs of the default run from each base.
const DEFAULT_PROGRAMS: usize = 64;

/// The most programs with an index that varies with the order that are
/// reduced for the failure message.
const MOST_REDUCED: usize = 3;

/// The environment variable that sets the number of programs.
const PROGRAMS_VARIABLE: &str = "RAF_ORACLE_PROGRAMS";

/// The classes of disagreement that are known to occur and are counted and
/// printed, not failed. A class leaves this list in the commit that fixes it,
/// together with its corpus entries.
const KNOWN: [Class; 3] = [Class::Documented, Class::Order, Class::Cycle];

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
    /// glob the reading module cannot access, is refused. The index models
    /// it, so a disagreement of this class, the index resolving what rustc
    /// refuses, is a failure.
    Rule2,
    /// A glob re-export that `rustc` only warns about
    /// (`ambiguous_glob_reexports`) and resolves, which the index refuses.
    Documented,
    /// A probe that `rustc` itself answers differently when the declarations
    /// of the program come in another order, accepting it in one and
    /// refusing it in another, so that no answer that does not depend on the
    /// order can be its answer in all of them.
    Order,
    /// A known gap of the index, in two directions. It finds the imports
    /// that make up a name in one name space to form a cycle and refuses the
    /// probe with [`ResolveError::Cyclic`] where `rustc` accepts the probe and
    /// the other name space agrees; or `rustc` gives up on the probe, "cannot
    /// determine resolution for the import", in a program whose imports have
    /// a cycle that reaches the probe's module, where the index resolves it;
    /// that test of the graph is loose, since a child module that names or
    /// globs an ancestor counts as a cycle, and the diagnostic is the real
    /// filter. No ambiguity or visibility rule is involved: the cycle is among globs
    /// and imports that bind the name, which the index does not settle as
    /// `rustc` does.
    Cycle,
    /// The index's own answer, to a probe or for the exported surface,
    /// changes when the declarations of the program come in another order.
    /// It is not about a disagreement with `rustc`, and no known class: it
    /// is never counted as `order`, which needs the index to be stable.
    IndexOrder,
    /// None of the above.
    Unclassified,
}

impl Class {
    fn name(self) -> &'static str {
        match self {
            Self::Rule1 => "rule1",
            Self::Rule2 => "rule2",
            Self::Documented => "documented",
            Self::Order => "order",
            Self::Cycle => "cycle",
            Self::IndexOrder => "index-order",
            Self::Unclassified => "unclassified",
        }
    }

    fn named(name: &str) -> Option<Self> {
        [
            Self::Rule1,
            Self::Rule2,
            Self::Documented,
            Self::Order,
            Self::Cycle,
        ]
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
    /// The imports that make up the name form a cycle.
    Cyclic,
    /// Any other refusal.
    Failed(String),
}

impl Answer {
    /// The answer without the route it took, which is not what the index
    /// says about the name.
    fn without_chain(&self) -> Self {
        match self {
            Self::Item { tag, .. } => Self::Item {
                tag: *tag,
                chain: Vec::new(),
            },
            other => other.clone(),
        }
    }

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
        let refusal = |answer: &Answer| {
            matches!(
                answer,
                Answer::Ambiguous | Answer::Cyclic | Answer::Failed(_)
            )
        };
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
        Err(ResolveError::Cyclic(_)) => Answer::Cyclic,
        Err(other) => Answer::Failed(other.to_string()),
    }
}

/// How one name space of an accepted probe disagrees: `rustc` names the item
/// tagged `wanted`, or none; the index answered `answer`. A name space that
/// `rustc` leaves without an item and the index finds ambiguous or missing
/// agrees: `rustc` drops a name space whose name is ambiguous from an import
/// that binds in another, in the declaration orders where it does not refuse
/// the import whole, and the index never answers an ambiguous name with an
/// item.
fn space_kind(wanted: Option<u32>, answer: &Answer) -> Option<Kind> {
    match (wanted, answer) {
        (Some(tag), Answer::Item { tag: found, .. }) => (tag != *found).then_some(Kind::OtherItem),
        (Some(_), _) | (None, Answer::Cyclic | Answer::Failed(_)) => Some(Kind::AcceptedButRefused),
        (None, Answer::Item { .. }) => Some(Kind::OtherItem),
        (None, Answer::Ambiguous | Answer::Missing) => None,
    }
}

/// The name spaces in which the two sides disagree about a probe: those where
/// `rustc` accepts the probe and the index answers otherwise, and, for a probe
/// `rustc` refuses, those where the index finds an item.
fn disagreeing_spaces(rustc: &Verdict, index: &IndexVerdict) -> Vec<Space> {
    let spaces = |wanted: &dyn Fn(Space, &Answer) -> bool| {
        [(Space::Type, &index.ty), (Space::Value, &index.value)]
            .into_iter()
            .filter(|(space, answer)| wanted(*space, answer))
            .map(|(space, _)| space)
            .collect()
    };
    match rustc {
        Verdict::Rejected(_) => spaces(&|_, answer| answer.tag().is_some()),
        Verdict::Accepted { ty, value } => spaces(&|space, answer| {
            let wanted = match space {
                Space::Type => *ty,
                Space::Value => *value,
            };
            space_kind(wanted, answer).is_some()
        }),
    }
}

/// Whether, in a probe `rustc` accepts, every name space the two sides
/// disagree about is one the index finds cyclic.
fn only_cycles_diverge(rustc: &Verdict, index: &IndexVerdict) -> bool {
    let Verdict::Accepted { ty, value } = rustc else {
        return false;
    };
    [(*ty, &index.ty), (*value, &index.value)]
        .into_iter()
        .all(|(wanted, answer)| *answer == Answer::Cyclic || space_kind(wanted, answer).is_none())
}

/// Whether the two sides disagree about one probe.
fn compare(rustc: &Verdict, index: &IndexVerdict) -> Option<Kind> {
    match rustc {
        Verdict::Rejected(_) => (!index.refuses()).then_some(Kind::RefusedButResolves),
        Verdict::Accepted { ty, value } => {
            let kinds = [space_kind(*ty, &index.ty), space_kind(*value, &index.value)];
            if kinds.contains(&Some(Kind::AcceptedButRefused)) {
                Some(Kind::AcceptedButRefused)
            } else {
                kinds.into_iter().flatten().next()
            }
        }
    }
}

/// What the classifier asks about a probe, each answered only when a
/// disagreement needs it.
struct Facts<'a> {
    /// Whether the two sides agree on the probe once nothing in the program
    /// is hidden.
    when_visible: &'a dyn Fn() -> bool,
    /// Whether `rustc` warns of an ambiguous glob re-export of the probed
    /// name once every module of the program is `pub`.
    warned_when_exported: &'a dyn Fn() -> bool,
    /// Whether the index finds the probed name ambiguous in some module of
    /// the program when that module reads it.
    ambiguous_in_a_module: &'a dyn Fn() -> bool,
    /// Whether `rustc` answers the probe differently in some of the name
    /// spaces given when the declarations of the program come in another
    /// order.
    order_dependent: &'a dyn Fn(&[Space]) -> bool,
    /// Whether the imports of the program form a cycle among the modules the
    /// probe's module reaches.
    cycle_reaches_probe: &'a dyn Fn() -> bool,
    /// Whether the index answers the probe the same in every order of the
    /// declarations tried. `rustc` answering it differently is `order` only
    /// then: an index that varies has a fault of its own, whatever `rustc`
    /// does.
    index_stable: bool,
}

/// Which class a disagreement is.
fn classify(kind: Kind, rustc: &Verdict, index: &IndexVerdict, facts: &Facts<'_>) -> Class {
    let when_visible = facts.when_visible;
    let ambiguous_in_a_module = facts.ambiguous_in_a_module;
    let spaces = disagreeing_spaces(rustc, index);
    if kind == Kind::AcceptedButRefused {
        // The index refuses a name that rustc resolves. The documented case
        // is the glob re-export that rustc only warns about.
        let ambiguous = index.ty == Answer::Ambiguous || index.value == Answer::Ambiguous;
        if ambiguous && (facts.warned_when_exported)() {
            return Class::Documented;
        }
        // An import cycle in a name space, with no ambiguity in either and
        // the other name space agreeing, is the gap of the index that the
        // class records.
        let cyclic = index.ty == Answer::Cyclic || index.value == Answer::Cyclic;
        if cyclic && !ambiguous && only_cycles_diverge(rustc, index) {
            return Class::Cycle;
        }
        // If rustc itself answers the probe differently in another order of
        // the declarations, no answer that does not depend on the order is
        // its answer. Anything else it accepts and the index refuses is a
        // disagreement to fix or to report.
        if facts.index_stable && (facts.order_dependent)(&spaces) {
            return Class::Order;
        }
        return Class::Unclassified;
    }
    // The index is more permissive than rustc. Both rules are about what a
    // hidden binding lets through a glob or a re-export, so the disagreement
    // must go when nothing is hidden; rustc then refused the name as an
    // ambiguity (rule 2) or did not see the binding at all (rule 1). A
    // disagreement that does not is `rustc` answering by the order of the
    // declarations, or one to fix or to report.
    let visible = when_visible();
    if facts.index_stable && (facts.order_dependent)(&spaces) {
        return Class::Order;
    }
    // The other direction of the cycle gap: rustc cannot settle an import
    // cycle and gives up on the probe, where the index resolves it.
    let undeterminable = matches!(rustc, Verdict::Rejected(codes) if codes.iter().any(|code| code == UNDETERMINABLE));
    if kind == Kind::RefusedButResolves && undeterminable && (facts.cycle_reaches_probe)() {
        return Class::Cycle;
    }
    if !visible {
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

/// Whether `rustc` warns of an ambiguous glob re-export of the name of the
/// only probe of `exported`, a program with what it re-exports made `pub`.
/// `rustc` raises the lint only for what a crate exports, and resolves the
/// name either way.
fn warns_of_ambiguous_re_export(exported: &Program) -> bool {
    let name = format!("`{}`", exported.probes[0].name());
    rustc::judge(exported).is_some_and(|judged| {
        judged.warnings.iter().any(|warning| {
            warning.lint == "ambiguous_glob_reexports" && warning.text.contains(&name)
        })
    })
}

/// The number of other orders of the declarations tried for a program.
const ORDERS_TRIED: usize = 12;

/// The seed of the orders tried, for `rustc` and for the index alike.
const ORDER_SEED: u64 = 0x0de2_0001;

/// Whether `rustc` answers the probe differently in some of `spaces` when
/// the declarations of the program come in another order, among the order as
/// written and [`ORDERS_TRIED`] orders drawn from a fixed seed (see
/// [`Program::shuffled`]): the name space is resolved in one, ambiguous in
/// another, or refused in a third. Only the probe's own errors count.
fn rustc_depends_on_order(program: &Program, probe: usize, spaces: &[Space]) -> bool {
    let alone = program.only_probe(probe);
    let mut draw = Draw::seeded(ORDER_SEED);
    let mut answers: Vec<Vec<SpaceVerdict>> = Vec::new();
    let mut note = |program: &Program| {
        let verdicts = rustc::space_verdicts(program);
        answers.push(
            spaces
                .iter()
                .map(|space| {
                    verdicts
                        .iter()
                        .find(|(asked, _)| asked == space)
                        .expect("both name spaces are asked")
                        .1
                })
                .collect(),
        );
    };
    note(&alone);
    for _ in 0..ORDERS_TRIED {
        note(&alone.shuffled(&mut draw));
    }
    (0..spaces.len()).any(|at| answers.iter().any(|each| each[at] != answers[0][at]))
}

/// Whether the index finds the name of `probe` ambiguous in some module of
/// `program` that has a `use`, when that module reads the name itself: a name
/// that two globs of the module bring in as different items.
fn ambiguous_in_a_module(ws: &Workspace, program: &Program, probe: usize) -> bool {
    let name = program.probes[probe].name();
    let modules: std::collections::BTreeSet<String> =
        program.uses().into_iter().map(|decl| decl.module).collect();
    modules.iter().any(|module| {
        [Namespace::Type, Namespace::Value].into_iter().any(|ns| {
            matches!(
                ws.resolve_from_in(module, &format!("self::{name}"), ns),
                Err(ResolveError::Ambiguous(_))
            )
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

/// What the index says about a whole program: its answer to each probe in
/// both name spaces, without the route taken, with the candidates of an
/// ambiguous answer, and its exported surface.
#[derive(Debug, Clone, PartialEq, Eq)]
struct View {
    probes: Vec<(IndexVerdict, String)>,
    surface: String,
}

/// The paths of the candidates of an ambiguous answer of the index to
/// `probe` in `space`, sorted, or nothing.
fn candidates(ws: &Workspace, probe: &Probe, space: Space) -> String {
    let namespace = match space {
        Space::Type => Namespace::Type,
        Space::Value => Namespace::Value,
    };
    match ws.resolve_from_in(&probe.module, &probe.path, namespace) {
        Err(ResolveError::Ambiguous(ids)) => {
            let mut paths: Vec<String> = ids
                .iter()
                .map(|id| {
                    ws.item(*id)
                        .map_or_else(String::new, |i| i.path.to_string())
                })
                .collect();
            paths.sort();
            format!("{paths:?}")
        }
        _ => String::new(),
    }
}

/// The public surface of the library of `ws`, as one comparable text: each
/// exported path with the item it names and its kind, and each re-export,
/// or the error that refuses the listing.
fn surface(ws: &Workspace) -> String {
    let id = ws
        .target_ids()
        .find(|id| ws.target(*id).is_some_and(|t| t.kind == TargetKind::Lib))
        .expect("the program is a library");
    match ws.exported_items(id) {
        Ok(exports) => {
            let mut items: Vec<String> = exports
                .items
                .iter()
                .map(|export| {
                    let defined = ws
                        .item(export.item)
                        .map_or_else(String::new, |item| item.path.to_string());
                    format!("{} = {defined} ({:?})", export.path, export.kind)
                })
                .collect();
            items.sort();
            let mut reexports: Vec<String> = exports
                .reexports
                .iter()
                .map(|reexport| format!("{} = {}", reexport.path, reexport.written))
                .collect();
            reexports.sort();
            format!("items {items:?}, re-exports {reexports:?}")
        }
        Err(error) => format!("refused: {error}"),
    }
}

/// The text of a panic payload.
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("a panic")
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// The start of the answer of a load that panicked.
const PANICKED: &str = "the load panicked";

/// What the index says about `program` written in this order: a load that
/// panics, as the check of the settled tables does in a debug build, is its
/// own answer for every probe and for the surface.
fn view_of(program: &Program, tags: &BTreeMap<(String, Space), u32>) -> View {
    let all: Vec<usize> = (0..program.probes.len()).collect();
    let source = program.render(&all, &BTreeMap::new()).source;
    let asked = panic::catch_unwind(AssertUnwindSafe(|| {
        let ws = load(&source);
        let probes = program
            .probes
            .iter()
            .map(|probe| {
                (
                    IndexVerdict {
                        ty: ask(&ws, tags, probe, Space::Type).without_chain(),
                        value: ask(&ws, tags, probe, Space::Value).without_chain(),
                    },
                    format!(
                        "{}/{}",
                        candidates(&ws, probe, Space::Type),
                        candidates(&ws, probe, Space::Value)
                    ),
                )
            })
            .collect();
        View {
            probes,
            surface: surface(&ws),
        }
    }));
    asked.unwrap_or_else(|payload| {
        let failed = Answer::Failed(format!("{PANICKED}: {}", panic_text(&*payload)));
        View {
            probes: program
                .probes
                .iter()
                .map(|_| {
                    (
                        IndexVerdict {
                            ty: failed.clone(),
                            value: failed.clone(),
                        },
                        String::new(),
                    )
                })
                .collect(),
            surface: format!("{PANICKED}: {}", panic_text(&*payload)),
        }
    })
}

/// A probe, or the exported surface, that the index answers differently in
/// different orders of the declarations.
#[derive(Debug, Clone)]
struct Unstable {
    /// The probe, or `None` for the exported surface.
    probe: Option<usize>,
    /// Each distinct answer with the first order that gave it, 0 being the
    /// order as written and the others the shuffles, in the order tried.
    answers: Vec<(usize, String)>,
}

/// Ask the index about `program` as written and in [`ORDERS_TRIED`] orders
/// drawn from a fixed seed (the orders [`rustc_depends_on_order`] tries), and
/// list each probe and the surface it answers differently in some of them.
/// `tamper` changes the answer to a probe in the order given.
fn index_instability(
    program: &Program,
    tags: &BTreeMap<(String, Space), u32>,
    tamper: &dyn Fn(usize, IndexVerdict) -> IndexVerdict,
) -> Vec<Unstable> {
    let mut draw = Draw::seeded(ORDER_SEED);
    let mut views = vec![view_of(program, tags)];
    for _ in 0..ORDERS_TRIED {
        views.push(view_of(&program.shuffled(&mut draw), tags));
    }
    let mut found = Vec::new();
    let distinct = |texts: Vec<String>| -> Vec<(usize, String)> {
        let mut seen: Vec<(usize, String)> = Vec::new();
        for (order, text) in texts.into_iter().enumerate() {
            if !seen.iter().any(|(_, known)| *known == text) {
                seen.push((order, text));
            }
        }
        seen
    };
    for probe in 0..program.probes.len() {
        let answers = distinct(
            views
                .iter()
                .enumerate()
                .map(|(order, view)| {
                    let (verdict, listed) = view.probes[probe].clone();
                    let told = tamper(order, verdict);
                    format!(
                        "type {:?}, value {:?}, candidates {listed}",
                        told.ty, told.value
                    )
                })
                .collect(),
        );
        if answers.len() > 1 {
            found.push(Unstable {
                probe: Some(probe),
                answers,
            });
        }
    }
    let answers = distinct(views.iter().map(|view| view.surface.clone()).collect());
    // A load that panics in every order is no stable answer.
    if answers.len() > 1 || answers[0].1.starts_with(PANICKED) {
        found.push(Unstable {
            probe: None,
            answers,
        });
    }
    found
}

/// The answers of the index that differ across the orders, for a failure
/// message.
fn describe_unstable(program: &Program, unstable: &Unstable) -> String {
    let what = unstable.probe.map_or_else(
        || "the exported surface".to_owned(),
        |at| {
            format!(
                "probe {at} `{}` in {}",
                program.probes[at].path, program.probes[at].module
            )
        },
    );
    let mut text = format!("{}: {what}\n", Class::IndexOrder.name());
    for (order, answer) in &unstable.answers {
        let which = if *order == 0 {
            "as written".to_owned()
        } else {
            format!("shuffle {order}")
        };
        writeln!(text, "  {which}: {answer}").expect("write");
    }
    text
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
    /// What the index answers differently across the orders of the
    /// declarations, whatever `rustc` says.
    unstable: Vec<Unstable>,
}

/// Judge `program` with both sides, the index answers passed through
/// `tamper`.
fn evaluate_with(program: &Program, tamper: &dyn Fn(IndexVerdict) -> IndexVerdict) -> Evaluation {
    evaluate_in(program, tamper, &|_, answer| answer)
}

/// [`evaluate_with`], the answers of the index in the orders of the
/// declarations other than the one written also passed through `by_order`,
/// given the order (0 is the order as written).
fn evaluate_in(
    program: &Program,
    tamper: &dyn Fn(IndexVerdict) -> IndexVerdict,
    by_order: &dyn Fn(usize, IndexVerdict) -> IndexVerdict,
) -> Evaluation {
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
            unstable: Vec::new(),
        };
    };
    let all: Vec<usize> = (0..program.probes.len()).collect();
    let tags = tags(&program);
    let unstable = index_instability(&program, &tags, by_order);
    // A load that panics, as the check of the settled tables makes it in a
    // debug build, is reported with the orders and has no answer to compare.
    let source = program.render(&all, &BTreeMap::new()).source;
    let Ok(ws) = panic::catch_unwind(AssertUnwindSafe(|| load(&source))) else {
        return Evaluation {
            program: program.clone(),
            valid: true,
            accepted: 0,
            rejected: 0,
            divergences: Vec::new(),
            unstable,
        };
    };
    let mut evaluation = Evaluation {
        program: program.clone(),
        valid: true,
        accepted: 0,
        rejected: 0,
        divergences: Vec::new(),
        unstable,
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
                    &Facts {
                        when_visible: &|| agree_when_visible(&program, at, tamper),
                        warned_when_exported: &|| {
                            warns_of_ambiguous_re_export(&program.with_public_modules(at))
                        },
                        ambiguous_in_a_module: &|| ambiguous_in_a_module(&ws, &program, at),
                        order_dependent: &|spaces| rustc_depends_on_order(&program, at, spaces),
                        cycle_reaches_probe: &|| program.import_cycle_reaches(at),
                        index_stable: !evaluation
                            .unstable
                            .iter()
                            .any(|unstable| unstable.probe == Some(at)),
                    },
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
    reduce(program, probe, &|evaluation| {
        evaluation.divergences.iter().any(wanted)
    })
}

/// [`minimise`] for an index that answers the probe `probe` differently in
/// some order of the declarations.
fn minimise_unstable(program: &Program, probe: Option<usize>) -> Program {
    // The reduced program holds the one probe, at 0.
    let kept = probe.map(|_| 0);
    reduce(program, probe.unwrap_or(0), &|evaluation| {
        evaluation
            .unstable
            .iter()
            .any(|unstable| unstable.probe == kept)
    })
}

/// The program reduced to the probe `probe` and as few lines as keep what
/// `shows` finds in an evaluation of it.
fn reduce(program: &Program, probe: usize, shows: &dyn Fn(&Evaluation) -> bool) -> Program {
    let mut current = program.clone();
    current.probes = vec![program.probes[probe].clone()];
    current.expect = None;
    let shows = |candidate: &Program| {
        let evaluation = evaluate(candidate);
        shows(&evaluation).then_some(evaluation.program)
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

/// The programs of the run: the seeds of each family in turn, each seed's
/// programs drawn from one generator.
fn generated() -> Vec<(String, Program)> {
    families()
        .into_iter()
        .flat_map(|(base, total)| generated_from(base, total))
        .collect()
}

/// `total` programs from the seeds of `base`.
fn generated_from(base: u64, total: usize) -> Vec<(String, Program)> {
    let seeds = seeds_of(base);
    let mut draws: Vec<Draw> = seeds.iter().map(|seed| Draw::seeded(*seed)).collect();
    (0..total)
        .map(|n| {
            let which = n % seeds.len();
            let program = random_program(&mut draws[which]);
            (
                format!("seed {:#x} program {}", seeds[which], n / seeds.len()),
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
    let programs = generated_from(BASE, program_count());
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

#[test]
fn an_index_answer_that_changes_with_the_order_is_reported_as_index_order() {
    let program = fixture(TWO_STRUCTS);
    // The index names the other struct in the third shuffle only.
    let changing = evaluate_in(&program, &|answer| answer, &|order, mut answer| {
        if let (3, Answer::Item { tag, .. }) = (order, &mut answer.ty) {
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
    assert_eq!(orders, [0, 3]);
    let report = describe_unstable(&changing.program, &changing.unstable[0]);
    assert!(
        report.starts_with("index-order: probe 0 `crate::m0::A` in demo\n"),
        "{report}"
    );
    assert!(
        report.contains("as written") && report.contains("shuffle 3"),
        "{report}"
    );
    assert!(!KNOWN.contains(&Class::IndexOrder));
    assert_eq!(Class::named("index-order"), None);
}

#[test]
fn a_probe_rustc_answers_by_the_order_is_not_order_when_the_index_does_too() {
    let entries = corpus();
    let (name, entry) = entries
        .iter()
        .find(|(_, program)| program.expect.as_deref() == Some(Class::Order.name()))
        .expect("the corpus holds an order entry");
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
    assert_eq!(parse_seed_base("0xa11ce00"), Ok(0x0a11_ce00));
    assert_eq!(parse_seed_base(" 0XC0FF_EE00 "), Ok(0xc0ff_ee00));
    assert_eq!(parse_seed_base("1000"), Ok(1000));
    assert_eq!(seeds_of(0x0a11_ce00)[0], 0x0a11_ce01);
    assert_eq!(seeds_of(0x0a11_ce00)[7], 0x0a11_ce08);
    for bad in ["", "0x", "0xzz", "seed", "-1", "12ab", "0xffffffffffffffff"] {
        let error = parse_seed_base(bad).expect_err(bad);
        assert!(
            error.contains(SEED_BASE_VARIABLE) && error.contains("is not a number")
                || error.contains("no room"),
            "{error}"
        );
    }
}
