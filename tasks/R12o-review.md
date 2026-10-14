Current status: VERDICT: PASS after recheck

# R12o review (raf-okf tests, FencedBlock accessors, validate cost)

## Scope
Uncommitted changes over 0617985: crates/raf-okf (src/body/{blocks,mod}.rs, src/profile/{mod,body_rules}.rs,
tests/{profile,render,refusals,body_structure,validate_cost}.rs, tests/fixtures/*.md), tests/docs_contract.rs,
docs/capabilities/okf-profile.md, Makefile, .github/workflows/ci.yaml, AGENTS.md, CONTRIBUTING.md.
Ignored: raf-source, source-index.md, SECURITY.md.

## Evidence
- Gates, all green on the working tree: `cargo test -p raf-okf --locked`, same with `--release` (validate_cost: 2 passed),
  `cargo test --locked --test docs_contract --test repo_gates`, workspace clippy `-D warnings`, `make reuse` (215/215).
- Old-code run (git archive 0617985 in /tmp, own CARGO_TARGET_DIR, new tests copied in):
  - validate_cost, release: both FAIL on old code. Capability probe (45310 sections, 10000 blocks): 324 ms; Change probe
    (27452 changes and affects): 512 ms; bound 100 ms. They pass on the new code (2 passed in 0.08 s).
  - G17: the new docs_contract test `a_tilde_fenced_usage_block_is_accepted_and_an_unclosed_block_is_refused`, added to the
    old docs_contract.rs, FAILS on old code and passes on new.
  - profile.rs, refusals.rs and the nested-key tests pass on old code. That is expected for G4, and for the Q-2 tests, which
    assert unchanged behaviour (the plan's mutation rule names G4 but not Q-2; see A-2).
  - render.rs cannot run in a git-less archive (`git ls-files` fails); the golden count was therefore checked by mutation.
- Mutations re-run on a scratch copy (reverted, nothing in the worktree touched):
  - G4: add `optional("affects", Affects)` to CAPABILITY in schema.rs -> `an_unknown_or_foreign_key_is_reported_on_every_type`
    FAILS (profile.rs:324).
  - Golden: delete `"docs/architecture"` from `tracked_documents` -> `every_tracked_document_renders_to_its_own_bytes` FAILS
    ("docs/architecture: 0 documents were round-tripped and 2 are tracked"). So the count check is not vacuous, and the
    `held >= 1` guard stops an empty listing passing.
- Output equivalence of the perf fix (old vs new, release): 15102 documents (every tracked doc, each with every line deleted,
  duplicated and swapped, plus whole-body duplication and an id rewrite; each validated under its own and two foreign
  paths), 47564 violations: the full `{:#?}` output is byte-identical (cmp). The 1 MiB capability probe's result is also
  byte-identical (4.26 MB of output). Violation order and content are unchanged.
- FencedBlock: content ranges are computed correctly for closed, unclosed, empty, CRLF, no-final-newline and quoted fences
  (quote prefixes kept, closing line excluded). `content()` borrows from the shared `Arc<str>`; no per-block copy.
  Doc examples run as doctests.

## Findings

### REQUIRED
- R-1 `FencedBlock` derives `Debug, PartialEq, Eq` over the new `text: Arc<str>` field. Measured: `format!("{block:?}")` of
  one block in a 600 KB body is 650 KB, so `{:?}` of a `Body`/`Document` with N fences prints the whole text N times, and `==` on
  two blocks compares the whole body text (O(body) per block; two distinct blocks of equal body compare on text again).
  The shared handle leaks into public trait output and equality, which contradicts "Arc design not leaking" and re-creates, in
  Debug output, the O(blocks x size) blow-up R1 removed. Implement `Debug` and `PartialEq` by hand over span, info, content
  range and closed (or compare `content()` and exclude `text`), and add a test that `Debug` of a block is small.
- R-2 `every_tracked_document_renders_to_its_own_bytes` asserts `held >= 1` for `work/changes`. The repository rule is that
  `change complete` deletes each change record (AGENTS.md: "deleted; Git keeps the history"); when the last record is
  completed, `work/changes` is legitimately empty and this test fails on a correct tree. Keep `>= 1` for `docs/capabilities`
  and `docs/architecture` and require only count equality for the Change directory (the committed change fixture covers
  the type independently), or state the floor per directory in a way that survives deletion.

### ADVISORY
- A-1 The new tests that exist are only partly bound in okf-profile.md `bindings.tests`: the fixture round-trip test,
  the golden count (existing test, changed), the BOM/NUL/trailing-space refusals, the nested-key test and the validate_cost
  probes are not listed (only the foreign-key and fenced-block tests were added). Bind them so the document's test list
  matches what proves the contract.
- A-2 Q-2 tests assert current behaviour and so cannot fail on 6c744af; the mutation rule only exempts G4/G5/Q-6/Q-7. Record
  them as at-limit tests with a named mutation (for example: strip a BOM in the fence check; trim the closing `---`),
  or accept the plan wording as covering them.
- A-3 Line wrapping: the AGENTS.md and CONTRIBUTING.md edits leave lines of 110 and 112 characters beside 80-column prose;
  rewrap the two paragraphs.
- A-4 The foreign-key test hard-codes the expected foreign lists per type; a legitimate schema addition means editing the
  test. Acceptable (it is the architecture page's assignment), noted only.
- A-5 The time bound is release-only; `make test` / debug CI run the probes with the size and entry-count asserts only. The
  release wiring is in `make test-release` and the CI release job and is accurate.

## Checked, no issue
Makefile target and help text, CI step, AGENTS.md and CONTRIBUTING.md statements are accurate and contain no process wording.
okf-profile.md edits (block model sentence, fixtures bullet, bindings) are accurate against the code; no lint, test or gate
was weakened. The binary-search in `visible_content` relies on generated blocks being ordered and non-overlapping, which
the scanner guarantees; the output comparison above confirms it.

VERDICT: BLOCKED

## Recheck
- R-1 fixed: hand-written `Debug` (span, info, content, closed, `finish_non_exhaustive`) and `PartialEq`/`Eq` (span, info, closed,
  content) exclude the shared text. Mutation: on a scratch copy, replacing them with the derives makes
  `a_fenced_block_prints_and_compares_without_the_body_around_it` FAIL. The test passes on the working tree.
- R-2 fixed: the `>= 1` floor is skipped for `work/changes`; count equality still applies to all three directories.
- Advisories spot-checked: bindings and the cost bullet are in okf-profile.md; AGENTS.md/CONTRIBUTING.md longest added line is
  now 84 characters. The Q-2 mutation runs are the implementer's record and were not re-run.
- Gates re-run green: raf-okf tests (debug and release), docs_contract + repo_gates, workspace clippy, make reuse, fmt check.

VERDICT: PASS
