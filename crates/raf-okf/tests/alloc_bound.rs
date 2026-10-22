// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! A refusal costs memory in proportion to the input, not to what the input
//! would expand to.

use std::fmt::Write as _;

use purrdf_alloc_probe::{CountingAllocator, CurrentThreadWindow};
use raf_okf::{Document, ErrorKind};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// The allowance on top of eight times the input: the error value, its key
/// name and the formatted message hold a few hundred bytes. 16 KiB leaves
/// room and is far below what an expanded alias tree would take.
const FIXED_ALLOWANCE: usize = 16 * 1024;

/// Build the text in one allocation, so the peak before the parse is the
/// input itself.
fn billion_laughs(levels: usize) -> String {
    let mut text = String::with_capacity(levels * 40 + 64);
    text.push_str("---\ntype: Capability\nlol0: &lol0 [\"lol\", \"lol\", \"lol\", \"lol\"]\n");
    for level in 1..levels {
        let previous = level - 1;
        writeln!(text, "lol{level}: &lol{level} [*lol{previous}, *lol{previous}, *lol{previous}, *lol{previous}]").expect("writing to a String cannot fail");
    }
    text.push_str("---\n");
    text
}

/// Each synchronous parse is measured on its calling thread, excluding the
/// test harness and any sibling tests.
#[test]
fn refused_documents_stay_within_eight_times_their_size() {
    billion_laughs_fails_fast_within_the_bound();
    large_refused_document_stays_within_the_bound();
}

fn billion_laughs_fails_fast_within_the_bound() {
    let text = billion_laughs(9);
    let window = CurrentThreadWindow::open();
    let result = Document::parse(&text);
    let growth = usize::try_from(window.close().peak_working_bytes)
        .expect("the parse peak is nonnegative and fits usize");
    assert!(
        growth > 0,
        "the native probe observes the refusal allocation"
    );
    let error = result.expect_err("anchors are refused");
    assert_eq!(
        error.kind(),
        &ErrorKind::Anchor {
            name: "lol0".to_string()
        }
    );
    assert_eq!(error.line(), 3);
    assert!(
        growth <= 8 * text.len() + FIXED_ALLOWANCE,
        "the parse peaked {growth} bytes above its start for a {} byte input",
        text.len()
    );
}

fn large_refused_document_stays_within_the_bound() {
    // Many keys, then an alias on the last line: the parse reads every line
    // before the refusal, so memory follows the entries kept so far.
    let mut text = String::with_capacity(1024 * 1024);
    text.push_str("---\n");
    let mut index = 0;
    while text.len() < 900 * 1024 {
        writeln!(text, "key{index}: \"value number {index} with some text\"")
            .expect("writing to a String cannot fail");
        index += 1;
    }
    text.push_str("alias: *key0\n---\n");
    let window = CurrentThreadWindow::open();
    let result = Document::parse(&text);
    let growth = usize::try_from(window.close().peak_working_bytes)
        .expect("the parse peak is nonnegative and fits usize");
    assert!(
        growth > 0,
        "the native probe observes the refusal allocation"
    );
    let error = result.expect_err("the alias is refused");
    assert_eq!(
        error.kind(),
        &ErrorKind::Alias {
            name: "key0".to_string()
        }
    );
    assert!(
        growth <= 8 * text.len() + FIXED_ALLOWANCE,
        "the parse peaked {growth} bytes above its start for a {} byte input",
        text.len()
    );
}
