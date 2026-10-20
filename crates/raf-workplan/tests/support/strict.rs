// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The strict-clean repository: one package and one delivered capability that
//! every rule of strict mode accepts, and the one-line edits that each break one
//! rule. The texts depend on nothing but `std`, so the command-line tests read
//! this file as their own and write the same repository to a disk.
//!
//! The package `demo` has a documented function `demo::run` with a runnable
//! documentation test and an integration test `demo::test_crate::it::case`. The
//! capability `cap:demo` binds the function in `code` and in `examples` and the
//! test in `tests`, names `demo::run` in a code span of its Usage, lists a
//! limitation in Constraints, holds a verification matrix in Verification and
//! [`PROSE`], twenty words, in each section, so its package is fully bound.

/// The capability document.
pub(crate) const CAPABILITY: &str = "docs/capabilities/demo.md";
/// The coverage record.
pub(crate) const COVERAGE: &str = "work/coverage/default.md";
/// The library of the package.
pub(crate) const LIB: &str = "src/lib.rs";

/// Twenty words of prose: [`MIN_SECTION_WORDS`](raf_workplan::strict::MIN_SECTION_WORDS).
pub(crate) const PROSE: &str = "This section of the demo capability states one plain fact in twenty words so that strict mode reads it whole.";
/// [`PROSE`] without its last word: nineteen words.
pub(crate) const THIN: &str = "This section of the demo capability states one plain fact in twenty words so that strict mode reads it.";

/// The manifest: a package of its own workspace, so `cargo metadata` run in the
/// directory reads no enclosing workspace.
pub(crate) const MANIFEST: &str =
    "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n";
/// The lock file of the manifest.
pub(crate) const LOCK: &str = "version = 4\n\n[[package]]\nname = \"demo\"\nversion = \"0.1.0\"\n";
/// The library: one documented function with a runnable documentation test.
pub(crate) const LIBRARY: &str =
    "/// Runs the demo.\n///\n/// ```\n/// demo::run();\n/// ```\npub fn run() {}\n";
/// The integration test.
pub(crate) const TEST: &str = "#[test]\nfn case() {}\n";

/// The Usage line that names the bound documentation test in a code span.
pub(crate) const USAGE_SPAN: &str = "Call `demo::run` once.";
/// The limitation that Constraints lists.
pub(crate) const LIMITATION: &str = "- One limitation of the demo.";
/// The generated block the Verification section holds, empty until a sync.
pub(crate) const MATRIX: &str =
    "<!-- raf:generated:verification-matrix -->\n<!-- raf:generated:end -->\n";
/// The examples list of the capability.
pub(crate) const EXAMPLES: &str = "  examples: [demo::run]\n";

/// The files of the source workspace, by path.
pub(crate) fn source_files() -> Vec<(&'static str, String)> {
    vec![
        ("Cargo.toml", MANIFEST.to_string()),
        (LIB, LIBRARY.to_string()),
        ("tests/it.rs", TEST.to_string()),
    ]
}

/// The capability document that every rule of strict mode accepts.
pub(crate) fn capability() -> String {
    let section = |name: &str, extra: &str| format!("## {name}\n\n{PROSE}\n{extra}");
    let sections = [
        section("Purpose", ""),
        section("Contract", ""),
        section("Usage", &format!("\n{USAGE_SPAN}\n")),
        section("Examples", ""),
        section("Constraints", &format!("\n{LIMITATION}\n")),
        section("Explanation", ""),
        section("Verification", &format!("\n{MATRIX}")),
    ]
    .join("\n");
    format!(
        "---\n# SPDX-FileCopyrightText: 2026 Example\n# SPDX-License-Identifier: CC-BY-4.0\n\
         type: Capability\nid: cap:demo\ntitle: \"Demo\"\n\
         description: \"A capability that strict mode accepts.\"\nstate: delivered\n\
         bindings:\n  code: [demo::run]\n  tests: [demo::test_crate::it::case]\n{EXAMPLES}\
         depends: []\n---\n\n# Demo\n\n{sections}"
    )
}

/// The coverage record with the `strictness` line `strictness` (empty for
/// none), enforcing no crate.
pub(crate) fn coverage(strictness: &str) -> String {
    format!(
        "---\ntype: Coverage\nid: coverage:default\n{strictness}scope: []\nexclusions: []\n---\n"
    )
}

/// The documents of the strict-clean repository, by path, under a coverage
/// record with no `strictness` key.
pub(crate) fn documents() -> Vec<(&'static str, String)> {
    vec![(CAPABILITY, capability()), (COVERAGE, coverage(""))]
}

/// `text` with its one `from` replaced by `to`.
pub(crate) fn replaced(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in {text}");
    text.replacen(from, to, 1)
}

/// The first `count` words of [`PROSE`], as a sentence.
pub(crate) fn words(count: usize) -> String {
    let words: Vec<&str> = PROSE.trim_end_matches('.').split(' ').take(count).collect();
    format!("{}.", words.join(" "))
}

/// The words a section holds beside its [`PROSE`]: the two of the Usage line
/// and the five of the limitation.
fn extra_words(section: &str) -> usize {
    match section {
        "Usage" => 2,
        "Constraints" => 5,
        _ => 0,
    }
}

/// The capability with the prose of `section` cut so that the section holds
/// nineteen words.
pub(crate) fn thin(section: &str) -> String {
    replaced(
        &capability(),
        &format!("## {section}\n\n{PROSE}\n"),
        &format!("## {section}\n\n{}\n", words(19 - extra_words(section))),
    )
}
