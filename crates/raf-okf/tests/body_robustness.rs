// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Arbitrary token soup never panics, and the spans of what parses lie in
//! the text.

use raf_okf::{Body, Document, Index};

const TOKENS: [&str; 40] = [
    "\n",
    "\n",
    "\r\n",
    " ",
    "  ",
    "    ",
    "# ",
    "## ",
    "###### ",
    "#",
    "> ",
    ">",
    "- ",
    "* ",
    "1. ",
    "```",
    "~~~",
    "````",
    "`",
    "``",
    "[",
    "]",
    "(",
    ")",
    "![",
    "](",
    "\\",
    "<!--",
    "-->",
    "<a href=",
    "src=\"",
    "\"",
    "'",
    "> [!PROPOSED change:x]",
    "<!-- raf:generated:k -->",
    "<!-- raf:generated:end -->",
    "text",
    "é",
    "---",
    "\t",
];

fn next(state: &mut u64) -> usize {
    let value = purrdf_hash::mix::lcg64_next(state, purrdf_hash::mix::LCG64_MMIX_INCREMENT);
    usize::try_from(value >> 33).expect("fits")
}

#[test]
fn random_token_soup_never_panics_and_parsed_spans_lie_in_the_text() {
    let mut state = 0x5eed_u64;
    for _ in 0..20_000 {
        let length = next(&mut state) % 60;
        let text: String = (0..length)
            .map(|_| TOKENS[next(&mut state) % TOKENS.len()])
            .collect();
        for candidate in [text.clone(), format!("---\na: b\n---\n{text}")] {
            if let Ok(body) = Body::parse(&candidate) {
                let whole = body.span().bytes.clone();
                assert_eq!(&candidate[whole], body.text());
                for section in body.sections() {
                    let span = section.content();
                    assert!(span.bytes.end <= candidate.len(), "{candidate:?}");
                    assert!(candidate.is_char_boundary(span.bytes.start));
                    assert!(candidate.is_char_boundary(span.bytes.end));
                }
                for block in body.generated_blocks() {
                    let inner = &candidate[block.content().bytes.clone()];
                    let outer = &candidate[block.outer().bytes.clone()];
                    assert!(outer.contains(inner));
                }
                // Replacing every section content with itself is the identity.
                let edits: Vec<_> = body
                    .sections()
                    .iter()
                    .map(|s| (s.content().clone(), &candidate[s.content().bytes.clone()]))
                    .collect();
                let mut again = body.clone();
                again.replace_spans(&edits).expect("identity edit");
                assert_eq!(again, body);
            }
            let _ = Document::parse(&candidate);
            let _ = Index::parse(&candidate);
        }
    }
}
