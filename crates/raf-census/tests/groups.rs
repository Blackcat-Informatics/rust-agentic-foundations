// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Grouping through `Census::load` over in-memory workspaces: body groups,
//! shim groups, their ordering, the hex tables, the constants shared across
//! packages, and job matching.

use std::fmt::Write as _;

use raf_census::{
    Census, CensusOptions, Fingerprint, Group, GroupKind, HexTable, Job, JobForbidden, JobMatch,
    MatchClass, RuleId, TableKind,
};
use raf_source::MemoryTree;

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn load(source: &str) -> Census {
    let tree: MemoryTree = [("Cargo.toml", MANIFEST), ("src/lib.rs", source)]
        .into_iter()
        .collect();
    Census::load(&tree, &["."], &CensusOptions::default()).expect("the workspace loads")
}

/// A workspace of packages, each with a library of the given source.
fn packages(sources: &[(&str, &str)]) -> Census {
    let members: Vec<String> = sources
        .iter()
        .map(|(name, _)| format!("\"{name}\""))
        .collect();
    let mut files = vec![(
        "Cargo.toml".to_owned(),
        format!(
            "[workspace]\nmembers = [{}]\nresolver = \"2\"\n",
            members.join(", ")
        ),
    )];
    for (name, source) in sources {
        files.push((
            format!("{name}/Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        ));
        files.push((format!("{name}/src/lib.rs"), (*source).to_owned()));
    }
    let tree: MemoryTree = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    Census::load(&tree, &["."], &CensusOptions::default()).expect("the workspace loads")
}

/// A function of exactly `count` (odd) structural tokens: `a op a op ... a`.
fn chain(name: &str, op: &str, count: usize) -> String {
    let sum = vec!["a"; count.div_ceil(2)].join(&format!(" {op} "));
    format!("pub fn {name}(a: u32) -> u32 {{ {sum} }}\n")
}

fn symbols(group: &Group) -> Vec<&str> {
    group.member_symbols().collect()
}

#[test]
fn groups_of_one_are_dropped_and_members_follow_symbol_order() {
    let census = load(&format!(
        "{}{}{}{}",
        chain("zulu", "+", 21),
        chain("alpha", "+", 21),
        chain("mike", "+", 21),
        chain("alone", "-", 21),
    ));
    assert_eq!(census.groups().len(), 1, "{:?}", census.groups());
    let group = &census.groups()[0];
    assert_eq!(group.kind, GroupKind::Body);
    assert!(matches!(group.fingerprint, Fingerprint::Body(_)));
    assert_eq!(group.tokens, 21);
    assert_eq!(
        symbols(group),
        ["demo::alpha", "demo::mike", "demo::zulu"],
        "ordered by symbol, whatever the source order"
    );
    assert_eq!(group.members[0].file, "src/lib.rs");
    assert_eq!(group.members[0].line, 2);
    assert_eq!(group.members[0].package, "demo");
    assert_eq!(census.body_groups().count(), 1);
    assert_eq!(census.shim_groups().count(), 0);
}

#[test]
fn bodies_under_min_tokens_never_group() {
    let census = load(&format!("{}{}", chain("a", "+", 19), chain("b", "+", 19)));
    assert_eq!(census.groups(), []);
    let census = load(&format!("{}{}", chain("a", "+", 21), chain("b", "+", 21)));
    assert_eq!(census.groups().len(), 1);
}

#[test]
fn groups_order_by_count_then_tokens_then_fingerprint() {
    let mut source = String::new();
    // Three members at the fewest tokens: count beats tokens.
    for name in ["c1", "c2", "c3"] {
        source.push_str(&chain(name, "+", 21));
    }
    // Two members each, differing in tokens.
    for name in ["t41a", "t41b"] {
        source.push_str(&chain(name, "+", 41));
    }
    for name in ["t31a", "t31b"] {
        source.push_str(&chain(name, "+", 31));
    }
    // Two groups tied on count and tokens: fingerprint ascending.
    for name in ["f_minus_a", "f_minus_b"] {
        source.push_str(&chain(name, "-", 21));
    }
    for name in ["f_times_a", "f_times_b"] {
        source.push_str(&chain(name, "*", 21));
    }
    source.push_str(&chain("loner", "/", 21));
    let census = load(&source);
    let shape: Vec<(usize, usize)> = census
        .groups()
        .iter()
        .map(|group| (group.members.len(), group.tokens))
        .collect();
    assert_eq!(shape, [(3, 21), (2, 41), (2, 31), (2, 21), (2, 21)]);
    let groups = census.groups();
    assert_eq!(symbols(&groups[0]), ["demo::c1", "demo::c2", "demo::c3"]);
    assert_eq!(symbols(&groups[1]), ["demo::t41a", "demo::t41b"]);
    assert_eq!(symbols(&groups[2]), ["demo::t31a", "demo::t31b"]);
    assert!(
        groups[3].fingerprint < groups[4].fingerprint,
        "tied groups run by fingerprint ascending"
    );
    let tied: Vec<&str> = groups[3..].iter().flat_map(symbols).collect();
    assert!(tied.iter().all(|s| s.starts_with("demo::f_")), "{tied:?}");
}

const FORWARDERS: &str = "\
pub trait Make {
    fn make(x: u32) -> u32;
    fn dflt(x: u32) -> u32 {
        helper(x, 1)
    }
}
pub struct S;
pub struct U;
impl Make for S {
    fn make(x: u32) -> u32 {
        helper(x, 1)
    }
}
impl S {
    pub fn build(x: u32) -> u32 {
        helper(x, 1)
    }
    pub fn plain(y: u32) -> u32 {
        helper(y, 1)
    }
    pub fn via(x: u32) -> u32 {
        Self::inner(x, 1)
    }
}
impl U {
    pub fn via(x: u32) -> u32 {
        Self::inner(x, 1)
    }
    pub fn neighbour(x: u32) -> u32 {
        Other::inner(x, 1)
    }
    pub fn twin(z: u32) -> u32 {
        Other::inner(z, 1)
    }
}
";

#[test]
fn shim_groups_use_the_shim_key_and_skip_trait_impls_and_self_forwarders() {
    let census = load(FORWARDERS);
    assert_eq!(census.body_groups().count(), 0);
    let shims: Vec<&Group> = census.shim_groups().collect();
    assert_eq!(shims.len(), 2, "{shims:?}");
    let named: Vec<Vec<&str>> = shims.iter().map(|group| symbols(group)).collect();
    // The larger group first.
    assert_eq!(
        named[0],
        ["demo::Make::dflt", "demo::S::build", "demo::S::plain"]
    );
    assert_eq!(named[1], ["demo::U::neighbour", "demo::U::twin"]);
    for group in &shims {
        assert_eq!(group.kind, GroupKind::Shim);
        assert!(matches!(group.fingerprint, Fingerprint::Shim(_)));
    }
    // Each excluded unit has a shim form, no key and no group.
    let trait_impl = census
        .units()
        .iter()
        .find(|u| u.trait_impl && u.name == "make")
        .expect("the trait implementation method exists");
    let excluded = [
        trait_impl,
        census.unit("demo::S::via").expect("exists"),
        census.unit("demo::U::via").expect("exists"),
    ];
    for unit in excluded {
        assert!(unit.print.as_ref().is_some_and(|p| p.shim.is_some()));
        assert!(unit.shim_key.is_none(), "{}", unit.symbol);
        assert!(
            census
                .groups()
                .iter()
                .all(|g| !g.members.iter().any(|m| m.symbol == unit.symbol)),
            "{} is in a group",
            unit.symbol
        );
    }
}

#[test]
fn a_forwarder_at_min_tokens_joins_a_shim_group_and_never_a_body_group() {
    let args = |first: &str| {
        let mut text = format!("f({first}");
        for n in 1..11 {
            write!(text, ", {n}").expect("a string takes a write");
        }
        text.push(')');
        text
    };
    // 24 tokens: above min_tokens, inside the shim bounds.
    let source = format!(
        "pub fn p(a: u32) -> u32 {{ {} }}\npub fn q(b: u32) -> u32 {{ {} }}\n",
        args("a"),
        args("b")
    );
    let census = load(&source);
    for symbol in ["demo::p", "demo::q"] {
        let print = census
            .unit(symbol)
            .and_then(|u| u.print.as_ref())
            .expect("a function has a print");
        assert_eq!(print.tokens, 24);
        assert!(print.body.is_some() && print.shim.is_some());
    }
    assert_eq!(census.body_groups().count(), 0);
    let shims: Vec<&Group> = census.shim_groups().collect();
    assert_eq!(shims.len(), 1);
    assert_eq!(symbols(shims[0]), ["demo::p", "demo::q"]);
}

fn digits(upper: bool) -> Vec<char> {
    (0..16)
        .map(|n| {
            let c = char::from_digit(n, 16).expect("a digit below 16");
            if upper { c.to_ascii_uppercase() } else { c }
        })
        .collect()
}

fn table_units(census: &Census) -> Vec<(&str, TableKind)> {
    census
        .hex_tables()
        .iter()
        .map(|t: &HexTable| (t.member.symbol.as_str(), t.table))
        .collect()
}

#[test]
fn every_form_of_a_hex_table_is_reported_in_both_cases() {
    let mut source = String::new();
    for (case, upper) in [("LO", false), ("UP", true)] {
        let d = digits(upper);
        let text: String = d.iter().collect();
        let chars: Vec<String> = d.iter().map(|c| format!("'{c}'")).collect();
        let bytes: Vec<String> = d.iter().map(|c| format!("b'{c}'")).collect();
        writeln!(source, "pub const {case}_STR: &str = \"{text}\";").expect("write");
        writeln!(source, "pub const {case}_BSTR: &[u8; 16] = b\"{text}\";").expect("write");
        writeln!(
            source,
            "pub const {case}_CHARS: [char; 16] = [{}];",
            chars.join(", ")
        )
        .expect("write");
        writeln!(
            source,
            "pub const {case}_BYTES: [u8; 16] = [{}];",
            bytes.join(", ")
        )
        .expect("write");
    }
    let census = load(&source);
    let found = table_units(&census);
    assert_eq!(found.len(), 8, "{found:?}");
    for form in ["STR", "BSTR", "CHARS", "BYTES"] {
        assert!(found.contains(&(&format!("demo::LO_{form}"), TableKind::HexLower)));
        assert!(found.contains(&(&format!("demo::UP_{form}"), TableKind::HexUpper)));
    }
}

#[test]
fn near_misses_of_a_hex_table_are_not_reported() {
    let lower: String = digits(false).iter().collect();
    let wrong = format!("{}g", &lower[..15]);
    let short = &lower[..15];
    let mixed = format!("{}{}", &lower[..14], "EF");
    let source = format!(
        "pub const OK: &str = \"{lower}\";\n\
         pub const WRONG: &str = \"{wrong}\";\n\
         pub const SHORT: &str = \"{short}\";\n\
         pub const MIXED: &str = \"{mixed}\";\n"
    );
    let census = load(&source);
    assert_eq!(table_units(&census), [("demo::OK", TableKind::HexLower)]);
}

#[test]
fn a_function_holding_a_table_is_reported_by_its_symbol() {
    let lower: String = digits(false).iter().collect();
    let census = load(&format!(
        "pub fn hex(n: u8) -> char {{\n    let t = \"{lower}\";\n    t.as_bytes()[n as usize] as char\n}}\n"
    ));
    let found = table_units(&census);
    assert_eq!(found, [("demo::hex", TableKind::HexLower)]);
    assert_eq!(census.hex_tables()[0].member.line, 1);
}

#[test]
fn constants_above_ffff_in_two_packages_are_shared() {
    let census = packages(&[
        (
            "alpha",
            "pub const A: u32 = 0x10000;\npub fn f() -> u32 { 0x10000 }\n\
             pub const SMALL: u32 = 0xffff;\npub const ONLY_A: u32 = 0x20000;\n",
        ),
        (
            "beta",
            "pub const B: u32 = 65536;\npub const SMALL_B: u32 = 0xffff;\n\
             pub const C: u32 = 0x30000;\n",
        ),
    ]);
    let shared = census.shared_constants();
    assert_eq!(shared.len(), 1, "{shared:?}");
    assert_eq!(shared[0].value, 0x10000);
    assert_eq!(shared[0].packages, 2);
    let members: Vec<(&str, &str)> = shared[0]
        .members
        .iter()
        .map(|m| (m.symbol.as_str(), m.package.as_str()))
        .collect();
    assert_eq!(
        members,
        [
            ("alpha::A", "alpha"),
            ("alpha::f", "alpha"),
            ("beta::B", "beta")
        ]
    );
}

#[test]
fn a_constant_in_one_package_is_not_shared() {
    let census = packages(&[
        (
            "alpha",
            "pub const A: u32 = 0x10000;\npub const B: u32 = 0x10000;\n",
        ),
        ("beta", "pub const C: u32 = 1;\n"),
    ]);
    assert_eq!(census.shared_constants(), []);
}

#[test]
fn shared_constants_run_by_value_ascending() {
    let census = packages(&[
        (
            "alpha",
            "pub const A: u32 = 0x30000;\npub const B: u32 = 0x10000;\n",
        ),
        (
            "beta",
            "pub const C: u32 = 0x10000;\npub const D: u32 = 0x30000;\n",
        ),
    ]);
    let values: Vec<u128> = census.shared_constants().iter().map(|c| c.value).collect();
    assert_eq!(values, [0x10000, 0x30000]);
}

const JOB_SOURCE: &str = "\
pub fn home(a: u32) -> u32 { a + a + a + a + a + a + a + a + a + a + a }
pub fn entry(b: u32) -> u32 { b + b + b + b + b + b + b + b + b + b + b }
pub fn variant(c: u32) -> u32 { c + c + c + c + c + c + c + c + c + c + c }
pub fn copy(d: u32) -> u32 { d + d + d + d + d + d + d + d + d + d + d }
pub const BIG: u32 = 65536;
pub const OTHER: u32 = 7;
";

fn matched(report: &raf_census::JobReport) -> Vec<(&str, MatchClass)> {
    report
        .matches
        .iter()
        .map(|m: &JobMatch| (m.symbol.as_str(), m.class))
        .collect()
}

#[test]
fn a_job_classifies_home_entry_point_variant_and_copies() {
    let census = load(JOB_SOURCE);
    let group = &census.groups()[0];
    assert_eq!(group.members.len(), 4);
    let job = Job::new("hash-bytes", "demo::home")
        .entry_points(["demo::entry"])
        .variants(["demo::variant"])
        .forbidding(JobForbidden {
            fingerprints: vec![group.fingerprint.clone()],
            constants: vec![0x10000],
        });
    let reports = census.match_jobs(&[job]);
    assert_eq!(reports.len(), 1);
    let report = &reports[0];
    assert_eq!(report.id, "hash-bytes");
    assert_eq!(report.home, "demo::home");
    assert_eq!(
        matched(report),
        [
            ("demo::BIG", MatchClass::Copy),
            ("demo::copy", MatchClass::Copy),
            ("demo::entry", MatchClass::Home),
            ("demo::home", MatchClass::Home),
            ("demo::variant", MatchClass::Variant),
        ]
    );
    assert_eq!(report.copies, 2);
    assert_eq!(report.matches[0].file, "src/lib.rs");
    assert_eq!(report.matches[0].line, 5);
}

#[test]
fn a_job_that_matches_nothing_has_an_empty_list() {
    let census = load(JOB_SOURCE);
    let none = Job::new("none", "demo::home").forbidding(JobForbidden {
        fingerprints: vec![Fingerprint::Body(1)],
        constants: vec![123_456],
    });
    // A rule id names no unit.
    let rule = Job::new("rule", "demo::home").forbidding(JobForbidden {
        fingerprints: vec![Fingerprint::Rule(
            RuleId::new("std-default-hasher").expect("a rule id"),
        )],
        constants: Vec::new(),
    });
    let empty = Job::new("empty", "demo::home");
    let reports = census.match_jobs(&[none, rule, empty]);
    let ids: Vec<&str> = reports.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["none", "rule", "empty"]);
    for report in &reports {
        assert!(report.matches.is_empty(), "{report:?}");
        assert_eq!(report.copies, 0);
    }
    assert_eq!(census.match_jobs(&[]), []);
}

#[test]
fn jobs_match_shim_and_table_fingerprints() {
    let lower: String = digits(false).iter().collect();
    let source = format!(
        "pub fn p(x: u32) -> u32 {{ helper(x, 1) }}\n\
         pub fn q(y: u32) -> u32 {{ helper(y, 1) }}\n\
         pub const T: &str = \"{lower}\";\n"
    );
    let census = load(&source);
    let shim = census.groups()[0].fingerprint.clone();
    assert!(matches!(shim, Fingerprint::Shim(_)));
    let job = Job::new("both", "demo::p").forbidding(JobForbidden {
        fingerprints: vec![shim, Fingerprint::Table(TableKind::HexLower)],
        constants: Vec::new(),
    });
    let report = census.match_jobs(&[job]).remove(0);
    assert_eq!(
        matched(&report),
        [
            ("demo::T", MatchClass::Copy),
            ("demo::p", MatchClass::Home),
            ("demo::q", MatchClass::Copy),
        ]
    );
    assert_eq!(report.copies, 2);
    let upper = Job::new("upper", "demo::p").forbidding(JobForbidden {
        fingerprints: vec![Fingerprint::Table(TableKind::HexUpper)],
        constants: Vec::new(),
    });
    assert_eq!(census.match_jobs(&[upper])[0].matches, []);
}

#[test]
fn a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones() {
    let census = load(FORWARDERS);
    let group = census
        .shim_groups()
        .find(|group| group.members.len() == 3)
        .expect("the three eligible forwarders group");
    let job = Job::new("forward", "demo::S::plain").forbidding(JobForbidden {
        fingerprints: vec![group.fingerprint.clone()],
        constants: Vec::new(),
    });
    let report = census.match_jobs(&[job]).remove(0);
    assert_eq!(
        matched(&report),
        [
            ("demo::Make::dflt", MatchClass::Copy),
            ("demo::S::build", MatchClass::Copy),
            ("demo::S::plain", MatchClass::Home),
        ]
    );
    assert_eq!(report.copies, 2);
    // The trait implementation method has the same shim form and is absent.
    let trait_impl = census
        .units()
        .iter()
        .find(|u| u.trait_impl && u.name == "make")
        .expect("the trait implementation method exists");
    assert!(trait_impl.print.as_ref().is_some_and(|p| p.shim.is_some()));
    assert!(report.matches.iter().all(|m| m.symbol != trait_impl.symbol));
    // A job forbidding the shim of the `Self::` forwarders matches nothing:
    // they have a shim form but no key, so no group carries that id.
    let via = census.unit("demo::S::via").expect("exists");
    let via_shim = via
        .print
        .as_ref()
        .and_then(|p| p.shim.as_ref())
        .expect("a `Self::` forwarder has a shim form")
        .fingerprint
        .clone();
    let job = Job::new("via", "demo::S::via").forbidding(JobForbidden {
        fingerprints: vec![via_shim],
        constants: Vec::new(),
    });
    assert_eq!(census.match_jobs(&[job])[0].matches, []);
}

#[test]
fn renamed_locals_inside_macro_arguments_land_in_one_body_group() {
    let census = load(
        "\
pub fn hex(b: u8) -> String { let s = format!(\"{x:02x}\", x = b); let t = vec![s.len(), s.len() + 1]; assert_eq!(t.len(), 2); s }
pub fn hexa(c: u8) -> String { let u = format!(\"{y:02x}\", y = c); let v = vec![u.len(), u.len() + 1]; assert_eq!(v.len(), 2); u }
pub fn other(c: u8) -> String { let u = format!(\"{y:02x}\", y = c); let v = vec![u.len(), u.capacity() + 1]; assert_eq!(v.len(), 2); u }
",
    );
    let groups: Vec<&Group> = census.body_groups().collect();
    assert_eq!(groups.len(), 1, "{groups:?}");
    assert_eq!(symbols(groups[0]), ["demo::hex", "demo::hexa"]);
    assert!(groups[0].tokens >= 20);
}
