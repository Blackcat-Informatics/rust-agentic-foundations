// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Units through `Census::load` over in-memory workspaces: one unit per source
//! item, the fields of each kind, the token sources, the constants and strings,
//! the thresholds and the load cap; and the census of this repository's own
//! `crates`.

use std::collections::BTreeSet;

use raf_census::{Census, CensusOptions, Error, StringKind, TableKind, Unit, UnitKind};
use raf_source::{DiskTree, MemoryTree};

const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

fn tree(files: &[(&str, &str)]) -> MemoryTree {
    let mut all = vec![("Cargo.toml", MANIFEST)];
    all.extend_from_slice(files);
    all.into_iter().collect()
}

fn load_with(files: &[(&str, &str)], options: &CensusOptions) -> Result<Census, Error> {
    Census::load(&tree(files), &["."], options)
}

fn load(files: &[(&str, &str)]) -> Census {
    load_with(files, &CensusOptions::default()).expect("the workspace loads")
}

fn unit<'a>(census: &'a Census, symbol: &str) -> &'a Unit {
    census
        .unit(symbol)
        .unwrap_or_else(|| panic!("no unit `{symbol}` in {:?}", symbols(census)))
}

fn symbols(census: &Census) -> Vec<&str> {
    census.units().iter().map(|u| u.symbol.as_str()).collect()
}

/// Every kind of unit and every kind that is not one. Line numbers are those of
/// the text below.
const KINDS: &str = "\
pub fn free(a: u8) -> u8 {
    a + 0xff
}
pub struct S;
pub enum E {}
pub type Alias = u8;
pub mod inner {}
macro_rules! make { () => {}; }
pub trait T {
    const REQ: u32;
    const DEF: u32 = 9;
    fn needed(&self) -> u32;
    fn provided(&self) -> u32 {
        7
    }
}
impl S {
    pub fn inherent(&self, n: u32) -> u32 {
        n + 1
    }
    pub const ASSOC: u32 = 0x10000;
}
impl T for S {
    const REQ: u32 = 2;
    fn needed(&self) -> u32 {
        1
    }
}
pub const C: u32 = 0x10000;
pub static ST: &str = \"hello\";
";

#[test]
fn each_kind_yields_one_unit_with_every_field() {
    let census = load(&[("src/lib.rs", KINDS)]);
    let mut found = symbols(&census);
    found.sort_unstable();
    assert_eq!(
        found,
        [
            "demo::C",
            "demo::S::ASSOC",
            "demo::S::REQ",
            "demo::S::inherent",
            "demo::S::needed",
            "demo::ST",
            "demo::T::DEF",
            "demo::T::provided",
            "demo::free",
        ]
    );

    let free = unit(&census, "demo::free");
    assert_eq!(
        (
            free.name.as_str(),
            free.package.as_str(),
            free.file.as_str()
        ),
        ("free", "demo", "src/lib.rs")
    );
    assert_eq!((free.line, free.kind), (1, UnitKind::Fn));
    assert!(free.shipping && !free.trait_impl && !free.via_self);
    assert_eq!(free.params, ["a"]);
    let print = free.print.as_ref().expect("a function has a print");
    assert_eq!(print.tokens, 3);
    assert!(print.body.is_none() && print.shim.is_none() && free.shim_key.is_none());
    assert_eq!(free.constants, BTreeSet::from([255]));
    assert!(free.strings.is_empty() && free.tables.is_empty());

    let inherent = unit(&census, "demo::S::inherent");
    assert_eq!((inherent.name.as_str(), inherent.line), ("inherent", 18));
    assert_eq!(inherent.kind, UnitKind::Method);
    assert!(!inherent.trait_impl && inherent.shipping);
    assert_eq!(inherent.params, ["n"]);
    assert_eq!(inherent.constants, BTreeSet::from([1]));

    let needed = unit(&census, "demo::S::needed");
    assert_eq!((needed.kind, needed.line), (UnitKind::Method, 25));
    assert!(needed.trait_impl && needed.shipping);
    assert_eq!(needed.params.len(), 0);

    let provided = unit(&census, "demo::T::provided");
    assert_eq!((provided.kind, provided.line), (UnitKind::Method, 13));
    assert!(!provided.trait_impl);
    assert_eq!(provided.constants, BTreeSet::from([7]));

    let konst = unit(&census, "demo::C");
    assert_eq!((konst.kind, konst.line), (UnitKind::Const, 29));
    assert!(konst.print.is_none() && konst.shim_key.is_none() && konst.params.is_empty());
    assert_eq!(konst.constants, BTreeSet::from([0x10000]));
    assert!(konst.shipping);

    let assoc = unit(&census, "demo::S::ASSOC");
    assert_eq!((assoc.kind, assoc.line), (UnitKind::Const, 21));
    assert_eq!(assoc.constants, BTreeSet::from([0x10000]));
    assert_eq!(unit(&census, "demo::T::DEF").constants, BTreeSet::from([9]));
    assert_eq!(unit(&census, "demo::S::REQ").constants, BTreeSet::from([2]));

    let stat = unit(&census, "demo::ST");
    assert_eq!((stat.kind, stat.line), (UnitKind::Static, 30));
    assert_eq!(stat.strings.len(), 1);
    assert_eq!(
        (stat.strings[0].value.as_str(), stat.strings[0].line),
        ("hello", 30)
    );
}

#[test]
fn a_module_declared_by_a_library_and_a_binary_is_one_unit() {
    let census = load(&[
        ("src/lib.rs", "mod shared;\n"),
        ("src/main.rs", "mod shared;\nfn main() {}\n"),
        ("src/shared.rs", "pub fn go(x: u8) -> u8 {\n    x\n}\n"),
    ]);
    let shared: Vec<&Unit> = census
        .units()
        .iter()
        .filter(|unit| unit.file == "src/shared.rs")
        .collect();
    assert_eq!(shared.len(), 1, "{:?}", symbols(&census));
    assert!(shared[0].shipping);
    assert_eq!(shared[0].line, 1);
    // The binary's own function is a unit of the binary alone.
    assert_eq!(
        census.units().iter().filter(|u| u.name == "main").count(),
        1
    );
}

#[test]
fn a_support_module_shared_by_two_test_binaries_is_one_unit_that_does_not_ship() {
    let census = load(&[
        ("src/lib.rs", "pub fn ship() {}\n"),
        ("tests/one.rs", "mod support;\n#[test]\nfn a() {}\n"),
        ("tests/two.rs", "mod support;\n#[test]\nfn b() {}\n"),
        (
            "tests/support/mod.rs",
            "pub fn helper(x: u8) -> u8 {\n    x\n}\n",
        ),
    ]);
    let support: Vec<&Unit> = census
        .units()
        .iter()
        .filter(|unit| unit.file == "tests/support/mod.rs")
        .collect();
    assert_eq!(support.len(), 1, "{:?}", symbols(&census));
    assert!(!support[0].shipping);
    assert_eq!(support[0].package, "demo");
    assert_eq!(
        support[0].symbol, "demo::test_crate::one::support::helper",
        "the smallest path of equally non-shipping reaches"
    );
    // The test functions are units of their targets and do not ship.
    assert!(!unit(&census, "demo::test_crate::one::a").shipping);
    assert!(unit(&census, "demo::ship").shipping);
}

#[test]
fn shipping_is_true_when_any_reach_ships_and_the_shipping_path_is_the_symbol() {
    let census = load(&[
        ("src/lib.rs", "mod zed;\n"),
        ("src/zed.rs", "pub fn go(x: u8) -> u8 {\n    x\n}\n"),
        (
            "tests/a.rs",
            "#[path = \"../src/zed.rs\"]\nmod zed;\n#[test]\nfn t() {}\n",
        ),
    ]);
    let go: Vec<&Unit> = census
        .units()
        .iter()
        .filter(|unit| unit.file == "src/zed.rs")
        .collect();
    assert_eq!(go.len(), 1, "{:?}", symbols(&census));
    // `demo::test_crate::a::zed::go` sorts before `demo::zed::go`, and still
    // loses to the reach that ships.
    assert_eq!(go[0].symbol, "demo::zed::go");
    assert!(go[0].shipping);
}

#[test]
fn test_code_of_a_shipping_target_does_not_ship() {
    let census = load(&[(
        "src/lib.rs",
        "pub fn real() {}\n#[cfg(test)]\nmod tests {\n    fn helper() {}\n    #[test]\n    fn case() {}\n}\n",
    )]);
    assert!(unit(&census, "demo::real").shipping);
    assert!(!unit(&census, "demo::tests::helper").shipping);
    assert!(!unit(&census, "demo::tests::case").shipping);
}

#[test]
fn file_context_names_package_path_and_shipping() {
    let census = load(&[
        ("src/lib.rs", "pub fn go() {}\n"),
        ("tests/it.rs", "#[test]\nfn t() {}\n"),
    ]);
    let lib = census.file("src/lib.rs").expect("the library file");
    assert_eq!(
        (lib.package.as_str(), lib.path.as_str(), lib.shipping),
        ("demo", "src/lib.rs", true)
    );
    let test = census.file("tests/it.rs").expect("the test file");
    assert_eq!((test.package.as_str(), test.shipping), ("demo", false));
    assert!(census.file("src/missing.rs").is_none());
    assert_eq!(census.files().len(), 2);
}

#[test]
fn parameters_of_every_pattern_are_renamed() {
    let census = load(&[(
        "src/lib.rs",
        "\
pub struct P { x: u8, y: u8 }
pub struct S;
impl S {
    pub fn pattern(&mut self, (a, b): (u8, u8), mut c: u8, P { x, y: z }: P) -> u8 {
        keep(a, b, c, x, z, 1)
    }
    pub fn twin(&mut self, (p, q): (u8, u8), mut r: u8, P { x: s, y: t }: P) -> u8 {
        keep(p, q, r, s, t, 1)
    }
    pub fn other(&mut self, (p, q): (u8, u8), mut r: u8, P { x: s, y: t }: P) -> u8 {
        keep(q, p, r, s, t, 1)
    }
}
",
    )]);
    let pattern = unit(&census, "demo::S::pattern");
    assert_eq!(pattern.params, ["a", "b", "c", "x", "z"]);
    let shim = |symbol: &str| {
        unit(&census, symbol)
            .print
            .as_ref()
            .and_then(|print| print.shim.as_ref())
            .map(|hashed| hashed.digest)
    };
    assert!(shim("demo::S::pattern").is_some());
    assert_eq!(shim("demo::S::pattern"), shim("demo::S::twin"));
    assert_ne!(shim("demo::S::pattern"), shim("demo::S::other"));
}

#[test]
fn a_constant_whose_type_names_a_number_does_not_hold_it() {
    let census = load(&[(
        "src/lib.rs",
        "\
pub const TABLE: [u8; 0x10000] = [0; 4];
#[allow(dead_code)]
pub static LIMIT: [u8; 65536] = [7; 2];
pub const BIG: u32 = 0x10000;
",
    )]);
    let table = unit(&census, "demo::TABLE");
    assert_eq!(table.constants, BTreeSet::from([0, 4]));
    assert!(!table.constants.contains(&65536));
    assert_eq!(
        unit(&census, "demo::LIMIT").constants,
        BTreeSet::from([2, 7])
    );
    assert!(unit(&census, "demo::BIG").constants.contains(&65536));
}

#[test]
fn every_spelling_of_an_integer_is_one_constant() {
    let census = load(&[(
        "src/lib.rs",
        "pub fn go() -> u32 {\n    0xff + 255 + 0b1111_1111 + 0o377 + 255u32\n}\n",
    )]);
    assert_eq!(unit(&census, "demo::go").constants, BTreeSet::from([255]));
}

#[test]
fn strings_carry_their_absolute_lines() {
    let census = load(&[(
        "src/lib.rs",
        "\
// a comment line
pub fn go(x: u8) -> String {
    let a = \"first\";
    let b = format!(
        \"{x}: {}\",
        r#\"raw\"#,
    );
    let c = b\"bytes\";
    let d = \"multi
line\";
    let e = \"after\";
    a
}
pub const LATE: &str =
    \"late\";
",
    )]);
    let go = unit(&census, "demo::go");
    let found: Vec<(&str, usize)> = go
        .strings
        .iter()
        .map(|s| (s.value.as_str(), s.line))
        .collect();
    assert_eq!(
        found,
        [
            ("first", 3),
            ("{x}: {}", 5),
            ("raw", 6),
            ("bytes", 8),
            ("multi\nline", 9),
            ("after", 11),
        ]
    );
    let late = unit(&census, "demo::LATE");
    assert_eq!(late.line, 14);
    assert_eq!(late.strings.len(), 1);
    assert_eq!(late.strings[0].line, 15);
}

#[test]
fn hex_tables_are_read_from_strings_and_arrays() {
    let census = load(&[(
        "src/lib.rs",
        "\
pub const LOWER: &str = \"0123456789abcdef\";
pub const UPPER: &[u8; 16] = b\"0123456789ABCDEF\";
pub const ARRAY: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
pub const MIXED: &str = \"0123456789abcdEF\";
",
    )]);
    assert_eq!(
        unit(&census, "demo::LOWER").tables,
        BTreeSet::from([TableKind::HexLower])
    );
    assert_eq!(
        unit(&census, "demo::UPPER").tables,
        BTreeSet::from([TableKind::HexUpper])
    );
    assert_eq!(
        unit(&census, "demo::ARRAY").tables,
        BTreeSet::from([TableKind::HexLower])
    );
    assert!(unit(&census, "demo::MIXED").tables.is_empty());
}

/// A function whose structural tokens are `count`: `a + a + ... + a`, then one
/// more token, `;`, when `count` is even.
fn body_of(name: &str, count: usize) -> String {
    let terms = count.div_ceil(2);
    let sum = vec!["a"; terms].join(" + ");
    let tail = if count.is_multiple_of(2) { ";" } else { "" };
    format!("pub fn {name}(a: u32) -> u32 {{ {sum}{tail} }}\n")
}

#[test]
fn min_tokens_decides_the_body_fingerprint_at_the_default_and_at_ten() {
    let source = [9, 10, 19, 20]
        .map(|count| body_of(&format!("f{count}"), count))
        .concat();
    let has_body = |census: &Census, count: usize| {
        let unit = unit(census, &format!("demo::f{count}"));
        let print = unit.print.as_ref().expect("a function has a print");
        assert_eq!(print.tokens, count);
        print.body.is_some()
    };
    let default = load(&[("src/lib.rs", &source)]);
    assert!(!has_body(&default, 19) && has_body(&default, 20));
    assert!(!has_body(&default, 9) && !has_body(&default, 10));

    let ten = load_with(
        &[("src/lib.rs", &source)],
        &CensusOptions::default().min_tokens(10).expect("valid"),
    )
    .expect("the workspace loads");
    assert!(!has_body(&ten, 9) && has_body(&ten, 10));
    assert!(has_body(&ten, 19) && has_body(&ten, 20));
}

/// `head` and comment lines up to exactly `len` bytes, each line ending in a
/// newline.
fn padded(head: &str, len: usize) -> String {
    let mut text = head.to_owned();
    while text.len() + 40 < len {
        text.push_str("// ................................\n");
    }
    let rest = len - text.len();
    text.push_str("//");
    text.push_str(&".".repeat(rest - 3));
    text.push('\n');
    assert_eq!(text.len(), len);
    text
}

#[test]
fn max_file_bytes_caps_a_file_and_the_cap_itself_loads() {
    let cap = 600;
    let at_cap = padded("pub fn go() {}\n", cap);
    let census = load_with(
        &[("src/lib.rs", &at_cap)],
        &CensusOptions::default().max_file_bytes(cap as u64),
    )
    .expect("a file at the cap loads");
    assert!(census.unit("demo::go").is_some());

    let above = padded("pub fn go() {}\n", cap + 1);
    let refused = load_with(
        &[("src/lib.rs", &above)],
        &CensusOptions::default().max_file_bytes(cap as u64),
    )
    .expect_err("a file above the cap is refused");
    assert!(matches!(refused, Error::Load(_)), "{refused:?}");
    let message = refused.to_string();
    assert!(message.contains("src/lib.rs"), "{message}");
}

#[test]
fn shim_keys_are_absent_for_trait_impl_methods_and_self_forwarders() {
    let census = load(&[(
        "src/lib.rs",
        "\
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
    pub fn via_turbofish(x: u32) -> u32 {
        Self::inner::<u8>(x, 1)
    }
    pub fn not_a_call(x: u32) -> u32 {
        helper(x, Self::ONE)
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
    pub fn fish(x: u32) -> u32 {
        Other::inner::<u8>(x, 1)
    }
    pub fn fish_twin(w: u32) -> u32 {
        Other::inner::<u8>(w, 1)
    }
}
",
    )]);
    let key = |symbol: &str| unit(&census, symbol).shim_key.clone();
    let shim = |symbol: &str| {
        unit(&census, symbol)
            .print
            .as_ref()
            .and_then(|print| print.shim.as_ref())
            .map(|hashed| hashed.fingerprint.clone())
    };

    // Forwarders with a key: the free-standing neighbours that do group.
    let plain = key("demo::S::plain").expect("an inherent forwarder has a key");
    assert_eq!(key("demo::S::build"), Some(plain.clone()));
    assert_eq!(key("demo::Make::dflt"), Some(plain.clone()));
    assert_eq!(shim("demo::S::plain"), Some(plain));
    let other = key("demo::U::neighbour").expect("a forwarder has a key");
    assert_eq!(key("demo::U::twin"), Some(other));
    assert_eq!(
        key("demo::S::not_a_call"),
        shim("demo::S::not_a_call"),
        "`Self::ONE` as an argument is not a call through `Self::`"
    );
    assert!(key("demo::S::not_a_call").is_some());

    // The same forwarder as a trait implementation method has a shim, no key.
    let trait_impls: Vec<&Unit> = census
        .units()
        .iter()
        .filter(|u| u.trait_impl && u.name == "make")
        .collect();
    assert_eq!(trait_impls.len(), 1);
    assert!(
        trait_impls[0]
            .print
            .as_ref()
            .is_some_and(|p| p.shim.is_some())
    );
    assert!(trait_impls[0].shim_key.is_none());

    // A forwarder through `Self::` has a shim and no key.
    for symbol in ["demo::S::via", "demo::U::via"] {
        let via = unit(&census, symbol);
        assert!(via.via_self, "{symbol}");
        assert!(
            via.print.as_ref().is_some_and(|p| p.shim.is_some()),
            "{symbol}"
        );
        assert!(via.shim_key.is_none(), "{symbol}");
    }
}

#[test]
fn a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin() {
    let census = load(&[(
        "src/lib.rs",
        "pub struct S;\nimpl S {\n    pub fn via(x: u32) -> u32 {\n        Self::inner::<u8>(x, 1)\n    }\n    pub fn fish(x: u32) -> u32 {\n        Other::inner::<u8>(x, 1)\n    }\n    pub fn fish_twin(w: u32) -> u32 {\n        Other::inner::<u8>(w, 1)\n    }\n    pub fn other(w: u32) -> u32 {\n        Other::inner::<u16>(w, 1)\n    }\n}\n",
    )]);
    let key = |symbol: &str| unit(&census, symbol).shim_key.clone();
    let turbofish = unit(&census, "demo::S::via");
    assert!(turbofish.via_self && turbofish.shim_key.is_none());
    assert!(turbofish.print.as_ref().is_some_and(|p| p.shim.is_some()));
    let keyed = key("demo::S::fish").expect("a turbofish forwarder has a key");
    assert_eq!(key("demo::S::fish_twin"), Some(keyed));
    assert_ne!(key("demo::S::fish"), key("demo::S::other"));
}

#[test]
fn a_self_call_at_any_depth_removes_the_shim_key() {
    let census = load(&[(
        "src/lib.rs",
        "\
pub struct S;
impl S {
    pub fn nested_argument(x: u32) -> u32 {
        helper(x, Self::f(1))
    }
    pub fn nested_callee_argument(x: u32) -> u32 {
        Self::f(helper(x, 1))
    }
    pub fn variant(x: u32) -> S {
        Self::V(x, 1)
    }
    pub fn constant_argument(x: u32) -> u32 {
        helper(x, Self::ONE)
    }
    pub fn type_path(x: u32) -> u32 {
        Vec::<u8>::new(x, 1)
    }
}
",
    )]);
    for symbol in [
        "demo::S::nested_argument",
        "demo::S::nested_callee_argument",
        "demo::S::variant",
    ] {
        let unit = unit(&census, symbol);
        assert!(unit.via_self, "{symbol}");
        assert!(
            unit.print.as_ref().is_some_and(|p| p.shim.is_some()),
            "{symbol}"
        );
        assert!(unit.shim_key.is_none(), "{symbol}");
    }
    for symbol in ["demo::S::constant_argument", "demo::S::type_path"] {
        let unit = unit(&census, symbol);
        assert!(!unit.via_self, "{symbol}");
        assert!(unit.shim_key.is_some(), "{symbol}");
    }
}

#[test]
fn byte_strings_and_c_strings_are_strings() {
    let census = load(&[(
        "src/lib.rs",
        "\
pub fn go() {
    let a = \"text\";
    let b = b\"bytes\";
    let c = c\"cstr\";
    let d = b\"\\xff\\xfe\";
}
pub const DIGITS: &[u8; 16] = b\"0123456789abcdef\";
pub const BROKEN: &[u8; 16] = b\"0123456789abcd\\xff\\xfe\";
",
    )]);
    let found: Vec<(&str, StringKind, usize)> = unit(&census, "demo::go")
        .strings
        .iter()
        .map(|s| (s.value.as_str(), s.kind, s.line))
        .collect();
    assert_eq!(
        found,
        [
            ("text", StringKind::Str, 2),
            ("bytes", StringKind::Bytes, 3),
            ("cstr", StringKind::CStr, 4),
            ("\u{fffd}\u{fffd}", StringKind::Bytes, 5),
        ]
    );
    assert_eq!(
        unit(&census, "demo::DIGITS").tables,
        BTreeSet::from([TableKind::HexLower])
    );
    let broken = unit(&census, "demo::BROKEN");
    assert_eq!(broken.strings.len(), 1);
    assert!(broken.tables.is_empty());
}

#[test]
fn a_unit_is_found_by_symbol() {
    let census = load(&[("src/lib.rs", "pub fn go() {}\npub fn stay() {}\n")]);
    assert_eq!(unit(&census, "demo::stay").line, 2);
    assert!(census.unit("demo::missing").is_none());
    assert_eq!(census.units_of("demo::go").count(), 1);
    assert_eq!(census.units_of("demo::missing").count(), 0);
}

#[test]
fn a_workspace_without_roots_does_not_load() {
    let refused = Census::load(&tree(&[]), &[], &CensusOptions::default());
    assert!(matches!(refused, Err(Error::Load(_))));
}

#[test]
fn this_repositorys_own_crates_load_into_distinct_units() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists");
    let disk = DiskTree::new(&root).expect("the repository root opens");
    let census =
        Census::load(&disk, &["crates"], &CensusOptions::default()).expect("the crates load");
    assert!(census.units().len() > 100, "{} units", census.units().len());
    let mut seen = BTreeSet::new();
    for unit in census.units() {
        assert!(
            seen.insert((unit.file.as_str(), unit.line)),
            "two units at {}:{}",
            unit.file,
            unit.line
        );
    }
    assert!(census.unit("raf_census::census::Census::load").is_some());
}
