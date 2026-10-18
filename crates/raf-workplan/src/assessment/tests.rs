// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The parts of the assessment module that need no repository: the id rule,
//! the subject form, the generated-only rule and the wording of a moved
//! input.

use std::collections::BTreeSet;

use raf_okf::{Body, CapabilitySection, Document, InputKind, Profile, Rule};
use raf_source::{MemoryTree, Workspace};

use super::{
    AssessError, InputChange, MovedInput, Naming, is_authored, naming, parse_subject, record_slug,
    symbol_inputs,
};
use crate::Workplan;
use crate::work::{Work, measure, measure_kind};

#[test]
fn the_record_slug_is_the_capability_slug_and_the_section_and_no_two_sections_share_one() {
    let capabilities = [
        "cap:a",
        "cap:a-contract",
        "cap:a-usage",
        "cap:usage",
        "cap:x-y-z",
    ];
    let mut seen = BTreeSet::new();
    for capability in capabilities {
        for section in CapabilitySection::ALL {
            let slug = record_slug(capability, section).expect("a capability id");
            assert!(raf_okf::is_kebab(&slug), "{slug}");
            assert!(
                slug.ends_with(&format!("-{}", section.slug())),
                "{capability} {section:?}"
            );
            assert!(seen.insert(slug.clone()), "{slug} is shared");
        }
    }
    for not_an_id in ["a", "cap:", "cap:A", "cap:a_b", "change:a", "cap:a--b"] {
        assert_eq!(
            record_slug(not_an_id, CapabilitySection::Usage),
            None,
            "{not_an_id}"
        );
    }
}

#[test]
fn a_subject_is_a_capability_and_one_of_the_seven_sections() {
    let id = parse_subject("cap:workplan-reconcile#explanation").expect("a subject");
    assert_eq!(id.capability(), "cap:workplan-reconcile");
    assert_eq!(id.section(), CapabilitySection::Explanation);
    assert_eq!(id.to_string(), "cap:workplan-reconcile#explanation");
    for bad in [
        "",
        "cap:x",
        "x#usage",
        "cap:X#usage",
        "cap:#usage",
        "evidence:x#usage",
        "cap:x y#usage",
    ] {
        assert!(
            matches!(parse_subject(bad), Err(AssessError::BadSubject { .. })),
            "{bad}"
        );
    }
    for missing in ["cap:x#intro", "cap:x#", "cap:x#Usage", "cap:x#usage#more"] {
        assert!(
            matches!(
                parse_subject(missing),
                Err(AssessError::NoSuchSection { .. })
            ),
            "{missing}"
        );
    }
}

fn body(contract: &str) -> Body {
    Body::parse(&format!(
        "# Title\n\n## Purpose\n\nWhy.\n\n## Contract\n{contract}\n## Usage\n\nHow.\n"
    ))
    .expect("a body")
}

#[test]
fn a_section_of_generated_blocks_and_comments_only_is_not_authored_and_one_word_is() {
    let generated = "\n<!-- raf:generated:relationships -->\n| Relation | Capability |\n<!-- raf:generated:end -->\n";
    assert!(!is_authored(&body(generated), CapabilitySection::Contract));
    let commented = format!("{generated}\n<!-- a note -->\n\n");
    assert!(!is_authored(&body(&commented), CapabilitySection::Contract));
    let worded = format!("{generated}\nOne word.\n");
    assert!(is_authored(&body(&worded), CapabilitySection::Contract));
    // The neighbours are authored either way.
    assert!(is_authored(&body(generated), CapabilitySection::Purpose));
    assert!(is_authored(&body(generated), CapabilitySection::Usage));
    // A section the body does not hold is judged, never passed over.
    assert!(is_authored(
        &body(generated),
        CapabilitySection::Verification
    ));
}

#[test]
fn a_moved_input_names_its_kind_its_name_and_how_it_moved() {
    let moved = |kind, change| {
        MovedInput {
            kind,
            name: "x".to_string(),
            change,
        }
        .to_string()
    };
    assert_eq!(
        moved(InputKind::Section, InputChange::Changed),
        "the section `x` has changed since"
    );
    assert_eq!(
        moved(
            InputKind::Symbol,
            InputChange::Unresolved("gone".to_string())
        ),
        "the symbol `x` does not resolve now: gone"
    );
    assert_eq!(
        moved(InputKind::Record, InputChange::NoLongerInput),
        "the evidence record `x` is no longer an input"
    );
    assert_eq!(
        moved(InputKind::Symbol, InputChange::NewInput),
        "the symbol `x` is an input now and was not then"
    );
}

/// A capability document whose Contract holds `contract` and whose other
/// sections each hold a sentence.
fn document(contract: &str) -> String {
    let mut text = String::from(
        "---\ntype: Capability\nid: cap:omega\ntitle: Omega\ndescription: Omega.\n\
         state: considered\nbindings:\n  code: []\n  tests: []\n  examples: []\ndepends: []\n---\n\n# Omega\n\n",
    );
    for section in CapabilitySection::ALL {
        text.push_str("## ");
        text.push_str(section.heading());
        text.push_str("\n\n");
        if section == CapabilitySection::Contract {
            text.push_str(contract);
        } else {
            text.push_str("A sentence.\n");
        }
        text.push('\n');
    }
    text
}

/// `is_authored` and the `section-empty` rule of the profile judge every
/// section alike, so a section the profile lets load is never passed over as
/// generated only, and the two rules cannot drift apart unseen.
#[test]
fn authored_text_is_what_the_profile_calls_a_section_that_is_not_empty() {
    let generated = "<!-- raf:generated:relationships -->\n| Relation | Capability |\n<!-- raf:generated:end -->\n";
    let contents = [
        String::new(),
        "   \n\t\n".to_string(),
        generated.to_string(),
        format!("{generated}\n<!-- a note -->\n"),
        "<!-- a comment\nover lines -->\n".to_string(),
        "<!-- never closed\nand text after it\n".to_string(),
        format!("{generated}Word.\n"),
        format!("Word.\n\n{generated}"),
        "`code`\n".to_string(),
        "```text\nfenced\n```\n".to_string(),
        "> [!PROPOSED change:grow]\n> A block.\n".to_string(),
        "a <!-- x --> b\n".to_string(),
    ];
    // The document is valid with a sentence in its Contract, so the rule the
    // cases meet is the one about the section.
    let valid = Document::parse(&document("Word.\n")).expect("a document");
    assert!(
        Profile
            .validate(&valid, "docs/capabilities/omega.md")
            .is_ok()
    );
    for contract in contents {
        let text = document(&contract);
        let parsed = Document::parse(&text).expect("a document");
        let empty = match Profile.validate(&parsed, "docs/capabilities/omega.md") {
            Ok(_) => false,
            Err(violations) => violations
                .iter()
                .any(|v| matches!(v.rule(), Rule::SectionEmpty)),
        };
        let body =
            Body::parse(&text[text.find("\n# Omega").expect("the title") + 1..]).expect("a body");
        assert_eq!(
            is_authored(&body, CapabilitySection::Contract),
            !empty,
            "{contract:?}"
        );
    }
}

fn named(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(ToString::to_string).collect()
}

#[test]
fn a_whole_path_or_a_suffix_of_two_segments_is_strong_and_the_last_segment_alone_is_weak() {
    let path = "raf_workplan::Workplan::check";
    assert_eq!(naming(&named(&[path]), path), Naming::Strong);
    assert_eq!(naming(&named(&["Workplan::check"]), path), Naming::Strong);
    assert_eq!(naming(&named(&["check"]), path), Naming::Weak);
    assert_eq!(
        naming(&named(&["check", "Workplan::check"]), path),
        Naming::Strong
    );
    for other in [
        "Workplan",
        "raf_workplan",
        "Workplan::chec",
        "heck",
        "raf_workplan::Workplan",
    ] {
        assert_eq!(naming(&named(&[other]), path), Naming::Unnamed, "{other}");
    }
}

#[test]
fn a_weak_name_adds_an_input_and_never_ends_the_every_binding_rule() {
    let code = [
        "demo::run",
        "demo::complete",
        "demo::Window::record",
        "demo::Window::len",
    ];
    let inputs = |names: &[&str]| -> Vec<&str> {
        symbol_inputs(&named(names), code.iter().copied())
            .into_iter()
            .map(|(path, _)| path)
            .collect()
    };
    // A common word names its binding weakly: every binding stays an input.
    assert_eq!(inputs(&["complete", "cargo"]), code);
    assert_eq!(inputs(&[]), code);
    // A strong name narrows the inputs to what the section names, weak names
    // included.
    assert_eq!(inputs(&["Window::record"]), ["demo::Window::record"]);
    assert_eq!(
        inputs(&["demo::run", "complete"]),
        ["demo::run", "demo::complete"]
    );
    // A suffix two bindings end in names both.
    let twins = ["demo::left::go", "demo::right::go", "demo::spare"];
    let got: Vec<&str> = symbol_inputs(&named(&["demo::spare", "go"]), twins.iter().copied())
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(got, twins);
}

/// A small generator of numbers, so the documents are the same on every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, below: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % (below as u64)).expect("small")
    }
}

/// For generated sections and bindings, a document that holds more code
/// spans of bare last segments, which name only weakly, never has fewer
/// symbol inputs: widening the match only ever adds an input.
#[test]
fn widening_a_section_with_weak_names_never_removes_an_input() {
    use crate::graph::code_span_paths;
    let modules = ["demo", "demo::inner", "demo::Window", "demo::left"];
    let names = ["run", "complete", "check", "record", "len", "go", "verify"];
    let mut random = Lcg(7);
    for _ in 0..2_000 {
        let mut code: Vec<String> = (0..=random.next(6))
            .map(|_| format!("{}::{}", modules[random.next(4)], names[random.next(7)]))
            .collect();
        code.sort();
        code.dedup();
        let span = |random: &mut Lcg| -> String {
            let path = &code[random.next(code.len())];
            let segments: Vec<&str> = path.split("::").collect();
            let from = random.next(segments.len());
            match random.next(3) {
                0 => names[random.next(7)].to_string(),
                _ => segments[from..].join("::"),
            }
        };
        let mut text = String::new();
        for _ in 0..random.next(4) {
            text.push_str("Some prose and `");
            text.push_str(&span(&mut random));
            text.push_str("` here.\n\n");
        }
        let mut wider = text.clone();
        for _ in 0..=random.next(3) {
            wider.push_str("And `");
            wider.push_str(names[random.next(7)]);
            wider.push_str("` too.\n\n");
        }
        let inputs = |text: &str| -> BTreeSet<String> {
            symbol_inputs(&code_span_paths(text), code.iter().map(String::as_str))
                .into_iter()
                .map(|(path, _)| path.to_string())
                .collect()
        };
        let (before, after) = (inputs(&text), inputs(&wider));
        assert!(before.is_subset(&after), "{code:?}\n{text}\n{wider}");
    }
}

// --- the work of judging a section --------------------------------------------

/// The most lines of a long Contract that may be read while it is judged: a
/// pass that reads them again for each line stops here at once instead of
/// running to the end.
const LINES: usize = 12_100;

/// A repository with the crate `demo` of `symbols` functions and the
/// capability `omega`, bound to all of them, whose Contract is `contract`.
fn omega(symbols: usize, contract: &str) -> (MemoryTree, Workspace) {
    use std::fmt::Write as _;

    let mut lib = String::new();
    let mut paths = Vec::new();
    for n in 0..symbols {
        writeln!(lib, "pub fn f{n}() {{}}").expect("a string takes the text");
        paths.push(format!("demo::f{n}"));
    }
    let source = Workspace::load(
        &MemoryTree::from([
            (
                "Cargo.toml",
                "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
            ),
            ("src/lib.rs", lib.as_str()),
        ]),
        &["."],
    )
    .expect("a workspace");
    let mut sections = String::new();
    for name in [
        "Purpose",
        "Contract",
        "Usage",
        "Examples",
        "Constraints",
        "Explanation",
        "Verification",
    ] {
        let text = if name == "Contract" {
            contract
        } else {
            "Text."
        };
        write!(sections, "## {name}\n\n{text}\n\n").expect("a string takes the text");
    }
    let document = format!(
        "---\ntype: Capability\nid: cap:omega\ntitle: Omega\ndescription: Omega.\n\
         state: implementing\nbindings:\n  code: [{}]\n  tests: []\n  examples: []\n\
         depends: []\n---\n\n# Omega\n\n{sections}",
        paths.join(", ")
    );
    let tree = MemoryTree::from([("docs/capabilities/omega.md", document.as_str())]);
    (tree, source)
}

/// Every binding is tested against the names of a section once to find
/// whether any is named strongly and once more to say how it is named, and
/// the first strong name ends the first look.
#[test]
fn the_bindings_are_tested_against_the_names_once_to_find_a_strong_one_and_once_to_name_them() {
    for count in [2_000, 10_000] {
        let paths: Vec<String> = (0..count).map(|n| format!("demo::f{n}")).collect();
        let tested = |names: &BTreeSet<String>| {
            let (inputs, tally) = measure(2 * count + 10, || {
                symbol_inputs(names, paths.iter().map(String::as_str))
            });
            (inputs.len(), tally.get(Work::NamingLookups))
        };
        // Nothing is named: every binding is an input, found out by looking
        // at all of them, then named.
        assert_eq!(tested(&named(&[])), (count, 2 * count));
        // The last is named: the first look runs to it.
        let last = format!("demo::f{}", count - 1);
        assert_eq!(tested(&named(&[last.as_str()])), (1, 2 * count));
        // The first is named: the first look ends at it.
        assert_eq!(tested(&named(&["demo::f0"])), (1, 1 + count));
    }
}

/// A Contract of twelve thousand lines is read once, line by line, for the
/// paths its code spans name, and each of the bound symbols is tested once
/// for how it is named.
#[test]
fn a_long_contract_is_read_once_and_its_symbols_are_named_in_one_pass() {
    use std::fmt::Write as _;

    let line = "The window keeps the values it was given, in order, and no more.\n";
    let contract = format!("\n{}", line.repeat(12_000));
    let (tree, source) = omega(2_000, &contract);
    let plan = Workplan::load(&tree, &source);
    let (context, tally) = measure_kind(Work::ContractLines, LINES, || {
        plan.assess_context("cap:omega#contract")
    });
    let context = context.expect("a context");
    // Nothing is named, so every binding is an input.
    assert_eq!(context.symbols.len(), 2_000);
    assert!(context.symbols.iter().all(|symbol| !symbol.named));
    let cap = &plan.capabilities()[0];
    let section = cap
        .body()
        .sections()
        .iter()
        .find(|section| section.slug() == "contract")
        .expect("the Contract");
    let prose = crate::graph::prose_of(cap.body(), section).expect("the section");
    // One look for each line, and one for the end of the text.
    assert!(prose.lines().count() > 12_000);
    assert_eq!(tally.get(Work::ContractLines), prose.lines().count() + 1);
    assert_eq!(tally.get(Work::NamingLookups), 2 * 2_000);

    let spans = (0..2_000).fold(String::new(), |mut text, n| {
        let _ = write!(text, "`demo::f{n}` ");
        text
    });
    let (tree, source) = omega(2_000, &format!("\n{spans}\n"));
    let plan = Workplan::load(&tree, &source);
    let (context, tally) = measure_kind(Work::NamingLookups, 2_010, || {
        plan.assess_context("cap:omega#contract")
    });
    let context = context.expect("a context");
    assert_eq!(context.symbols.len(), 2_000);
    assert!(context.symbols.iter().all(|symbol| symbol.named));
    assert_eq!(tally.get(Work::NamingLookups), 1 + 2_000);
}
