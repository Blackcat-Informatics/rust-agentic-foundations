// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The story of the first scenario: one capability, `streaming-parser`, over
//! the crate `chunk-parser`, redesigned four times.
//!
//! The capability starts delivered (version 0) and passes through four
//! changes. Each change amends a decision, in the manner of an architecture
//! decision record that gains an "Amended <date>" paragraph: the amendment is
//! written as a PROPOSED block while the change is open, and the current
//! account is rewritten when the change completes, so the amendments are never
//! on the reading path.
//!
//! | Change | Decision it moves |
//! | --- | --- |
//! | `chunked-input` | decision 1: the input may arrive in chunks |
//! | `strict-errors` | decision 2 is reversed: a malformed line is an error |
//! | `incremental-output` | decision 3 is made: entries are returned as their line completes; `push` and `finish` are renamed |
//! | `line-limit` | decision 4 is made, amended inside the change, and the one-shot function is removed |

use super::story::Bindings;
use super::streaming_code::{
    V0_EXAMPLES_DUMP, V0_SRC_LIB, V0_TESTS_PARSER, V1_EXAMPLES_CHUNKS, V1_EXAMPLES_DUMP,
    V1_SRC_LIB, V1_TESTS_PARSER, V2_EXAMPLES_CHUNKS, V2_SRC_LIB, V2_TESTS_PARSER,
    V3_EXAMPLES_STREAM, V3_SRC_LIB, V3_TESTS_PARSER, V4_EXAMPLES_LIMITS, V4_EXAMPLES_STREAM,
    V4_SRC_LIB, V4_TESTS_PARSER,
};

/// The package of the crate.
pub(crate) const PACKAGE: &str = "chunk-parser";

/// The slug of the capability.
pub(crate) const SLUG: &str = "streaming-parser";

/// The id of the capability.
pub(crate) const ID: &str = "cap:streaming-parser";

/// The path of the capability document.
pub(crate) const DOCUMENT: &str = "docs/capabilities/streaming-parser.md";

/// The title of the capability.
pub(crate) const TITLE: &str = "Streaming parser";

/// The description of the capability.
pub(crate) const DESCRIPTION: &str = "Reads key=value text into entries, whole or in chunks.";

/// The path of the architecture page the capability links to.
pub(crate) const ARCHITECTURE: &str = "docs/architecture/framing.md";

/// The architecture page.
pub(crate) const FRAMING: &str = r#"---
# SPDX-FileCopyrightText: 2026 Example Org <dev@example.org>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:framing
title: "Framing"
description: "How text becomes entries and entries become records."
---

# Framing

## Line framing

Text is cut into lines first and into entries second. A stage that works on entries
never looks at raw text, and a stage that works on raw text never builds an entry.
"#;

/// The README, which links to the capability.
pub(crate) fn readme() -> String {
    format!(
        "{}\n# chunk-parser\n\nA parser for `key=value` text. Read the\n\
         [streaming parser](docs/capabilities/streaming-parser.md#contract) for what it does and\n\
         the [capability catalogue](docs/capabilities/index.md) for the rest.\n",
        super::project::spdx_comments()
    )
}

/// What the bindings of each version are.
pub(crate) fn bindings(version: usize) -> Bindings {
    const V0: Bindings = Bindings {
        code: &["chunk_parser::parse", "chunk_parser::Entry"],
        tests: &[
            "chunk_parser::test_crate::parser::a_whole_text_is_parsed_in_order",
            "chunk_parser::test_crate::parser::a_line_that_is_not_a_pair_is_skipped",
        ],
        examples: &["chunk_parser::example_crate::dump"],
    };
    const V1: Bindings = Bindings {
        code: &[
            "chunk_parser::Entry",
            "chunk_parser::Parser",
            "chunk_parser::Parser::new",
            "chunk_parser::Parser::push",
            "chunk_parser::Parser::finish",
            "chunk_parser::parse",
        ],
        tests: &[
            "chunk_parser::test_crate::parser::a_whole_text_is_parsed_in_order",
            "chunk_parser::test_crate::parser::a_line_that_is_not_a_pair_is_skipped",
            "chunk_parser::test_crate::parser::chunks_split_anywhere_give_the_same_entries",
            "chunk_parser::test_crate::parser::a_last_line_without_a_line_ending_is_kept",
        ],
        examples: &[
            "chunk_parser::example_crate::dump",
            "chunk_parser::example_crate::chunks",
        ],
    };
    const V2: Bindings = Bindings {
        code: &[
            "chunk_parser::Entry",
            "chunk_parser::ParseError",
            "chunk_parser::Parser",
            "chunk_parser::Parser::new",
            "chunk_parser::Parser::push",
            "chunk_parser::Parser::finish",
            "chunk_parser::parse",
        ],
        tests: &[
            "chunk_parser::test_crate::parser::a_whole_text_is_parsed_in_order",
            "chunk_parser::test_crate::parser::chunks_split_anywhere_give_the_same_entries",
            "chunk_parser::test_crate::parser::a_last_line_without_a_line_ending_is_kept",
            "chunk_parser::test_crate::parser::a_blank_line_is_not_an_entry_and_not_an_error",
            "chunk_parser::test_crate::parser::a_line_that_is_not_a_pair_is_an_error_that_names_its_line",
            "chunk_parser::test_crate::parser::the_first_error_ends_the_parse",
        ],
        examples: &["chunk_parser::example_crate::chunks"],
    };
    const V3: Bindings = Bindings {
        code: &[
            "chunk_parser::Entry",
            "chunk_parser::ParseError",
            "chunk_parser::Parser",
            "chunk_parser::Parser::new",
            "chunk_parser::Parser::feed",
            "chunk_parser::Parser::close",
            "chunk_parser::parse",
        ],
        tests: &[
            "chunk_parser::test_crate::parser::a_whole_text_is_parsed_in_order",
            "chunk_parser::test_crate::parser::chunks_split_anywhere_give_the_same_entries",
            "chunk_parser::test_crate::parser::an_entry_is_returned_by_the_chunk_that_completes_its_line",
            "chunk_parser::test_crate::parser::a_blank_line_is_not_an_entry_and_not_an_error",
            "chunk_parser::test_crate::parser::a_line_that_is_not_a_pair_is_an_error_that_names_its_line",
            "chunk_parser::test_crate::parser::the_first_error_ends_the_parse",
        ],
        examples: &[
            "chunk_parser::example_crate::stream",
            "chunk_parser::Parser::feed",
        ],
    };
    const V4: Bindings = Bindings {
        code: &[
            "chunk_parser::DEFAULT_MAX_LINE",
            "chunk_parser::Entry",
            "chunk_parser::ParseError",
            "chunk_parser::Parser",
            "chunk_parser::Parser::new",
            "chunk_parser::Parser::with_max_line",
            "chunk_parser::Parser::feed",
            "chunk_parser::Parser::close",
            "chunk_parser::Reason",
        ],
        tests: &[
            "chunk_parser::test_crate::parser::a_whole_text_is_one_chunk",
            "chunk_parser::test_crate::parser::chunks_split_anywhere_give_the_same_entries",
            "chunk_parser::test_crate::parser::an_entry_is_returned_by_the_chunk_that_completes_its_line",
            "chunk_parser::test_crate::parser::a_blank_line_is_not_an_entry_and_not_an_error",
            "chunk_parser::test_crate::parser::a_line_that_is_not_a_pair_is_an_error_that_names_its_line",
            "chunk_parser::test_crate::parser::the_first_error_ends_the_parse",
            "chunk_parser::test_crate::parser::a_line_over_the_limit_is_an_error_before_its_line_ending_arrives",
            "chunk_parser::test_crate::parser::the_default_limit_is_sixty_five_thousand_five_hundred_thirty_six_bytes",
        ],
        examples: &[
            "chunk_parser::example_crate::stream",
            "chunk_parser::example_crate::limits",
            "chunk_parser::Parser::feed",
        ],
    };
    [V0, V1, V2, V3, V4][version]
}

/// The files of the crate at a version, by path.
pub(crate) fn files(version: usize) -> Vec<(&'static str, &'static str)> {
    match version {
        0 => vec![
            ("src/lib.rs", V0_SRC_LIB),
            ("tests/parser.rs", V0_TESTS_PARSER),
            ("examples/dump.rs", V0_EXAMPLES_DUMP),
        ],
        1 => vec![
            ("src/lib.rs", V1_SRC_LIB),
            ("tests/parser.rs", V1_TESTS_PARSER),
            ("examples/dump.rs", V1_EXAMPLES_DUMP),
            ("examples/chunks.rs", V1_EXAMPLES_CHUNKS),
        ],
        2 => vec![
            ("src/lib.rs", V2_SRC_LIB),
            ("tests/parser.rs", V2_TESTS_PARSER),
            ("examples/chunks.rs", V2_EXAMPLES_CHUNKS),
        ],
        3 => vec![
            ("src/lib.rs", V3_SRC_LIB),
            ("tests/parser.rs", V3_TESTS_PARSER),
            ("examples/stream.rs", V3_EXAMPLES_STREAM),
        ],
        4 => vec![
            ("src/lib.rs", V4_SRC_LIB),
            ("tests/parser.rs", V4_TESTS_PARSER),
            ("examples/stream.rs", V4_EXAMPLES_STREAM),
            ("examples/limits.rs", V4_EXAMPLES_LIMITS),
        ],
        _ => panic!("there is no version {version}"),
    }
}

const PURPOSE: &str = r"The streaming parser reads `key=value` text into entries. It is the first stage of
[line framing](../architecture/framing.md#line-framing): every later stage works on
entries and never on raw text.";

const VERIFICATION: &str = r"Each bound test checks one claim of the [Contract](#contract) or of the
[Constraints](#constraints), and each bound example is built and run. The table lists
them.";

/// The text of the five sections that change, by version: contract, usage,
/// examples, constraints, explanation.
const CHANGING: [[&str; 5]; 5] = [
    // Version 0.
    [
        r"`parse(input)` reads a whole text in one call and returns its entries in the order
they appear.

A line of the form `key=value` is one entry. The key is the text before the first `=`
and is not empty; the value is the rest of the line and may be empty.

Lines that are not of that form are skipped.",
        r#"Call `chunk_parser::parse` with the complete text:

```rust
let entries = chunk_parser::parse("a=1\nb=2\n");
```"#,
        r"The example [dump](../../examples/dump.rs) parses a fixed text and prints one
`key=value` pair per line.",
        r"- The whole text is in memory before `parse` is called.
- Lines are split at `\n`; a `\r` before it is removed.
- Duplicate keys give separate entries, in order.",
        r"Decision 1, input model: the text is parsed whole. Callers hold their text in memory
already, and one function is the smallest interface that does the job.

Decision 2, error policy: a line that is not a pair is skipped. The texts read so far
mix data with comments and blank lines, and none of them needs a report of what was
skipped.",
    ],
    // Version 1: chunked input.
    [
        r"A `Parser` reads text that arrives in chunks of any size, split anywhere, even inside
a key or a value. `Parser::push` takes the next chunk and `Parser::finish` ends the
input and returns the entries in the order they appear. Where the chunks are split
never changes the entries.

`parse(input)` reads a complete text in one call. It is a `Parser` that is given one
chunk.

A line of the form `key=value` is one entry. The key is the text before the first `=`
and is not empty; the value is the rest of the line and may be empty.

Lines that are not of that form are skipped.",
        r#"Make a parser, push each chunk as it arrives, and finish when the input ends:

```rust
let mut parser = chunk_parser::Parser::new();
parser.push("a=1\nb");
parser.push("=2\n");
let entries = parser.finish();
```

`chunk_parser::parse` does the same for a text that is complete."#,
        r"The example [chunks](../../examples/chunks.rs) pushes a fixed text in three chunks and
prints one `key=value` pair per line. The example [dump](../../examples/dump.rs) parses
the same text in one call.",
        r"- A line that is not complete is held until its line ending arrives or `finish` is
  called. The parser holds one partial line and every entry found so far.
- Lines are split at `\n`; a `\r` before it is removed.
- Duplicate keys give separate entries, in order.",
        r"Decision 1, input model: the text may arrive in chunks of any size, because sockets
and pipes deliver text in pieces and the caller should not have to join them first.
`parse` stays as the shortest path for a complete text.

Decision 2, error policy: a line that is not a pair is skipped. The texts read so far
mix data with comments and blank lines, and none of them needs a report of what was
skipped.",
    ],
    // Version 2: strict errors.
    [
        r"A `Parser` reads text that arrives in chunks of any size, split anywhere, even inside
a key or a value. `Parser::push` takes the next chunk and `Parser::finish` ends the
input and returns the entries in the order they appear. Where the chunks are split
never changes the entries.

`parse(input)` reads a complete text in one call. It is a `Parser` that is given one
chunk.

A line of the form `key=value` is one entry. The key is the text before the first `=`
and is not empty; the value is the rest of the line and may be empty. A blank line is
not an entry and is not an error.

Any other line is an error. `push`, `finish` and `parse` return a `Result`, and the
`ParseError` names the line, counted from 1. The first error ends the parse: the
parser reports the same error for every later call.",
        r#"Make a parser, push each chunk as it arrives, and finish when the input ends. Both
calls can fail:

```rust
let mut parser = chunk_parser::Parser::new();
parser.push("a=1\nb")?;
parser.push("=2\n")?;
let entries = parser.finish()?;
```

`chunk_parser::parse` does the same for a text that is complete."#,
        r"The example [chunks](../../examples/chunks.rs) pushes a fixed text in three chunks and
prints one `key=value` pair per line, then pushes a malformed line and prints the
error.",
        r"- A line that is not complete is held until its line ending arrives or `finish` is
  called. The parser holds one partial line and every entry found so far.
- An error names the line, not the column.
- Lines are split at `\n`; a `\r` before it is removed.
- Duplicate keys give separate entries, in order.",
        r"Decision 1, input model: the text may arrive in chunks of any size, because sockets
and pipes deliver text in pieces and the caller should not have to join them first.
`parse` stays as the shortest path for a complete text.

Decision 2, error policy: a line that is not a pair is an error. Skipping hid a
truncated feed, so a bad feed is seen at the line where it breaks. The first error ends
the parse because the entries after it cannot be trusted to line up with the input.",
    ],
    // Version 3: incremental output.
    [
        r"A `Parser` reads text that arrives in chunks of any size, split anywhere, even inside
a key or a value. `Parser::feed` takes the next chunk and returns the entries whose
lines it completed, in the order they appear. `Parser::close` ends the input and
returns the entry of a last line that has no line ending. Where the chunks are split
never changes the entries.

`parse(input)` reads a complete text in one call. It feeds one chunk and closes.

A line of the form `key=value` is one entry. The key is the text before the first `=`
and is not empty; the value is the rest of the line and may be empty. A blank line is
not an entry and is not an error.

Any other line is an error. `feed`, `close` and `parse` return a `Result`, and the
`ParseError` names the line, counted from 1. The first error ends the parse: the
parser reports the same error for every later call.",
        r#"Make a parser, feed each chunk as it arrives and use the entries it returns, then
close when the input ends:

```rust
let mut parser = chunk_parser::Parser::new();
for entry in parser.feed("a=1\nb")? {
    println!("{}={}", entry.key, entry.value);
}
let rest = parser.close()?;
```

`chunk_parser::parse` does the same for a text that is complete."#,
        r"The example [stream](../../examples/stream.rs) feeds a fixed text in three chunks and
prints each entry as its line completes.",
        r"- The parser holds one partial line and nothing else. The caller holds the entries.
- An error names the line, not the column.
- Lines are split at `\n`; a `\r` before it is removed.
- Duplicate keys give separate entries, in order.
- A line may be any length, so one line without a line ending can fill the memory.",
        r"Decision 1, input model: the text may arrive in chunks of any size, because sockets
and pipes deliver text in pieces and the caller should not have to join them first.
`parse` stays as the shortest path for a complete text.

Decision 2, error policy: a line that is not a pair is an error. Skipping hid a
truncated feed, so a bad feed is seen at the line where it breaks. The first error ends
the parse because the entries after it cannot be trusted to line up with the input.

Decision 3, output model: entries are returned as their lines complete, because a
caller that reads a stream with no end cannot wait for the end to see the first entry,
and the parser has no reason to keep what the caller already has.",
    ],
    // Version 4: line limit.
    [
        r"A `Parser` reads text that arrives in chunks of any size, split anywhere, even inside
a key or a value. `Parser::feed` takes the next chunk and returns the entries whose
lines it completed, in the order they appear. `Parser::close` ends the input and
returns the entry of a last line that has no line ending. Where the chunks are split
never changes the entries. A complete text is one chunk.

A line of the form `key=value` is one entry. The key is the text before the first `=`
and is not empty; the value is the rest of the line and may be empty. A blank line is
not an entry and is not an error.

A line is at most 65536 bytes long without its line ending, unless
`Parser::with_max_line` sets another limit. A longer line is an error, found as soon
as the limit is passed and without waiting for the line ending.

Any other line is an error. `feed` and `close` return a `Result`, and the `ParseError`
names the line, counted from 1, and the `Reason`: `Reason::Malformed` or
`Reason::TooLong`. The first error ends the parse: the parser reports the same error
for every later call.",
        r#"Make a parser, with a line limit when the default does not suit, feed each chunk as
it arrives and use the entries it returns, then close when the input ends:

```rust
let mut parser = chunk_parser::Parser::with_max_line(1024);
for entry in parser.feed("a=1\nb")? {
    println!("{}={}", entry.key, entry.value);
}
let rest = parser.close()?;
```

A complete text is fed as one chunk and then closed."#,
        r"The example [stream](../../examples/stream.rs) feeds a fixed text in three chunks and
prints each entry as its line completes. The example [limits](../../examples/limits.rs)
feeds a line over a limit of 8 bytes and prints the error.",
        r"- The parser holds at most one partial line, and a line is at most the limit, so its
  memory is bounded by the limit.
- The limit counts bytes, not characters.
- An error names the line, not the column.
- Lines are split at `\n`; a `\r` before it is removed.
- Duplicate keys give separate entries, in order.",
        r"Decision 1, input model: the text may arrive in chunks of any size, because sockets
and pipes deliver text in pieces and the caller should not have to join them first. A
complete text is one chunk, so there is one way in and no second entry point to keep
consistent with it.

Decision 2, error policy: a line that is not a pair is an error. Skipping hid a
truncated feed, so a bad feed is seen at the line where it breaks. The first error ends
the parse because the entries after it cannot be trusted to line up with the input.

Decision 3, output model: entries are returned as their lines complete, because a
caller that reads a stream with no end cannot wait for the end to see the first entry,
and the parser has no reason to keep what the caller already has.

Decision 4, line limit: a line is at most 65536 bytes unless the caller sets another
limit, because a stream that never sends a line ending would otherwise fill the
memory. The limit counts bytes since that is what the parser holds.",
    ],
];

/// The seven sections of the document at a version, in the order of the
/// profile.
pub(crate) fn sections(version: usize) -> [&'static str; 7] {
    let [contract, usage, examples, constraints, explanation] = CHANGING[version];
    [
        PURPOSE,
        contract,
        usage,
        examples,
        constraints,
        explanation,
        VERIFICATION,
    ]
}

/// A draft of the proposal of a change: the text for each section it
/// proposes for.
pub(crate) type Draft = &'static [(&'static str, &'static str)];

/// One redesign: the change that opens, the sections it affects, the drafts
/// of its proposals (the last is what is built), its items and the version it
/// delivers.
pub(crate) struct Redesign {
    pub(crate) slug: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) affects: &'static [&'static str],
    pub(crate) drafts: &'static [Draft],
    /// Each item: the claim and the outcome it ends with.
    pub(crate) items: &'static [(&'static str, &'static str)],
    pub(crate) version: usize,
}

/// The sections every redesign rewrites.
const AFFECTS: &[&str] = &[
    "contract",
    "usage",
    "examples",
    "constraints",
    "explanation",
];

/// The four redesigns, in order.
pub(crate) const REDESIGNS: [Redesign; 4] = [
    Redesign {
        slug: "chunked-input",
        title: "Chunked input",
        description: "The parser accepts its text in chunks.",
        summary: "Amended 2026-02-03, decision 1: the text may arrive in chunks of any size. A parser takes each chunk with push and returns the entries with finish; parse stays for a complete text.",
        affects: AFFECTS,
        drafts: &[&[
            (
                "contract",
                r"Amended 2026-02-03, decision 1: the text may arrive in chunks of any size, split
anywhere. A `Parser` takes each chunk with `push` and returns the entries with
`finish`. `parse` stays for a complete text and is a parser given one chunk.",
            ),
            (
                "constraints",
                r"Amended 2026-02-03: a line that is not complete is held until its line ending
arrives or `finish` is called.",
            ),
            (
                "explanation",
                r"Amended 2026-02-03, decision 1: sockets and pipes deliver text in pieces, so the
parser accepts the pieces and the caller does not join them first. Decision 2 is
unchanged.",
            ),
        ]],
        items: &[
            (
                "Text may arrive in chunks of any size, split anywhere",
                "implemented",
            ),
            ("A complete text still parses in one call", "implemented"),
        ],
        version: 1,
    },
    Redesign {
        slug: "strict-errors",
        title: "Strict errors",
        description: "A line that is not a pair is an error.",
        summary: "Amended 2026-03-10, decision 2 is reversed: a line that is not a pair is an error that names its line, and the first error ends the parse.",
        affects: AFFECTS,
        drafts: &[&[
            (
                "contract",
                r"Amended 2026-03-10, decision 2 is reversed: a line that is not a pair is an error
that names its line number. `push`, `finish` and `parse` return a `Result`, and the
first error ends the parse.",
            ),
            (
                "constraints",
                r"Amended 2026-03-10: a blank line is not an error. An error names the line and the
column.",
            ),
            (
                "explanation",
                r"Amended 2026-03-10, decision 2: skipping hid a truncated feed, so a malformed line
is an error and a bad feed is seen where it breaks.",
            ),
        ]],
        items: &[
            ("A line that is not a pair is an error", "implemented"),
            ("An error names the line and the column", "dropped"),
            ("The first error ends the parse", "implemented"),
        ],
        version: 2,
    },
    Redesign {
        slug: "incremental-output",
        title: "Incremental output",
        description: "Entries are returned as their lines complete.",
        summary: "Amended 2026-04-21, decision 3: entries are returned as soon as their line is complete. Push is renamed feed and finish is renamed close.",
        affects: AFFECTS,
        drafts: &[&[
            (
                "contract",
                r"Amended 2026-04-21, decision 3: entries are returned as soon as their line is
complete. `push` is renamed `feed` and returns the entries that the chunk completed;
`finish` is renamed `close` and returns the entry of a last line that has no line
ending.",
            ),
            (
                "constraints",
                r"Amended 2026-04-21: the parser holds one partial line and nothing else; the
caller holds the entries.",
            ),
            (
                "explanation",
                r"Amended 2026-04-21, decision 3: a caller that reads a stream with no end cannot
wait for the end to see the first entry.",
            ),
        ]],
        items: &[
            (
                "Entries are returned as their line completes",
                "implemented",
            ),
            (
                "The method push is renamed feed and finish is renamed close",
                "implemented",
            ),
        ],
        version: 3,
    },
    Redesign {
        slug: "line-limit",
        title: "Line limit",
        description: "A line has a length limit and the one-shot function is removed.",
        summary: "Amended 2026-05-30, decision 4: a line has a length limit of 65536 bytes, which the first draft set at 4096. The one-shot parse function is removed, since a complete text is one chunk.",
        affects: AFFECTS,
        drafts: &[
            &[
                (
                    "contract",
                    r"Amended 2026-05-12, decision 4: a line has a length limit, 4096 bytes unless
`Parser::with_max_line` sets another. A longer line is an error. `parse` is removed,
since a complete text is one chunk.",
                ),
                (
                    "constraints",
                    r"Amended 2026-05-12: the memory of a parser is bounded by the limit.",
                ),
                (
                    "explanation",
                    r"Amended 2026-05-12, decision 4: a stream that never sends a line ending would
otherwise fill the memory.",
                ),
            ],
            &[
                (
                    "contract",
                    r"Amended 2026-05-30, decision 4: the default limit is 65536 bytes and not 4096,
because feeds in use carry lines near 40000 bytes. `Parser::with_max_line` sets
another limit. A longer line is an error. `parse` is removed, since a complete text
is one chunk.",
                ),
                (
                    "constraints",
                    r"Amended 2026-05-30: the memory of a parser is bounded by the limit, which counts
bytes.",
                ),
                (
                    "explanation",
                    r"Amended 2026-05-30, decision 4: a stream that never sends a line ending would
otherwise fill the memory; the limit is large enough for the feeds in use.",
                ),
            ],
        ],
        items: &[
            ("A line has a length limit", "implemented"),
            ("The one-shot parse function is removed", "implemented"),
            ("The limit counts characters and not bytes", "dropped"),
        ],
        version: 4,
    },
];

/// A sentence of the contract that applies at a version, and the version at
/// which it stops applying (`None` while it still applies at the last).
pub(crate) struct Wording {
    pub(crate) text: &'static str,
    /// The first version whose document states it.
    pub(crate) from: usize,
    /// The first version whose document no longer states it.
    pub(crate) until: Option<usize>,
}

/// The sentences a reader of an old version relied on that no longer apply,
/// and the sentences that apply now.
pub(crate) const WORDING: [Wording; 10] = [
    Wording {
        text: "Lines that are not of that form are skipped.",
        from: 0,
        until: Some(2),
    },
    Wording {
        text: "reads a whole text in one call",
        from: 0,
        until: Some(1),
    },
    Wording {
        text: "`Parser::push` takes the next chunk",
        from: 1,
        until: Some(3),
    },
    Wording {
        text: "`Parser::finish` ends the",
        from: 1,
        until: Some(3),
    },
    Wording {
        text: "every entry found so far",
        from: 1,
        until: Some(3),
    },
    Wording {
        text: "`parse(input)` reads a complete text in one call.",
        from: 1,
        until: Some(4),
    },
    Wording {
        text: "A line may be any length",
        from: 3,
        until: Some(4),
    },
    Wording {
        text: "`Parser::feed` takes the next chunk",
        from: 3,
        until: None,
    },
    Wording {
        text: "A line is at most 65536 bytes long without its line ending",
        from: 4,
        until: None,
    },
    Wording {
        text: "The first error ends the parse",
        from: 2,
        until: None,
    },
];
