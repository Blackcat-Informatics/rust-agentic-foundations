// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The text of a module file as `Workspace::load` meets it: a file that is not
//! UTF-8 is refused naming it, and a byte-order mark or CRLF line endings leave
//! every line number and byte span exactly where they are in the text.

mod support;

use std::fs;

use raf_source::{
    Budget, DiskTree, Item, LoadError, LoadErrorKind, LoadOptions, MemoryTree, RelPath, Tree,
    TreeError, Workspace,
};

const DEMO: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
const BOM: &str = "\u{feff}";

fn load(lib: &str) -> Result<Workspace, LoadError> {
    let tree: MemoryTree = [("Cargo.toml", DEMO), ("src/lib.rs", lib)]
        .into_iter()
        .collect();
    Workspace::load(&tree, &["."])
}

fn item<'w>(ws: &'w Workspace, path: &str) -> &'w Item {
    let resolved = ws.resolve(path).unwrap_or_else(|error| panic!("{error}"));
    ws.item(resolved.sites[0]).expect("the item")
}

/// The text a span covers in the file the workspace holds.
fn covered<'w>(ws: &'w Workspace, item: &Item) -> &'w str {
    let text = ws.source(&item.file).expect("the file's text");
    &text[item.span.start_byte..item.span.end_byte]
}

/// `a` on line 2, `b` on lines 4 to 5, whatever the line endings, with a
/// non-ASCII character before `b` to move bytes from columns.
fn body(nl: &str) -> String {
    format!("// caf\u{e9}{nl}pub fn a() {{}}{nl}{nl}pub fn b() {{{nl}}}{nl}")
}

fn check_lines(ws: &Workspace) {
    let (a, b) = (item(ws, "demo::a"), item(ws, "demo::b"));
    assert_eq!((a.name_line, a.span.start_line, a.span.end_line), (2, 2, 2));
    assert_eq!((b.name_line, b.span.start_line, b.span.end_line), (4, 4, 5));
    assert_eq!(covered(ws, a), "pub fn a() {}");
    assert!(covered(ws, b).starts_with("pub fn b() {") && covered(ws, b).ends_with('}'));
}

#[test]
fn crlf_line_endings_give_the_same_lines_and_spans_as_lf() {
    for newline in ["\n", "\r\n"] {
        let ws = load(&body(newline)).expect("loads");
        check_lines(&ws);
    }
}

#[test]
fn a_byte_order_mark_leaves_the_lines_of_the_text_where_they_are() {
    for newline in ["\n", "\r\n"] {
        let ws = load(&format!("{BOM}{}", body(newline))).expect("loads with a BOM");
        check_lines(&ws);
    }
}

#[test]
fn a_byte_order_mark_on_the_first_line_starts_the_span_of_its_item_after_the_mark() {
    let ws = load(&format!("{BOM}pub fn first() {{}} pub fn second() {{}}\n")).expect("loads");
    let first = item(&ws, "demo::first");
    assert_eq!(first.span.start_line, 1);
    assert_eq!(first.span.start_byte, BOM.len());
    assert_eq!(covered(&ws, first), "pub fn first() {}");
    // A second item on the same line checks the column arithmetic after the mark.
    let second = item(&ws, "demo::second");
    assert_eq!(second.span.start_line, 1);
    assert_eq!(
        second.span.start_byte,
        BOM.len() + "pub fn first() {} ".len()
    );
    assert_eq!(covered(&ws, second), "pub fn second() {}");
    let at = ws.item_at(&first.file, BOM.len());
    assert_eq!(at.len(), 1);
    assert_eq!(at[0].name.as_ref(), "first");
}

#[test]
fn a_syntax_error_in_a_crlf_file_or_after_a_byte_order_mark_names_its_own_line() {
    for text in [
        "fn ok() {}\n\nfn 1 {}\n",
        "fn ok() {}\r\n\r\nfn 1 {}\r\n",
        &format!("{BOM}fn ok() {{}}\n\nfn 1 {{}}\n"),
        &format!("{BOM}fn ok() {{}}\r\n\r\nfn 1 {{}}\r\n"),
    ] {
        let error = load(text).expect_err("syntax error");
        assert_eq!(error.line, Some(3), "{text:?}: {error}");
        assert!(
            matches!(error.kind, LoadErrorKind::Syntax { .. }),
            "{error}"
        );
        assert_eq!(error.file.as_str(), "src/lib.rs");
    }
}

#[test]
fn a_budget_refusal_in_a_crlf_file_names_the_same_line_as_in_an_lf_file() {
    // The delimiter budget is refused at the 129th open parenthesis, on line 4.
    let nested = format!("{}{}", "(".repeat(129), ")".repeat(129));
    for nl in ["\n", "\r\n"] {
        let text = format!("fn a() {{}}{nl}{nl}{nl}const C: () = {nested};{nl}");
        let error = load(&text).expect_err("over the budget");
        assert!(
            matches!(
                error.kind,
                LoadErrorKind::Budget {
                    budget: Budget::Delimiters,
                    ..
                }
            ),
            "{error}"
        );
        assert_eq!(error.line, Some(4), "{nl:?}: {error}");
    }
}

#[test]
fn a_module_file_that_is_not_utf8_is_refused_on_disk_naming_the_file() {
    let scratch = support::Scratch::new();
    scratch.dir("root/src");
    scratch.file("root/Cargo.toml", DEMO);
    scratch.file("root/src/lib.rs", "pub mod child;\n");
    // A lone continuation byte is not UTF-8, in a comment that would parse.
    let mut bytes = b"// caf".to_vec();
    bytes.extend_from_slice(&[0xe9, b'\n']);
    bytes.extend_from_slice(b"pub fn f() {}\n");
    fs::write(scratch.path("root/src/child.rs"), &bytes).expect("write");

    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let direct = tree
        .read(&RelPath::new("src/child.rs").expect("path"), 1 << 20)
        .expect_err("not UTF-8");
    assert!(matches!(direct, TreeError::NotUtf8 { .. }), "{direct:?}");

    let error = Workspace::load_with(&tree, &["."], &LoadOptions::default()).expect_err("refused");
    assert!(
        matches!(&error.kind, LoadErrorKind::Tree(inner) if matches!(**inner, TreeError::NotUtf8 { .. })),
        "{error}"
    );
    assert!(error.to_string().contains("src/child.rs"), "{error}");
    assert_eq!(error.line, Some(1), "the line of `mod child;`: {error}");

    // The neighbour: the same file in UTF-8 loads.
    fs::write(
        scratch.path("root/src/child.rs"),
        "// caf\u{e9}\npub fn f() {}\n",
    )
    .expect("write");
    let tree = DiskTree::new(scratch.path("root")).expect("tree");
    let ws = Workspace::load(&tree, &["."]).expect("UTF-8 loads");
    assert_eq!(item(&ws, "demo::child::f").name_line, 2);
}
