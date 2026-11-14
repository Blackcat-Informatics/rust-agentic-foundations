// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The peak storage of a parsed syntax tree stays within the bound that is
//! computed before the parse, over a reference table of shapes. Allocations are
//! counted, never timed.

use purrdf_alloc_probe::{CountingAllocator, CurrentThreadWindow};
use raf_source::{RelPath, SYNTAX_NODE_SIZES, SyntaxBound, prepare_source, syntax_held};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    /// Many small nodes; the per-node term governs.
    Node,
    /// Few nodes over many bytes; the per-byte term governs.
    Byte,
}

struct Shape {
    name: String,
    class: Class,
    text: String,
}

struct Row {
    name: String,
    class: Class,
    nodes: usize,
    bytes: usize,
    peak: usize,
}

/// Every native token-tree node, groups included, counted with an explicit
/// stack of the iterators that are still open.
fn native_nodes(tokens: proc_macro2::TokenStream) -> usize {
    let mut count = 0;
    let mut open = vec![tokens.into_iter()];
    while let Some(mut level) = open.pop() {
        while let Some(token) = level.next() {
            count += 1;
            if let proc_macro2::TokenTree::Group(group) = token {
                open.push(level);
                open.push(group.stream().into_iter());
                break;
            }
        }
    }
    count
}

/// Peak working bytes of the syntax parse alone, with the exact native node
/// count read before it. The lexer, the token stream and the text are already
/// live when the window opens, so they are not part of the peak.
fn measure(name: &str, class: Class, text: &str) -> Row {
    let path = RelPath::new("src/shape.rs").expect("fixture path");
    let (nodes, peak) = prepare_source(&path, text)
        .unwrap_or_else(|error| panic!("{name} is not guard-admitted: {error:?}"))
        .with_lexed(|lexed| {
            let nodes = native_nodes(lexed.tokens().clone());
            let window = CurrentThreadWindow::open();
            let peak = lexed
                .parse(|_| window.close().peak_working_bytes)
                .unwrap_or_else(|error| panic!("{name} is not valid Rust: {error:?}"));
            (nodes, usize::try_from(peak).expect("a nonnegative peak"))
        })
        .expect("the shape lexes");
    Row {
        name: name.to_owned(),
        class,
        nodes,
        bytes: text.len(),
        peak,
    }
}

fn node_shape(name: &str, n: usize, text: String) -> Shape {
    Shape {
        name: format!("{name} x{n}"),
        class: Class::Node,
        text,
    }
}

fn shapes() -> Vec<Shape> {
    let mut all = Vec::new();
    for k in [10_u32, 14, 17] {
        for n in [(1_usize << k) + 1, 1_usize << k] {
            all.push(node_shape(
                "fn body of empty statements",
                n,
                format!("fn f() {{ {} }}", ";".repeat(n)),
            ));
        }
    }
    let n = 4096;
    all.push(node_shape("unit structs", n, "struct S;\n".repeat(n)));
    all.push(node_shape(
        "impl body of macro items",
        256,
        format!("impl X {{ {} }}", "m!{} ".repeat(256)),
    ));
    all.push(node_shape(
        "trait body of macro items",
        256,
        format!("trait T {{ {} }}", "m!{} ".repeat(256)),
    ));
    all.push(node_shape(
        "extern block of macro items",
        256,
        format!("extern \"C\" {{ {} }}", "m!{} ".repeat(256)),
    ));
    all.push(node_shape(
        "match arms",
        n,
        format!("fn f(x: u8) {{ match x {{ {} }} }}", "_ => 0,".repeat(n)),
    ));
    all.push(node_shape(
        "tuple of one-token elements",
        n,
        format!("fn f() {{ ({}); }}", "a,".repeat(n)),
    ));
    all.push(node_shape(
        "generic comparisons",
        n,
        format!("fn f() {{ {} }}", "a<b; ".repeat(n)),
    ));
    all.push(node_shape(
        "nested generic arguments",
        60,
        format!("type T = {}u8{};", "A<".repeat(60), ">".repeat(60)),
    ));
    all.push(node_shape(
        "attribute run",
        n,
        format!("{}fn f() {{}}", "#[a] ".repeat(n)),
    ));
    all.push(node_shape(
        "macro statements",
        256,
        format!("fn f() {{ {} }}", "m!{} ".repeat(256)),
    ));
    all.push(node_shape(
        "numeric field accesses",
        n,
        format!("fn f() {{ {} }}", "x.1.2; ".repeat(n)),
    ));
    all.push(node_shape(
        "doc comment items",
        n,
        "/// x\nstruct S;\n".repeat(n),
    ));
    let mib = 1 << 20;
    all.push(Shape {
        name: "one 1 MiB string literal".to_owned(),
        class: Class::Byte,
        text: format!("const S: &str = \"{}\";", "x".repeat(mib)),
    });
    all.push(Shape {
        name: "one 1 MiB identifier".to_owned(),
        class: Class::Byte,
        text: format!("fn {}() {{}}", "i".repeat(mib)),
    });
    all.push(Shape {
        name: "one 1 MiB doc comment".to_owned(),
        class: Class::Byte,
        text: format!("///{}\nfn f() {{}}", "d".repeat(mib)),
    });
    all.push(Shape {
        name: "one 1 MiB raw string literal".to_owned(),
        class: Class::Byte,
        text: format!("const S: &str = r#\"{}\"#;", "r".repeat(mib)),
    });
    all.push(Shape {
        name: "one 1 MiB byte string literal".to_owned(),
        class: Class::Byte,
        text: format!("const S: &[u8] = b\"{}\";", "b".repeat(mib)),
    });
    all
}

fn table(rows: &[Row]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for row in rows {
        let _ = writeln!(
            out,
            "{:<44} nodes={:>7} bytes={:>8} peak={:>9} per_node={:>5} per_byte={:>5}",
            row.name,
            row.nodes,
            row.bytes,
            row.peak,
            row.peak / row.nodes.max(1),
            row.peak / row.bytes.max(1),
        );
    }
    out
}

fn measured() -> Vec<Row> {
    shapes()
        .iter()
        .map(|shape| measure(&shape.name, shape.class, &shape.text))
        .collect()
}

/// INVARIANT: for every shape of the reference table, the peak storage of the
/// syntax parse is at most `nodes * per_node + bytes * per_byte`, and the
/// worst shape of each class reaches at least 90 percent of the bound, so the
/// constants are tight and not vacuous.
#[test]
fn syntax_tree_peak_storage_stays_within_the_per_node_and_per_byte_bound() {
    let rows = measured();
    eprintln!("{}", table(&rows));
    let bound = SyntaxBound::MEASURED;
    for class in [Class::Node, Class::Byte] {
        let mut highest = 0.0_f64;
        for row in rows.iter().filter(|row| row.class == class) {
            let limit = bound.bytes(row.nodes, row.bytes).expect("the bound fits");
            assert!(
                row.peak <= limit,
                "{} peaked at {} over the bound {limit}\n{}",
                row.name,
                row.peak,
                table(&rows)
            );
            #[allow(clippy::cast_precision_loss, reason = "a ratio of two small counts")]
            let ratio = row.peak as f64 / limit as f64;
            highest = highest.max(ratio);
        }
        assert!(
            highest >= 0.90,
            "the worst {class:?} shape reaches {highest:.3} of the bound\n{}",
            table(&rows)
        );
    }
}

/// INVARIANT: the per-node term is at least two elements of every counted node
/// enum, shared over the fewest nodes one element takes, because a vector that
/// doubles holds up to twice its retained elements at the doubling (the table
/// shows exactly this for statements); a pin change that enlarges these enums
/// fails here before any input does.
#[test]
fn syntax_node_bound_is_at_least_two_vec_elements_of_every_counted_enum() {
    for (name, size, minimum) in SYNTAX_NODE_SIZES {
        let needed = (2 * size).div_ceil(minimum);
        assert!(
            SyntaxBound::MEASURED.per_node >= needed,
            "per node {} under {needed} for {name}",
            SyntaxBound::MEASURED.per_node
        );
    }
}

/// INVARIANT: the bound also covers every Rust source this repository tracks.
#[test]
fn syntax_tree_storage_bound_covers_every_source_of_this_repository() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    collect(&root.join("crates"), &mut files);
    collect(&root.join("tests"), &mut files);
    assert!(files.len() > 100, "the walk found the repository sources");
    let bound = SyntaxBound::MEASURED;
    for file in files {
        let text = std::fs::read_to_string(&file).expect("tracked source reads");
        let row = measure(&file.display().to_string(), Class::Node, &text);
        let limit = bound.bytes(row.nodes, row.bytes).expect("the bound fits");
        assert!(
            row.peak <= limit,
            "{} peaked at {} over the bound {limit}",
            row.name,
            row.peak
        );
    }
}

fn collect(directory: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("directory reads") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "target") {
                collect(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

/// INVARIANT: the retained-tree estimate of the nine-enum walk is available to
/// callers and names the dominant kind of a statement-dense body.
#[test]
fn syntax_held_names_the_dominant_kind_of_a_statement_dense_body() {
    let path = RelPath::new("src/shape.rs").expect("fixture path");
    let text = format!("fn f() {{ {} }}", ";".repeat(64));
    let (kind, count) =
        raf_source::parse_source(&path, &text, |tree| syntax_held(tree).dominant()).expect("valid");
    assert_eq!((kind, count), ("Stmt", 64));
}
