# T4 review: bindings resolved through the source index

## Scope
Staged, uncommitted Task 4 on top of b6dff17 (14 files): `crates/raf-workplan/src/bindings.rs` (new), `graph.rs`, `finding.rs`,
`lib.rs`, `model.rs`, `Cargo.toml`, `Cargo.lock` (direct `proc-macro2` dependency), tests `bindings.rs` (new), `kinds.rs`,
`graph.rs`, `states.rs`, `documents.rs`, `support/{mod,cases}.rs`. Checked against plan Task 4, G7, G9, the R13/R19 row, the
document-model capability ("Binding resolution", Verification "Binding fixtures"), `$STAGE/tasks/T4-ported.md`, and the old
`tests/docs_contract.rs` at HEAD (`wrong_role`, `check_bindings`, `demo_workspace`, `bound`, the binding tests, and
`repository_bindings_resolve_in_the_source_index`). Gates and mutations ran on a `git checkout-index` copy of the staged state in
`/tmp/t4rev` (a throw-away git repo); the worktree was not touched and holds only the staged change plus this file.

## Gates on the staged state (all green)
- `cargo test --locked -p raf-workplan` debug and `--release`: all pass (bindings 17, kinds 4, graph 22, documents 26, states 16, ...; 3 doctests).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean. `make fmt`: clean. `make doc` (`-Dwarnings`): clean.
- `cargo test --locked --test docs_contract --test repo_gates`: pass (57 repo_gates). `make issue-refs`: pass.
- `make reuse`: compliant, 304/304. `make deny`: advisories, bans, licenses, sources ok. `make msrv` (rust-validate 1.98.1, root and fuzz): ok.
- Real-repository probe (scratch test over `DiskTree` of the staged tree, roots `["."]`, 21 capabilities): `Workplan::check()` returns 0 findings,
  so Task 7 can switch to this rule set without a new violation. No capability binds `examples` today.
- Added/changed lines contain no issue number, process wording, attribution or the forbidden domain (grep of the diff; `issue-refs` green). SPDX
  headers present on both new files.

## (1) Parity with the old gate
- `wrong_role`: `code` (any site not `is_shipping`) and `tests` (any site not `TestRole::TestFn`) are the same predicates; the three messages
  (`... does not resolve: <error>`, `... names an item that is test code or is not in a shipping target`, `... names an item that is not a
  function marked `#[test]``) are byte-identical, now asserted with `assert_eq!` for the role messages and the missing-path message.
  `examples` was resolve-only and is now role-checked (G9), intended.
- Every old binding test has a same-named successor (map in T4-ported.md). Differences: the macro-generated message is extended (intended);
  `a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found` was an exact-message assertion and is now
  `message.contains("`demo::body::FenceBlock` names nothing in the workspace")` (A8).
- Not ported, owned by Task 7: `repository_bindings_resolve_in_the_source_index` and its anti-vacuity guard (`resolve("raf_okf::Document")`,
  `resolve("raf_source::Workspace")` before judging). T7 must keep an equivalent guard (A9). Old roots were `["crates"]`; G7 says `["."]`.
- Nothing else the old gate enforced is missing: the by-state rules (`bindings` must be empty before delivery, delivered needs a code and a test
  binding) are raf-okf rules surfaced in states.rs, which this change keeps passing (`demo::real` replaces `src/lib.rs` in the fixtures).

## (2) Macro-generated detection and the dependency
- Semantics read from `invocation_naming`: tokens of the parent module's text; for an inline module the braced body; `name ! (group)` where
  the token before `!` is an identifier and the group contains the missing name as an identifier at any depth. `macro_rules! m {..}`, `#![..]`,
  `impl !Send` and `= !(..)` do not match; function bodies, nested inline modules and struct/enum bodies are not item level and are not read.
  Text that does not lex yields no hint (the finding stays `binding-unresolved`, so it is still an error).
- False positives (probed): `pub fn reall() {}  register!(real);` with binding `demo::real` gives `binding-macro-generated` although nothing is
  generated; so does `const N: usize = size_of!(real);` (an expression macro in an item initialiser is at item level). Severity: low, the finding is
  an error either way and the message names the invocation line, so the reader can see at once; but the message asserts "a generated symbol cannot
  be bound" as fact (A1).
- False negatives: derive and attribute macros (`#[derive(Builder)]` yielding `FooBuilder`), names built by concatenation (`paste!`) or given as a
  string, names inside a module a macro generates (parent does not resolve), and an `UnresolvedGlob` through a macro-generated module all stay
  `binding-unresolved`. Acceptable given "fail hard"; the capability text says only that a generated symbol cannot be bound (A2).
- Panic safety: lexing is only over text that already passed the raf-source pre-scan (probe: 128-level delimiter nesting is refused at load,
  `Budget::Delimiters`; 100 levels lexed and walked fine; a 1.4 MB, 200k-invocation file lexed). `text.parse::<TokenStream>().ok()?` turns
  lexer errors into "no hint". `text.get(range)?` is used for the body slice. No panic found.
- Boundedness: see R1. The lexing is linear in the module text per call but is repeated for every unresolved binding.
- Dependency: `proc-macro2 = { version = "1", default-features = false, features = ["span-locations"] }`, identical spec to raf-source's, version
  1.0.107 already in the lockfile (diff to `Cargo.lock` is only the new edge). `make deny` ok (licences MIT or Apache-2.0), `--locked` builds,
  MSRV ok. No new crate enters the build graph. Acceptable.

## (3) Doctest rule against Cargo and the index
- Cargo reference: `doctest` "is only relevant for libraries, it has no effect on other sections"; raf-source's `Target::doctest` documents the
  same ("The value is reported as the manifest writes it; cargo honours the key only for the library"). The implementer's rule (target kind
  `Lib` and `doctest` on) is therefore right and G9's "a target whose `doctest` is true" is wrong for `[[example]]`/`[[bin]]` tables that
  write `doctest = true`. Rustdoc also skips `#[cfg(test)]` code, so `item.test == TestRole::None` is right. Private and `#[doc(hidden)]` items
  do get doctests, and the rule does not exclude them (right). Both stricter conditions are pinned by fixtures
  (`example-doctest-key-on-a-non-library`, the `demo::unit::helper` row) and by mutations below.
- Plan text to correct (G9, `plan.md` line 42): replace "or a documented item (`documented == true`) in a target whose `doctest` is true" with
  "or a documented item (`documented == true`, not test code) in the library target when its `doctest` is true (cargo ignores the key on other
  targets)". The R13/R19 row's "a non-doctest example" should read "an example that is not an example target and not a documented library item".
  The capability's wording ("an example target or a documentation test item") and Task 9's reword of lines 98-106 should say the same (A3).
- Limits not modelled by the index, so not judged: a library whose `crate-type` excludes `lib`/`rlib`/`proc-macro` runs no doctests; and a
  documented item need not hold a fenced code block, so `documented` is a necessary, not a sufficient, condition. Both belong to the complete-gate
  capability that G9 names (A4).

## (4) Examples bindings, macro exports, ambiguity, mixed sites
- Crate-root path: `demo::example_crate::my_ex` (hyphenated example `my-ex.rs`) resolves and is `ExampleKind::Target`; the library root `demo`
  is a `Doctest` item when its crate docs exist and `binding-wrong-role` when they do not. Any other module inside an example crate is not a
  target (kind Mod and path equal to `crate_path`; see R2 for the untested arm).
- Macro-exported items: `#[macro_export] macro_rules! m` at the root resolves for `code` and, with docs, for `examples` as a `Doctest`; a
  `#[macro_export]` macro inside `pub mod a` resolves at `demo::inner_m` (the crate root, as rustc exports it) and not at `demo::a::inner_m`
  (`binding-unresolved`), which is correct.
- Ambiguity: two globs bringing the same name give `binding-ambiguous` naming both definitions with `file:line`; the arm is killed by tests.
- Mixed sites (`#[cfg(test)] fn a` beside `#[cfg(not(test))] fn a`, or a documented and an undocumented twin): every site is judged and one
  failing site fails the binding (`any` in all three arms). That is the precedent's choice and is sound (a binding is a claim about the symbol),
  but no test pins it (R2).

## (5) Falsifiability
Mutations were applied one at a time to a copy of `bindings.rs` and `cargo test -p raf-workplan --tests` was run (control mutation, a no-op,
survived as it must).

| Mutation | Result |
|---|---|
| `Ambiguous` finding kind changed to `binding-unresolved` | killed |
| `Ambiguous` arm disabled (falls to `not_found`) | killed |
| macro hint: inline module body not read (`None`) | killed |
| macro hint: any file reached by any module counts (`.any(|_| true)`) | killed |
| macro hint: base line offset dropped | killed |
| `target.kind == Lib` dropped (doctest on any target) | killed |
| `target.doctest` dropped | killed |
| `item.test == None` dropped from the doctest rule | killed |
| `item.documented` dropped | killed |
| example target by `kind != Lib` | killed |
| `tests` fix text shortened | killed |
| macro hint: `names` not recursive (`Group(_) => false`) | SURVIVED |
| macro hint: identifier before `!` not required (admits `#![cfg_attr(x, name)]`) | SURVIVED |
| `any` to `all` in the `code`, `tests` and `examples` arms (three runs) | SURVIVED (3) |
| example root: `path == crate_path` removed (any module of an example crate is a target) | SURVIVED |
| macro hint: parent `kind == Mod` guard removed | SURVIVED (A5) |
| example root: `kind == Mod` removed | survives, equivalent (only the root has `crate_path`) |

## Findings

### REQUIRED
- **R1. The macro hint re-lexes the whole parent module for every unresolved binding, with no bound on the product.** `macro_site` runs per
  `NotFound` finding and `invocation_naming` parses the module's full text each time. Measured (release): a 1.4 MB root file and 300 unresolved
  `code` bindings take 19.4 s and peak at 1.06 GB resident, against 0.5 s and 233 MB for one binding. The memory is real retention: with
  `span-locations`, proc-macro2 records every parse in a thread-local source map that is never freed, so cost grows per lex, and a long-lived
  caller that checks repeatedly (a watcher, the CLI) keeps growing. A hostile or merely large document that lists many wrong paths slows the
  gate in proportion. Fix: memoise within one `check` (a map from module path or file to the list of `(identifier, line)` pairs of its item-level
  invocations, built at most once per module and per check), and look the name up in it. Add a test (many missing bindings under one big
  module, asserting the lex count or the time bound) or at least a unit test of the memo.
- **R2. Tests do not pin four behaviours of the rules, and the review mutations survive.** Add fixtures in `tests/bindings.rs`:
  (a) a macro argument that holds the name only inside a nested group (`make!(wrap(generated))`, `make! { fn generated() {} }`) gives
  `binding-macro-generated` (kills the non-recursive `names`); (b) a file starting `#![allow(generated)]`, and a `const A: bool = !(generated);`,
  give `binding-unresolved` (kills the identifier-before-`!` mutant; the second also documents that an expression `!` is not an invocation);
  (c) a binding that resolves to two sites, one failing (`#[cfg(test)] pub fn a` twin for `code`; a non-`#[test]` twin for `tests`; an
  undocumented twin for `examples`) gives `binding-wrong-role` in each list (kills `any` to `all` in the three arms) and states the chosen rule;
  (d) a bound `pub mod` (or other module) inside an example crate gives `binding-wrong-role` (kills the dropped `path == crate_path`).
- **R3. `CHANGELOG.md` is not updated.** Each earlier raf-workplan commit adds the kinds it introduces to the `raf-workplan` entry
  (13 kinds in the previous one). This change adds `binding-unresolved`, `binding-ambiguous`, `binding-wrong-role`,
  `binding-macro-generated`, `Graph::bindings_of`, `Binding::example_kind` and `ExampleKind`; the entry still says only that the graph holds
  "the symbols, tests and examples the bindings resolve to". Add the sentence (including the examples role and the macro hint).

### ADVISORY
- **A1.** The hint over-claims: "the macro invocation at X names it, and the index does not expand macros, so a generated symbol cannot be bound".
  Use "may generate it" (it is a name match, not an expansion). Consider ignoring an invocation whose macro is an expression-position one only if the
  parser could tell; it cannot, so hedge the wording instead.
- **A2.** Derive and attribute macros, `UnresolvedGlob` (whose own docs say a macro-generated module is globbed this way) and `UnresolvedSelf` all get
  the generic `binding-unresolved` fix text, which does not mention macros. A sentence in the fix ("or the name is generated by a macro or derive")
  would cost nothing.
- **A3.** Correct G9 and the R13/R19 row as in section (3); fold the library-only and not-test-code precision into the capability's Binding
  resolution paragraph when Task 9 rewrites lines 98-106.
- **A4.** Document in `ExampleKind::Doctest` and in the complete-gate capability that `documented` neither proves a fenced block nor `crate-type`
  doctestability; the rule is an index fact, the run is the gate.
- **A5.** The parent `kind == Mod` guard in `macro_site` is untested: without it a function parent (`demo::f::made` with `make!(made)` inside `f`'s
  body) would be hinted. Add one row.
- **A6.** The "macro-generated" fixture rows in `binding_cases()` assert kind, file and location only; give `missing-example-target` a message
  assertion naming the capability, list and binding (R13: "each failure names the capability, the list and the binding").
- **A7.** `check` finds the capability of each binding by a linear scan of `capabilities` (O(bindings x capabilities)); an index by id is trivial.
- **A8.** Restore the full-message assertion in `a_binding_to_an_item_under_the_module_that_declares_its_file_is_not_found`
  (`assert_eq!` on `cap:gamma code binding ... does not resolve: ... names nothing in the workspace`), as the precedent had.
- **A9.** Task 7 must keep the anti-vacuity guard of `repository_bindings_resolve_in_the_source_index` (known symbols resolve before the verdict is
  trusted) in the disposition table.

## Verdict
The staged change reproduces every rule of the old gate, adds the examples role correctly for Cargo's semantics, resolves macro exports, re-exports
and ambiguity as intended, and passes every gate (tests debug and release, clippy, fmt, doc, docs_contract and repo_gates, reuse, deny, msrv).
It is blocked on R1 (unbounded repeated lexing and retained memory), R2 (survivors in the macro hint, mixed-site judgment and example root), and
R3 (CHANGELOG), all small.

VERDICT: BLOCKED

## Recheck

Re-run on a fresh `git checkout-index` copy of the re-staged change (15 files, adds CHANGELOG.md; `/tmp/t4rev`). The worktree was not touched.

- **R1 (cost and memory): resolved.** The same probe (1.4 MB root file of 200k invocations, 300 missing `code` bindings, release) now takes
  0.57 s with a 232 MB peak, against 19.4 s and 1.06 GB before (and 233 MB for a single binding earlier: the cost is now flat in the number of
  bindings). `Macros` lexes each module once per `check` and answers by lookup. The new unit test
  (`the_invocations_of_a_module_are_listed_once_however_many_bindings_miss`: 300 missing names, 2000-call module, `lexes == 1`) is falsifiable:
  turning the cache off (`if true`) fails it. Residual, accepted: one lex per module per `check` still adds to proc-macro2's thread-local
  source map, bounded by the module texts the load already holds.
- **R2 (survivors): resolved.** Re-run, all KILLED, each by a single failing test of `tests/bindings.rs`: non-recursive argument search; the
  identifier-before-`!` guard dropped; `any` to `all` in the `code`, `tests` and `examples` arms (three runs); `path == crate_path` dropped from
  the example root; and the earlier advisory mutant (parent `kind == Mod` guard dropped).
- **R3 (CHANGELOG): resolved.** The `raf-workplan` entry names `Graph::bindings_of`, `Binding::example_kind`/`ExampleKind`, the four kinds, the
  roles, the several-definitions rule and the macro hint.
- **Advisories:** A1 (hedged message "which may generate it", full-message assertion), A2 (fix text names macro, derive or attribute), A3 (G9
  reworded in plan.md; correct), A5 (parent-is-a-module test), A6 (missing example target asserts the whole message), A7 (map by id), A8 and A9
  (anti-vacuity note in T4-ported.md) are done. A4 is covered in the `ExampleKind::Doctest` doc comment.
- **Gates (all green):** `cargo test --locked -p raf-workplan` debug and `--release` (bindings 23 plus the lib unit test, kinds 4, graph 22,
  documents 26, states 16, 3 doctests); `cargo clippy --workspace --all-targets --locked -- -D warnings`; `make fmt`; `make doc`;
  `docs_contract` and `repo_gates` (57); `make issue-refs`; `make reuse` (304/304); `make deny`; `make msrv` (root and fuzz). The diff adds no
  attribution, issue number or process wording.

VERDICT: PASS
