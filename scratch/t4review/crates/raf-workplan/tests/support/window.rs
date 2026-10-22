// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The story of the second scenario: one capability, `rolling-window`, over
//! the crate `window-stats`.
//!
//! The capability is `considered` with two open questions and a candidate
//! contract in PROPOSED blocks. The implementation renames the candidate's
//! `add` to `record`, and keeps its values in a growable buffer, which drops
//! the guarantee that recording never allocates. The change record lists three
//! claims, and the one about allocation ends `dropped`.

use super::story::Bindings;
use super::window_code::{W_LIB, W_ROLLING, W_TESTS};

/// The package of the crate.
pub(crate) const PACKAGE: &str = "window-stats";

/// The id of the capability.
pub(crate) const ID: &str = "cap:rolling-window";

/// The slug of the capability.
pub(crate) const SLUG: &str = "rolling-window";

/// The path of the capability document.
pub(crate) const DOCUMENT: &str = "docs/capabilities/rolling-window.md";

/// The slug of the change.
pub(crate) const CHANGE: &str = "first-window-api";

/// The path of the change record.
pub(crate) const RECORD: &str = "work/changes/first-window-api.md";

/// The title of the capability.
pub(crate) const TITLE: &str = "Rolling window";

/// The description of the capability.
pub(crate) const DESCRIPTION: &str = "The most recent values of a series, and their mean.";

/// The sections the change affects: all seven.
pub(crate) const AFFECTS: [&str; 7] = [
    "purpose",
    "contract",
    "usage",
    "examples",
    "constraints",
    "explanation",
    "verification",
];

/// The title, description and summary of the change record.
pub(crate) const RECORD_TITLE: &str = "First window API";
pub(crate) const RECORD_DESCRIPTION: &str =
    "Settles the open questions of the rolling window and builds it.";
pub(crate) const RECORD_SUMMARY: &str =
    "Answers the two open questions, builds the window and writes the capability as it is built.";

/// The claims of the change, in order. The second is the guarantee the
/// implementation does not keep.
pub(crate) const CLAIMS: [&str; 3] = [
    "The window keeps the most recent values up to its capacity",
    "Adding a value never allocates after the window is made",
    "The mean of the stored values is available at any time",
];

/// The crate before it is implemented.
pub(crate) const EMPTY_LIB: &str = "//! The rolling window crate.\n";

/// The crate as built.
pub(crate) fn built() -> Vec<(&'static str, &'static str)> {
    vec![
        ("src/lib.rs", W_LIB),
        ("tests/window.rs", W_TESTS),
        ("examples/rolling.rs", W_ROLLING),
    ]
}

/// An example that fails: it exits with status 1.
pub(crate) const FAILING_EXAMPLE: &str = "fn main() {\n    std::process::exit(1);\n}\n";

/// The candidate's text for the sections that carry a PROPOSED block, by
/// section.
pub(crate) const CANDIDATE: [(&str, &str); 4] = [
    (
        "contract",
        "`Window::new(capacity)` makes an empty window. `Window::add(value)` stores a value\nand drops the oldest one when the window is full. `Window::mean()` returns the mean\nof the stored values, or nothing when the window is empty. `add` never allocates\nafter the window is made.",
    ),
    (
        "usage",
        "Make a window with `Window::new`, store values with `Window::add` and ask for the\nmean with `Window::mean`.",
    ),
    (
        "examples",
        "An example target named `rolling` adds four values to a window of three and prints\nthe mean after each.",
    ),
    (
        "constraints",
        "The window holds at most `capacity` values. `add` never allocates after the window\nis made.",
    ),
];

/// The sections of the capability while it is considered: plain statements
/// of where things stand. The candidate contract is in the blocks.
pub(crate) const CONSIDERED: [&str; 7] = [
    r"The rolling window keeps the most recent values of a series and answers questions
about them. Two questions are open. Should the window be bounded by a count or by an
age? Should a value that is not a number be refused or stored? The candidate contract
answers the first with a count and leaves the second open.",
    "The contract is a candidate. It sits in the block below, and no implementation supports it.",
    "Usage follows from the contract once the questions above are answered.",
    "No example is written for a candidate.",
    "The limits of the candidate sit in the block below.",
    "The reasons for the choices are written when the questions above are answered.",
    "No test substantiates the candidate contract.",
];

const PURPOSE: &str = r"The rolling window keeps the most recent values of a series and answers the mean of
them. It is bounded by a count. A `NaN` value is stored like any other.";

const CONTRACT: &str = r"`Window::new(capacity)` makes an empty window that keeps at most `capacity` values.
`Window::record(value)` stores a value and drops the oldest one when the window is
full. `Window::mean()` returns the mean of the stored values, or `None` when the
window is empty. `Window::len()` and `Window::is_empty()` report how many values are
stored. A window of capacity 0 stores nothing. The API is in
[the source](../../src/lib.rs).";

const USAGE: &str = r#"Make a window, record values as they arrive and ask for the mean when it is needed:

```rust
let mut window = window_stats::Window::new(3);
window.record(1.0);
window.record(3.0);
println!("{:?}", window.mean());
```"#;

const EXAMPLES: &str = r"The example [rolling](../../examples/rolling.rs) records four values into a window of
three and prints the mean after each. The documentation example of `Window::new` runs
as a test.";

const CONSTRAINTS: &str = r"- Recording a value can allocate while the window fills, because the values are kept in
  a `VecDeque` that grows up to the capacity.
- The mean is computed over the stored values on each call, so its cost grows with the
  capacity.
- The window is bounded by a count and has no notion of age.
- A `NaN` value makes the mean `NaN` until it leaves the window.";

const EXPLANATION: &str = r"The window is bounded by a count because callers feed it at a steady rate, so a count
is also an age and needs no clock. The values are kept in a `VecDeque` that grows up to
the capacity: a fixed array would remove the allocation, but it needs the capacity at
compile time, and callers choose it at run time. The mean is computed on demand and not
kept as a running sum, because a running sum drifts as values leave the window and the
stored values are few.";

const VERIFICATION: &str = r"Each bound test checks one claim of the [Contract](#contract) or of the
[Constraints](#constraints), the documentation example of `Window::new` runs as a test,
and the example target is built and run. The table lists them.";

/// The sections once the contract, the constraints, the explanation and the
/// verification text are settled, with usage and examples still the candidate's.
pub(crate) fn partly_settled() -> [&'static str; 7] {
    [
        PURPOSE,
        CONTRACT,
        CONSIDERED[2],
        CONSIDERED[3],
        CONSTRAINTS,
        EXPLANATION,
        VERIFICATION,
    ]
}

/// The sections of the capability as built.
pub(crate) fn settled() -> [&'static str; 7] {
    [
        PURPOSE,
        CONTRACT,
        USAGE,
        EXAMPLES,
        CONSTRAINTS,
        EXPLANATION,
        VERIFICATION,
    ]
}

/// The tests the capability binds.
const TESTS: &[&str] = &[
    "window_stats::test_crate::window::the_window_keeps_only_the_most_recent_values",
    "window_stats::test_crate::window::the_mean_of_an_empty_window_is_none",
    "window_stats::test_crate::window::a_window_of_capacity_zero_keeps_nothing",
];

/// Nothing is bound while the capability is considered.
pub(crate) const UNBOUND: Bindings = Bindings {
    code: &[],
    tests: &[],
    examples: &[],
};

/// The bindings as the candidate named them: the API that was renamed.
pub(crate) const CANDIDATE_BINDINGS: Bindings = Bindings {
    code: &[
        "window_stats::Window",
        "window_stats::Window::new",
        "window_stats::Window::add",
        "window_stats::Window::mean",
    ],
    tests: TESTS,
    examples: &[
        "window_stats::example_crate::rolling",
        "window_stats::Window::add",
    ],
};

/// The bindings of the API as built.
pub(crate) const CURRENT_BINDINGS: Bindings = Bindings {
    code: &[
        "window_stats::Window",
        "window_stats::Window::new",
        "window_stats::Window::record",
        "window_stats::Window::mean",
        "window_stats::Window::len",
        "window_stats::Window::is_empty",
    ],
    tests: TESTS,
    examples: &[
        "window_stats::example_crate::rolling",
        "window_stats::Window::new",
    ],
};

/// A document that exists and says nothing: every section is a single
/// sentence that names the capability and nothing else.
pub(crate) const THIN: [&str; 7] = [
    "The rolling window.",
    "The rolling window.",
    "The rolling window.",
    "The rolling window.",
    "The rolling window.",
    "The rolling window.",
    "The rolling window.",
];

/// Sentences of the document as built that the scenario reads.
pub(crate) const ACCURATE: [&str; 4] = [
    "`Window::record(value)` stores a value",
    "Recording a value can allocate while the window fills",
    "a `VecDeque` that grows up to the capacity",
    "The window is bounded by a count and has no notion of age.",
];
