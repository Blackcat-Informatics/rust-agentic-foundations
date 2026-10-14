# Historical fit: issue 2 / PR 13 (raf-okf + raf-source), Stage 2

Branch `paudley/2-shared-crates-raf-okf-raf-source` at 6c744af against `origin/main` (2c41082).
`W` is the worktree. Evidence is read from the worktree and the two local precedent checkouts. No
file other than this one was written and nothing was committed.

Precedent checkouts found locally (read directly, not only through the assessment):

- `/home/paudley/Active/katamari/crates/katamari-helper-census/src/source.rs` (1438 lines)
- `/home/paudley/Active/purrdf/crates/helper-census/src/source.rs` (1367 lines)

Line cites into the precedents match `prior-art-assessment.md` (for example the `TEST_TARGETS` table at
purrdf 50-53, `walk_mod` at katamari 560-592, `roots` / `test_roots` at katamari 1157-1258). I re-read
those spans and found no disagreement with the assessment.

## 1. Precedent fit

Verdict: the port preserves the precedents' observable behaviour where the precedent matched rustc, and
deliberately diverges where the precedent was a census-internal walker or was wrong. Every divergence
traces to the recorded assessment lessons L1-L14 or to a user decision. Exceptions are listed under
"Divergences without a recorded user decision" below.

### Preserved

| Behaviour | Precedent | Port | Verdict |
|---|---|---|---|
| Test-target naming `<pkg>::{test,bench,example}_crate::<target>`, `-` to `_` | purrdf 50-53, katamari `test_roots` 1214-1258 (`format!("{package_ident}::{kind}::{}", ...)`) | `crates/raf-source/src/target.rs:285-295` (`test_crate`, `bench_crate`, `example_crate`), `ident()` on package and target | Preserved. Issue text names the format; `docs/capabilities/source-index.md:256-258` states it. |
| Build script crate `<pkg>::build_script` | katamari 1201-1206 | `target.rs:367-380` | Preserved. Now also a documented, shipping target. |
| Auto-discovery of `*.rs` and `*/main.rs` under tests/benches/examples/src/bin | katamari 1214-1258 | `target.rs:296-320` (`infer_from`) | Preserved. |
| Explicit `[[test]]`/`[[bench]]`/`[[example]]` default path `<dir>/<name>.rs` | katamari 1246-1258 | `target.rs:322-352` also tries `<dir>/<name>/main.rs` (Cargo's rule) | Preserved and extended to Cargo's two candidates. |
| Flat-then-`mod.rs` lookup, nested directory, `#[path]` file's children at its parent directory | katamari 574-592 | `modtree.rs:1-20` rules; `tests/module_tree.rs` fixtures at 51, 166, 178, 253 | Preserved (assessment confirmed these were correct). |
| Test-only predicate matrix (`all(test,x)`, `any(test,test)` test; `any(test,unix)`, `not(test)` not) | katamari 979-1013, test 1305-1330 | `cfg.rs:33-40` (`implies_test`) | Preserved; the action is inverted (tag, do not drop) as the issue requires. |
| `use` alias table: renames, groups, globs, `self`, `crate`/`self`/`super` absolutise | katamari 594-648, 785-818 | `facts.rs` use extraction, `resolve.rs`, `index.rs` | Preserved and extended with `use ::x` and `extern crate x as y` (L9). |
| BFS with a `seen` set as the cycle guard | katamari 820-859 | `resolve.rs`; fixture `tests/use_resolution.rs:268` | Preserved, silent 4096 cap removed (L8). |
| Sorted, `BTreeMap`-based determinism | katamari throughout | `modtree.rs:77` (`BTreeMap`), sorted `items()` | Preserved. |
| Raw identifiers: file name strips `r#` | katamari 565-567, purrdf 543-550 | `facts.rs:359` `unraw`; fixture `tests/module_tree.rs:370` | Preserved. |
| `Tree` seam (`read`, `list`) with disk and memory backends | purrdf 57-122 = katamari 67-132 | `tree/mod.rs:40-83`, `tree/disk.rs`, `tree/memory.rs` | Seam kept; signatures changed (D1). |

Note on the precedent `Tree`: it is `pub(crate)` in both census crates and byte-identical between them
(assessment 1.2). The port makes it public API with `non_exhaustive` `EntryKind`.

### Deliberate divergences, and whether each is recorded

| # | Divergence | Cite | Driver | Recorded? |
|---|---|---|---|---|
| D1 | `Tree` returns `Result<_, TreeError>`, takes `&RelPath` and `max_bytes`, `list` returns `Entry{name,kind}` with `EntryKind::{File,Dir,Symlink,Other}`, `exists` replaced by `entry() -> Result<Option<EntryKind>>` | `tree/mod.rs:40-83` vs precedent `Option<String>` / `Vec<(String,bool)>` | L5 (Disk swallows errors, follows symlinks, opens FIFOs, no cap) | In the capability contract and assessment 1.1 "Adapt". Not a user decision. The capability says "test existence" (`source-index.md:27`); the shipped method is `entry`, a superset. Acceptable. |
| D2 | `DiskTree` canonicalises the root once, opens non-blocking with `O_NOFOLLOW`, `fstat`s the handle, verifies the handle against the root | `tree/disk.rs:221,239` | L5, L4; contract text | User decision 3 (per-platform containment) plus capability contract. Justified. |
| D3 | Path escape is refused with a typed error, never clamped or recorded as a finding | `tests/module_tree.rs:510`, `tests/workspace_containment.rs` | L4; issue "a `#[path]` escape that is rejected" | Issue acceptance criterion. Justified. |
| D4 | `#[path]` resolution follows rustc: inside an inline module relative to the module directory; on an inline module replaces the component; `cfg_attr(pred, path=..)` is another site; two unconditional `path` attributes are an error | `modtree.rs:6-20`; fixtures `tests/module_tree.rs:206,229,275,299` | L1, L2, L7 | Plan C6; dictated by "mod-tree following as rustc does it" in the issue scope. Justified. |
| D5 | Both `name.rs` and `name/mod.rs` is an error naming both files | `tests/module_tree.rs:85` | L3 (rustc E0761) | Plan C6. Justified. |
| D6 | Module cycle detected on the active-file stack, error names both files | `tests/module_tree.rs:388`, `modtree.rs` (`active`) | assessment row "Module cycle" | Plan C6. Justified. |
| D7 | Manifests read with the `toml` crate under the byte cap, replacing the line parser | `crates/raf-source/Cargo.toml:17`, `manifest.rs` | L6 | Plan C5; contract requires it. Justified. |
| D8 | `resolve` returns `Result<Resolution, ResolveError>` with `NotFound`/`Ambiguous`/`Cyclic`, glob reported, no cap | `resolve.rs`; `tests/use_resolution.rs:268` | L8 | Plan C7. Justified. |
| D9 | Definitions with the same path, namespace and kind are one symbol with several sites (cfg twins); real conflicts are `Ambiguous` | plan C7 | L8 (first-wins `or_insert`) | Plan C7. Justified. |
| D10 | Inherent impl on an unresolvable self type is an error (`InherentImplSelf`), not a `module::Last` fallback | plan C7 impl rules, `impls.rs` | assessment row "Impl placement settle" | Plan C7; matches the fail-hard doctrine. Justified. |
| D11 | Explicit `roots` argument; no `["crates"]` default | `workspace.rs:65-75`; the consumer passes `&["crates"]` at `tests/docs_contract.rs:912` | Issue says "(default `crates/`)" | **User decision 2**, recorded at `plan.md:87`. |
| D12 | Census policy not ported (exclusions of tests/benches/examples/generated, fingerprints, `scan_rules` over unreachable files) | whole crate | assessment rows "Do not port" | Issue scope ("index of every named item"). Justified. |
| D13 | Raw-identifier spelling: symbols are stored unraw (the katamari spelling) and `resolve` strips `r#` per segment so both spellings resolve | `facts.rs:359`, `resolve.rs:110-111,496`; `source-index.md:448` | assessment 1.2: the two precedents disagree; RAF must pick and accept both | Chosen in the capability text and plan C7; not a user decision. Follows the assessment's recommendation. Acceptable. |
| D14 | `syn = "3"` with `printing`, plus `quote`; the precedents pin `syn = "2"` | `crates/raf-source/Cargo.toml:15-16`; `Cargo.lock` syn 3.0.8 | `quote::ToTokens` renders written self types (`facts.rs:12`) | The assessment advised dropping `printing` and avoiding `quote`. The branch keeps both. Not recorded as a decision anywhere I found. Low risk; extra graph surface for `cargo deny`. |

### Divergences without a recorded user decision (flag for the gap analyzer)

1. **Binary crate path is `<pkg>::bin_crate::<bin>`.** Katamari roots a binary at the bare identifier
   (`ident(&name)`, katamari 1169-1186). The issue names only `<pkg>::test_crate::<target>` "etc." and
   `build.rs`; `origin/main`'s `source-index.md` never mentions `bin_crate`. The format enters through plan
   C5 (`plan.md:300`) and was exercised by the completion adversary
   (`reviews/completion-adversary.md:198-202`, F-6). It fixes a real collision (a lib and a bin both named
   `demo_a` share a path in the precedent; fixture `tests/target_roots.rs:127-170` shows the collision and
   its resolution). It is consistent with the `test_crate` family. But it is not one of user decisions
   1-5, and it changes every binding that names a binary item. It is a plan-recorded divergence that
   needs the owner's visible sign-off in the PR body.
2. **`Rule: Sync` was added to `docs/capabilities/census-rule-api.md`** (hunk around lines 31-41,
   `plan.md:560`). That edits the contract of a different capability (`raf-census`, not delivered). It
   follows from the dedicated parse thread in `raf-source`, but it constrains a crate that does not exist
   yet and was never a user decision. See R5.
3. **Items beyond the issue text** that the owner's handoff already lists: canonical trait keys,
   Cargo-accurate workspace membership (`members`/`exclude` globs, implied path-dependency members),
   crate-root module items. Disclosed, not decided by the user.

## 2. Doctrine compliance (AGENTS.md / CLAUDE.md)

Method: grep over `git diff origin/main...HEAD` and over tracked files at HEAD. `make check` was run by
the author on 6c744af and passed (`validation.md`); I did not rerun it.

| Rule | Evidence | Result |
|---|---|---|
| No `#![feature]` | grep `#!?\[feature` over the diff: 0 hits; over `crates/ fuzz/ tests/` (`*.rs`): 0 hits | Pass |
| No `unsafe` | grep `unsafe ` over `crates fuzz tests`: 0 code hits (`unsafe_code = deny` stays; `libc` supplies open-flag constants only: `crates/raf-source/Cargo.toml:20-22`, `tree/disk.rs:221,239`, `error.rs:112`) | Pass |
| Workspace lints not weakened | `diff` of the `[workspace.lints.*]` tables, `origin/main:Cargo.toml` against `W/Cargo.toml`: identical. `fuzz/Cargo.toml` lint tables equal the root's, and `tests/repo_gates.rs` (`lints_fuzz_workspace_tables_equal_the_root_tables`, `lints_every_workspace_manifest_inherits_the_workspace_lints`, `lints_rejects_a_weakened_level_in_the_fuzz_tables`) enforces both with negative fixtures. Both new crates carry `[lints] workspace = true` (`crates/raf-okf/Cargo.toml:19-20`, `crates/raf-source/Cargo.toml:24-25`) | Pass |
| Every `#[allow]` carries a reason | 4 sites, all with `reason =`: `facts.rs:569`, `index.rs:273`, `tests/support/mod.rs:6`, `tests/symbol_index.rs:55`. `allow_attributes_without_reason` stays denied | Pass |
| Fail hard, no fallback | Typed errors throughout (`TreeError`, `LoadError`, `ResolveError`); no `.ok()`, `unwrap_or_default` or silent cap in `crates/*/src` (the only `unwrap_or*` hits, `target.rs:214,219` and `index.rs:329`, apply Cargo's own defaults or a rename, not an error swallow). Panics from the parse thread are re-raised (`parse.rs:24`). One explicit, user-approved portability fallback: non-Linux containment (decision 3, AGENTS.md "Path containment is per platform"). A residual stack-overflow abort is a stated outcome (`SECURITY.md` Bounds), not a silent pass | Pass, with residual risks R1 and R2 |
| No issue/PR numbers or process wording in committed files | Regex over every tracked file (corpus and `Cargo.lock` excluded): 0 `#NNN`, `issues/N`, `pull/N`, `PR N` hits outside the gate's own fixtures. Process-word grep over added lines hits only ordinary English (`later`, `temporary directory`, `silently` in prose describing refusal). The authoritative gate `make issue-refs` passed per `validation.md` | Pass |
| Naming: RAF in prose, `raf-*` / `cargo-*` for crates | Added-line grep for `Raf`, `Raf-`, `RAF-okf` forms: 0 hits except the env var `RAF_GATE_ROOT`, a legitimate constant. Crate names: `raf-okf`, `raf-source`, `raf-source-fuzz` | Pass |
| No attribution | `git log origin/main..HEAD` bodies: 0 co-author trailers, 0 generated-by lines, 0 agent names. Diff grep for those words finds only the Cargo.lock banner and a test string | Pass |
| No deferrals | Both capabilities moved to `state: delivered` (`docs/capabilities/okf-profile.md:8`, `source-index.md:8`); both change records removed (`work/changes/` holds none for them); AGENTS.md and README.md status flipped | Pass on form; see R3 for the evidence behind `delivered` |
| `.deficiencies` untouched | not in the 144-file list | Pass |
| Census fingerprint / `normal_form` | no fingerprint hashing in this diff | N/A |
| MSRV 1.98.1, SPDX headers, REUSE | `make msrv` passed including the fuzz manifest; `REUSE.toml` gains aggregate annotations for `fuzz/Cargo.lock` and `fuzz/corpus/**`, each with a stated reason | Pass |

Doctrine note: `fuzz/Cargo.toml` copies the whole `[workspace.lints]` block because the fuzz crate is a
separate workspace. That is a duplicated table, not a weakened one, and it is policed by a gate. It is
the only structural deviation from "one lint table".

## 3. Recurring failures

Counted from `git log origin/main` (24 commits). The only `GhpRsq-Defect-Class` trailer is `none` on
`2c41082`, so the trailer tally is 1 and says nothing about class recurrence. Recurrence below comes from
fix-commit subjects and diffs.

| Past defect class | Evidence on main | Does this branch repeat it? |
|---|---|---|
| Issue-reference gate evasion (a `#` after a word character, `Fixes:#123`) | `45bc98e` | No. The banned-line logic was not weakened; `tests/support/mod.rs` has a 2-line diff; `make issue-refs` passes. |
| Frontmatter-subset hardening gaps (unterminated quotes, unquoted `: `, indicators, comments, markers, FIFOs, symlinks) | `dba307f` (6+ classes in one commit) | The same grammar is re-derived in `raf-okf`; the negative corpus was carried over (docs_contract tests 114 to 137, 2 renamed, ~23 new; the rest moved to `crates/raf-okf/tests`). Parity decisions are listed in `validation.md` "Task 2". Residual risk: parity rests on fixtures, not a differential run of old against new (R4). |
| Contract text contradicting itself or lagging the build | `edf93d5`, `80c14d7`, `6120029` (spec hardened before any code existed) | Partly repeats, in miniature: the branch needed two follow-up fixes after the feature commits, `52155fa` (inherit workspace keys) and `e6f3e9b` (crate root as a module item), each because the resolver did not match the contract the plan promised. Both were found in review, fixed with fixtures, and the doc was finalised after the code (`6c744af`). Same class as the earlier doc-before-code drift. |
| Link to a tracked symlink | `edf93d5` | Not repeated; raf-okf extracts links and leaves resolution to `docs_contract`. |
| Assertion that passes if resolution is broken outright (katamari `unfalsifiable-check` x15) | assessment section 2 | Mitigated: bindings tests first resolve known symbols (`tests/docs_contract.rs:913-916`), and `module_tree.rs` / `use_resolution.rs` pair negatives with positives. A residual weak spot from `validation.md` "Open findings": a golden count assertion is a lower bound (`> 20`). |
| Katamari/purrdf raw-identifier divergence (2 fixes, 2 spellings) | assessment | Addressed (D13). |
| Target coverage patching (tests/bench/example targets missing, double counting) | purrdf `7a5d95a89`, `9ecc73c28` | Addressed: all six target kinds, shared-root error (`target.rs:420-436`), one parse per file with several paths (`tests/module_tree.rs:414`). |

Within-branch pattern: the resolver (`resolve.rs`, `index.rs`) was rewritten more than once under review
(owner handoff, "least confident" item 3), and the hostile-input pre-scan was found incomplete twice
(evasion paths) and is still incomplete by design. Those are the two places a future defect is most likely.

## 4. Consumers and blast radius

### Shipped callers of the new crates (on the branch)

| Caller | Evidence | Nature |
|---|---|---|
| Root package `tests/docs_contract.rs` | `Cargo.toml:15-17` (path dev-dependencies with `version`), `tests/docs_contract.rs:26-30` (`raf_okf::{...}`, `raf_source::{DiskTree, MemoryTree, Workspace}`), `:912` (`Workspace::load(&DiskTree::new(&root)?, &["crates"])`), `:3190-3200` (doctest pairs for the two capabilities) | Reads every Markdown document through `Document::parse` / `Body::parse` / `Profile::validate`; resolves capability bindings through the symbol index. The file shrank 3851 to 3481 lines. |
| `crates/raf-okf/tests/repo_markdown.rs` | `:9`, `:55` | Runs `Body` over the repository's Markdown for split links. |
| `crates/raf-source/tests/this_repository.rs` | in diff | Indexes the repository itself. |
| `.githooks/pre-commit` (unchanged) | runs `repo_gates` and `docs_contract` on the staged snapshot via `RAF_GATE_ROOT` | Transitively exercises both crates on every commit; hook compile time now includes `syn`. |
| Fuzz harness `fuzz/fuzz_targets/parse_source.rs` | `fuzz/Cargo.toml` dependency on `raf-source` | Separate workspace; consumes `parse_source`. |

No other crate depends on either one yet; `raf-census`, `raf-workplan` and `cargo-census` are still
`designed`. `raf_okf::Document` and `raf_source::Workspace` are code bindings of the two capabilities and
`docs_contract` resolves them (`tests/docs_contract.rs:913-916`), so the repository binds its own
documents to its own source index. Renaming either type fails `cargo test` at the root.

### Gate and CI changes

| Change | Cite | Impact |
|---|---|---|
| `make check` gains `test-release`, `fuzz`, `cross-check` | `Makefile:50,76,86,99-113` | New required tooling: `cargo-fuzz` 0.13.2 and rustup targets `x86_64-apple-darwin` and `x86_64-pc-windows-gnu`. A missing tool or target is a hard failure by design (`Makefile:77-78,87-90`), consistent with AGENTS.md "a missing tool is a hard failure". AGENTS.md and CONTRIBUTING.md document both. |
| `make deny` now `--workspace` | `Makefile:60-62`, `ci.yaml:139` | Fixes a pre-existing narrowing: before this branch `cargo deny check` saw only the root package, which had no dependencies. A tightening. |
| `fuzz/` is its own workspace, excluded from the root | `Cargo.toml:25` (`exclude = ["fuzz"]`), `fuzz/Cargo.toml` | The fuzz graph is not covered by root `make deny`; only `make fuzz` (`cd fuzz && cargo deny`, using `fuzz/deny.toml`) covers it. |
| `make fmt`, `make msrv` cover `fuzz/Cargo.toml` | `Makefile:37-38,68-74` | Documented in AGENTS.md and CONTRIBUTING.md. |
| New CI jobs and steps | `ci.yaml:71` (`test-release`), `:86` (`fuzz`), `:106` (`cross-check`), extra steps in `fmt` and `msrv` | Same SHA-pinned action style as existing jobs; `fuzz` installs `cargo-deny@0.20.2` and `cargo-fuzz@0.13.2` with `--locked`. **Hosted CI has never run on this branch** (`validation.md` "Hosted CI": the PR's jobs failed to start on an account billing block; main also fails). All new jobs, and the `fuzz` job's `FUZZ_DENY=--locked` path (advisory-database fetch), are unobserved. |
| REUSE | `REUSE.toml` +15 lines | Two aggregate annotations; the corpus files are exact byte inputs, so headers would change the tests. Justified in the file. |
| `repo_gates` extensions | `tests/repo_gates.rs:384` (`deny_problem`), `:881-906` fixtures | New gates: fuzz lint tables equal root; every workspace member inherits workspace lints; `fuzz/deny.toml` equals `deny.toml` plus exactly the NCSA exception. Real gates with negative fixtures. |

### Licence exceptions against `deny.toml`'s stated policy

`deny.toml:9-11` (policy): "The list is deliberately short: an addition should require someone to look
at the new crate and decide, rather than pass silently." The allow list
`["MIT","Apache-2.0","MulanPSL-2.0"]` (`deny.toml:12`) is **not widened**. The branch adds crate-scoped
exceptions instead:

- Root `deny.toml:20-26`: `unicode-ident` allowed `Unicode-3.0`. `unicode-ident` is
  `(MIT OR Apache-2.0) AND Unicode-3.0`, so the exception is unavoidable once `proc-macro2` (hence `syn`)
  is in the graph. It is scoped to one crate and carries a reason comment. That matches the policy's
  intent (a visible, reasoned decision), and the assessment (1.1) flagged it as a blocker needing "a
  stated decision". The stated decision is the comment plus the owner handoff asking for confirmation.
  **The owner has not yet confirmed it**: the handoff says "your merge is that look". Acceptable if the
  PR body quotes the governing lines, which `plan.md:570` promises.
- `fuzz/deny.toml:28-34`: `libfuzzer-sys` allowed `NCSA`. Scoped to the harness graph, which is not
  shipped, with a reason comment. Fits the policy; the assessment predicted it and offered "allow NCSA
  explicitly" as an option. The gate "equal to root plus one exception" stops it growing silently.
- `fuzz/deny.toml:20-26` repeats the `unicode-ident` exception because it is a copy of the root file, as
  `deny_problem` requires.

`multiple-versions = "deny"`, `wildcards = "deny"`, `unknown-registry = "deny"` and
`[graph] all-features = true` are unchanged. No advisory `ignore` entries were added.

## 5. Risks for the gap analyzer (ranked)

R1. **Hostile Rust input can abort the process, and the docs call that acceptable.** The pre-scan covers
five recursion shapes (`budget.rs:36-41`); composite shapes and uncounted repetitions (`return return ...`,
`&mut &mut ...`, `a = a = ...`) overflow the 64 MiB stack (`validation.md` Task 5; owner handoff item 1;
`SECURITY.md` Bounds). The fuzz run is capped at `-max_len=4096` (`Makefile` `FUZZ_MAX_LEN`) so the large
committed inputs are never fuzzed in process; they are judged only by `corpus_replay.rs` in child
processes. Check that the three-outcome harness cannot classify an unexpected abort as a pass, that every
committed corpus file has a classified outcome, and that `make fuzz` (5000 runs, fixed seed) is not a
no-op on the hostile shapes. Any future consumer that loads untrusted source gets aborts, not typed
errors. Neither precedent had any bound, so this improves on them, but "never a wrong index" rests on a
bounded guard, not a proof.

R2. **Non-Linux containment is compiled but never run.** `cross-check` is only `cargo check` for Apple
and Windows. The "other unix" and "non-unix" verifiers are std functions unit-tested on Linux (decision 3),
but nothing runs on those platforms and a rename race remains open there by design (AGENTS.md,
SECURITY.md). The precedent had no containment, so this is no regression. The gap is the unverified claim
that the `(dev, ino)` comparison and the regular-file check behave on macOS and Windows. Inspect the
`cfg(not(unix))` code in `tree/disk.rs` and its tests.

R3. **Capabilities moved to `delivered` with no hosted CI run and a resolver rewritten under review.**
Local `make check` passes (963 tests, 1m27s) but PR 13's jobs never started, and four jobs are new. The
owner names the resolver as the least certain code: it "rests on fixtures, not on a check against rustc".
Precedent lessons L1-L3 are verified against rustc only through the assessment's probes, never in CI.
Confirm the 1,440-declaration-order test runs in the release profile too (`make test-release`), and that
debug and release test counts are now equal (`validation.md` records 227 vs 225 at one point).

R4. **The `docs_contract` rewrite changes the repository's own gate without a differential run.** About
2738 changed lines in one file; 114 to 137 tests; two renamed (`comment_stripping_keeps_visible_text`,
`parses_quoted_and_block_forms`). The old std-only reader is deleted, so there is no old-versus-new run on
the repository's current Markdown proving that nothing the old reader refused is now accepted. Verify by
mutating one rule at a time (the repo's own "break exactly one thing" discipline) rather than by reading
the diff. The optional `tags`/`timestamp` keys (user decision 1) widen the accepted language of every
document type: confirm `resource` is still refused and a duplicate tag is a violation
(`rejects_duplicate_tag` exists).

R5. **Cross-capability contract edit.** `docs/capabilities/census-rule-api.md` now requires `Rule: Sync`
and says a rule "runs on the parse thread", while `source-index.md:293` says file facts are read on the
parse thread and leave it as owned data. The two statements may not be consistent. `Rule: Sync` is a hard
bound on every future detector and was not a user decision (the recorded decisions concern tags and
timestamp, roots, containment and split links).

R6. **Binary crate path `<pkg>::bin_crate::<bin>`** (section 1). It diverges from both precedents and
from the issue's silence; bindings that name a binary item must use it, and a census ledger home written
against the katamari spelling will not resolve.

R7. **Licence exceptions are unconfirmed by the owner** (`unicode-ident` / `Unicode-3.0` in the root
policy, `libfuzzer-sys` / `NCSA` in the harness). Well scoped and gated, so technical risk is low, but the
stated policy asks for a human decision and the handoff asks for it. Gate the merge on that confirmation.

R8. **`make check` is heavier and its tool preconditions are new.** `cargo-fuzz` and two rustup targets
are required; a contributor without them fails at `fuzz` before `licenses` and `issue-refs` run (`make
check` stops at the first failure, and `validation.md` notes the silent `cross-check` recipe). The 1m27s
runtime is from a warm local cache; a cold CI run is unmeasured. The pre-commit hook also compiles `syn`
on first use.

R9. **Dependency surface grew.** `syn 3.0.8` (a major above both precedents), `quote`, `toml 1` with
`serde`, `proc-macro2` with `span-locations`, `libc`, and for the harness `libfuzzer-sys` (`Cargo.lock`
+165 lines, `fuzz/Cargo.lock` 232 lines). The assessment suggested trimming `printing` and `quote`.
`cargo deny` passed locally, but advisories were checked offline only; CI's fetching variant is
unobserved (R3).

R10. **Spec size and drift.** `docs/capabilities/source-index.md` grew by 601 lines and `okf-profile.md`
by 250. `docs_contract` pins Usage and Examples to crate doctests (`DOCTEST_PAIRS`,
`tests/docs_contract.rs:3190`) and resolves the listed test bindings, so a rename fails loudly. Residual:
confirm that each listed binding is the test that exercises its claim, not merely a test that exists.

### Checked and clean

No `#![feature]`, no weakened lint, no new `unsafe`, no issue or PR numbers, no process wording, no
attribution lines, no widened licence allow list, no `.deficiencies` change, no edit to the issue-ref
gate's banned list. The 13 commits use conventional subjects and carry no `GhpRsq-Defect-Class` trailer
(the squash step adds it).
