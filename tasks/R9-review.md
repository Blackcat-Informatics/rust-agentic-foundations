# R9 + G15 review

## Scope
Uncommitted change in /tmp/raf-par/p2-r9 (base e472918): `LoadErrorKind::NoRoots` (error.rs), the
empty-roots guard in `Workspace::load_with` (workspace.rs), `Found.declared: PackageDecl` carried
from discovery (discovery.rs, workspace.rs; G15), inverted test in discovery_targets.rs, and
source-index.md (binding, Package discovery paragraph, Verification line).

## Evidence
- Guard is the first statement of `load_with`, before `RelPath::new`, `Meter::new`, `discover` and any
  tree call: no IO precedes it. `load` delegates to `load_with`, so both are covered. Only
  `roots.is_empty()` triggers it; `["."]` still loads (asserted in the same test).
- Display text: "`.: no roots were given; name at least one, such as `.``" (RelPath root renders as `.`);
  accurate and actionable. Doc comments on `load` and `load_with` name the error.
- G15: `Found` now holds `declared` moved out with `parsed.package.take()`. No clone added. The only
  consumers of `manifest.package` after discovery were the removed `let ... else continue`; grep shows
  none remain (members.rs:234 parses a separate manifest). `descend` is computed before the take.
  The impossible branch is gone; behaviour is unchanged (all discovery tests pass unmodified).
- Falsification: `git archive e472918` copy plus the new test file fails to compile (no `NoRoots`);
  matches $STAGE/raw/R9-falsification.txt, which also records the runtime failure
  ("no roots is an error" on a successful empty Workspace). The test therefore fails on the base.
  Not a pre-correct test, so the mutation rule does not apply.
- Gates in the worktree: `cargo test -p raf-source --locked` 357 passed/0 failed; `--release`
  357/0; target test `an_empty_root_list_is_a_no_roots_error_beside_dot_which_loads` 1 passed;
  workspace clippy `-D warnings` clean; docs_contract 139 passed, repo_gates 55 passed;
  `make reuse` compliant; `cargo fmt --check` clean; `cargo doc -p raf-source` clean (intra-doc
  link resolves).
- Docs: binding added to tests list; "no default roots ... an empty list is a `NoRoots` error";
  Verification "no roots is an error". No issue numbers or process wording. All in-repo callers pass
  non-empty roots (grep of `Workspace::load`, tests, docs, fuzz, examples).

## Findings
REQUIRED: none.

ADVISORY:
1. workspace.rs `# Errors` line "Returns a [`LoadError`] for an empty `roots` list, a root that is
   missing or not a directory, a" is 103+ columns; rustfmt does not wrap comments, so no gate trips,
   but rewrap it for consistency with the surrounding 80-column prose.
2. 3-way apply risk (workspace.rs): hunks are at the doc comments (~86, ~105), the new guard after
   the signature (~116), and in the entry loop (`let Some(declared)` removal and the two `declared`
   uses, ~147-167). A concurrent resolver change that edits the loop's `Context { ... package: ... }`
   or the `package_info(...)` call arguments would conflict textually; resolver-only edits elsewhere
   in the file should apply cleanly. Re-run the target test and the Found-using paths after the merge.
   discovery.rs/error.rs/tests/docs hunks are independent of resolver work except source-index.md
   bindings (adjacent list lines) and the Verification paragraph.
3. `Found.declared` and `Found.manifest.package == None` is an implicit invariant (documented on the
   struct). Acceptable; nothing reads `manifest.package` afterwards.

VERDICT: PASS
