"""Write the files of one commit state, A, B or C, into the worktree.

C is the final state kept in scratch/final. B is C without the hunks of the
command line, A is B without the hunks of claim coverage. A file that a state
does not hold is restored to HEAD, or removed when HEAD has none.

Run from the worktree root: python3 -I <this> A|B|C
"""

import os
import re
import subprocess
import sys

ROOT = os.getcwd()
assert ROOT.endswith("/.worktrees/16-raf-workplan-strict-mode-force-coverage"), ROOT
FINAL = os.path.join(ROOT, ".stage/raf-workplan-strict-mode-force-coverage/scratch/final")
STATE = sys.argv[1]
assert STATE in ("0", "A", "B", "C")


def final(path):
    with open(os.path.join(FINAL, path), encoding="utf-8") as handle:
        return handle.read()


def head(path):
    run = subprocess.run(["git", "show", f"HEAD:{path}"], capture_output=True, text=True)
    return run.stdout if run.returncode == 0 else None


def cut(text, old, new="", count=1):
    assert text.count(old) == count, (old[:80], text.count(old))
    return text.replace(old, new)


def cut_between(text, start, end):
    """Remove from `start` up to, not including, `end`."""
    a = text.index(start)
    b = text.index(end, a)
    return text[:a] + text[b:]


# The files of each step.
OKF = [
    "crates/raf-okf/src/body/marker.rs",
    "crates/raf-okf/src/body/mod.rs",
    "crates/raf-okf/src/error.rs",
    "crates/raf-okf/src/lib.rs",
    "crates/raf-okf/src/profile/body_rules.rs",
    "crates/raf-okf/src/profile/violation.rs",
    "crates/raf-okf/tests/tested_by.rs",
    "crates/raf-workplan/src/plain.rs",
    "crates/raf-workplan/tests/plain.rs",
    "docs/architecture/okf-profile.md",
    "docs/capabilities/okf-profile.md",
    "work/changes/workplan-strict.md",
]
WORKPLAN = [
    "crates/raf-workplan/src/coverage/render.rs",
    "crates/raf-workplan/src/graph.rs",
    "crates/raf-workplan/src/lib.rs",
    "crates/raf-workplan/src/strict.rs",
    "crates/raf-workplan/src/strict/claims.rs",
    "crates/raf-workplan/tests/claim_coverage.rs",
    "crates/raf-workplan/tests/generated.rs",
    "crates/raf-workplan/tests/graph.rs",
    "crates/raf-workplan/tests/kinds.rs",
    "crates/raf-workplan/tests/support/strict.rs",
    "docs/capabilities/workplan-generated-docs.md",
]
CLI = [
    "AGENTS.md",
    "README.md",
    "crates/cargo-workplan/src/cli.rs",
    "crates/cargo-workplan/src/command.rs",
    "crates/cargo-workplan/src/coverage.rs",
    "crates/cargo-workplan/src/main.rs",
    "crates/cargo-workplan/tests/claims.rs",
    "crates/cargo-workplan/tests/json.rs",
    "docs/capabilities/workplan-cli.md",
]
# Files whose hunks belong to more than one step.
SHARED = [
    "CHANGELOG.md",
    "crates/raf-workplan/src/finding.rs",
    "crates/raf-workplan/tests/support/cases.rs",
    "docs/capabilities/workplan-document-model.md",
    "docs/capabilities/workplan-strict.md",
]


def changelog(text):
    if STATE in ("A", "B"):
        text = cut_between(text, "- **`cargo-workplan`:** `coverage claims", "- **`raf-source`:** a work budget")
    if STATE == "A":
        text = cut_between(text, "- **`raf-workplan`:** claim coverage of strict mode.", "- **`raf-source`:** a work budget")
    return text


def finding(text):
    if STATE == "A":
        text = cut(text, '    CLAIM_TEST_UNBOUND = "claim-test-unbound";\n')
        text = cut(text, '    STRICT_CLAIM_UNCOVERED = "strict-claim-uncovered";\n    STRICT_CLAIM_UNIT_COLLAPSED = "strict-claim-unit-collapsed";\n')
        text = cut(text, "    ///   `claim-test-unbound`: an entry of a tested-by marker of a capability\n    ///   that is no path of the capability's `bindings.tests` or\n    ///   `bindings.examples`, at the line of the marker, under either\n    ///   strictness.\n")
        text = cut(text, "    ///   `strict-claim-uncovered`, `strict-claim-unit-collapsed`,\n")
    return text


def cases(text):
    if STATE == "A":
        text = cut(text, "    cases.extend(claim_cases());\n")
        text = cut_between(text, "/// The cases of the tested-by marker that are judged under either\n", "/// The strict-clean repository with the edit `edit` made to its files.\n")
        text = cut_between(text, "        // The Contract's one claim without its marker.\n", "    ];\n    cases.extend(strict_coverage_cases());")
    return text


def document_model(text):
    if STATE == "A":
        text = cut(text, "    - raf_workplan::Graph::tested_by\n    - raf_workplan::TestedByEdge\n")
        text = cut_between(text, "- Tested-by edges: from the section of a capability that holds a tested-by marker\n", "\nThe graph must have no `depends` cycle.")
        text = cut(text, "record (`Graph::claims_on`), the tested-by edges (`Graph::tested_by`), the subjects", "record (`Graph::claims_on`), the subjects")
        text = cut_between(text, "- under either strictness, a tested-by marker entry that is no path of the\n", "- under strict, the rules of [strict mode]")
        head_text = head("docs/capabilities/workplan-document-model.md")
        start = head_text.index("> [!PROPOSED change:workplan-strict]\n> - under either strictness")
        end = head_text.index("\nMarkdown outside the document directories", start)
        a = text.index("> [!PROPOSED change:workplan-strict]\n> - under either strictness")
        b = text.index("\nMarkdown outside the document directories", a)
        text = text[:a] + head_text[start:end] + text[b:]
    return text


def strict_doc(text):
    if STATE in ("A", "B"):
        for test in [
            "every_claim_names_a_test_prints_the_report_and_exits_zero",
            "a_claim_that_names_no_test_is_a_finding_and_exits_one_in_text_and_json_alike",
            "a_capability_that_is_not_loaded_exits_two_and_says_why",
        ]:
            text = cut(text, f"    - cargo_workplan::test_crate::claims::{test}\n")
        text = cut_between(text, "\nThe claims report of the same capability names the test of its one marked\n", "\nThe reverse coverage of one package,")
        text = cut_between(text, "- Report fixtures: the table and the JSON of `coverage claims` equal literal strings\n", "- A bound package outside `scope`")
    if STATE == "A":
        text = head("docs/capabilities/workplan-strict.md")
    return text


SHAPERS = {
    "CHANGELOG.md": changelog,
    "crates/raf-workplan/src/finding.rs": finding,
    "crates/raf-workplan/tests/support/cases.rs": cases,
    "docs/capabilities/workplan-document-model.md": document_model,
    "docs/capabilities/workplan-strict.md": strict_doc,
}

held = {"0": [], "A": OKF, "B": OKF + WORKPLAN, "C": OKF + WORKPLAN + CLI}[STATE]


def write(path, text):
    full = os.path.join(ROOT, path)
    if text is None:
        if os.path.exists(full):
            os.remove(full)
        return
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(text)


for path in OKF + WORKPLAN + CLI:
    write(path, final(path) if path in held else head(path))
for path in SHARED:
    write(path, head(path) if STATE == "0" else SHAPERS[path](final(path)))
print("state", STATE, "written")
