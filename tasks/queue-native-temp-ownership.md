<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native temporary-directory ownership

Pinned `purrdf-testkit` supplies public `TempDir::new_in` and owned cleanup in
`crates/testkit/src/temp.rs`. Compatible workplan, CLI and root test/example
owners now use it. Source-lane owners remain assigned to their existing owner;
no `raf-source` source or helper was edited for this migration.

RAF wrappers retain fixture-specific file population, source loading, Git
isolation, Cargo configuration and snapshot methods. Shared fixture directories
and their Cargo target directories have separate native owners, both outside one
another. Canonical temporary parents preserve the existing canonical-path
guarantees of the root/Git fixtures and Unix CLI callers. There is no fabricated
`CACHEDIR.TAG`, fallback constructor, local collision retry or copied cleanup.
Domain tests that intentionally delete or replace files/directories retain those
operations; they are the tested behavior, not generic scratch ownership.

Ordinary temporary roots used solely as a command working directory or as an
elsewhere path do not create or own a temporary directory and remain path values.
The missing-directory and empty-search-path controls now have exclusive native
parents, so stale entries cannot turn their intended negatives into another case.
The process-wide empty Git home is a native owner retained for the process
lifetime; Rust does not run static destructors, so this is not claimed as
end-of-process cleanup evidence.

Native names read host time for uniqueness; no test measures or asserts that
reading. The provider exclusively creates its directories and refuses collisions.
Its ordinary cleanup failure is actionable, rather than the old ignored cleanup
result. There is no supplied public process/FIFO/pidfd supervisor harness in the
inspected native testkit APIs; RAF's domain runner proofs still need that OS
coordination.

The root manifests and lock wire the exact existing native revision. Testkit's
closure uses native hash, SHA-2, regex syntax and wasm-bindgen already in the
resolved graph; it has the workspace code license and needs no blanket license
allowance. Locked offline root and fuzz metadata both completed with exit 0;
`queue-native-temp-{root,fuzz}-metadata.json` retain their actual outputs.
`make deny` completed with exit 0 in `queue-native-temp-deny.log`: advisories,
bans, licenses and sources all passed. The fuzz workspace does not inherit the
root package's development-only testkit dependency.

Early real-caller compilation in `queue-native-temp-first-callers.log` refused
unused imports left by removal of the local counter. Those imports are removed.
The wider compile in `queue-native-temp-all-callers-current.log` refused four
remaining path accesses in the real runner's nested Cargo fixtures and empty-PATH
control. They now call the native `path()` accessor. Warnings-denied all-target
Clippy for workplan, CLI and root passed with exit 0 in
`queue-native-temp-all-clippy-settled.log`.

Sequential runtime batch 79368 ended with exit 101. Evidence units (20), evidence
integration callers (16), gate (18), root (6) and preview passed. Sync passed 15
tests and failed five exact generated-file/count expectations that omitted the
real auxiliary capability introduced by the earlier source-ownership fixture
repair. No native cleanup failure occurred. The expectations now include that
capability's exact generated path and contents, and the roadmap asserts both
delivered entries before checking distinct state ordering. Batch 1115 completed
with exit 0: sync 20, CLI completion 12, CLI evidence 14, CLI root 6 and CLI
verification 10. The final warnings-denied all-target Clippy passed too. Exact
logs are `queue-native-temp-sync-current.log`,
`queue-native-temp-cli-callers.log` and `queue-native-temp-final-clippy.log`.
The earlier preview target passed all 23 tests. These are focused caller and
dependency checks, not full gate, assessment or lifecycle acceptance.

Additional directly affected process fixtures passed with terminal exit 0 in
batch 39025: system-runner units 22, bound callers 18, execution callers 35,
Git callers 22 and the exact native RDF command registry 1. Logs are
`queue-native-temp-system-units.log`, `queue-native-temp-process-callers.log`
and `queue-native-temp-registry.log`. The registry preserves exact canonical
bytes and its existing independently pinned graph expectations despite the
new temporary-directory names.

After binding the genuine new evidence owners and running production docs sync,
the repaired document-contract target passed all 31 tests. Repository gates
passed all 72. The actual default Workplan check was clean, then self-workplan
failed on 59 stale or unaccepted assessments; this is not an accepted gate.
The fresh delivered-state candidate completed with exit 0 and retained exactly
97 findings, zero code/test gaps and 66 uncovered claims. Its eight package rows
all have zero orphan, excepted and debt lines. This measured snapshot precedes
the subsequent evidence publication-limit proof slice.
