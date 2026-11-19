// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Actual migration and debt commands, with read-only inputs and refusal neighbours.

use raf_fixtures::{files, git_repo, layout};

use raf_census::{
    Baseline, Census, CensusOptions, DebtRecord, Gate, Ledger, LedgerExtension,
    LedgerExtensionGraph, MAX_LEDGER_BYTES,
};
use raf_rdf::{Builder, Format, TermValue as Term, Triple, vocab};
use raf_source::DiskTree;
use std::{
    ffi::OsString,
    fmt::Write as _,
    fs,
    path::Path,
    process::{Command, ExitCode, Output},
};

fn put(graph: &mut Builder, subject: &str, key: &str, object: Term) {
    graph
        .insert(Triple {
            subject: Term::iri(subject),
            predicate: Term::iri(if key == "type" {
                vocab::TYPE.into()
            } else {
                format!("{}{key}", vocab::REPORT)
            }),
            object,
        })
        .unwrap();
}

fn ledger(version: u32) -> String {
    ledger_graph(version)
        .finish()
        .unwrap()
        .to_text(Format::Turtle)
        .unwrap()
}

fn ledger_graph(version: u32) -> Builder {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        "type",
        Term::iri(format!("{}Ledger", vocab::REPORT)),
    );
    for (key, value) in [
        ("normalForm", version),
        ("reportForm", raf_census::REPORT_FORM),
    ] {
        put(
            &mut graph,
            "",
            key,
            Term::typed_literal(value.to_string(), vocab::INTEGER),
        );
    }
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
        ("homeSymbol", "demo::home"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    put(&mut graph, "#job", "enforced", Term::boolean(false));
    put(
        &mut graph,
        "#job",
        "forbiddenConstant",
        Term::typed_literal("7", vocab::INTEGER),
    );
    graph
}

fn fixture() -> purrdf_testkit::TempDir {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    source_fixture(scratch.path());
    fs::write(scratch.path().join("helpers-ledger.ttl"), ledger(1)).unwrap();
    scratch
}

fn source_fixture(root: &Path) {
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[workspace]\n",
    )
    .unwrap();
    source(root, 2);
}

fn source(root: &Path, copies: usize) {
    let mut text = "/// The operation.\npub fn home()->u32{7}\n".to_owned();
    for index in 0..copies {
        writeln!(text, "fn copy_{index}()->u32{{7}}").unwrap();
    }
    fs::write(root.join("src/lib.rs"), text).unwrap();
}

/// Every child of this binary is started and awaited under this lock. A child
/// forked by one test thread holds a copy of every descriptor the other test
/// threads have open until it execs; a copy of the descriptor of a directory
/// lock that a test holds, or has just released, keeps the lock busy for the
/// command another test starts next. With one child alive at a time, a command
/// only ever sees the locks of its own test.
static CHILD: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Start a command and wait for it under [`CHILD`]; every command of this
/// binary goes through it.
fn alone(command: &mut Command) -> Output {
    let _alone = CHILD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    command.output().unwrap()
}

fn run(root: &Path, words: &[&str]) -> Output {
    alone(
        Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(root)
            .args(words),
    )
}

fn capnp_history(root: &Path, base: &str, current: &str) -> Output {
    run(
        root,
        &[
            "capnp-history",
            "--base",
            base,
            "--current",
            current,
            "--base-descriptor",
            "schema.bin",
            "--current-descriptor",
            "schema.bin",
            "--type",
            "0xf1804b9b18ae6505",
            "--type",
            "0xc8a351b6b42dc6a5",
        ],
    )
}

/// The shipped command follows compiled references from the selected type and
/// compares the committed layouts even when the worktree contains other bytes.
#[test]
fn capnp_history_checks_committed_compiler_layouts() {
    let target = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let scratch = purrdf_testkit::TempDir::new_in(target).unwrap();
    let root = scratch.path();
    git_repo::init(root);
    let descriptor = root.join("schema.bin");
    fs::write(&descriptor, include_bytes!("fixtures/capnp/base.bin")).unwrap();
    let base = git_repo::commit_all(root);
    fs::write(&descriptor, include_bytes!("fixtures/capnp/compatible.bin")).unwrap();
    let compatible = git_repo::commit_all(root);
    fs::write(&descriptor, include_bytes!("fixtures/capnp/changed.bin")).unwrap();
    let changed = git_repo::commit_all(root);
    fs::write(&descriptor, b"not a request").unwrap();
    let malformed = git_repo::commit_all(root);
    fs::write(
        &descriptor,
        include_bytes!("fixtures/capnp/missing-node.bin"),
    )
    .unwrap();
    let unresolved = git_repo::commit_all(root);
    fs::write(&descriptor, b"uncommitted bytes").unwrap();

    let clean = capnp_history(root, &base, &compatible);
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, b"");

    let changed = capnp_history(root, &base, &changed);
    assert_eq!(changed.status.code(), Some(1), "{changed:?}");
    let findings = String::from_utf8(changed.stdout).unwrap();
    for expected in [
        ".title: member renamed to heading",
        ".counter: ordinal changed",
        ".selected: type changed",
        ".state: slot offset changed",
        ".options.retries: default changed",
        ".payload: default changed",
        ".boxed: type changed",
        "0xc5fa75a0c95aa56a.obsolete: member removed",
        "0xc5fa75a0c95aa56a.numbers: ordinal changed",
        "0xc8a351b6b42dc6a5.back: type changed",
        "0xe70e425d8d63c329.ready: member renamed to waiting",
        "0xe70e425d8d63c329.retired: member removed",
    ] {
        assert!(
            findings.contains(expected),
            "{expected:?} absent from {findings}"
        );
    }
    assert!(!findings.contains("ignored"), "{findings}");

    let bad = capnp_history(root, &base, &malformed);
    assert_eq!(bad.status.code(), Some(1), "{bad:?}");
    assert_eq!(bad.stdout, b"");
    let diagnostic = String::from_utf8(bad.stderr).unwrap();
    assert!(
        diagnostic.contains("regenerate and commit the complete descriptor"),
        "{diagnostic}"
    );

    let missing_node = capnp_history(root, &base, &unresolved);
    assert_eq!(missing_node.status.code(), Some(1), "{missing_node:?}");
    assert_eq!(missing_node.stdout, b"");
    let diagnostic = String::from_utf8(missing_node.stderr).unwrap();
    assert!(diagnostic.contains("missing schema node"), "{diagnostic}");

    let missing = "1111111111111111111111111111111111111111";
    let refused = capnp_history(root, missing, &compatible);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert_eq!(refused.stdout, b"");
    let diagnostic = String::from_utf8(refused.stderr).unwrap();
    assert!(
        diagnostic.contains(&format!("Cap'n Proto revision {missing}")),
        "{diagnostic}"
    );
}

fn visibility_nodes<'a>(
    graph: &'a raf_rdf::Graph,
    from: raf_rdf::Node<'a>,
    key: &str,
) -> impl Iterator<Item = raf_rdf::Node<'a>> {
    from.values(&format!("{}{key}", vocab::REPORT))
        .map(move |value| graph.get(value.iri().unwrap()).unwrap())
}

fn visibility_value<'a>(node: raf_rdf::Node<'a>, key: &str) -> &'a str {
    node.values(&format!("{}{key}", vocab::REPORT))
        .next()
        .unwrap()
        .literal()
        .unwrap()
        .0
}

/// Count distinct definitions and attribute written paths to their owner
/// library, including a definition reached only through a member re-export.
#[test]
fn visibility_counts_definitions_and_cross_member_paths_in_dependency_tiers() {
    let target = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let scratch = purrdf_testkit::TempDir::new_in(target).unwrap();
    let root = scratch.path();
    let base_manifest = "[package]\nname='base'\nversion='0.1.0'\nedition='2024'\n";
    files::write(
        root,
        &[
            (
                "Cargo.toml",
                "[workspace]\nmembers=['base','bridge','client']\nexclude=['excluded']\n",
            ),
            ("base/Cargo.toml", base_manifest),
            (
                "base/src/lib.rs",
                "mod implementation { pub fn used() {} pub fn internal_only() {} }\n\
                 pub use implementation::{internal_only, used as api, used as other_alias};\n\
                 pub fn local() { implementation::internal_only(); }\n\
                 pub fn spare() {}\n#[doc(hidden)] pub fn hidden() {}\n\
                 pub struct Data { pub number: u32 }\n\
                 pub fn only_in_import() {}\npub fn dev_only() {}\npub trait Marker {}\n\
                 pub mod glob_symbols {\n\
                     pub const LIMIT: u8 = 3;\n\
                     pub fn via_glob() {} pub fn shadowed() {}\n\
                     pub mod branch { pub mod leaf { pub fn via_chain() {} pub fn via_direct() {} } }\n\
                 }",
            ),
            (
                "bridge/Cargo.toml",
                "[package]\nname='bridge'\nversion='0.1.0'\nedition='2024'\n\
                 [dependencies]\nupstream={path='../base',package='base'}\n",
            ),
            (
                "bridge/src/lib.rs",
                "pub use upstream::{api as facade, glob_symbols, only_in_import, Marker};\n\
                 pub fn unused_bridge() {}\n\
                 #[macro_export] macro_rules! emitted { () => { () } }\n\
                 pub mod macro_api { pub use crate::emitted as emit; }",
            ),
            (
                "client/Cargo.toml",
                "[package]\nname='client'\nversion='0.1.0'\nedition='2024'\n\
                 [dependencies]\nbridge={path='../bridge'}\n\
                 [dev-dependencies]\ndev={path='../base',package='base'}\n",
            ),
            (
                "client/src/lib.rs",
                "mod loose;\nstruct Local;\nimpl bridge::Marker for Local {}\n\
                 pub fn call(value: u8) {\n\
                     use { ::bridge::facade as chosen };\n\
                     use bridge::glob_symbols::*;\n\
                     fn shadowed() {}\n\
                     chosen();\n\
                     via_glob();\n\
                     shadowed();\n\
                     match value { LIMIT => (), _ => () }\n\
                     { use leaf::*; use branch::*; via_chain(); }\n\
                     { use bridge::glob_symbols::branch::leaf::*; via_direct(); }\n\
                     bridge::macro_api::emit!();\n\
                 }",
            ),
            (
                "client/src/loose.rs",
                "use { ::bridge::only_in_import };\n#[cfg(test)] use dev::dev_only;",
            ),
            (
                "client/build.rs",
                "fn main() { std::fs::write(\"consumer-executed\", b\"bad\").unwrap(); panic!(\"static data\"); }",
            ),
            (
                "excluded/Cargo.toml",
                "[package]\nname='excluded'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("excluded/src/lib.rs", "this source must remain unparsed"),
        ],
    );
    let first = run(root, &["visibility"]);
    assert!(first.status.success(), "{first:?}");
    assert_eq!(
        run(&root.join("client"), &["visibility"]).stdout,
        first.stdout
    );
    let graph = raf_rdf::Graph::from_text(
        std::str::from_utf8(&first.stdout).unwrap(),
        Format::NTriples,
    )
    .unwrap();
    assert_visibility_report(&graph);
    assert!(!root.join("client/consumer-executed").exists());
    assert!(!root.join("target").exists());

    assert_block_glob_scope(root);
    assert_visibility_item_scopes(root);
    fs::write(
        root.join("base/Cargo.toml"),
        format!("{base_manifest}[dependencies]\nclient={{path='../client'}}\n"),
    )
    .unwrap();
    let cycle = run(root, &["visibility"]);
    assert_eq!(cycle.status.code(), Some(1));
    assert_eq!(cycle.stdout, Vec::<u8>::new());
    assert!(String::from_utf8_lossy(&cycle.stderr).contains("normal library dependency cycle"));
}

fn assert_block_glob_scope(root: &Path) {
    fs::write(
        root.join("client/src/loose.rs"),
        "fn call() { use std::io::*; { use ::bridge::glob_symbols::branch::leaf::*; via_direct(); } }",
    )
    .unwrap();
    let branch = run(root, &["visibility"]);
    assert!(branch.status.success(), "{branch:?}");

    for globs in [
        "use std::io::*;",
        "use first::second::*; use second::first::*;",
    ] {
        fs::write(
            root.join("client/src/loose.rs"),
            format!("use bridge::facade;\nfn call() {{ {globs} facade(); }}"),
        )
        .unwrap();
        let branch = run(root, &["visibility"]);
        assert_eq!(branch.status.code(), Some(1));
        assert_eq!(branch.stdout, Vec::<u8>::new());
        assert!(String::from_utf8_lossy(&branch.stderr).contains("unindexed block glob"));
    }

    fs::write(
        root.join("client/src/loose.rs"),
        "mod branch {}\nfn call() { use ::bridge::glob_symbols::*; use branch::*; }",
    )
    .unwrap();
    let ambiguous = run(root, &["visibility"]);
    assert_eq!(ambiguous.status.code(), Some(1));
    assert_eq!(ambiguous.stdout, Vec::<u8>::new());
    assert!(
        String::from_utf8_lossy(&ambiguous.stderr).contains("distinct glob and outer bindings")
    );
}

fn assert_visibility_item_scopes(root: &Path) {
    let bridge = root.join("bridge/src/lib.rs");
    let mut source = fs::read_to_string(&bridge).unwrap();
    source.push_str(
        "\npub enum Empty { Spare } impl Empty { pub fn fallback() {} }\n\
        pub enum Tuple { concealed(()) } impl Tuple { pub fn concealed() {} }\n\
        pub enum Brace { named { value: u8 } }\n\
         #[cfg(unix)] pub enum Branched { divergent(()) }\n\
         #[cfg(windows)] pub enum Branched { Spare }\n\
         pub fn fallback() {} pub fn concealed() {} pub fn named() {} pub fn divergent(_: ()) {}",
    );
    fs::write(bridge, source).unwrap();
    fs::write(
        root.join("client/src/loose.rs"),
        "use bridge::{fallback, concealed, named};\n\
        fn calls() {\n\
            { use bridge::Empty::*; fallback(); bridge::Empty::fallback(); }\n\
            { use bridge::Tuple::*; let _ = concealed(()); }\n\
            { use bridge::Brace::*; named(); }\n\
            use bridge::facade as nested_call;\n\
            fn inner() { nested_call(); }\n\
            let nested_call = || ();\n\
            use bridge::facade as captured_call; let captured_call = || ();\n\
            let _ = || captured_call();\n\
        }\n\
        fn alias_chain() {\n\
            use bridge::glob_symbols::*; use branch as local; use local::*;\n\
            leaf::via_chain();\n\
        }",
    )
    .unwrap();
    let output = run(root, &["visibility"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let graph = raf_rdf::Graph::from_text(
        std::str::from_utf8(&output.stdout).unwrap(),
        Format::NTriples,
    )
    .unwrap();
    for (package, symbol, written, expected) in [
        ("bridge", "bridge::fallback", "fallback", true),
        ("bridge", "bridge::Empty::fallback", "fallback", false),
        (
            "bridge",
            "bridge::Empty::fallback",
            "bridge::Empty::fallback",
            true,
        ),
        ("bridge", "bridge::concealed", "concealed", false),
        ("bridge", "bridge::Tuple::concealed", "concealed", false),
        ("bridge", "bridge::named", "named", true),
        ("base", "base::implementation::used", "nested_call", true),
        ("base", "base::implementation::used", "captured_call", false),
        ("base", "base::glob_symbols::branch", "local::*", true),
        (
            "base",
            "base::glob_symbols::branch::leaf::via_chain",
            "leaf::via_chain",
            true,
        ),
    ] {
        let library = visibility_nodes(&graph, graph.get("").unwrap(), "library")
            .find(|node| visibility_value(*node, "package") == package)
            .unwrap();
        let observed = visibility_nodes(&graph, library, "use").any(|row| {
            visibility_value(row, "symbol") == symbol
                && visibility_value(row, "consumer") == "client"
                && row
                    .values(&format!("{}writtenPath", vocab::REPORT))
                    .any(|value| value.literal().unwrap().0 == written)
        });
        assert_eq!(observed, expected, "written path {written} for {symbol}");
    }

    fs::write(
        root.join("client/src/loose.rs"),
        "use bridge::divergent; fn call() { use bridge::Branched::*; let _ = divergent(()); }",
    )
    .unwrap();
    let ambiguous = run(root, &["visibility"]);
    assert_eq!(ambiguous.status.code(), Some(1));
    assert_eq!(ambiguous.stdout, Vec::<u8>::new());
    assert!(
        String::from_utf8_lossy(&ambiguous.stderr).contains("differs across indexed declarations")
    );
    assert_visibility_glob_namespaces(root);
}

fn assert_visibility_glob_namespaces(root: &Path) {
    let bridge = root.join("bridge/src/lib.rs");
    let mut source = fs::read_to_string(&bridge).unwrap();
    source.push_str(
        "\npub fn ping() {}\n\
         pub mod types { pub struct Record { pub value: u8 } pub struct GenericOnly; #[derive(Clone, Copy)] pub struct Used; }\n\
         pub mod left { pub mod branch { pub fn run() {} pub use crate::ping as same; } }\n\
         pub mod right { pub mod branch { pub fn run() {} pub use crate::ping as same; } }\n\
         pub enum First { Ready } pub enum Second { Ready } pub use First as Same;\n\
         pub mod api { pub mod entry { pub fn member() {} } pub fn entry() {} }\n\
         #[macro_export] macro_rules! stamp { () => {} }\n\
         #[macro_export] macro_rules! early_stamp { () => {} }",
    );
    fs::write(bridge, source).unwrap();
    let report = |source: &str| {
        fs::write(root.join("client/src/loose.rs"), source).unwrap();
        run(root, &["visibility"])
    };
    let output = report(
        "use bridge::ping as picked; use bridge::left::branch;\n\
         fn split() { use bridge::types::*; use Record as picked; picked(); let _: Option<picked> = None; }\n\
         fn union_fields() { use bridge::types::*; union Generic<GenericOnly: Copy> { field: GenericOnly } union Ordinary { field: Used } }\n\
         fn qualified() { use bridge::right::*; branch::run(); }\n\
         fn coincident() { use bridge::left::*; use branch::run as chosen; chosen(); }\n\
         fn coincident_block() { use bridge::right::*; use bridge::left::branch; { use bridge::left::*; use branch::run as chosen_in_block; chosen_in_block(); } }\n\
         fn coincident_glob() { use bridge::left::branch; { use bridge::left::*; use branch::*; run(); } }\n\
         fn explicit_glob() { use bridge::left::branch; { use bridge::right::*; use bridge::left::branch::*; run(); } }\n\
         fn repeated() { use bridge::First::*; use bridge::Same::*; let _ = Ready; }",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let graph = raf_rdf::Graph::from_text(
        std::str::from_utf8(&output.stdout).unwrap(),
        Format::NTriples,
    )
    .unwrap();
    let bridge = visibility_nodes(&graph, graph.get("").unwrap(), "library")
        .find(|node| visibility_value(*node, "package") == "bridge")
        .unwrap();
    for (symbol, written, expected) in [
        ("bridge::ping", "picked", true),
        ("bridge::types::Record", "picked", true),
        ("bridge::types::GenericOnly", "GenericOnly", false),
        ("bridge::types::Used", "Used", true),
        ("bridge::right::branch::run", "branch::run", true),
        ("bridge::left::branch::run", "chosen", true),
        ("bridge::left::branch::run", "chosen_in_block", true),
        ("bridge::left::branch", "branch::*", true),
        ("bridge::left::branch", "bridge::left::branch::*", true),
    ] {
        assert_eq!(
            visibility_nodes(&graph, bridge, "use").any(|row| {
                visibility_value(row, "symbol") == symbol
                    && visibility_value(row, "consumer") == "client"
                    && row
                        .values(&format!("{}writtenPath", vocab::REPORT))
                        .any(|value| value.literal().unwrap().0 == written)
            }),
            expected,
            "written path {written} for {symbol}"
        );
    }
    assert_visibility_glob_refusals(&report);
    assert_visibility_grouped_self_and_macros(&report);
}

fn assert_visibility_glob_refusals(report: &impl Fn(&str) -> Output) {
    for (source, reason) in [
        (
            "use bridge::left::branch; fn call() { use bridge::right::*; use branch::run as chosen; chosen(); }",
            "distinct glob and outer bindings",
        ),
        (
            "fn call() { use bridge::left::*; { use bridge::right::*; use branch::run as chosen; chosen(); } }",
            "competing block glob bindings",
        ),
        (
            "fn call() { use bridge::left::branch; { use bridge::right::*; use branch::same as chosen; chosen(); } }",
            "distinct glob and outer bindings",
        ),
        (
            "fn call() { use bridge::left::branch; { use bridge::right::*; use branch::*; run(); } }",
            "distinct glob and outer bindings",
        ),
        (
            "fn call() { macro_rules! stamp { () => {} } { use bridge::*; stamp!(); } }",
            "competing textual and path-based bindings",
        ),
        (
            "fn call() { use bridge::left::branch; { use bridge::right::branch; { use bridge::right::*; use branch::same as chosen; chosen(); } } }",
            "distinct glob and outer bindings",
        ),
        (
            "fn call() { use bridge::left::*; { use bridge::right::branch; { use bridge::right::*; use branch::same as chosen; chosen(); } } }",
            "distinct glob and outer bindings",
        ),
        (
            "fn call() { use bridge::First::*; use bridge::Second::*; let _ = Ready; }",
            "competing block glob bindings",
        ),
    ] {
        let ambiguous = report(source);
        assert_eq!(ambiguous.status.code(), Some(1));
        assert_eq!(ambiguous.stdout, Vec::<u8>::new());
        let stderr = String::from_utf8_lossy(&ambiguous.stderr);
        assert!(stderr.contains(reason), "{stderr}");
    }
    let internal = report(
        "enum First { Ready } enum Second { Ready } fn call() { use self::First::*; use self::Second::*; let _ = Ready; }",
    );
    assert!(
        internal.status.success(),
        "{}",
        String::from_utf8_lossy(&internal.stderr)
    );
}

fn assert_visibility_grouped_self_and_macros(report: &impl Fn(&str) -> Output) {
    let output = report(
        "use bridge::api::entry::{self as grouped}; use bridge::ping as grouped;\n\
         use bridge::api as ordinary_api; use ordinary_api::entry as ordinary;\n\
         use bridge::ping as local_module;\n\
         fn grouped_paths() { grouped(); grouped::member(); ordinary(); ordinary::member();\n\
             use bridge::api as api_block; use api_block::entry::{self as local_module};\n\
             local_module(); local_module::member(); }\n\
         fn macros() { use bridge::*; early_stamp!(); macro_rules! early_stamp { () => {} }\n\
             macro_rules! stamp { () => {} } stamp!(); bridge::stamp!(); }",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let graph = raf_rdf::Graph::from_text(
        std::str::from_utf8(&output.stdout).unwrap(),
        Format::NTriples,
    )
    .unwrap();
    let bridge = visibility_nodes(&graph, graph.get("").unwrap(), "library")
        .find(|node| visibility_value(*node, "package") == "bridge")
        .unwrap();
    for (symbol, kind, written, expected) in [
        ("bridge::api::entry", "module", "bridge::api::entry", true),
        (
            "bridge::api::entry",
            "function",
            "bridge::api::entry",
            false,
        ),
        ("bridge::api::entry", "module", "api_block::entry", true),
        ("bridge::api::entry", "function", "api_block::entry", false),
        ("bridge::api::entry", "module", "ordinary_api::entry", true),
        (
            "bridge::api::entry",
            "function",
            "ordinary_api::entry",
            true,
        ),
        ("bridge::api::entry", "function", "ordinary", true),
        ("bridge::api::entry", "function", "local_module", false),
        (
            "bridge::api::entry::member",
            "function",
            "grouped::member",
            true,
        ),
        (
            "bridge::api::entry::member",
            "function",
            "local_module::member",
            true,
        ),
        ("bridge::ping", "function", "grouped", true),
        ("bridge::ping", "function", "local_module", true),
        ("bridge::stamp", "macro-rules", "stamp", false),
        ("bridge::stamp", "macro-rules", "bridge::stamp", true),
        ("bridge::early_stamp", "macro-rules", "early_stamp", true),
    ] {
        let declaration = (symbol == "bridge::api::entry").then(|| {
            visibility_nodes(&graph, bridge, "declaration")
                .find(|row| {
                    visibility_value(*row, "symbol") == symbol
                        && visibility_value(*row, "kind") == kind
                })
                .expect("public definition in the selected namespace")
        });
        assert_eq!(
            visibility_nodes(&graph, bridge, "use").any(|row| {
                visibility_value(row, "symbol") == symbol
                    && declaration.is_none_or(|declaration| {
                        visibility_value(row, "startByte")
                            == visibility_value(declaration, "startByte")
                    })
                    && visibility_value(row, "consumer") == "client"
                    && row
                        .values(&format!("{}writtenPath", vocab::REPORT))
                        .any(|value| value.literal().unwrap().0 == written)
            }),
            expected,
            "written path {written} for {symbol} ({kind})"
        );
    }
}

fn assert_visibility_report(graph: &raf_rdf::Graph) {
    let root = graph.get("").unwrap();
    let libraries: std::collections::BTreeMap<_, _> = visibility_nodes(graph, root, "library")
        .map(|library| (visibility_value(library, "package"), library))
        .collect();
    assert_eq!(libraries.len(), 3);
    for (package, tier) in [("base", "0"), ("bridge", "1"), ("client", "2")] {
        assert_eq!(visibility_value(libraries[package], "tier"), tier);
    }
    let base = libraries["base"];
    assert_eq!(visibility_value(base, "publicCount"), "17");
    assert_eq!(visibility_value(base, "exportedCount"), "17");
    assert_eq!(visibility_value(base, "usedExportedCount"), "11");
    assert_eq!(visibility_value(base, "writtenPathCount"), "19");
    let counts: std::collections::BTreeMap<_, _> = visibility_nodes(graph, base, "kindCount")
        .map(|row| {
            (
                visibility_value(row, "kind"),
                visibility_value(row, "count"),
            )
        })
        .collect();
    assert_eq!(
        counts,
        std::collections::BTreeMap::from([
            ("constant", "1"),
            ("function", "11"),
            ("module", "3"),
            ("struct", "1"),
            ("trait", "1")
        ])
    );
    let unused: std::collections::BTreeSet<_> = visibility_nodes(graph, base, "unusedDeclaration")
        .map(|row| visibility_value(row, "symbol"))
        .collect();
    assert!(unused.contains("base::implementation::internal_only"));
    assert!(!unused.contains("base::implementation::used"));
    assert!(unused.contains("base::glob_symbols::shadowed"));
    assert!(!unused.contains("base::glob_symbols::LIMIT"));
    assert_visibility_paths(graph, base, libraries["bridge"]);
}

fn assert_visibility_paths(
    graph: &raf_rdf::Graph,
    base: raf_rdf::Node<'_>,
    bridge: raf_rdf::Node<'_>,
) {
    for (definition, written) in [
        ("bridge::macro_api", "bridge::macro_api"),
        ("bridge::emitted", "bridge::macro_api::emit"),
    ] {
        assert!(
            visibility_nodes(graph, bridge, "use").any(|row| {
                visibility_value(row, "symbol") == definition
                    && visibility_value(row, "consumer") == "client"
                    && visibility_value(row, "writtenPath") == written
            }),
            "missing {definition} through {written}"
        );
    }
    let client_uses: std::collections::BTreeMap<_, _> = visibility_nodes(graph, base, "use")
        .filter(|row| visibility_value(*row, "consumer") == "client")
        .map(|row| {
            let written: std::collections::BTreeSet<_> = row
                .values(&format!("{}writtenPath", vocab::REPORT))
                .map(|value| value.literal().unwrap().0)
                .collect();
            (visibility_value(row, "symbol"), written)
        })
        .collect();
    assert_eq!(
        client_uses,
        std::collections::BTreeMap::from([
            ("base::Marker", ["bridge::Marker"].into_iter().collect()),
            ("base::dev_only", ["dev::dev_only"].into_iter().collect()),
            (
                "base::glob_symbols",
                ["bridge::glob_symbols", "bridge::glob_symbols::*"]
                    .into_iter()
                    .collect()
            ),
            ("base::glob_symbols::LIMIT", ["LIMIT"].into_iter().collect()),
            (
                "base::glob_symbols::branch",
                ["branch::*", "bridge::glob_symbols::branch"]
                    .into_iter()
                    .collect()
            ),
            (
                "base::glob_symbols::branch::leaf",
                ["bridge::glob_symbols::branch::leaf::*", "leaf::*"]
                    .into_iter()
                    .collect()
            ),
            (
                "base::glob_symbols::branch::leaf::via_chain",
                ["via_chain"].into_iter().collect()
            ),
            (
                "base::glob_symbols::branch::leaf::via_direct",
                ["via_direct"].into_iter().collect()
            ),
            (
                "base::glob_symbols::via_glob",
                ["via_glob"].into_iter().collect()
            ),
            (
                "base::implementation::used",
                ["::bridge::facade", "chosen"].into_iter().collect()
            ),
            (
                "base::only_in_import",
                ["::bridge::only_in_import"].into_iter().collect()
            ),
        ])
    );
}

const OWNER: &str = "urn:example:query-owner";

struct QueryOwner;

impl LedgerExtension for QueryOwner {
    fn validate(&self, _: &str, graph: LedgerExtensionGraph<'_>) -> Option<Result<(), String>> {
        let node = graph.root();
        let mut owners = node.values(OWNER);
        let owner = owners.next()?;
        Some(
            if owner.literal() == Some(("demo", vocab::STRING)) && owners.next().is_none() {
                Ok(())
            } else {
                Err("query owner must be demo".into())
            },
        )
    }
}

fn query_ledger(owner: Option<&str>, max_file_bytes: Option<u32>) -> String {
    let mut graph = ledger_graph(raf_census::NORMAL_FORM);
    put(&mut graph, "", "minTokens", Term::integer(1));
    put(&mut graph, "", "pattern", Term::iri("#pattern"));
    put(
        &mut graph,
        "#pattern",
        "type",
        Term::iri(format!("{}Pattern", vocab::REPORT)),
    );
    for (key, value) in [
        ("patternId", "literal-seven"),
        ("summary", "Use the approved constant"),
        ("tokens", "7"),
    ] {
        put(&mut graph, "#pattern", key, Term::simple_literal(value));
    }
    for (key, value) in [
        ("forbiddenPattern", "literal-seven"),
        ("forbiddenFingerprint", "rule:literal-seven"),
    ] {
        put(&mut graph, "#job", key, Term::simple_literal(value));
    }
    if let Some(limit) = max_file_bytes {
        put(&mut graph, "", "maxFileBytes", Term::integer(limit));
    }
    if let Some(owner) = owner {
        put(&mut graph, "#job", "extension", Term::iri("#owner"));
        put(
            &mut graph,
            "#owner",
            "type",
            Term::iri(format!("{}Extension", vocab::REPORT)),
        );
        graph
            .insert(Triple {
                subject: Term::iri("#owner"),
                predicate: Term::iri(OWNER),
                object: Term::simple_literal(owner),
            })
            .unwrap();
    }
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

fn query_gate(gate: Gate) -> Gate {
    gate.extensions([Box::new(QueryOwner) as Box<dyn LedgerExtension>])
}

fn registered_query(
    root: &Path,
    words: &[&str],
    configure: impl FnOnce(Gate) -> Gate,
) -> (ExitCode, Vec<u8>, String) {
    let args = [
        OsString::from("repository-census"),
        OsString::from("--workspace"),
        root.as_os_str().into(),
    ]
    .into_iter()
    .chain(words.iter().copied().map(OsString::from));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let status = cargo_census::run(args, Some(configure), &mut stdout, &mut stderr);
    (status, stdout, String::from_utf8(stderr).unwrap())
}

/// Registered queries keep the stock report bytes and require accepted policy
/// before publishing a result. Argument refusal precedes Gate configuration.
#[test]
fn registered_queries_share_cli_bytes_and_refuse_before_output() {
    let target = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let scratch = purrdf_testkit::TempDir::new_in(target).unwrap();
    let root = scratch.path();
    source_fixture(root);
    let ledger = root.join(raf_census::LEDGER_FILE);
    fs::write(&ledger, query_ledger(None, None)).unwrap();
    let commands: &[&[&str]] = &[
        &["job", "number"],
        &["count", "number"],
        &["fingerprint", "demo::home", "--format", "turtle"],
        &["baseline"],
    ];
    let expected: Vec<_> = commands
        .iter()
        .map(|words| {
            let result = run(root, words);
            assert!(
                result.status.success() && !result.stdout.is_empty(),
                "{result:?}"
            );
            result.stdout
        })
        .collect();

    fs::write(&ledger, query_ledger(Some("demo"), None)).unwrap();
    for (words, expected) in commands.iter().zip(&expected) {
        let (status, output, diagnostics) = registered_query(root, words, query_gate);
        assert_eq!(status, ExitCode::SUCCESS, "{words:?}: {diagnostics}");
        assert_eq!(&output, expected, "{words:?}");
        assert_eq!(diagnostics, "");
    }

    let (status, output, diagnostics) =
        registered_query(root, &["baseline"], std::convert::identity);
    assert_eq!(status, ExitCode::from(2));
    assert!(output.is_empty() && diagnostics.contains("unclaimed extension"));

    // Ledger source limits belong to Gate evaluation. The registered
    // baseline admits the same extension and still uses the Census defaults.
    fs::write(&ledger, query_ledger(Some("demo"), Some(1))).unwrap();
    assert!(query_gate(Gate::workspace(root)).check().is_err());
    let (status, output, diagnostics) = registered_query(root, &["baseline"], query_gate);
    assert_eq!(status, ExitCode::SUCCESS, "{diagnostics}");
    assert_eq!(output, expected[3]);

    fs::write(&ledger, query_ledger(Some("foreign"), None)).unwrap();
    for words in commands {
        let (status, output, diagnostics) = registered_query(root, words, query_gate);
        assert_eq!(status, ExitCode::FAILURE, "{words:?}: {diagnostics}");
        assert!(output.is_empty() && diagnostics.contains("query owner must be demo"));
    }
    let destination = root.join("refused.nt");
    let args = ["baseline", "--out", destination.to_str().unwrap()];
    let (status, output, diagnostics) = registered_query(root, &args, query_gate);
    assert_eq!(status, ExitCode::FAILURE, "{diagnostics}");
    assert!(output.is_empty() && !destination.exists());

    let (status, output, diagnostics) =
        registered_query(root, &["job", "number", "--unknown"], |_| {
            panic!("invalid arguments reached Gate configuration")
        });
    assert_eq!(status, ExitCode::from(2));
    assert!(output.is_empty() && diagnostics.contains("--unknown"));
}

#[test]
fn native_index_never_builds_or_executes_consumer_targets() {
    let scratch = fixture();
    let root = scratch.path();
    fs::create_dir(root.join("examples")).unwrap();
    fs::create_dir(root.join("tests")).unwrap();
    let dangerous = "fn main() { std::fs::write(\"consumer-executed\", b\"bad\").unwrap(); panic!(\"must not execute\"); }\n";
    for path in ["build.rs", "examples/registered.rs", "tests/destructive.rs"] {
        fs::write(root.join(path), dangerous).unwrap();
    }
    let report = Gate::workspace(root).check().unwrap();
    let expected = report
        .census()
        .index(report.jobs())
        .unwrap()
        .to_ntriples()
        .unwrap();
    let result = run(root, &["index"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, expected.as_bytes());
    let checked = run(root, &["check"]);
    assert_eq!(checked.status.code(), Some(i32::from(!report.is_clean())));
    let bounded = run(root, &["index", "--max-index-bytes", "1"]);
    assert_eq!(bounded.status.code(), Some(1));
    assert_eq!(bounded.stdout, Vec::<u8>::new());
    assert!(!root.join("consumer-executed").exists());
    assert!(!root.join("target").exists());
    for path in ["build.rs", "examples/registered.rs", "tests/destructive.rs"] {
        assert_eq!(fs::read_to_string(root.join(path)).unwrap(), dangerous);
    }
    let refused = run(
        root,
        &["check", "--package", "demo", "--example", "registered"],
    );
    assert_eq!(refused.status.code(), Some(2));
}

#[cfg(target_os = "linux")]
#[test]
fn embedded_report_uses_static_source_and_requires_existing_exact_debt() {
    let scratch = fixture();
    let root = scratch.path();
    embedded_source(root, 2);
    let dangerous = "fn main() { std::fs::write(\"consumer-executed\", b\"bad\").unwrap(); }\n";
    fs::create_dir(root.join("examples")).unwrap();
    fs::create_dir(root.join("tests")).unwrap();
    for path in [
        "build.rs",
        "examples/destructive.rs",
        "tests/destructive.rs",
    ] {
        fs::write(root.join(path), dangerous).unwrap();
    }
    let tree = DiskTree::new(root).unwrap();
    let source = raf_source::Workspace::load(&tree, &["."]).unwrap();
    let analysis =
        raf_census::embedded::Run::analyze(&source, &raf_census::embedded::Options::default())
            .unwrap();
    assert!(analysis.witnesses().len() > 0);
    let expected = analysis.report(|outcome| {
        outcome
            .unwrap()
            .to_text(Format::NTriples, |text| text.unwrap().as_str().to_owned())
    });
    let report = run(root, &["embedded", "report"]);
    assert!(report.status.success(), "{report:?}");
    assert_eq!(report.stdout, expected.as_bytes());
    let missing = run(root, &["embedded", "check"]);
    assert_eq!(missing.status.code(), Some(1), "{missing:?}");
    assert_eq!(missing.stdout, [] as [u8; 0]);
    let debt = analysis
        .adopt_debt(|outcome| outcome.unwrap())
        .to_text(Format::Turtle, |outcome| outcome.unwrap());
    let debt_path = root.join(raf_census::embedded::DEBT_FILE);
    let created = run(root, &["embedded", "debt", "create"]);
    assert!(created.status.success(), "{created:?}");
    assert_eq!(fs::read_to_string(&debt_path).unwrap(), debt.as_str());
    let refused = run(root, &["embedded", "debt", "create"]);
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert_eq!(fs::read_to_string(&debt_path).unwrap(), debt.as_str());
    let checked = run(root, &["embedded", "check"]);
    assert!(checked.status.success(), "{checked:?}");
    embedded_source(root, 1);
    let stale = run(root, &["embedded", "check"]);
    assert_eq!(stale.status.code(), Some(1), "{stale:?}");
    let shrunk = run(root, &["embedded", "debt", "shrink"]);
    assert!(shrunk.status.success(), "{shrunk:?}");
    let exact = run(root, &["embedded", "check"]);
    assert!(exact.status.success(), "{exact:?}");
    let retained = fs::read(&debt_path).unwrap();
    embedded_source(root, 3);
    for command in [
        vec!["embedded", "check"],
        vec!["embedded", "debt", "shrink"],
    ] {
        let growth = run(root, &command);
        assert_eq!(growth.status.code(), Some(1), "{growth:?}");
        assert_eq!(fs::read(&debt_path).unwrap(), retained);
    }
    assert!(!root.join("consumer-executed").exists());
    assert!(!root.join("target").exists());
    for path in [
        "build.rs",
        "examples/destructive.rs",
        "tests/destructive.rs",
    ] {
        assert_eq!(fs::read_to_string(root.join(path)).unwrap(), dangerous);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn embedded_debt_publication_never_creates_a_declared_missing_rust_target() {
    for (name, arguments) in [
        (
            ".census-embedded-debt.ttl.lock",
            vec!["embedded", "debt", "create"],
        ),
        (
            ".census-embedded-debt.ttl.write",
            vec!["embedded", "debt", "create"],
        ),
        (
            "census-embedded-debt.ttl",
            vec!["embedded", "debt", "create"],
        ),
    ] {
        let scratch = fixture();
        let root = scratch.path();
        fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[lib]\npath='{name}'\n[workspace]\n"),
        ).unwrap();
        let result = run(root, &arguments);
        assert!(
            !root.join(name).exists(),
            "publication created a declared Rust target: {result:?}"
        );
        assert_eq!(result.status.code(), Some(1), "{result:?}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn embedded_debt_publication_preserves_unowned_entries_and_busy_records() {
    use std::os::unix::fs::symlink;

    for name in [
        ".census-embedded-debt.ttl.write",
        "census-embedded-debt.ttl",
    ] {
        let scratch = fixture();
        let root = scratch.path();
        let entry = root.join(name);
        let protected = root.join("protected-data");
        let source = fs::read(root.join("src/lib.rs")).unwrap();
        fs::write(&protected, b"preserve").unwrap();
        symlink(&protected, &entry).unwrap();
        let refused = run(root, &["embedded", "debt", "create"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert_eq!(fs::read_link(&entry).unwrap(), protected);
        assert_eq!(fs::read(&protected).unwrap(), b"preserve");
        fs::remove_file(&entry).unwrap();
        fs::create_dir(&entry).unwrap();
        let refused = run(root, &["embedded", "debt", "create"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert!(entry.is_dir());
        fs::remove_dir(&entry).unwrap();
        fs::write(&entry, b"pre-existing metadata").unwrap();
        let refused = run(root, &["embedded", "debt", "create"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert_eq!(fs::read(&entry).unwrap(), b"pre-existing metadata");
        fs::remove_file(&entry).unwrap();
        let lock = fs::File::open(root).unwrap();
        lock.try_lock().unwrap();
        for command in [vec!["embedded", "debt", "create"], vec!["debt", "--write"]] {
            let busy = run(root, &command);
            assert_eq!(busy.status.code(), Some(1), "{busy:?}");
            assert!(
                String::from_utf8(busy.stderr)
                    .unwrap()
                    .contains("publication is busy")
            );
        }
        drop(lock);
        assert!(!root.join(raf_census::embedded::DEBT_FILE).exists());
        let valid = run(root, &["embedded", "debt", "create"]);
        assert!(valid.status.success(), "{valid:?}");
        assert_eq!(fs::read(root.join("src/lib.rs")).unwrap(), source);
        assert!(!root.join(".census-embedded-debt.ttl.lock").exists());
        assert!(!root.join(".census-debt.ttl.lock").exists());
    }
}

#[cfg(target_os = "linux")]
fn embedded_source(root: &Path, copies: usize) {
    let helpers = [
        "fn one(input:i32) { let a=input+1; let b=a*3; let c=b+a; consume(c,b,input,7); }\n",
        "fn two(other:i32) { let x=other+1; let y=x*3; let z=y+x; consume(z,y,other,7); }\n",
        "fn three(value:i32) { let p=value+1; let q=p*3; let r=q+p; consume(r,q,value,7); }\n",
    ];
    fs::write(
        root.join("src/lib.rs"),
        format!("#![no_implicit_prelude]\n{}", helpers[..copies].concat()),
    )
    .unwrap();
}

#[test]
fn report_exports_refuse_source_existing_files_and_git_administration() {
    let scratch = fixture();
    let root = scratch.path();
    let original = fs::read(root.join("src/lib.rs")).unwrap();
    for path in ["src/lib.rs", "Cargo.toml", "helpers-ledger.ttl"] {
        let result = run(root, &["baseline", "--out", path]);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
    }
    fs::create_dir(root.join(".git")).unwrap();
    let result = run(root, &["baseline", "--out", ".git/report.ttl"]);
    assert_eq!(result.status.code(), Some(1));
    assert!(!root.join(".git/report.ttl").exists());
    fs::remove_dir(root.join(".git")).unwrap();
    let administration = scratch.path().join("bare-administration");
    fs::create_dir_all(administration.join("objects")).unwrap();
    fs::create_dir_all(administration.join("refs/heads")).unwrap();
    fs::write(administration.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::write(
        administration.join("config"),
        "[core]\nrepositoryformatversion=0\nbare=true\n",
    )
    .unwrap();
    let destination = administration.join("report.ttl");
    let result = alone(
        Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(root)
            .arg("baseline")
            .arg("--out")
            .arg(&destination),
    );
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert!(!destination.exists());
    let result = run(root, &["baseline", "--out", "report.ttl"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, Vec::<u8>::new());
    assert_eq!(fs::read(root.join("src/lib.rs")).unwrap(), original);
}

#[cfg(target_os = "linux")]
#[test]
fn debt_publication_never_creates_a_declared_missing_rust_target() {
    for name in [
        ".census-debt.ttl.lock",
        ".census-debt.ttl.write",
        "census-debt.ttl",
    ] {
        let scratch = fixture();
        let root = scratch.path();
        fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[lib]\npath='{name}'\n[workspace]\n"),
        ).unwrap();
        let result = run(root, &["debt", "--write"]);
        assert!(
            !root.join(name).exists(),
            "publication created a declared Rust target: {result:?}"
        );
        assert_eq!(result.status.code(), Some(1), "{result:?}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn debt_publication_preserves_unowned_entries_and_busy_records() {
    use std::os::unix::fs::symlink;

    for name in [".census-debt.ttl.write", "census-debt.ttl"] {
        let scratch = fixture();
        let root = scratch.path();
        let entry = root.join(name);
        let protected = root.join("protected-data");
        let source = fs::read(root.join("src/lib.rs")).unwrap();
        fs::write(&protected, b"preserve").unwrap();
        symlink(&protected, &entry).unwrap();
        let refused = run(root, &["debt", "--write"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert_eq!(fs::read_link(&entry).unwrap(), protected);
        assert_eq!(fs::read(&protected).unwrap(), b"preserve");
        fs::remove_file(&entry).unwrap();
        fs::create_dir(&entry).unwrap();
        let refused = run(root, &["debt", "--write"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert!(entry.is_dir());
        fs::remove_dir(&entry).unwrap();
        fs::write(&entry, b"pre-existing metadata").unwrap();
        let refused = run(root, &["debt", "--write"]);
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert_eq!(fs::read(&entry).unwrap(), b"pre-existing metadata");
        fs::remove_file(&entry).unwrap();
        let lock = fs::File::open(root).unwrap();
        lock.try_lock().unwrap();
        let busy = run(root, &["debt", "--write"]);
        assert_eq!(busy.status.code(), Some(1), "{busy:?}");
        assert!(
            String::from_utf8(busy.stderr)
                .unwrap()
                .contains("publication is busy")
        );
        drop(lock);
        let valid = run(root, &["debt", "--write"]);
        assert!(valid.status.success(), "{valid:?}");
        assert_eq!(fs::read(root.join("src/lib.rs")).unwrap(), source);
        assert!(!root.join(".census-debt.ttl.lock").exists());
    }
}

#[test]
fn debt_command_adopts_refuses_growth_and_records_shrink_through_the_real_gate() {
    let scratch = fixture();
    let root = scratch.path();
    let first = run(root, &["census", "debt", "--write"]);
    assert!(first.status.success(), "{first:?}");
    let path = root.join("census-debt.ttl");
    let before = fs::read_to_string(&path).unwrap();
    let policy = DebtRecord::from_turtle(&before).unwrap();
    assert_eq!(policy.job_copies()["number"], 2);
    Gate::workspace(root).write_debt().unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    source(root, 3);
    let growth = run(root, &["debt", "--write"]);
    assert_eq!(growth.status.code(), Some(1), "{growth:?}");
    assert!(
        String::from_utf8_lossy(&growth.stderr).contains("growth cannot be recorded"),
        "{growth:?}"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
    source(root, 1);
    let shrink = run(root, &["debt", "--write"]);
    assert!(shrink.status.success(), "{shrink:?}");
    assert!(Gate::workspace(root).check().unwrap().is_clean());
    let lower = fs::read_to_string(&path).unwrap();
    assert_eq!(
        DebtRecord::from_turtle(&lower).unwrap().job_copies()["number"],
        1
    );
    fs::write(
        root.join("helpers-ledger.ttl"),
        ledger(1).replace("demo::home", "demo::gone"),
    )
    .unwrap();
    let stale = run(root, &["debt", "--write"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(fs::read_to_string(path).unwrap(), lower);
    for builtin in [false, true] {
        ratcheted_hits_adopt_refuse_growth_and_lower(builtin);
    }
}

/// A ratcheted pattern (`builtin` false) or built-in detector (`builtin` true):
/// the write adopts its hits as counted sites, a fourth hit is refused, and a
/// removed hit fails the check until the record is lowered.
fn ratcheted_hits_adopt_refuse_growth_and_lower(builtin: bool) {
    let ratcheted = fixture();
    let root = ratcheted.path();
    fs::write(root.join("helpers-ledger.ttl"), ratcheted_ledger(builtin)).unwrap();
    ratcheted_source(root, &["one", "two", "three"], builtin);
    let path = root.join("census-debt.ttl");
    let unadopted = run(root, &["check"]);
    assert_eq!(unadopted.status.code(), Some(1), "{unadopted:?}");
    let adopted = run(root, &["debt", "--write"]);
    assert!(adopted.status.success(), "{adopted:?}");
    assert!(
        String::from_utf8_lossy(&adopted.stdout).contains("3 ratcheted sites"),
        "{adopted:?}"
    );
    let record = fs::read_to_string(&path).unwrap();
    assert_eq!(DebtRecord::from_turtle(&record).unwrap().sites().len(), 3);
    let held = run(root, &["check"]);
    assert!(held.status.success(), "{held:?}");
    ratcheted_source(root, &["one", "two", "three", "four"], builtin);
    for words in [&["check"][..], &["debt", "--write"][..]] {
        let grown = run(root, words);
        assert_eq!(grown.status.code(), Some(1), "{grown:?}");
        assert!(
            [&grown.stdout, &grown.stderr].iter().any(|text| {
                String::from_utf8_lossy(text).contains("growth cannot be recorded")
            }),
            "{grown:?}"
        );
    }
    assert_eq!(fs::read_to_string(&path).unwrap(), record);
    ratcheted_source(root, &["one", "two"], builtin);
    let fallen = run(root, &["check"]);
    assert_eq!(fallen.status.code(), Some(1), "{fallen:?}");
    let lowered = run(root, &["debt", "--write"]);
    assert!(lowered.status.success(), "{lowered:?}");
    assert_eq!(
        DebtRecord::from_turtle(&fs::read_to_string(&path).unwrap())
            .unwrap()
            .sites()
            .len(),
        2
    );
    let settled = run(root, &["check"]);
    assert!(settled.status.success(), "{settled:?}");
}

/// An enforced job whose forbidden pattern or built-in rule is ratcheted: its hits are recorded
/// as counted sites instead of failing the gate.
fn ratcheted_ledger(builtin: bool) -> String {
    let mut graph = Builder::from_graph(Ledger::from_turtle(&ledger(1)).unwrap().graph());
    let enforced = |value| Triple {
        subject: Term::iri("#job"),
        predicate: Term::iri(format!("{}enforced", vocab::REPORT)),
        object: Term::boolean(value),
    };
    assert!(graph.remove(enforced(false)).unwrap());
    graph.insert(enforced(true)).unwrap();
    let constant = Triple {
        subject: Term::iri("#job"),
        predicate: Term::iri(format!("{}forbiddenConstant", vocab::REPORT)),
        object: Term::typed_literal("7", vocab::INTEGER),
    };
    assert!(graph.remove(constant).unwrap());
    if builtin {
        for (key, value) in [
            ("forbiddenFingerprint", "rule:serde-json"),
            ("ratchetedRule", "serde-json"),
        ] {
            put(&mut graph, "#job", key, Term::simple_literal(value));
        }
        return graph.finish().unwrap().to_text(Format::Turtle).unwrap();
    }
    put(&mut graph, "", "pattern", Term::iri("#pattern"));
    put(
        &mut graph,
        "#pattern",
        "type",
        Term::iri(format!("{}Pattern", vocab::REPORT)),
    );
    for (key, value) in [
        ("patternId", "banned"),
        ("summary", "The banned drop call"),
        ("call", "std::mem::drop"),
    ] {
        put(&mut graph, "#pattern", key, Term::simple_literal(value));
    }
    put(&mut graph, "#pattern", "ratcheted", Term::boolean(true));
    put(
        &mut graph,
        "#job",
        "forbiddenPattern",
        Term::simple_literal("banned"),
    );
    graph.finish().unwrap().to_text(Format::Turtle).unwrap()
}

/// One drop call per name, each followed by a distinct string literal.
fn ratcheted_source(root: &Path, names: &[&str], builtin: bool) {
    let hit = if builtin {
        "serde_json::to_string(&0)"
    } else {
        "std::mem::drop(0)"
    };
    let mut text = "/// The operation.\npub fn home()->u32{7}\n".to_owned();
    for name in names {
        writeln!(text, "pub fn hit_{name}(){{{hit};let _=\"{name}\";}}").unwrap();
    }
    fs::write(root.join("src/lib.rs"), text).unwrap();
}

#[test]
fn migration_commands_preserve_inputs_and_emit_the_same_readable_native_policy() {
    let scratch = fixture();
    let root = scratch.path();
    let old = converted_carrier(root, "ledger", &ledger(0));
    fs::write(root.join("old.ttl"), &old).unwrap();
    let tree = DiskTree::new(root).unwrap();
    let census = Census::load(&tree, &["."], &CensusOptions::default()).unwrap();
    let mut index = census.index(&[]).unwrap();
    index.normal_form = 0;
    let previous = converted_carrier(root, "index", &index.to_turtle().unwrap());
    converted_carrier(root, "baseline", &census.baseline().to_turtle().unwrap());
    converted_carrier(
        root,
        "debt",
        &DebtRecord::new(
            raf_census::NORMAL_FORM,
            [],
            std::collections::BTreeMap::new(),
        )
        .unwrap()
        .to_turtle()
        .unwrap(),
    );
    fs::write(root.join("old-index.ttl"), &previous).unwrap();
    let result = run(
        root,
        &["migrate", "--ledger", "old.ttl", "--index", "old-index.ttl"],
    );
    assert!(result.status.success(), "{result:?}");
    let text = purrdf_testkit::text::stdout_utf8(&result);
    assert_eq!(
        Ledger::from_turtle(&text).unwrap().jobs()[0].job.id,
        "number"
    );
    assert_eq!(fs::read_to_string(root.join("old.ttl")).unwrap(), old);
    assert_eq!(
        fs::read_to_string(root.join("old-index.ttl")).unwrap(),
        previous
    );
    let foreign = "[census]\n[[job]]\nid='number'\nsummary='The approved constant'\nhome='demo::home'\nentry_points=[]\nenforced=true\n";
    let baseline = r#"{"min_tokens":20,"units":3,"body_groups":[],"shim_groups":[],"hex_tables":[],"shared_constants":[]}"#;
    // Separate report values exceed the ledger byte cap without
    // exceeding the baseline decoder's per-string or value-count bounds.
    let metric = format!("\"{}\",", "x".repeat(MAX_LEDGER_BYTES / 8));
    let baseline = baseline.replace(
        "\"hex_tables\":[]",
        &format!("\"hex_tables\":[{}null]", metric.repeat(9)),
    );
    assert!(baseline.len() > MAX_LEDGER_BYTES);
    fs::write(root.join("old.toml"), foreign).unwrap();
    fs::write(root.join("old.json"), &baseline).unwrap();
    let words = [
        "migrate",
        "--legacy",
        "--ledger",
        "old.toml",
        "--baseline",
        "old.json",
    ];
    let result = run(root, &words);
    assert!(result.status.success(), "{result:?}");
    Ledger::from_turtle(&purrdf_testkit::text::stdout_utf8(&result)).unwrap();
    assert_eq!(fs::read_to_string(root.join("old.toml")).unwrap(), foreign);
    assert_eq!(fs::read_to_string(root.join("old.json")).unwrap(), baseline);
    fs::write(
        root.join("old.toml"),
        format!("{foreign}unrecognized='original'\n"),
    )
    .unwrap();
    let unmapped = run(root, &words);
    assert_eq!(unmapped.status.code(), Some(1));
    assert!(purrdf_testkit::text::stdout_utf8(&unmapped).contains("# unmapped:"));
}

fn converted_carrier(root: &Path, name: &str, current: &str) -> String {
    let kind = match name {
        "baseline" => raf_census::ReportKind::Baseline,
        "index" => raf_census::ReportKind::Index,
        "ledger" => raf_census::ReportKind::Ledger,
        "debt" => raf_census::ReportKind::Debt,
        _ => panic!("the fixture declares one of four carrier kinds"),
    };
    let graph = raf_rdf::Graph::from_text(current, Format::Turtle).unwrap();
    for (syntax, format) in [("turtle", Format::Turtle), ("ntriples", Format::NTriples)] {
        let expected = graph.to_text(format).unwrap();
        layout::assert_upgrade(kind, &expected, format);
        let prior = layout::form3(&expected, format);
        let path = format!("prior-{name}.{syntax}");
        fs::write(root.join(&path), &prior).unwrap();
        let result = run(
            root,
            &[
                "migrate",
                "--report-form",
                name,
                "--input",
                &path,
                "--format",
                syntax,
            ],
        );
        assert!(result.status.success(), "{result:?}");
        assert_eq!(result.stdout, expected.as_bytes());
        assert_eq!(fs::read(root.join(&path)).unwrap(), prior.as_bytes());
        fs::write(root.join(&path), &expected).unwrap();
        let refused = run(
            root,
            &[
                "migrate",
                "--report-form",
                name,
                "--input",
                &path,
                "--format",
                syntax,
            ],
        );
        assert_eq!(refused.status.code(), Some(1), "{refused:?}");
        assert_eq!(refused.stdout, b"");
        assert_eq!(fs::read(root.join(&path)).unwrap(), expected.as_bytes());
    }
    current.into()
}

#[test]
fn command_usage_and_bounded_input_failures_do_not_publish_a_record() {
    baseline_discovery();
    ledger_inspection();
    let scratch = fixture();
    let root = scratch.path();
    for words in [
        vec!["debt"],
        vec!["migrate", "--legacy", "--ledger", "missing"],
        vec!["migrate", "--ledger", "missing"],
        vec!["baseline", "--format", "json"],
        vec!["baseline", "--out"],
        vec!["dump-ledger", "--format", "json"],
        vec!["dump-ledger", "--out"],
        vec![
            "check",
            "--package",
            "demo",
            "--example",
            "registered",
            "--max-index-bytes",
            "1",
        ],
        vec!["debt", "--write", "--max-index-bytes", "1"],
    ] {
        assert_eq!(run(root, &words).status.code(), Some(2), "{words:?}");
    }
    let above = (raf_census::MAX_INDEX_BYTES + 1).to_string();
    for (command, selector) in [
        ("index", None),
        ("job", Some("copies")),
        ("count", Some("copies")),
        ("fingerprint", Some("demo::home")),
    ] {
        for limit in ["0", above.as_str()] {
            let mut words = vec![command];
            words.extend(selector);
            words.extend(["--max-index-bytes", limit]);
            assert_eq!(run(root, &words).status.code(), Some(2), "{words:?}");
        }
    }
    for words in [vec!["--help"], vec!["census", "--version"]] {
        assert!(run(root, &words).status.success());
    }
    fs::write(root.join("large.toml"), " ".repeat(MAX_LEDGER_BYTES + 1)).unwrap();
    let result = run(
        root,
        &[
            "migrate",
            "--legacy",
            "--ledger",
            "large.toml",
            "--baseline",
            "missing",
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("large.toml"));
    assert_eq!(result.stdout, [] as [u8; 0]);
    assert!(!root.join("census-debt.ttl").exists());
}

fn ledger_inspection() {
    let scratch = fixture();
    let root = scratch.path();
    let policy = Ledger::from_turtle(&ledger(1)).unwrap();
    let mut graph = Builder::from_graph(policy.graph());
    put(
        &mut graph,
        "",
        "reference",
        Term::simple_literal("dependency"),
    );
    put(&mut graph, "#job", "extension", Term::iri("#extension"));
    put(
        &mut graph,
        "#extension",
        "type",
        Term::iri(format!("{}Extension", vocab::REPORT)),
    );
    graph
        .insert(Triple {
            subject: Term::iri("#extension"),
            predicate: Term::iri("urn:example:owner"),
            object: Term::simple_literal("unverified declaration"),
        })
        .unwrap();
    let original = graph.finish().unwrap().to_text(Format::Turtle).unwrap();
    fs::write(root.join("helpers-ledger.ttl"), &original).unwrap();
    fs::write(root.join("src/lib.rs"), "fn broken(\n").unwrap();
    let native = Ledger::from_turtle(&original).unwrap();
    let paths = ["Cargo.toml", "src/lib.rs", "helpers-ledger.ttl"];
    let before = paths.map(|path| fs::read(root.join(path)).unwrap());
    let result = run(root, &["dump-ledger"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(
        result.stdout,
        native.graph().to_text(Format::NTriples).unwrap().as_bytes()
    );
    assert_eq!(run(root, &["dump-ledger"]).stdout, result.stdout);
    assert_eq!(
        Ledger::from_ntriples(std::str::from_utf8(&result.stdout).unwrap())
            .unwrap()
            .graph()
            .snapshot()
            .unwrap(),
        native.graph().snapshot().unwrap()
    );
    let error = Gate::workspace(root).check().unwrap_err();
    assert!(
        matches!(&error, raf_census::Error::Load(source)
            if source.file.as_str() == "src/lib.rs"
                && matches!(&source.kind, raf_source::LoadErrorKind::Syntax { .. })),
        "{error}"
    );
    let caller = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let result = alone(
        Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(caller.path())
            .args(["--workspace"])
            .arg(root)
            .args(["dump-ledger", "--format", "turtle", "--out", "policy.ttl"]),
    );
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, [] as [u8; 0]);
    let text = fs::read_to_string(caller.path().join("policy.ttl")).unwrap();
    assert_eq!(text, native.graph().to_text(Format::Turtle).unwrap());
    assert_eq!(
        Ledger::from_turtle(&text)
            .unwrap()
            .graph()
            .snapshot()
            .unwrap(),
        native.graph().snapshot().unwrap()
    );
    assert_eq!(paths.map(|path| fs::read(root.join(path)).unwrap()), before);
    assert!(!root.join("census-debt.ttl").exists());
    ledger_inspection_refusals(root, caller.path(), &original);
}

fn ledger_inspection_refusals(root: &Path, caller: &Path, original: &str) {
    let input = root.join("helpers-ledger.ttl");
    let destination = caller.join("policy.ttl");
    for invalid in [
        None,
        Some("not a ledger".to_owned()),
        Some(" ".repeat(MAX_LEDGER_BYTES + 1)),
        Some(ledger(0)),
    ] {
        if let Some(text) = &invalid {
            fs::write(&input, text).unwrap();
        } else {
            fs::remove_file(&input).unwrap();
        }
        for present in [true, false] {
            if present {
                fs::write(&destination, "preserve prior output").unwrap();
            } else {
                fs::remove_file(&destination).unwrap();
            }
            let result = alone(
                Command::new(env!("CARGO_BIN_EXE_cargo-census"))
                    .current_dir(root)
                    .args(["dump-ledger", "--out"])
                    .arg(&destination),
            );
            assert_eq!(result.status.code(), Some(1), "{result:?}");
            assert_eq!(result.stdout, [] as [u8; 0]);
            assert!(String::from_utf8_lossy(&result.stderr).contains("helpers-ledger.ttl"));
            if present {
                assert_eq!(fs::read(&destination).unwrap(), b"preserve prior output");
            } else {
                assert!(!destination.exists());
            }
        }
        if let Some(text) = &invalid {
            assert_eq!(fs::read_to_string(&input).unwrap(), *text);
        } else {
            assert!(!input.exists());
        }
    }
    fs::write(&input, original).unwrap();
    let result = run(root, &["dump-ledger", "--out", "."]);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert_eq!(result.stdout, [] as [u8; 0]);
    assert_ne!(result.stderr, [] as [u8; 0]);
    #[cfg(unix)]
    {
        let external = caller.join("external-ledger.ttl");
        fs::write(&external, original).unwrap();
        fs::remove_file(&input).unwrap();
        std::os::unix::fs::symlink(&external, &input).unwrap();
        let result = run(root, &["dump-ledger"]);
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(result.stdout, [] as [u8; 0]);
        assert!(String::from_utf8_lossy(&result.stderr).contains("helpers-ledger.ttl"));
        assert_eq!(fs::read_to_string(&external).unwrap(), original);
        fs::remove_file(&input).unwrap();
        fs::write(&input, original).unwrap();
    }
    assert!(run(root, &["dump-ledger"]).status.success());
}

fn baseline_discovery() {
    let scratch = fixture();
    let root = scratch.path();
    fs::write(
        root.join("src/lib.rs"),
        "pub fn first(x:u32)->u32{let a=x.wrapping_add(1);let b=a.wrapping_mul(3);b.wrapping_sub(2)}\n\
         pub fn second(y:u32)->u32{let c=y.wrapping_add(1);let d=c.wrapping_mul(3);d.wrapping_sub(2)}\n",
    )
    .unwrap();
    // Discovery never treats a missing or malformed policy as a clean verdict.
    fs::write(root.join("helpers-ledger.ttl"), "not a ledger").unwrap();
    let paths = ["Cargo.toml", "src/lib.rs", "helpers-ledger.ttl"];
    let before = paths.map(|path| fs::read(root.join(path)).unwrap());
    let native = Census::load(
        &DiskTree::new(root).unwrap(),
        &["."],
        &CensusOptions::default(),
    )
    .unwrap()
    .baseline();
    assert_ne!(native.body_groups, [] as [raf_census::BaselineGroup; 0]);
    let result = run(root, &["baseline"]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, native.to_ntriples().unwrap().as_bytes());
    assert_eq!(run(root, &["baseline"]).stdout, result.stdout);
    assert_eq!(
        Baseline::from_ntriples(std::str::from_utf8(&result.stdout).unwrap()).unwrap(),
        native
    );
    let caller = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let turtle = alone(
        Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(caller.path())
            .args(["--workspace"])
            .arg(root)
            .args(["baseline", "--format", "turtle", "--out", "report.ttl"]),
    );
    assert!(turtle.status.success(), "{turtle:?}");
    assert_eq!(turtle.stdout, [] as [u8; 0]);
    let text = fs::read_to_string(caller.path().join("report.ttl")).unwrap();
    assert_eq!(text, native.to_turtle().unwrap());
    assert_eq!(Baseline::from_turtle(&text).unwrap(), native);
    assert_eq!(paths.map(|path| fs::read(root.join(path)).unwrap()), before);
    assert!(!root.join("census-debt.ttl").exists());
    baseline_refusals(root, caller.path());
}

fn baseline_refusals(root: &Path, caller: &Path) {
    let destination = caller.join("report.ttl");
    let manifest = fs::read(root.join("Cargo.toml")).unwrap();
    fs::write(root.join("Cargo.toml"), "not a manifest [").unwrap();
    for present in [true, false] {
        if present {
            fs::write(&destination, "preserve prior output").unwrap();
        } else {
            fs::remove_file(&destination).unwrap();
        }
        let result = alone(
            Command::new(env!("CARGO_BIN_EXE_cargo-census"))
                .current_dir(root)
                .arg("baseline")
                .arg("--out")
                .arg(&destination),
        );
        assert_eq!(result.status.code(), Some(1), "{result:?}");
        assert_eq!(result.stdout, [] as [u8; 0]);
        assert!(String::from_utf8_lossy(&result.stderr).contains("Cargo.toml"));
        if present {
            assert_eq!(fs::read(&destination).unwrap(), b"preserve prior output");
        } else {
            assert!(!destination.exists());
        }
    }
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let result = run(root, &["baseline", "--out", "."]);
    assert_eq!(result.status.code(), Some(1), "{result:?}");
    assert_eq!(result.stdout, [] as [u8; 0]);
    assert_ne!(result.stderr, [] as [u8; 0]);
    assert!(run(root, &["baseline"]).status.success());
}

#[test]
fn registered_code_is_required_explicitly_without_recording_partial_debt() {
    let scratch = fixture();
    let root = scratch.path();
    for kind in ["rule", "extension", "variant"] {
        let policy = Ledger::from_turtle(&ledger(1)).unwrap();
        let mut graph = Builder::from_graph(policy.graph());
        if kind == "extension" {
            put(&mut graph, "#job", "extension", Term::iri("#extension"));
            put(
                &mut graph,
                "#extension",
                "type",
                Term::iri(format!("{}Extension", vocab::REPORT)),
            );
            graph
                .insert(Triple {
                    subject: Term::iri("#extension"),
                    predicate: Term::iri("urn:example:owner"),
                    object: Term::simple_literal("runtime"),
                })
                .unwrap();
        } else if kind == "variant" {
            put(&mut graph, "#job", "variant", Term::iri("#variant"));
            put(
                &mut graph,
                "#variant",
                "type",
                Term::iri(format!("{}Variant", vocab::REPORT)),
            );
            for (key, value) in [
                ("symbol", "demo::copy_0"),
                ("file", "src/lib.rs"),
                ("detector", "custom-check"),
                ("criterion", "layering"),
                ("reason", "Independent fixture implementation"),
            ] {
                put(&mut graph, "#variant", key, Term::simple_literal(value));
            }
            put(
                &mut graph,
                "#variant",
                "anchor",
                Term::iri("urn:raf:code:demo::home"),
            );
        } else {
            put(
                &mut graph,
                "#job",
                "forbiddenFingerprint",
                Term::simple_literal("rule:custom-check"),
            );
        }
        fs::write(
            root.join("helpers-ledger.ttl"),
            graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
        )
        .unwrap();
        let result = run(root, &["debt", "--write"]);
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("Gate::write_debt"));
        assert!(!root.join("census-debt.ttl").exists());
    }
}

#[test]
fn self_test_needs_no_workspace_writes_nothing_and_prints_stable_case_lines() {
    let empty = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let first = run(empty.path(), &["census", "self-test"]);
    let second = run(empty.path(), &["census", "self-test"]);
    let text = String::from_utf8(first.stdout.clone()).unwrap();
    assert!(first.status.success(), "{text}{:?}", first.stderr);
    assert_eq!(first.stdout, second.stdout);
    assert!(
        text.contains("pass detector/hex-table/positive\n"),
        "{text}"
    );
    assert!(text.contains("pass debt/rising-copies-refused\n"), "{text}");
    assert!(text.ends_with(" cases, 0 failed\n"), "{text}");
    assert!(
        !text.lines().any(|line| line.starts_with("fail ")),
        "{text}"
    );
    assert_eq!(fs::read_dir(empty.path()).unwrap().count(), 0);
}

#[test]
fn self_test_ignores_the_workspace_flag_even_for_a_path_that_does_not_exist() {
    let empty = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let plain = run(empty.path(), &["census", "self-test"]);
    let flagged = run(
        empty.path(),
        &[
            "census",
            "--workspace",
            "/nonexistent/self-test-workspace",
            "self-test",
        ],
    );
    assert!(plain.status.success(), "{plain:?}");
    assert_eq!(flagged.status.code(), plain.status.code(), "{flagged:?}");
    assert_eq!(flagged.stdout, plain.stdout);
    assert!(flagged.stderr.is_empty(), "{flagged:?}");
}

/// The command line declares one reference tree for the evaluating commands:
/// a repository at a commit given directly or through a pin file, or an
/// unpinned directory whose index receipts carry the `unpinned-tree` label.
/// Incomplete or conflicting reference options refuse, and a command
/// that does not evaluate the Gate refuses them rather than ignoring them.
#[test]
fn reference_options_select_a_pinned_commit_or_a_labelled_unpinned_directory() {
    let scratch = fixture();
    let root = scratch.path();
    let body = "(value:u32)->u32 { let scaled=value*3; let shifted=scaled+7; if shifted>100 { shifted-100 } else { shifted*2 } }";
    let consumer = format!("pub fn copy{body}\n");
    let mut text = fs::read_to_string(root.join("src/lib.rs")).unwrap();
    text.push_str(&consumer);
    fs::write(root.join("src/lib.rs"), text).unwrap();
    let outside = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let repository = outside.path().join("upstream");
    let plain = outside.path().join("plain");
    let library = format!("pub fn provided{body}\n");
    let files = [
        (
            "Cargo.toml",
            "[package]\nname='upstream'\nversion='1.0.0'\nedition='2024'\n",
        ),
        ("src/lib.rs", library.as_str()),
    ];
    git_repo::init(&repository);
    files::write(&repository, &files);
    let commit = git_repo::commit_all(&repository);
    files::write(&plain, &files);
    fs::write(
        root.join("pins.toml"),
        format!("[upstream]\nrev = \"{commit}\"\n"),
    )
    .unwrap();
    let repository = repository.to_str().unwrap();
    let plain = plain.to_str().unwrap();

    let pinned = run(
        root,
        &[
            "check",
            "--reference-repo",
            repository,
            "--reference-commit",
            &commit,
        ],
    );
    assert_eq!(pinned.status.code(), Some(1), "{pinned:?}");
    let stdout = String::from_utf8(pinned.stdout.clone()).unwrap();
    assert!(
        stdout.contains("REFERENCE-COPY src/lib.rs:5 demo::copy matches upstream::provided "),
        "{stdout}"
    );
    let from_file = run(
        root,
        &[
            "check",
            "--reference-repo",
            repository,
            "--reference-pin-file",
            "pins.toml",
            "--reference-pin-key",
            "upstream.rev",
        ],
    );
    assert_eq!(from_file.stdout, pinned.stdout);
    let unpinned = run(root, &["check", "--reference-unpinned", plain]);
    assert_eq!(unpinned.stdout, pinned.stdout);
    unpinned_notice(root, plain, outside.path(), &pinned, &from_file);
    let index = |words: &[&str]| {
        let result = run(root, words);
        assert!(result.status.success(), "{result:?}");
        String::from_utf8(result.stdout).unwrap()
    };
    let labelled = index(&["index", "--reference-unpinned", plain]);
    assert!(labelled.contains("\"unpinned-tree\""), "{labelled}");
    let committed = index(&[
        "index",
        "--reference-repo",
        repository,
        "--reference-commit",
        &commit,
    ]);
    assert!(committed.contains("\"committed-git\""), "{committed}");
    assert!(!committed.contains("\"unpinned-tree\""));
    reference_option_refusals(root, repository, &commit, plain);
}

/// The refusals of the reference options, each beside the valid uses above.
fn reference_option_refusals(root: &Path, repository: &str, commit: &str, plain: &str) {
    for words in [
        &["check", "--reference-repo", repository][..],
        &["check", "--reference-commit", commit],
        &[
            "check",
            "--reference-repo",
            repository,
            "--reference-pin-file",
            "pins.toml",
        ],
        &[
            "check",
            "--reference-repo",
            repository,
            "--reference-commit",
            commit,
            "--reference-unpinned",
            plain,
        ],
    ] {
        let refused = run(root, words);
        assert_eq!(refused.status.code(), Some(1), "{words:?}");
        assert!(
            String::from_utf8_lossy(&refused.stderr)
                .contains("reference options: give --reference-repo"),
            "{refused:?}"
        );
    }
    let refused = run(root, &["baseline", "--reference-unpinned", plain]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("reference options apply only to"),
        "{refused:?}"
    );
}

/// An unpinned run names its directory on stderr beside findings that match
/// the pinned run, an unpinned run without reference findings says so too, a pinned run is silent,
/// and an unpinned directory that does not exist refuses.
fn unpinned_notice(root: &Path, plain: &str, outside: &Path, pinned: &Output, from_file: &Output) {
    let unpinned = run(root, &["check", "--reference-unpinned", plain]);
    let notice = String::from_utf8_lossy(&unpinned.stderr);
    assert!(
        notice.contains("unpinned reference") && notice.contains(plain),
        "{notice}"
    );
    assert!(pinned.stderr.is_empty(), "{pinned:?}");
    assert!(from_file.stderr.is_empty(), "{from_file:?}");
    let other = outside.join("other");
    files::write(
        &other,
        &[
            (
                "Cargo.toml",
                "[package]\nname='other'\nversion='1.0.0'\nedition='2024'\n",
            ),
            ("src/lib.rs", "pub fn unrelated() -> u8 { 1 }\n"),
        ],
    );
    let clean = run(
        root,
        &["check", "--reference-unpinned", other.to_str().unwrap()],
    );
    assert!(!String::from_utf8_lossy(&clean.stdout).contains("REFERENCE-COPY"));
    assert!(String::from_utf8_lossy(&clean.stderr).contains("unpinned reference"));
    let missing = outside.join("absent");
    let refused = run(
        root,
        &["check", "--reference-unpinned", missing.to_str().unwrap()],
    );
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert!(String::from_utf8_lossy(&refused.stderr).contains("absent"));
}

/// The body `provided` and a consumer copy repeat: `value*factor` plus `offset`.
fn scaled(factor: u32, offset: u32) -> String {
    format!("(value:u32)->u32 {{ let scaled=value*{factor}; scaled+{offset} }}")
}

/// Write the ledger of a consumer that declares `selector` as a reference and
/// homes it with no job.
fn write_reference_ledger(root: &Path, selector: &str) {
    write_ledger(root, &[selector]);
}

/// Write the ledger of a consumer that declares each of `selectors` as a
/// reference, none when the list is empty.
fn write_ledger(root: &Path, selectors: &[&str]) {
    write_ledger_with(root, selectors, false);
}

/// [`write_ledger`], and when `job` is set also the job `number`, homed at
/// `app::home`, so that `job` and `count` have a selector to answer.
fn write_ledger_with(root: &Path, selectors: &[&str], job: bool) {
    let mut graph = Builder::new();
    put(
        &mut graph,
        "",
        "type",
        Term::iri(format!("{}Ledger", vocab::REPORT)),
    );
    for (key, value) in [
        ("normalForm", 1),
        ("reportForm", raf_census::REPORT_FORM),
        ("minTokens", 1),
    ] {
        put(
            &mut graph,
            "",
            key,
            Term::typed_literal(value.to_string(), vocab::INTEGER),
        );
    }
    put(&mut graph, "", "root", Term::simple_literal("."));
    for selector in selectors {
        put(&mut graph, "", "reference", Term::simple_literal(*selector));
    }
    if job {
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
            ("homeSymbol", "app::home"),
        ] {
            put(&mut graph, "#job", key, Term::simple_literal(value));
        }
        put(&mut graph, "#job", "enforced", Term::boolean(false));
        put(
            &mut graph,
            "#job",
            "forbiddenConstant",
            Term::typed_literal("7", vocab::INTEGER),
        );
    }
    fs::write(
        root.join("helpers-ledger.ttl"),
        graph.finish().unwrap().to_text(Format::Turtle).unwrap(),
    )
    .unwrap();
}

/// A workspace `ws` depending on the path package `acme-a` beside it, whose
/// `provided` body is `scaled(2, 7)`, and a ledger referencing the family
/// `acme-*`. The consumer source is written by the caller.
fn ban_workspace(scratch: &Path) -> std::path::PathBuf {
    let root = scratch.join("ws");
    files::write(
        &root,
        &[
            (
                "Cargo.toml",
                "[package]\nname='demo'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-a={path='../vendor/acme-a'}\n[workspace]\n",
            ),
            (
                "Cargo.lock",
                "version=4\n[[package]]\nname='acme-a'\nversion='1.0.0'\n[[package]]\nname='demo'\nversion='1.0.0'\ndependencies=['acme-a']\n",
            ),
        ],
    );
    files::write(
        scratch,
        &[
            (
                "vendor/acme-a/Cargo.toml",
                "[package]\nname='acme-a'\nversion='1.0.0'\nedition='2024'\n",
            ),
            (
                "vendor/acme-a/src/lib.rs",
                format!("pub fn provided{}\n", scaled(2, 7)).as_str(),
            ),
        ],
    );
    write_reference_ledger(&root, "acme-*");
    root
}

fn write_consumer(root: &Path, copy: &str, extra: &str) {
    files::write(
        root,
        &[("src/lib.rs", format!("pub fn copy{copy}\n{extra}").as_str())],
    );
}

/// A consumer adopts the copy ban with a ledger `reference` and no job:
/// `check` fails with exit 1 on a copied body and passes on a changed one,
/// `debt --write` refuses the same finding and records nothing, and `index`
/// and `fingerprint` read the same inputs and carry the same witness. An
/// unusable declaration or a missing source refuses every evaluating command
/// with no verdict on stdout, and a ledger reference composes with a pinned
/// external commit.
#[test]
fn ledger_reference_ban_fails_check_and_refuses_unusable_inputs_without_a_partial_verdict() {
    let scratch = purrdf_testkit::TempDir::new_in(std::env::temp_dir()).unwrap();
    let root = ban_workspace(scratch.path());
    write_consumer(&root, &scaled(2, 7), "");
    let checked = run(&root, &["check"]);
    let stdout = String::from_utf8(checked.stdout).unwrap();
    assert_eq!(checked.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.starts_with("REFERENCE-COPY src/lib.rs:1 demo::copy matches acme_a::provided "),
        "{stdout}"
    );
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert_eq!(checked.stderr, Vec::<u8>::new());
    let debt = run(&root, &["debt", "--write"]);
    assert_eq!(debt.status.code(), Some(1), "{debt:?}");
    assert!(String::from_utf8_lossy(&debt.stderr).contains("REFERENCE-COPY"));
    assert!(!root.join(raf_census::DEBT_FILE).exists());
    let index = run(&root, &["index"]);
    assert!(index.status.success(), "{index:?}");
    assert!(String::from_utf8_lossy(&index.stdout).contains("ReferenceWitness"));
    let fingerprint = run(&root, &["fingerprint", "demo::copy"]);
    assert!(fingerprint.status.success(), "{fingerprint:?}");
    assert!(
        String::from_utf8_lossy(&fingerprint.stdout).contains("acme_a::provided"),
        "{fingerprint:?}"
    );

    // The neighbour with a changed body is clean and carries no witness.
    write_consumer(&root, &scaled(2, 8), "");
    let clean = run(&root, &["check"]);
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, Vec::<u8>::new());
    let index = run(&root, &["index"]);
    assert!(!String::from_utf8_lossy(&index.stdout).contains("ReferenceWitness"));

    ledger_reference_composes_with_a_pinned_commit(scratch.path(), &root);
    unusable_reference_inputs_refuse_without_a_verdict(scratch.path(), &root);
}

/// The ledger's lock-selected reference and a pinned external commit are both
/// compared in one run.
fn ledger_reference_composes_with_a_pinned_commit(scratch: &Path, root: &Path) {
    let upstream = scratch.join("upstream");
    git_repo::init(&upstream);
    files::write(
        &upstream,
        &[
            (
                "Cargo.toml",
                "[package]\nname='upstream'\nversion='1.0.0'\nedition='2024'\n",
            ),
            (
                "src/lib.rs",
                format!("pub fn provided{}\n", scaled(5, 3)).as_str(),
            ),
        ],
    );
    let commit = git_repo::commit_all(&upstream);
    write_consumer(
        root,
        &scaled(2, 7),
        &format!("pub fn more{}\n", scaled(5, 3)),
    );
    let both = run(
        root,
        &[
            "check",
            "--reference-repo",
            upstream.to_str().unwrap(),
            "--reference-commit",
            &commit,
        ],
    );
    let stdout = String::from_utf8(both.stdout).unwrap();
    assert_eq!(both.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.contains("demo::copy matches acme_a::provided "),
        "{stdout}"
    );
    assert!(
        stdout.contains("demo::more matches upstream::provided "),
        "{stdout}"
    );
}

/// A family that selects nothing and a vanished source each refuse `check`,
/// `index` and `fingerprint` with a message and an empty stdout.
fn unusable_reference_inputs_refuse_without_a_verdict(scratch: &Path, root: &Path) {
    write_reference_ledger(root, "zz-*");
    for words in [&["check"][..], &["index"], &["fingerprint", "demo::copy"]] {
        let refused = run(root, words);
        assert_eq!(refused.status.code(), Some(1), "{words:?}");
        assert_eq!(refused.stdout, Vec::<u8>::new(), "{words:?}");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            stderr.contains("\"zz-*\"") && stderr.contains("matches no locked dependency"),
            "{stderr}"
        );
    }
    write_reference_ledger(root, "acme-*");
    fs::remove_dir_all(scratch.join("vendor")).unwrap();
    for words in [&["check"][..], &["index"]] {
        let refused = run(root, words);
        assert_eq!(refused.status.code(), Some(1), "{words:?}");
        assert_eq!(refused.stdout, Vec::<u8>::new(), "{words:?}");
        assert!(String::from_utf8_lossy(&refused.stderr).starts_with("cargo-census: "));
    }
}

const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// The provider archive `acme-core 1.0.0` whose `provided` body the consumer
/// may copy, with the library manifest `manifest`.
fn acme_core_archive(manifest: &str) -> Vec<u8> {
    raf_fixtures::registry::crate_archive(
        "acme-core",
        "1.0.0",
        &[
            ("Cargo.toml", manifest),
            (
                "src/lib.rs",
                "pub fn provided(input:u32)->u32 { let result=input*2; result+7 }\n",
            ),
        ],
    )
}

const ACME_CORE: &str = "[package]\nname='acme-core'\nversion='1.0.0'\nedition='2024'\n";

/// A consumer `app` below `scratch` whose registry dependency `acme-core` is
/// locked to the checksum of `locked`, with the ledger referencing it, and an
/// empty cache directory. Returns the consumer root and the cache directory.
fn registry_workspace(scratch: &Path, locked: &[u8]) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = scratch.join("ws");
    let cache = scratch.join("cache");
    fs::create_dir_all(&cache).unwrap();
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n",
        raf_fixtures::registry::checksum(locked)
    );
    files::write(
        &root,
        &[
            (
                "Cargo.toml",
                "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='1'\n[workspace]\n",
            ),
            ("Cargo.lock", lock.as_str()),
        ],
    );
    write_reference_ledger(&root, "acme-core");
    (root, cache)
}

fn sealed_words<'a>(command: &[&'a str], cache: &'a Path) -> Vec<&'a str> {
    let mut words = command.to_vec();
    words.extend([
        "--registry-cache",
        cache.to_str().unwrap(),
        "--registry-features",
        "acme-core=",
    ]);
    words
}

/// A copied body of a registry dependency sealed from the lock file fails
/// `check` with exit 1 and one `REFERENCE-COPY` line, `debt --write` refuses it
/// and records nothing, and `index` and `fingerprint` carry the same witness;
/// a changed body is clean. The consumer's build script, tests and examples
/// never run, and the cache is only read.
#[test]
fn registry_reference_ban_fails_check_and_every_command_reads_the_same_sealed_set() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let archive = acme_core_archive(ACME_CORE);
    let (root, cache) = registry_workspace(scratch.path(), &archive);
    raf_fixtures::registry::place(&cache, "index.crates.io-0", "acme-core", "1.0.0", &archive);
    // Each target differs in length, so none is a copy of another.
    let dangerous = |repeats: usize| {
        let write = "std::fs::write(\"consumer-executed\", b\"bad\").unwrap();";
        format!(
            "fn main() {{ {} panic!(\"must not execute\"); }}\n",
            write.repeat(repeats)
        )
    };
    let targets = ["build.rs", "examples/registered.rs", "tests/destructive.rs"];
    for (repeats, path) in targets.iter().enumerate() {
        files::write(&root, &[(*path, dangerous(repeats + 1).as_str())]);
    }
    write_consumer(&root, &scaled(2, 7), "");

    let checked = run(&root, &sealed_words(&["check"], &cache));
    let stdout = String::from_utf8(checked.stdout).unwrap();
    assert_eq!(checked.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.starts_with("REFERENCE-COPY src/lib.rs:1 app::copy matches acme_core::provided "),
        "{stdout}"
    );
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    assert_eq!(checked.stderr, Vec::<u8>::new());
    let debt = run(&root, &sealed_words(&["debt", "--write"], &cache));
    assert_eq!(debt.status.code(), Some(1), "{debt:?}");
    assert!(String::from_utf8_lossy(&debt.stderr).contains("REFERENCE-COPY"));
    assert!(!root.join(raf_census::DEBT_FILE).exists());
    let index = run(&root, &sealed_words(&["index"], &cache));
    assert!(index.status.success(), "{index:?}");
    assert!(String::from_utf8_lossy(&index.stdout).contains("ReferenceWitness"));
    let fingerprint = run(&root, &sealed_words(&["fingerprint", "app::copy"], &cache));
    assert!(fingerprint.status.success(), "{fingerprint:?}");
    assert!(String::from_utf8_lossy(&fingerprint.stdout).contains("acme_core::provided"));
    let ordinary = run(&root, &["check"]);
    assert!(!String::from_utf8_lossy(&ordinary.stdout).contains("acme_core::provided"));

    // The default cache is the Cargo home's, named by the environment.
    let home = scratch.path().join("home");
    fs::create_dir_all(home.join("registry")).unwrap();
    fs::rename(&cache, home.join("registry/cache")).unwrap();
    let from_home = alone(
        Command::new(env!("CARGO_BIN_EXE_cargo-census"))
            .current_dir(&root)
            .env("CARGO_HOME", &home)
            .args(["check", "--registry-features", "acme-core="]),
    );
    assert_eq!(from_home.status.code(), Some(1), "{from_home:?}");
    assert_eq!(from_home.stdout, stdout.as_bytes());
    fs::rename(home.join("registry/cache"), &cache).unwrap();

    // The neighbour with a changed body is clean and carries no witness.
    write_consumer(&root, &scaled(2, 8), "");
    let clean = run(&root, &sealed_words(&["check"], &cache));
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, Vec::<u8>::new());
    let index = run(&root, &sealed_words(&["index"], &cache));
    assert!(!String::from_utf8_lossy(&index.stdout).contains("ReferenceWitness"));

    assert!(!root.join("consumer-executed").exists());
    assert!(!root.join("target").exists());
    for (repeats, path) in targets.iter().enumerate() {
        assert_eq!(
            fs::read_to_string(root.join(path)).unwrap(),
            dangerous(repeats + 1)
        );
    }
}

/// An absent, mismatched or oversized archive and an undeclared, incomplete
/// or misdirected feature declaration each refuse `check` and `index` with
/// exit 1, the named refusal on stderr and an empty stdout, beside the same
/// command with everything in order.
#[test]
fn registry_options_refuse_by_name_without_a_partial_verdict() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let featured = "[package]\nname='acme-core'\nversion='1.0.0'\nedition='2024'\n[features]\ndefault=['std']\nstd=[]\n";
    let archive = acme_core_archive(featured);
    let (root, cache) = registry_workspace(scratch.path(), &archive);
    write_consumer(&root, &scaled(2, 7), "");
    let refuses = |words: &[&str], expected: &[&str]| {
        for command in ["check", "index"] {
            let mut all = vec![command];
            all.extend_from_slice(words);
            let refused = run(&root, &all);
            assert_eq!(refused.status.code(), Some(1), "{all:?}: {refused:?}");
            assert_eq!(refused.stdout, Vec::<u8>::new(), "{all:?}");
            let stderr = String::from_utf8_lossy(&refused.stderr);
            for text in expected {
                assert!(stderr.contains(text), "{text} missing: {stderr}");
            }
        }
    };
    let cache_text = cache.to_str().unwrap();
    let both = ["--registry-cache", cache_text, "--registry-features"];
    let declare = |set: &'static str| {
        let mut words = both.to_vec();
        words.push(set);
        words
    };

    refuses(
        &declare("acme-core=default,std"),
        &["acme-core-1.0.0.crate is not in"],
    );

    let index = cache.join("index.crates.io-0");
    fs::create_dir_all(&index).unwrap();
    fs::write(
        index.join("acme-core-1.0.0.crate"),
        b"not the locked archive",
    )
    .unwrap();
    refuses(
        &declare("acme-core=default,std"),
        &["does not match the lock checksum"],
    );
    // A sparse file one byte over the archive budget: nothing is allocated.
    fs::File::create(index.join("acme-core-1.0.0.crate"))
        .unwrap()
        .set_len((256 << 20) + 1)
        .unwrap();
    refuses(&declare("acme-core=default,std"), &["byte archive budget"]);

    raf_fixtures::registry::place(&cache, "index.crates.io-0", "acme-core", "1.0.0", &archive);
    refuses(&both[..2], &["acme-core 1.0.0", "no declared feature set"]);
    refuses(
        &declare("acme-core=default"),
        &["missing an implied feature"],
    );
    refuses(&declare("acme-core=nope"), &["unknown enabled feature"]);
    refuses(&declare("misspelt="), &["features declared for misspelt"]);
    refuses(&declare("acme-core"), &["expected PACKAGE=FEATURE"]);
    let mut twice = declare("acme-core=default,std");
    twice.extend(["--registry-features", "acme-core="]);
    refuses(&twice, &["name each package once"]);

    // The neighbour with the complete set reads the sealed archive.
    for command in ["check", "index"] {
        let mut words = vec![command];
        words.extend(declare("acme-core=default,std"));
        let accepted = run(&root, &words);
        assert_eq!(
            accepted.status.code(),
            Some(i32::from(command == "check")),
            "{accepted:?}"
        );
    }
}

/// A sealed package is a reference for the ban exactly as a ledger `reference`
/// is: a consumer whose ledger names no reference fails `check` on a copied
/// body with one `REFERENCE-COPY` line when `--seal-registry` names the
/// provider, by exact name or by `prefix-*` family, and is clean on a changed
/// body; the same copy passes without the flag, since nothing is implied.
#[test]
fn a_sealed_registry_selector_is_a_reference_without_a_ledger_entry() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let archive = acme_core_archive(ACME_CORE);
    let (root, cache) = registry_workspace(scratch.path(), &archive);
    raf_fixtures::registry::place(&cache, "index.crates.io-0", "acme-core", "1.0.0", &archive);
    write_ledger(&root, &[]);
    let cache_text = cache.to_str().unwrap();
    let sealed = |selector: &'static str| {
        vec![
            "check",
            "--registry-cache",
            cache_text,
            "--registry-features",
            "acme-core=",
            "--seal-registry",
            selector,
        ]
    };
    write_consumer(&root, &scaled(2, 7), "");
    let unflagged = run(&root, &["check"]);
    assert!(unflagged.status.success(), "{unflagged:?}");
    assert_eq!(unflagged.stdout, Vec::<u8>::new());
    for selector in ["acme-core", "acme-*"] {
        let copied = run(&root, &sealed(selector));
        let stdout = String::from_utf8(copied.stdout).unwrap();
        assert_eq!(copied.status.code(), Some(1), "{selector}: {stdout}");
        assert!(
            stdout
                .starts_with("REFERENCE-COPY src/lib.rs:1 app::copy matches acme_core::provided "),
            "{selector}: {stdout}"
        );
        assert_eq!(stdout.lines().count(), 1, "{selector}: {stdout}");
    }
    write_consumer(&root, &scaled(2, 8), "");
    for selector in ["acme-core", "acme-*"] {
        let changed = run(&root, &sealed(selector));
        assert!(changed.status.success(), "{selector}: {changed:?}");
        assert_eq!(changed.stdout, Vec::<u8>::new(), "{selector}");
    }
}

/// A consumer `app` of the registry package `acme-core` and of the Git package
/// `kit`, both locked, with the registry archive in the cache and a job `number`
/// homed at `app::home`. The consumer copy has a body no provider holds, so the
/// only refusal a command can meet is the one the ledger's `reference` values
/// cause. Returns the consumer root and the cache directory.
fn git_and_registry_workspace(scratch: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    let archive = acme_core_archive(ACME_CORE);
    let (root, cache) = registry_workspace(scratch, &archive);
    raf_fixtures::registry::place(&cache, "index.crates.io-0", "acme-core", "1.0.0", &archive);
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['acme-core','kit']\n[[package]]\nname='acme-core'\nversion='1.0.0'\nsource='{CRATES_IO}'\nchecksum='{}'\n[[package]]\nname='kit'\nversion='1.0.0'\nsource='git+https://example.test/kit?rev={COMMIT}#{COMMIT}'\n",
        raf_fixtures::registry::checksum(&archive)
    );
    files::write(
        &root,
        &[
            (
                "Cargo.toml",
                format!(
                    "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nacme-core='1'\nkit={{git='https://example.test/kit',rev='{COMMIT}'}}\n[workspace]\n"
                )
                .as_str(),
            ),
            ("Cargo.lock", lock.as_str()),
        ],
    );
    write_consumer(&root, &scaled(2, 8), "pub fn home()->u32{7}\n");
    (root, cache)
}

/// The commands that share the registry evaluation with `check` and `index`
/// and the registry debt writer, each with the selector it needs.
const REGISTRY_COMMANDS: [&[&str]; 4] = [
    &["job", "number"],
    &["count", "number"],
    &["fingerprint", "app::copy"],
    &["debt", "--write"],
];

/// A ledger `reference` that names a Git package makes `job`, `count`,
/// `fingerprint` and `debt --write` refuse on the registry route as `check`
/// and `index` do, with exit 1, the refusal on stderr, an empty stdout and no
/// debt record. The same commands over the same lock file succeed when the
/// ledger's references name only the sealed registry package.
#[test]
fn a_git_reference_is_refused_by_job_count_fingerprint_and_debt_beside_a_clean_registry_neighbour()
{
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let (root, cache) = git_and_registry_workspace(scratch.path());

    write_ledger_with(&root, &["acme-core", "kit"], true);
    for command in REGISTRY_COMMANDS {
        let refused = run(&root, &sealed_words(command, &cache));
        assert_eq!(refused.status.code(), Some(1), "{command:?}: {refused:?}");
        assert_eq!(refused.stdout, Vec::<u8>::new(), "{command:?}");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            stderr.contains("package \"kit\" has no admitted local reference source"),
            "{command:?}: {stderr}"
        );
    }
    assert!(!root.join(raf_census::DEBT_FILE).exists());

    // An unenforced job with no copies fails the gate itself, so only the two
    // commands that select a job read a ledger that declares one.
    for command in REGISTRY_COMMANDS {
        write_ledger_with(&root, &["acme-core"], matches!(command[0], "job" | "count"));
        let accepted = run(&root, &sealed_words(command, &cache));
        assert!(accepted.status.success(), "{command:?}: {accepted:?}");
        assert_eq!(accepted.stderr, Vec::<u8>::new(), "{command:?}");
    }
}

/// A `reference` naming a Git dependency is refused by the registry options for
/// `check` and `index`: exit 1, the named refusal on stderr and an empty stdout.
/// The same options over a registry consumer with a changed body pass.
#[test]
fn a_git_reference_is_refused_by_the_registry_options_beside_a_clean_registry_neighbour() {
    let scratch = purrdf_testkit::TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
    let archive = acme_core_archive(ACME_CORE);
    let (root, cache) = registry_workspace(scratch.path(), &archive);
    let commit = "0123456789abcdef0123456789abcdef01234567";
    let lock = format!(
        "version=4\n[[package]]\nname='app'\nversion='1.0.0'\ndependencies=['kit']\n[[package]]\nname='kit'\nversion='1.0.0'\nsource='git+https://example.test/kit?rev={commit}#{commit}'\n"
    );
    let git = scratch.path().join("git");
    files::write(
        &git,
        &[
            (
                "Cargo.toml",
                "[package]\nname='app'\nversion='1.0.0'\nedition='2024'\n[dependencies]\nkit={git='https://example.test/kit'}\n[workspace]\n",
            ),
            ("Cargo.lock", lock.as_str()),
            (
                "src/lib.rs",
                "pub fn copy(input:u32)->u32 { let result=input*2; result+8 }\n",
            ),
        ],
    );
    write_reference_ledger(&git, "kit");
    let cache_text = cache.to_str().unwrap();
    for command in ["check", "index"] {
        let refused = run(&git, &[command, "--registry-cache", cache_text]);
        assert_eq!(refused.status.code(), Some(1), "{command}: {refused:?}");
        assert_eq!(refused.stdout, Vec::<u8>::new(), "{command}");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            stderr.contains("package \"kit\" has no admitted local reference source"),
            "{command}: {stderr}"
        );
    }

    raf_fixtures::registry::place(&cache, "index.crates.io-0", "acme-core", "1.0.0", &archive);
    write_consumer(&root, &scaled(2, 8), "");
    let clean = run(&root, &sealed_words(&["check"], &cache));
    assert!(clean.status.success(), "{clean:?}");
    assert_eq!(clean.stdout, Vec::<u8>::new());
}

/// The module-path requirement is a command-line choice for one run: `check`
/// and `debt --write` refuse a module whose `path` attribute leaves its package
/// only when asked, the same workspace is clean without the option, and the
/// debt record keeps its exact bytes either way.
#[test]
fn module_paths_in_package_are_required_by_the_flag_beside_a_clean_run_without_it() {
    let scratch = fixture();
    let root = scratch.path();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname='demo'\nversion='0.1.0'\nedition='2021'\n[workspace]\nmembers=['other']\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("other/src")).unwrap();
    fs::write(
        root.join("other/Cargo.toml"),
        "[package]\nname='other'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    fs::write(root.join("other/src/lib.rs"), "").unwrap();
    fs::write(root.join("other/src/shared.rs"), "").unwrap();
    let lib = root.join("src/lib.rs");
    let mut text = fs::read_to_string(&lib).unwrap();
    text.push_str("#[path = \"../other/src/shared.rs\"]\nmod shared;\n");
    fs::write(&lib, text).unwrap();

    let debt = root.join("census-debt.ttl");
    let written = run(root, &["debt", "--write"]);
    assert!(written.status.success(), "{written:?}");
    let plain = run(root, &["check"]);
    assert!(plain.status.success(), "{plain:?}");
    let record = fs::read(&debt).unwrap();

    let required = run(root, &["check", "--require-module-paths-in-package"]);
    assert_eq!(required.status.code(), Some(1), "{required:?}");
    let stdout = String::from_utf8_lossy(&required.stdout);
    assert!(
        stdout.contains("module path attribute: src/lib.rs:")
            && stdout.contains("in package `other`"),
        "{stdout}"
    );
    let refused = run(
        root,
        &["debt", "--write", "--require-module-paths-in-package"],
    );
    assert_eq!(refused.status.code(), Some(1), "{refused:?}");
    assert_eq!(fs::read(&debt).unwrap(), record);

    source(root, 2);
    let contained = run(root, &["check", "--require-module-paths-in-package"]);
    assert!(contained.status.success(), "{contained:?}");
    let rewritten = run(
        root,
        &["debt", "--write", "--require-module-paths-in-package"],
    );
    assert!(rewritten.status.success(), "{rewritten:?}");
}
