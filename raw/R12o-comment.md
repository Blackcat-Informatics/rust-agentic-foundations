**R12o fixed: raf-okf tests, fenced-block accessors and validation cost** (commits 100bf2f, d89b78f, 671850b, f5ede44, pushed)

- **G4 / Q-3:** the unknown-key check runs on every document type. Each type gets an unknown key, a key that belongs only to another type, and unknown keys nested under `bindings` and Change items.
  - Mutation: adding a key to the Capability table makes the test fail.
- **Q-1:** the round-trip golden test asserts, per directory, that it rendered exactly as many documents as `git ls-files` tracks there.
  - It also asserts at least one document for capabilities and architecture. It does not for change records, which are legitimately deleted when they complete.
  - Two mutations, skipping a directory and dropping a file, each make it fail.
  - Three committed fixture documents, one per type, round-trip independently of the repository's own documents.
- **Q-2:** tests now pin a leading BOM, a NUL in the body and a `--- ` fence. The Contract is silent on all three, and each test has a recorded mutation that fails it.
- **G17:** `FencedBlock::content()`, which borrows from the shared body text with no copy per block, and `FencedBlock::is_closed()`.
  - `Debug` and equality cover only the block. The derived forms would have printed and compared the whole body; a test catches that.
  - `docs_contract` uses the accessors, and a `~~~rust` Usage block is now accepted. That test fails on the old code.
- **P-F2 / P-F3:** `Profile::validate` no longer scans quadratically. `changes` is held in a set, and generated blocks are found by binary search.
  - On the old code the release probes took 324 ms and 512 ms. They are now under the 100 ms bound, which `validate_cost.rs` asserts in `make test-release` and the release CI job.
  - The reviewer compared 47,564 violations over 15,102 mutated documents before and after: byte-identical.
- **Gates:** raf-okf tests 176 (debug and release), docs_contract 139, repo_gates 55, workspace clippy, rustdoc, `make reuse`, fmt, and the hook on each commit.
- **Review:** `tasks/R12o-review.md`, VERDICT: PASS after recheck. Two REQUIRED findings were fixed: the Debug and equality blow-up through the shared body, and a brittle floor on change records.
