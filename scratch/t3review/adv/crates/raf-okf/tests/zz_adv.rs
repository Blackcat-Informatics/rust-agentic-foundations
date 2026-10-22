// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Review probes of the tested-by marker (scratch only).

use raf_okf::Body;

fn show(label: &str, text: &str) {
    match Body::parse(text) {
        Ok(body) => {
            let markers: Vec<_> = body
                .tested_by()
                .iter()
                .map(|m| {
                    (
                        m.line(),
                        m.paths().to_vec(),
                        text[m.statement().bytes.clone()].to_string(),
                        m.section().map(str::to_string),
                    )
                })
                .collect();
            println!("PROBE {label}: OK claims={} markers={markers:?}", body.claims().len());
        }
        Err(error) => println!("PROBE {label}: ERR {:?} line {}", error.kind(), error.line()),
    }
}

#[test]
fn probes() {
    let m = "<!-- raf:tested-by demo::a -->";
    show("crlf", &format!("## Contract\r\n\r\nA statement.\r\n{m}\r\n"));
    show("crlf-two-paths", "## Contract\r\n\r\nA statement.\r\n<!-- raf:tested-by demo::a,demo::b -->\r\n");
    show("tab-after-word", "A statement.\n<!-- raf:tested-by\tdemo::a -->\n");
    show("tab-before-close", "A statement.\n<!-- raf:tested-by demo::a\t-->\n");
    show("tab-indent", &format!("A statement.\n\t{m}\n"));
    show("trailing-tab", &format!("A statement.\n{m}\t\n"));
    show("unicode-path", "A statement.\n<!-- raf:tested-by demo::caf\u{e9} -->\n");
    show("unicode-ident-start", "A statement.\n<!-- raf:tested-by \u{e9}demo::a -->\n");
    show("two-markers-one-line", &format!("A statement.\n{m} {m}\n"));
    show("two-markers-stacked", &format!("A statement.\n{m}\n{m}\n"));
    show("marker-eof-no-newline", &format!("A statement.\n{m}"));
    show("marker-eof-crlf-no-newline", &format!("A statement.\r\n{m}"));
    show("blockquote", &format!("> A quoted statement.\n> {m}\n"));
    show("blockquote-bare-marker", &format!("> A quoted statement.\n{m}\n"));
    show("list-item", &format!("- An item.\n  {m}\n"));
    show("nested-list-item", &format!("- Top.\n  - Nested item.\n    {m}\n"));
    show("list-item-after-blank", &format!("- An item.\n\n  {m}\n"));
    show("table-cell", &format!("| a | {m} |\n| --- | --- |\n"));
    show("after-table", &format!("| a | b |\n| --- | --- |\n| c | d |\n{m}\n"));
    show("inline-code-only-line", &format!("`code`\n{m}\n"));
    show("setext-heading", &format!("A heading\n---------\n{m}\n"));
    show("h5-heading", &format!("##### Five\n{m}\n"));
    show("fence-tilde", &format!("~~~\n{m}\n~~~\n"));
    show("fence-unclosed-inside-list", &format!("- x\n  ```\n  {m}\n  ```\n"));
    show("marker-then-claim-marker", &format!("Statement.\n{m}\n<!-- raf:claim evidence=x -->\n"));
    show("claim-then-marker", &format!("Statement.\n<!-- raf:claim evidence=x -->\n{m}\n"));
    show("marker-then-claim-after-blank", &format!("Statement.\n{m}\n\n<!-- raf:claim evidence=x -->\n"));
    show("upper-raf-only", "Statement.\n<!-- RAF:tested-by demo::a -->\n");
    show("generic-with-space", "Statement.\n<!-- raf:tested-by demo::Pair<A, B>::get -->\n");
    show("generic-nested", "Statement.\n<!-- raf:tested-by demo::Pair<Vec<A>>::get -->\n");
    show("lifetime-generic", "Statement.\n<!-- raf:tested-by demo::Pair<'a>::get -->\n");
    show("path-single-segment", "Statement.\n<!-- raf:tested-by demo -->\n");
    show("leading-colons", "Statement.\n<!-- raf:tested-by ::demo::a -->\n");
    show("html-block-above", &format!("<div>\n{m}\n"));
    show("comment-same-line-as-statement-above", &format!("Statement. <!-- note -->\n{m}\n"));
    show("visible-text-after-comment-above", &format!("<!-- note --> Statement.\n{m}\n"));
}
