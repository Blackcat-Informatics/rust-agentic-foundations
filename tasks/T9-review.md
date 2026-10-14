# T9 review: Capabilities delivered (phase 2, staged on e6f3e9b)

## Scope
Staged diff (12 files): both capability docs, tests/docs_contract.rs, both change records deleted,
workplan-document-model and census-rule-api sentences, AGENTS.md, README.md, SECURITY.md,
CHANGELOG.md, crates/raf-source/src/error.rs. Read-only review; no code or docs modified.

## Evidence
- Read okf-profile.md and source-index.md in full (Contract, Usage, Examples, Constraints, Verification).
- 322 test-function names in delivery-checklist.md were each grepped as `fn <name>` under crates/ and tests/:
  all exist (the only non-hit, `no_io_in_source`, is a file name).
- Constants checked against code: MAX_TEXT_BYTES 1 MiB, MAX_YAML_DEPTH 32, MAX_BLOCK_NESTING 64,
  MAX_LINK_PART 4096 (okf limits.rs, body/link.rs); budget table 128/64/256/128/256/128/262144, 8 MiB default
  cap, PARSE_STACK_BYTES 64 MiB (raf-source budget.rs, options.rs, parse.rs); FUZZ_MAX_LEN 4096 (Makefile).
- Ran `cargo test -p raf-source --test overflow_harness composites -- --nocapture`: the five composite shapes
  and uncounted return, &mut, *const, fn() ->, a = abort; uncounted_move_closure is Refused. This matches the
  Contract ("committed composite shapes and the uncounted shapes that repeat return, &mut, ... end in that
  abort") and the Constraints ("most of the uncounted shapes"), in debug.
- Other claims spot-checked against code or tests (20): TreeError variant list; prelude table by edition
  (FromIterator/TryFrom/TryInto 2021, Future/IntoFuture 2024); glob syntax refusals (`**`, backslash, unclosed
  bracket, empty segment, one trailing `/` allowed); root items left out of by_source (index.rs:603) and root
  item fields (documented from file, test role CfgTest for Test/Bench only); test_fns selectors; Linux
  /proc/self/fd handle check, Unix dev/ino identity, portable fallback (disk.rs); crates-root `Ambiguous`
  test; delivered binds code+tests (profile/mod.rs:469); four states; `resource` rule on every type;
  4096/4097 link test; 1 MiB test; membership test (glob, class, exclude, not-listed); permutation test
  loads == 1440; cross-check targets (Apple and Windows).
- Stale grep (git grep over tracked, excluding crates and work): none of "designed, not yet implemented"
  for these crates, "`members` is empty", "designed — not yet published" for these crates, "binding waits
  for the source index", "will enforce the whole profile once", "Once implemented" (SECURITY.md) remain.
  Remaining hits are legitimate: AGENTS.md:54 (other four crates), README.md:29-32 (the four other crates),
  docs/architecture/okf-profile.md:174 "None exist yet." (about generated blocks, which truly do not exist in
  the repository: no `raf:generated` marker in any tracked document), AGENTS.md:194 and the architecture
  page line 40 (about raf-workplan).
- Staged-additions grep for tracker numbers, plan/task labels, process wording, and agent
  attribution (credit trailers, agent and vendor names, generated-with lines): no hits.

## Findings

1. (non-blocking) docs/capabilities/census-rule-api.md:41 states "`Rule: Sync`" but the trait block at
   line 31 is `pub trait Rule {` with no supertrait. A reader sees a sentence and a signature that disagree.
   Suggest changing the block to `pub trait Rule: Sync {` (same Contract section, same in-place edit allowed
   for a `designed` capability whose change record lists #contract; the record's item 1 claims the
   "documented signatures"). The sentence is a design statement with no executing test; the checklist row
   (checklist line 109) maps it to the parse-thread test, which is indirect. Acceptable for a `designed`
   capability with no crate yet, provided the block and sentence agree.

2. (non-blocking) crates/raf-source/src/error.rs:502 test name
   `a_parse_thread_that_cannot_start_is_a_spawn_error_naming_the_file_and_its_cause` overclaims. The test
   constructs `LoadErrorKind::Spawn` by hand and checks Display and `source()`. It does not make a spawn fail,
   so it does not prove parse.rs:62 produces Spawn (that mapping is only compile-checked). It is honest in
   what it asserts (file-prefixed message, "cannot start the parse thread", cause preserved), and the
   Contract sentence ("a thread that cannot start is a `Spawn` error") is adequately backed for a path that
   cannot be provoked portably, but the name should say "a_spawn_error_names_the_file_and_keeps_its_cause".
   The same name is bound in source-index.md bindings.tests; rename in both if changed.

3. (non-blocking) docs/capabilities/source-index.md Contract, module-path budget paragraph: "this repository
   records a few hundred" module paths is a measured-sounding figure with no test or gate behind it.
   Unchecked by me. Either drop it or leave it as illustrative; it is not a promised behaviour.

4. (non-blocking) The Contract's abort claim ("today ... end in that abort") is verified here in debug
   only; the harness itself accepts any of three outcomes, so nothing in `cargo test` keeps the sentence
   true if a budget later starts refusing those shapes. The doc already frames it as "today" and the
   harness tolerates both; not a gap against the delivered rules.

5. (non-blocking, wording) docs/capabilities/okf-profile.md Contract "Validate" list, second bullet: a
   mid-sentence line break leaves "A document otherwise sits directly in the directory of its type" on an
   over-long line; the sentence's "otherwise" follows a clause about the unknown type, which reads slightly
   oddly. No factual issue.

## Judged questions
1. Delivered rules: state `delivered`; no PROPOSED blocks in either doc; both change records deleted after
   the outcome commit; bindings.code and bindings.tests non-empty in both; every binding resolves
   (repository_bindings_resolve_in_the_source_index passes, orchestrator-reported). The chosen tests map to
   Verification sections (okf: parse refusals, bounds, alias, validation per rule, round trip, splice,
   index, markers, links, repo-markdown; source-index: containment, discovery, mod tree, prescan budgets,
   harness, corpus replay, caps, trait keys, crate roots, disk/memory equality). Not arbitrary. Gap noted:
   okf bindings do not list `bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`,
   which is the test of the very rule this change relies on; optional addition.
2. Docs match the build: no sentence found promising absent behaviour or a wrong number among ~32 checked
   claims (bounds, platform containment, trait keys and prelude, membership rule, crate roots, splice,
   placeholder rule via `Body::placeholder_lines` test, split links, glob syntax, order/determinism).
3. Two-way checklist: all named tests exist; 10 mappings read for assertion fit (1 MiB, 4096 link part,
   membership, alloc bound 8x, resource key, delivered binding rule, harness outcomes, containment platform
   bullets, root items, permutation count): real matches. Gate-backed rows (make fuzz, make cross-check,
   `missing_docs` lint) are acceptable because the Makefile targets and workspace lint exist and are named in
   the Contract/Verification; the `Rule: Sync` row is acceptable as a design sentence once finding 1 is fixed.
4. Stale strings gone for these two crates (see grep). Keeping AGENTS.md "Every other crate below is
   **designed, not yet implemented**" for the other four crates is correct and acceptable.
5. workplan-document-model and census-rule-api edits are in place, present tense, listed under #contract in
   their change records (both `affects` list #contract). Accurate except finding 1.
6. Spawn test: see finding 2.
7. CHANGELOG entries accurate and user-facing; AGENTS.md section 2 containment bullet matches
   source-index and disk.rs; README table and prose accurate; SECURITY.md wording accurate.
8. No tracker numbers, labels, process wording or attribution in staged additions.

VERDICT: PASS

## Orchestrator note (non-blocking findings applied before commit)

1. census-rule-api trait block now `pub trait Rule: Sync {`.
2. Spawn unit test renamed `a_spawn_error_displays_the_file_and_carries_its_cause` (binding and checklist updated) so its name claims only what it checks.
3. The unbacked "this repository records a few hundred" figure is removed.
4. The "today ... end in that abort" claims (source-index Contract and Constraints, SECURITY.md) now state the stable fact: no budget counts those shapes, so a file of one of them can end in an abort.
5. okf-profile Contract directory sentence split into its own bullet.
Optional: okf-profile tests bindings gain `raf_okf::test_crate::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests`.
docs_contract 137 (bindings resolve), repo_gates 55, raf-source lib tests, fmt pass. The PASS verdict above stands.

VERDICT: PASS
