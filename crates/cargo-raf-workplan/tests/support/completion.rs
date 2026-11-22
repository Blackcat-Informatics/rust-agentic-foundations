// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The completed and refused lifecycle fixtures shared by command projections.

use super::{BLOCKS, Repo, capability, change};

pub(crate) const RECORD: &str = "work/changes/shape.md";
pub(crate) const CAP: &str = "docs/capabilities/fixture.md";

pub(crate) const PASSES: &str = "[demo::tests::passes]";
pub(crate) const GOOD: &str = "[demo::example_crate::good]";

/// An item of the record that has its outcome.
pub(crate) const DONE: &str = "items:\n  - claim: \"fixture is shaped\"\n    outcome: implemented";

/// Keep only the proof targets this lifecycle fixture actually binds. The
/// negative targets remain real failing programs when the case requests them.
pub(crate) fn proof_targets(repo: &Repo, tests: &str, examples: &str) {
    let mut library = repo
        .read("src/lib.rs")
        .replace("pub fn real()", "fn real()")
        .replace("    a + b", "    real();\n    a + b");
    for (name, body) in [
        (
            "passes",
            "    #[test]\n    fn passes() {\n        assert_eq!(super::add(1, 2), 3);\n    }\n",
        ),
        (
            "fails",
            "    #[test]\n    fn fails() {\n        panic!(\"this test fails\");\n    }\n",
        ),
    ] {
        if !tests.contains(&format!("demo::tests::{name}")) {
            assert!(library.contains(body), "the fixture holds {name}");
            library = library.replace(body, "");
        }
    }
    repo.put("src/lib.rs", &library);
    std::fs::remove_file(repo.root().join("tests/it.rs")).expect("the bare fixture wrote its test");
    for name in ["good", "exiting"] {
        if !examples.contains(&format!("demo::example_crate::{name}")) {
            std::fs::remove_file(repo.root().join(format!("examples/{name}.rs")))
                .expect("the real fixture wrote its example");
        }
    }
}

/// A checkout of a workspace with the capability `fixture` in `state`, bound
/// as given, and the change `shape` that lists it with `items`. The contract
/// of the capability is edited after the fork, so its section is touched.
pub(crate) fn shaped(tag: &str, state: &str, tests: &str, examples: &str, items: &str) -> Repo {
    let repo = Repo::real(tag);
    proof_targets(&repo, tests, examples);
    repo.put(
        CAP,
        &capability(
            "fixture",
            "delivered",
            ["[demo::add]", tests, examples],
            "[]",
            &[("Verification", BLOCKS)],
        )
        .replacen("state: delivered", &format!("state: {state}"), 1),
    );
    repo.put(
        RECORD,
        &change("shape", "[cap:fixture]", &["cap:fixture#contract"]).replace("items: []", items),
    );
    repo.sync();
    repo.checkout();
    repo.edit(
        CAP,
        "Text for Contract of fixture.",
        "Fixture states its contract here.",
    );
    repo.sync();
    // The contract as it now reads is assessed.
    repo.assess("shape");
    repo
}

/// A delivered capability and a resolved record: `complete` completes.
pub(crate) fn ready(tag: &str) -> Repo {
    shaped(tag, "delivered", PASSES, GOOD, DONE)
}
