// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The work and memory budgets of a load: the tokens of one file, the bytes a
//! load reads, the bytes of index records, the bytes of file facts and the
//! bytes of glob tables. Each is shown at its limit, where the load succeeds,
//! and one over, where it is refused with a `Budget` error that names the
//! budget and the file.

use std::fmt::Write as _;

use raf_source::{
    Budget, LoadError, LoadErrorKind, LoadOptions, MemoryTree, PARSE_BYTES_PER_TOKEN, RelPath,
    Usage, Workspace, parse_source,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// A token limit lowered for speed; the default behaves the same and is
/// checked on its own.
const TOKENS: usize = 100_000;

/// Options with the token limit lowered to [`TOKENS`].
fn lowered() -> LoadOptions {
    let mut options = LoadOptions::default();
    options.max_file_tokens = TOKENS;
    options
}

/// The tokens a load at the lowered limit counted.
fn tokens_at_lowered(text: &str) -> usize {
    load_with(&[("src/lib.rs", text)], &lowered())
        .unwrap_or_else(|error| panic!("loads: {error}"))
        .usage()
        .max_file_tokens
}

fn tree(files: &[(&str, &str)]) -> MemoryTree {
    let mut all = vec![("Cargo.toml", DEMO)];
    all.extend_from_slice(files);
    all.into_iter().collect()
}

fn load_with(files: &[(&str, &str)], options: &LoadOptions) -> Result<Workspace, LoadError> {
    Workspace::load_with(&tree(files), &["."], options)
}

fn usage(files: &[(&str, &str)]) -> Usage {
    load_with(files, &LoadOptions::default())
        .unwrap_or_else(|error| panic!("loads: {error}"))
        .usage()
}

/// The budget and limit of a refusal, after checking that it names `file`.
fn refused(result: Result<Workspace, LoadError>, file: &str) -> (Budget, usize, Option<usize>) {
    let error = result.expect_err("over the budget");
    assert_eq!(error.file.as_str(), file, "{error}");
    match error.kind {
        LoadErrorKind::Budget { budget, limit } => {
            let text = error.to_string();
            assert!(
                text.contains(file) && text.contains(budget.name()),
                "{text}"
            );
            (budget, limit, error.line)
        }
        other => panic!("not a budget refusal: {other:?}"),
    }
}

/// `count` empty statements, each one token.
fn semis(count: usize) -> String {
    "; ".repeat(count)
}

/// A file of exactly `tokens` tokens: `head`, a body of empty statements that
/// pads it, and the closing brace. `head` opens a function body and holds
/// `head_tokens` tokens.
fn padded(head: &str, head_tokens: usize, tokens: usize) -> String {
    let pad = tokens - head_tokens - 1;
    format!("{head}{}}}\n", semis(pad))
}

/// `a<b; ` repeated, the comparison chain of the pre-scan's measurements.
fn comparisons(tokens: usize) -> String {
    // `fn f ( ) { let _ = a < b ;` is twelve tokens, each further `a<b;` four.
    let head = format!(
        "fn f() {{ let _ = {}",
        "a<b; ".repeat((tokens - 13) / 4 + 1)
    );
    let head_tokens = 8 + 4 * ((tokens - 13) / 4 + 1);
    padded(&head, head_tokens, tokens)
}

/// Statements that each name a type nested 64 generic arguments deep.
fn deep_generic_arguments(tokens: usize) -> String {
    let one = format!("let _: {}u8{}; ", "A<".repeat(64), ">".repeat(64));
    // `let _ :` is three tokens, `A <` two per level, `u8` one, `>` one per
    // level and `;` one.
    let per = 3 + 2 * 64 + 1 + 64 + 1;
    let count = (tokens - 7) / per;
    padded(
        &format!("fn f() {{ {}", one.repeat(count)),
        5 + per * count,
        tokens,
    )
}

/// A run of outer attributes on one function.
fn attribute_run(tokens: usize) -> String {
    // `#[a]` is four tokens; `fn f ( ) {` five.
    let count = (tokens - 6) / 4;
    padded(
        &format!("{}fn f() {{ ", "#[a] ".repeat(count)),
        4 * count + 5,
        tokens,
    )
}

/// The costliest shape the other budgets admit: one function body of a power of
/// two and one more empty statements, whose list of statements doubles its
/// storage for the last one, after a body that takes the remaining tokens.
fn statement_lists(tokens: usize) -> String {
    // `fn a ( ) {` and `}` are six tokens around each body.
    let big = (1_usize << (tokens - 13).ilog2()) + 1;
    let rest = tokens - big - 6;
    format!(
        "fn p() {{ {}}}\nfn a() {{ {}}}\n",
        semis(rest - 6),
        semis(big)
    )
}

#[test]
fn a_file_at_the_token_limit_parses_and_one_more_token_is_refused_naming_its_line() {
    // `m ! {` and `}` are four tokens around the identifiers.
    let at = format!("m!{{ {} }}\n", "a ".repeat(TOKENS - 4));
    assert_eq!(tokens_at_lowered(&at), TOKENS);
    let over = format!("m!{{ {}\n}}\n", "a ".repeat(TOKENS - 3));
    let error = load_with(&[("src/lib.rs", &over)], &lowered()).expect_err("one over");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::FileTokens,
                limit: TOKENS
            }
        ),
        "{error}"
    );
    assert_eq!(error.file.as_str(), "src/lib.rs");
    assert_eq!(error.line, Some(2));
    // `parse_source` applies the default limit, refused before any parse.
    let file = RelPath::new("src/lib.rs").expect("path");
    let default = Budget::FileTokens.limit();
    let past_default = format!("m!{{ {}\n}}\n", "a ".repeat(default - 3));
    let error = parse_source(&file, &past_default, |_| ()).expect_err("over the default");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::FileTokens,
                limit
            } if limit == default
        ),
        "{error}"
    );
    assert_eq!(error.line, Some(2));
}

/// A shape: a name and the text of it with a given number of tokens.
type Shape = (&'static str, fn(usize) -> String);

#[test]
fn each_costly_shape_at_the_token_limit_loads_and_one_more_token_is_refused() {
    let shapes: [Shape; 4] = [
        ("comparisons", comparisons),
        ("deep generic arguments", deep_generic_arguments),
        ("attribute run", attribute_run),
        ("statement lists", statement_lists),
    ];
    for (name, shape) in shapes {
        assert_eq!(tokens_at_lowered(&shape(TOKENS)), TOKENS, "{name}");
        let over = shape(TOKENS + 1);
        let (budget, limit, _) = refused(
            load_with(&[("src/lib.rs", &over)], &lowered()),
            "src/lib.rs",
        );
        assert_eq!((budget, limit), (Budget::FileTokens, TOKENS), "{name}");
    }
}

#[test]
fn a_doc_comment_counts_as_the_tokens_of_its_attribute_and_other_comments_count_nothing() {
    let text = "//! inner\n/*! inner block */\n/// outer\n/** outer block */\n\
                //// plain\n/*** plain */\n/**/\n// plain\nfn f() {}\n";
    // Seven for each inner doc comment, six for each outer one and six for
    // `fn f ( ) { }`.
    assert_eq!(
        usage(&[("src/lib.rs", text)]).max_file_tokens,
        7 + 7 + 6 + 6 + 6
    );
    // A file of doc comments alone passes the limit: each counts.
    let docs = format!("{}fn f() {{}}\n", "/// a\n".repeat(TOKENS / 6));
    let (budget, _, line) = refused(
        load_with(&[("src/lib.rs", &docs)], &lowered()),
        "src/lib.rs",
    );
    assert_eq!(budget, Budget::FileTokens);
    assert_eq!(line, Some(TOKENS / 6 + 1));
}

/// The facts of each adversarial fixture: a 1 MiB doc comment, a 1 MiB
/// written self type and a `use` group of long names, with the line of the
/// fact that holds the payload.
fn large_payloads() -> [(&'static str, String, usize); 3] {
    let long = |index: usize| format!("N{index}_{}", "x".repeat(64 * 1024));
    let names: Vec<String> = (0..16).map(long).collect();
    [
        (
            "doc comment",
            format!("/// {}\npub fn f() {{}}\n", "d".repeat(1 << 20)),
            1,
        ),
        (
            "written self type",
            format!(
                "pub trait Tr {{}}\nimpl Tr for ({}) {{}}\n",
                names.join(", ")
            ),
            2,
        ),
        (
            "use group",
            format!("use std::{{{}}};\n", names.join(", ")),
            1,
        ),
    ]
}

#[test]
fn each_large_fact_payload_is_charged_and_refused_at_its_line_once_over_the_fact_budget() {
    for (kind, text, line) in large_payloads() {
        let files = [("src/lib.rs", text.as_str())];
        let spent = usage(&files).fact_bytes;
        assert!(spent > 1 << 20, "{kind}: {spent}");
        // At the limit the load succeeds; one byte less is refused.
        let mut options = LoadOptions::default();
        options.max_fact_bytes = spent;
        load_with(&files, &options).unwrap_or_else(|e| panic!("{kind} at the limit: {e}"));
        options.max_fact_bytes = spent - 1;
        let (budget, limit, _) = refused(load_with(&files, &options), "src/lib.rs");
        assert_eq!((budget, limit), (Budget::FactBytes, spent - 1), "{kind}");
        // A limit below the payload is passed at the payload's own line.
        options.max_fact_bytes = 1 << 20;
        let (budget, _, at) = refused(load_with(&files, &options), "src/lib.rs");
        assert_eq!((budget, at), (Budget::FactBytes, Some(line)), "{kind}");
    }
}

#[test]
fn the_fact_budget_counts_over_every_file_of_the_load() {
    let doc = format!("/// {}\npub fn f() {{}}\n", "d".repeat(4096));
    let files = [
        ("src/lib.rs", "mod a;\nmod b;\n"),
        ("src/a.rs", doc.as_str()),
        ("src/b.rs", doc.as_str()),
    ];
    let spent = usage(&files).fact_bytes;
    let mut options = LoadOptions::default();
    options.max_fact_bytes = spent;
    load_with(&files, &options).expect("at the limit");
    options.max_fact_bytes = spent - 1;
    // The second file passes the budget that the first left.
    let (budget, _, _) = refused(load_with(&files, &options), "src/b.rs");
    assert_eq!(budget, Budget::FactBytes);
}

#[test]
fn index_records_at_the_item_budget_load_and_one_byte_less_is_refused() {
    let files = [
        (
            "src/lib.rs",
            "pub mod a;\npub use a::f as g;\npub fn top() {}\n",
        ),
        (
            "src/a.rs",
            "pub fn f() {}\npub struct S;\nimpl S { pub fn m() {} }\n",
        ),
    ];
    let spent = usage(&files).item_bytes;
    let mut options = LoadOptions::default();
    options.max_item_bytes = spent;
    load_with(&files, &options).expect("at the limit");
    options.max_item_bytes = spent - 1;
    let error = load_with(&files, &options).expect_err("one byte over");
    assert!(
        matches!(
            error.kind,
            LoadErrorKind::Budget {
                budget: Budget::ItemBytes,
                limit
            } if limit == spent - 1
        ),
        "{error}"
    );
    assert!(
        error.to_string().contains(Budget::ItemBytes.name()),
        "{error}"
    );
    // The last records charged are the members of the impl block in `a.rs`.
    assert_eq!(error.file.as_str(), "src/a.rs", "{error}");
    assert_eq!(error.line, Some(3), "{error}");
}

#[test]
fn a_long_module_name_is_charged_in_every_item_path_and_refused_over_the_item_budget() {
    let name = format!("m{}", "x".repeat(64 * 1024));
    let fns = (0..100).fold(String::new(), |mut text, i| {
        writeln!(text, "    pub fn f{i}() {{}}").expect("write to a string");
        text
    });
    let text = format!("pub mod {name} {{\n{fns}}}\n");
    let files = [("src/lib.rs", text.as_str())];
    // Each of the hundred item paths holds the long name.
    assert!(usage(&files).item_bytes > 100 * 64 * 1024);
    let mut options = LoadOptions::default();
    options.max_item_bytes = 1 << 20;
    let (budget, limit, line) = refused(load_with(&files, &options), "src/lib.rs");
    assert_eq!((budget, limit), (Budget::ItemBytes, 1 << 20));
    assert!(line.is_some_and(|line| line > 1), "{line:?}");
}

#[test]
fn a_file_reached_by_many_module_paths_is_charged_once_per_path_and_refused_over_the_item_budget() {
    let fns = (0..100).fold(String::new(), |mut text, i| {
        writeln!(text, "pub fn f{i}() {{}}").expect("write to a string");
        text
    });
    let decls = (0..100).fold(String::new(), |mut text, i| {
        writeln!(text, "#[path = \"leaf.rs\"] pub mod p{i};").expect("write to a string");
        text
    });
    let files = [
        ("src/lib.rs", decls.as_str()),
        ("src/leaf.rs", fns.as_str()),
    ];
    let spent = usage(&files).item_bytes;
    // Ten thousand items, a few hundred bytes each at the least.
    assert!(spent > 100 * 100 * 256, "{spent}");
    let mut options = LoadOptions::default();
    options.max_item_bytes = spent / 2;
    let (budget, _, _) = refused(load_with(&files, &options), "src/leaf.rs");
    assert_eq!(budget, Budget::ItemBytes);
}

#[test]
fn the_bytes_read_at_the_total_budget_load_and_one_byte_less_is_refused_naming_the_file() {
    let padding = format!("// {}\npub fn f() {{}}\n", "p".repeat(1000));
    let files = [
        ("src/lib.rs", "pub mod a;\n"),
        ("src/a.rs", padding.as_str()),
    ];
    let spent = usage(&files).total_bytes;
    assert_eq!(spent, DEMO.len() + "pub mod a;\n".len() + padding.len());
    let mut options = LoadOptions::default();
    options.max_total_bytes = spent;
    load_with(&files, &options).expect("at the limit");
    options.max_total_bytes = spent - 1;
    let (budget, limit, _) = refused(load_with(&files, &options), "src/a.rs");
    assert_eq!((budget, limit), (Budget::TotalBytes, spent - 1));
    // The manifests count too: a limit below the manifest refuses it.
    options.max_total_bytes = DEMO.len() - 1;
    let (budget, _, _) = refused(load_with(&files, &options), "Cargo.toml");
    assert_eq!(budget, Budget::TotalBytes);
}

#[test]
fn the_default_budgets_are_the_stated_limits() {
    let options = LoadOptions::default();
    assert_eq!(options.max_file_tokens, Budget::FileTokens.limit());
    assert_eq!(Budget::FileTokens.limit(), 2_000_000);
    // The recommended limit for a host with little memory keeps one file's
    // parse peak below 512 MiB.
    assert_eq!(PARSE_BYTES_PER_TOKEN, 1664);
    const { assert!(320_000 * PARSE_BYTES_PER_TOKEN <= 512 << 20) };
    assert_eq!(options.max_total_bytes, Budget::TotalBytes.limit());
    assert_eq!(options.max_item_bytes, Budget::ItemBytes.limit());
    assert_eq!(options.max_fact_bytes, Budget::FactBytes.limit());
    assert_eq!(options.max_glob_bytes, Budget::GlobBytes.limit());
    assert_eq!(Budget::TotalBytes.limit(), 256 << 20);
    assert_eq!(Budget::ItemBytes.limit(), 512 << 20);
    assert_eq!(Budget::FactBytes.limit(), 256 << 20);
    assert_eq!(Budget::GlobBytes.limit(), 512 << 20);
    assert_eq!(Budget::DirectoryDepth.limit(), 128);
}

#[test]
fn usage_reports_what_a_load_spent() {
    // A root package with a `[workspace]` table, so discovery walks below it.
    let manifest = format!("{DEMO}[workspace]\n");
    let tree: MemoryTree = [
        ("Cargo.toml", manifest.as_str()),
        ("src/lib.rs", "pub mod a;\n"),
        ("src/a/mod.rs", "pub fn f() {}\n"),
    ]
    .into_iter()
    .collect();
    let spent = Workspace::load(&tree, &["."]).expect("loads").usage();
    assert_eq!(
        spent.total_bytes,
        manifest.len() + "pub mod a;\n".len() + "pub fn f() {}\n".len()
    );
    assert!(spent.item_bytes > 0 && spent.fact_bytes > 0);
    // `pub fn f ( ) { }` is seven tokens.
    assert_eq!(spent.max_file_tokens, 7);
    // `fn` and `(`, then the body's `{` and the level it opens.
    assert_eq!(spent.max_recursion, 4);
    // Discovery entered `src/a`, two levels below the root.
    assert_eq!(spent.max_directory_depth, 2);
}

/// A library whose `leaf.rs` is reached under 2^`levels` module paths: each
/// level's file declares the next one twice. `leaf` is the leaf's text.
fn reached_many_times(levels: usize, leaf: &str) -> Vec<(String, String)> {
    let mut files = vec![(
        "src/lib.rs".to_owned(),
        "#[path = \"l0.rs\"]\npub mod a;\n".to_owned(),
    )];
    for level in 0..levels {
        let next = if level + 1 == levels {
            "leaf.rs".to_owned()
        } else {
            format!("l{}.rs", level + 1)
        };
        files.push((
            format!("src/l{level}.rs"),
            format!("#[path = \"{next}\"]\npub mod a;\n#[path = \"{next}\"]\npub mod b;\n"),
        ));
    }
    files.push(("src/leaf.rs".to_owned(), leaf.to_owned()));
    files
}

fn item_bytes_of(files: &[(String, String)]) -> usize {
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    usage(&borrowed).item_bytes
}

#[test]
fn a_long_alias_path_in_a_file_reached_many_times_is_held_and_charged_once() {
    // Twenty thousand segments, about half a megabyte once owned.
    let long = format!("pub type T = {}a;\n", "a::".repeat(20_000));
    let short = "pub type T = a;\n";
    let arrivals = 8;
    let charged = item_bytes_of(&reached_many_times(arrivals, &long))
        - item_bytes_of(&reached_many_times(arrivals, short));
    let owned = 20_000 * (size_of::<String>() + 1);
    // Charged once, not once for each of the 256 module paths.
    assert!(
        charged >= owned && charged < 2 * owned,
        "{charged} for {owned}"
    );
}

#[test]
fn a_long_extern_crate_name_in_a_file_reached_many_times_is_held_and_charged_once() {
    let name = format!("c{}", "x".repeat(64 * 1024));
    let long = format!("extern crate {name} as x;\n");
    let short = "extern crate c as x;\n";
    let arrivals = 8;
    let charged = item_bytes_of(&reached_many_times(arrivals, &long))
        - item_bytes_of(&reached_many_times(arrivals, short));
    // The short name `c` is charged in the neighbour too.
    let owned = name.len() - 1;
    assert!(
        charged >= owned && charged < 2 * owned,
        "{charged} for {owned}"
    );
}

#[test]
fn a_chain_of_files_that_each_declare_the_next_twice_is_charged_what_it_keeps() {
    // Twelve levels: 8191 arrivals of the module walk, two modules and their
    // items at each arrival but the last, and one item at each of those.
    let files = reached_many_times(12, "pub type T = a;\n");
    let arrivals = (1 << 13) - 1;
    // A load of this shape keeps 1511 bytes of index for each arrival,
    // measured with a counting allocator in a release build over 131071
    // arrivals; what it is charged must cover that.
    let kept = 1_511 * arrivals;
    let charged = item_bytes_of(&files);
    assert!(charged >= kept, "{charged} charged for {kept} kept");
}

#[test]
fn nested_inline_modules_whose_scopes_hold_one_name_are_charged_what_they_keep() {
    // Twenty modules, each holding a chain of a hundred inline modules `a`:
    // 2020 modules whose scope tables hold one name each.
    let one = format!("{}{}", "mod a { ".repeat(100), " }".repeat(100));
    let text: String = (0..20).fold(String::new(), |mut text, i| {
        writeln!(text, "mod b{i} {{ {one} }}").expect("write to a string");
        text
    });
    let modules = 20 * 101;
    // Such a load keeps 1443 bytes of index for each module, measured with a
    // counting allocator in a release build over 202000 of them; what it is
    // charged must cover that, as every charge is an upper bound of the
    // allocations it covers.
    let kept = 1_443 * modules;
    let charged = usage(&[("src/lib.rs", &text)]).item_bytes;
    assert!(charged >= kept, "{charged} charged for {kept} kept");
}

/// A library whose module `big` defines `names` functions and whose
/// `importers` modules each glob it.
fn glob_fan(names: usize, importers: usize) -> String {
    let mut text = String::from("pub mod big {\n");
    for i in 0..names {
        writeln!(text, "    pub fn f{i}() {{}}").expect("write to a string");
    }
    text.push_str("}\n");
    for i in 0..importers {
        writeln!(text, "pub mod g{i} {{ pub use super::big::*; }}").expect("write to a string");
    }
    text
}

/// A library of `length` modules, each globbing the next, the last defining
/// one function.
fn glob_chain(length: usize) -> String {
    let mut text = String::new();
    for i in 0..length {
        writeln!(text, "mod m{i} {{ pub use super::m{}::*; }}", i + 1).expect("write to a string");
    }
    writeln!(text, "pub mod m{length} {{ pub fn f() {{}} }}").expect("write to a string");
    text
}

/// A library of `count` modules that each glob the crate root, which globs
/// each of them, each defining one function: every module brings in every
/// name of every other.
fn mutual_globs(count: usize) -> String {
    let mut text = String::new();
    for i in 0..count {
        writeln!(
            text,
            "pub use m{i}::*;\npub mod m{i} {{ pub use crate::*; pub fn g{i}() {{}} }}"
        )
        .expect("write to a string");
    }
    text
}

/// A crate in the shape of a generated binding crate: a private module of
/// `children` modules, each of a foreign type and methods, twelve of them
/// named as in every other child, each child globbing its parent, the parent
/// globbing each child and the root globbing the parent.
fn generated_bindings(children: usize) -> Vec<(String, String)> {
    let mut files = vec![(
        "src/lib.rs".to_owned(),
        "mod features;\npub use features::*;\npub fn top() {}\n".to_owned(),
    )];
    let mut parent = String::new();
    for i in 0..children {
        writeln!(parent, "mod gen_{i};\npub use gen_{i}::*;").expect("write to a string");
        let mut child = format!("use super::*;\nextern \"C\" {{\n    pub type T{i};\n");
        for m in 0..12 {
            writeln!(
                child,
                "    pub fn m{m}(this: &T{i});\n    pub fn own_{i}_{m}(this: &T{i});"
            )
            .expect("write to a string");
        }
        child.push_str("}\n");
        files.push((format!("src/features/gen_{i}.rs"), child));
    }
    files.push(("src/features/mod.rs".to_owned(), parent));
    files
}

fn glob_bytes_of(text: &str) -> usize {
    usage(&[("src/lib.rs", text)]).glob_bytes
}

#[test]
fn glob_tables_at_the_glob_budget_load_and_one_byte_less_is_refused() {
    let text = "pub mod a { pub fn f() {} pub struct S; }\n\
                pub mod b { pub use crate::a::*; }\n\
                pub mod c { pub use crate::b::*; }\n";
    let files = [("src/lib.rs", text)];
    let spent = usage(&files).glob_bytes;
    assert!(spent > 0, "a load with globs charges its glob tables");
    let mut options = LoadOptions::default();
    options.max_glob_bytes = spent;
    let ws = load_with(&files, &options).expect("at the limit");
    assert_eq!(
        ws.resolve("demo::c::f").expect("resolves").path,
        "demo::a::f"
    );
    options.max_glob_bytes = spent - 1;
    let (budget, limit, line) = refused(load_with(&files, &options), "src/lib.rs");
    assert_eq!((budget, limit), (Budget::GlobBytes, spent - 1));
    // The glob of `b` or of `c` passes the limit.
    assert!(matches!(line, Some(2 | 3)), "{line:?}");
    // A load without a glob charges none.
    assert_eq!(
        glob_bytes_of("pub mod a { pub fn f() {} }\npub use a::f;\n"),
        0
    );
}

#[test]
fn a_module_that_many_modules_glob_is_kept_once_for_all_of_them() {
    // A thousand names globbed by one module and by four hundred: each more
    // importer reads the one table of the module, and costs that read, not a
    // copy of the thousand names.
    let one = glob_bytes_of(&glob_fan(1000, 1));
    let many = glob_bytes_of(&glob_fan(1000, 400));
    let per_importer = (many - one) / 399;
    assert!(
        per_importer < 1024,
        "{per_importer} bytes for each importer of a thousand names"
    );
    assert!(one > 1000 * 64, "{one} bytes for a thousand names");
}

#[test]
fn a_chain_of_globs_is_charged_what_each_link_brings_in() {
    // Each link of a chain of globs brings in one name, so twice the links
    // cost twice the bytes, not four times.
    let short = glob_bytes_of(&glob_chain(1000));
    let long = glob_bytes_of(&glob_chain(2000));
    assert!(
        long < 2 * short + short / 10,
        "{long} bytes for 2000 links, {short} for 1000"
    );
    let ws = load_with(
        &[("src/lib.rs", &glob_chain(2000))],
        &LoadOptions::default(),
    )
    .expect("loads");
    assert_eq!(
        ws.resolve("demo::m0::f").expect("resolves").path,
        "demo::m2000::f"
    );
}

#[test]
fn modules_that_glob_one_another_are_refused_over_the_glob_budget_naming_a_glob() {
    // Each of 200 modules brings in all 400 names of the others, which the
    // tables hold once for each module; a limit of 4 MiB refuses them, and
    // allows 20.
    let mut options = LoadOptions::default();
    options.max_glob_bytes = 4 << 20;
    let text = mutual_globs(200);
    let (budget, limit, line) =
        refused(load_with(&[("src/lib.rs", &text)], &options), "src/lib.rs");
    assert_eq!((budget, limit), (Budget::GlobBytes, 4 << 20));
    let line = line.expect("the line of a glob");
    assert!(
        text.lines()
            .nth(line - 1)
            .is_some_and(|glob| glob.contains("::*;")),
        "{line}"
    );
    let ws = load_with(&[("src/lib.rs", &mutual_globs(20))], &options).expect("loads");
    assert_eq!(
        ws.resolve("demo::m0::g19").expect("resolves").path,
        "demo::m19::g19"
    );
}

#[test]
fn glob_tables_of_a_generated_binding_crate_are_charged_what_they_keep() {
    // Two hundred children of one module, each globbing it, read one table
    // of all that the module brings in. Such a load keeps 1976740 bytes of
    // glob tables, measured with a counting allocator in a release build;
    // what it is charged must cover that, as every charge is an upper bound
    // of the allocations it covers.
    let files = generated_bindings(200);
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    let ws = load_with(&borrowed, &LoadOptions::default()).expect("loads");
    let charged = ws.usage().glob_bytes;
    assert!(charged >= 1_976_740, "{charged} charged");
    assert_eq!(
        ws.resolve("demo::T7").expect("resolves").path,
        "demo::features::gen_7::T7"
    );
}

/// A library whose module `a` defines `names` structs, whose module `t` globs
/// `a` `globs` times over and whose module `u` globs `t`, with `u` declared
/// first when `u_first`.
fn repeated_globs(names: usize, globs: usize, u_first: bool) -> Vec<(String, String)> {
    let mut a = String::new();
    for i in 0..names {
        writeln!(a, "pub struct S{i};").expect("write to a string");
    }
    let lib = if u_first {
        "pub mod u;\npub mod a;\npub mod t;\n"
    } else {
        "pub mod a;\npub mod t;\npub mod u;\n"
    };
    vec![
        ("src/lib.rs".to_owned(), lib.to_owned()),
        ("src/a.rs".to_owned(), a),
        (
            "src/t.rs".to_owned(),
            "pub use crate::a::*;\n".repeat(globs),
        ),
        ("src/u.rs".to_owned(), "pub use crate::t::*;\n".to_owned()),
    ]
}

fn borrowed(files: &[(String, String)]) -> Vec<(&str, &str)> {
    files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect()
}

#[test]
fn repeated_globs_of_one_module_are_charged_once_for_what_it_holds() {
    // `t` globs a module of 2000 names 400 times over, which rustc accepts,
    // and `u` globs `t`. Every glob of `t` reads the one table of `a`, and
    // gathering what `t` exports reads that table once, so each more glob
    // costs its read and not a copy of the 2000 names, whichever module is
    // declared first.
    for u_first in [false, true] {
        let one = usage(&borrowed(&repeated_globs(2000, 1, u_first))).glob_bytes;
        let files = repeated_globs(2000, 400, u_first);
        let ws = load_with(&borrowed(&files), &LoadOptions::default()).expect("loads");
        let many = ws.usage().glob_bytes;
        let per_glob = (many - one) / 399;
        assert!(
            per_glob < 1024,
            "{per_glob} bytes for each more glob of two thousand names ({u_first})"
        );
        assert_eq!(
            ws.resolve("demo::u::S1999").expect("resolves").path,
            "demo::a::S1999"
        );
        // Below what the tables take, the load is refused by the glob budget.
        let mut options = LoadOptions::default();
        options.max_glob_bytes = many - 1;
        let error = load_with(&borrowed(&files), &options).expect_err("over the budget");
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Budget {
                    budget: Budget::GlobBytes,
                    ..
                }
            ),
            "{error}"
        );
    }
}
