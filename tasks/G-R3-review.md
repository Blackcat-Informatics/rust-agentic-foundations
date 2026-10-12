# G-R3 review: docs(security), covering trust model, path containment and invocation rules

- Reviewed: the unstaged worktree diff on head `6f8891e` (20 files, +292/−19), against `remediation-plan.md` §R3 and its Check list. The governing decision is "Workspace = build input (Recommended)".
- Mode: read-only. Nothing was edited, staged or committed. All experiments ran in throwaway `mktemp -d` directories, which were removed afterwards.

VERDICT: BLOCKED

There are three required findings:

- **R-1.** The Rust parse bound does not prevent a stack overflow, although the spec claims that it does.
- **R-2.** The `cargo test` argv forms pass vacuously. The doc-test form can never select a doc test.
- **R-3.** SECURITY.md overclaims against the specs.

Everything else in R3 is sound or advisory.

## Scope

- `SECURITY.md`
- `docs/architecture/okf-profile.md`
- The capability docs `source-index`, `census-reference-scan`, `okf-profile`, `census-ledger`, `census-migration`, `workplan-cli`, `workplan-complete-gate`, `workplan-entity-history` and `workplan-reconcile`
- Their `work/changes/*` records

## Finding closure (security-review M1, M2, L3, L4)

| Finding | Closed? | Basis |
| --- | --- | --- |
| M1 path containment | **Yes**, with advisories A-3 and A-7 | `source-index.md:32-43`: canonical root and canonical candidate, a prefix test, the `link/..` case named, no descent into directory symlinks, refusal of non-regular files, Cargo target paths covered. Six `DiskTree` fixtures, `:162-174`, plus the MemoryTree caveat `:159-161`. `okf-profile.md` (architecture) `:136-142`: escaping links are neither opened nor stat'ed, and lookup is against the walked set. `census-reference-scan.md:66-73`. |
| M2 trust model / invocation | **Yes** for the trust model and the git/metadata argv. **No** for the `cargo test` argv (R-2). | The SECURITY.md trust model `:35-52` matches the user decision. The reference-scan and workplan-cli metadata argv are verified (E1). The git argv is verified (E3). |
| L3 PGP | **Yes**, for the in-repo part | The fingerprint matches (E5). The user-ID mismatch is explained. Renewal before 2026-12-24 is an owner action outside the repo, per the plan. |
| L4 resource bounds | **Partly** | YAML, TOML, Markdown and JSON bounds are sane (see Bounds). The Rust bound does not prevent the abort that it claims to prevent (R-1). |

## Evidence

### E0. R3 Check list and gates (all pass)

```
C1  grep -n -i 'build input' SECURITY.md                        -> 37: ...**trusted build input**...         (1 line)
C2  grep -n -iE 'canonical' source-index / reference-scan / arch okf-profile -> 9 lines                    (>=3)
C3a git grep -n -- '--frozen' -- docs/capabilities               -> 13 lines                                 (>=1)
C3b git grep -n -- '--no-deps' -- docs/capabilities/workplan-cli.md -> 34, 37, 129                         (>=1)
C4  per-file count of nesting|depth|maximum file size|alias: source-index 7, okf-profile 6, census-ledger 3 (>=1 each)
C5  git grep -n -e ' -- ' (history, complete-gate, reference-scan) -> complete-gate:71,73; entity-history:53,137 (4 >= 3)
C6  grep -n -iE 'fifo|loop' source-index.md                      -> 42, 43, 173, 174                         (>=2)
C7  gpg --show-keys (throwaway GNUPGHOME) fpr = 52E54B131C297D1EAB143BAA0C54057C8F9A0090
    SECURITY.md:124 `52E5 4B13 1C29 7D1E AB14  3BAA 0C54 057C 8F9A 0090` -> equal; block sha256 6bb700ee…fa62 unchanged
CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target cargo test --test docs_contract --locked
    -> test result: ok. 46 passed; 0 failed      rc=0
make issue-refs                                    -> rc=0
uvx reuse@6.2.0 lint                               -> 77/77, compliant with REUSE 3.3   rc=0
```

### E1. Root discovery versus my original E6 probe, with clean cargo

This used the real toolchain cargo (`…/nightly-2026-09-14…/bin/cargo`, cargo 1.100.0-nightly) under `env -i` with an isolated `CARGO_HOME`. The machine's `cargo` shim forces `--config build.rustc-wrapper="kache"`, so it was bypassed. The scripts lived outside the probe package.

```
build.rustc, cargo metadata --format-version 1 --frozen:
    marker: repo-configured rustc invoked: -vV
            repo-configured rustc invoked: - --crate-name ___ --print=file-names ... --print=cfg -Wwarnings
    (then rc=101: cannot create the lock file ... because --frozen was passed)
build.rustc, cargo metadata --format-version 1 --no-deps --frozen:          rc=0, marker: (not invoked)
build.rustc-wrapper, cargo metadata --format-version 1 --frozen:
    marker: repo-configured program invoked: rustc -vV  /  rustc - --crate-name ___ ... --print=cfg
build.rustc-wrapper, cargo metadata --format-version 1 --no-deps --frozen:  rc=0, marker: (not invoked)
```

Three conclusions follow:

- The `workplan-cli` fixture (`:127-130`) is correct: `--no-deps --frozen` runs neither `build.rustc` nor `build.rustc-wrapper`.
- The reference-scan "Trust" paragraph (`:59-64`) is accurate: full resolution runs both, even under `--frozen`, and even when it then fails.
- The fixture also guards against a future cargo changing this behaviour.

### E2. libtest semantics of the complete-gate argv (throwaway crate)

The crate had a lib with a doc-tested `foo`, `tests::it`, an `#[ignore]`d failing test, and `tests/a.rs::it`.

```
cargo test --frozen --package probe --doc -- --exact foo
    running 0 tests / test result: ok. 0 passed; ... 1 filtered out        rc=0   <- doc test never selected
cargo test --frozen --package probe --no-fail-fast -- --exact does_not_exist
    0 passed in every target (lib, tests/a.rs, Doc-tests)                    rc=0   <- vacuous pass
cargo test ... -- --exact tests::it              -> 1 passed (lib); 0 in other targets      rc=0
cargo test ... -- --exact -- --include-ignored   -> 0 run in every target                   rc=0   <- libtest honours `--`
cargo test ... -- --exact --include-ignored      -> ignored test ran and FAILED             rc=101 <- option injection is real
(doc test name as libtest reports it: "src/lib.rs - foo (line 4)")
```

### E3. git argv forms (git 2.56.0, throwaway repo)

```
git --no-pager log -L1,3:-rf.rs --no-patch --format=... --   -> two formatted lines, no patch     rc=0
git --no-pager log --follow --format=... -- -rf.rs             -> three commits                     rc=0
git update-ref 'refs/heads/--output=x' HEAD                    -> a branch named like an option exists
git rev-parse --verify --end-of-options '--output=x^{commit}'  -> 6bded8c…; no file x created       rc=0
git rev-parse --verify '--output=x^{commit}'                   -> fatal: Needed a single revision
git cat-file blob "<mb>:-rf.rs"                                -> content                           rc=0
```

### E4. syn recursion versus the delimiter-only bound (syn 2.0.119, `full` + `parsing`)

`syn::parse_file` ran on a thread with a 2 MiB stack, which is libtest's default thread stack. rc=134 is a stack-overflow abort.

```
input (delimiter depth)                bytes   debug   release
126 nested parens (127)                ~  -    ok      ok
'A<' x500 'u8' '>' x500       (0)      1513    134     134
'!' x1000 'x'                 (2)      1022    134     ok
'!' x4000 'x'                 (2)      4022    134     134
'&' x20000 'u8'               (0)     20 kB    134     134
'|| ' x10000 'x'              (2)     30 kB    134     134
'else if a {}' x20000         (2)    260 kB    134     ok
with a 256 MiB stack, debug: unary20k ok, generics5k ok, closures10k ok, refs20k aborted
```

### E5. Other probes

- **toml 1.1.6.** `a = [[[...` and `a = {b={b=...` at depths of 100, 10 000 and 100 000 each give `Err("TOML parse error at line 1, column 85/245")`, with no crash. The source-index claim (`:48-49`) that the toml parser's own limit turns deep nesting into an error holds. The ledger's 32-level bound is stricter, so it needs its own check, which the spec states.
- **Real `.rs` files over 2 MiB in the local registry:**
  - `encoding_rs-0.8.42/src/data.rs`, 2.5 MiB
  - `chromiumoxide_cdp-0.9.1/src/cdp.rs`, 4.9 MiB
  - `chrono-tz-0.10.4/src/prebuilt/timezones.rs`, 6.9 MiB

## Findings

### R-1. REQUIRED: the Rust nesting bound counts only delimiters, so a small file still aborts the process

**Where:**
- `docs/capabilities/source-index.md:44-50` ("scans its token delimiters … refuses nesting deeper than 128 levels … so a hostile file cannot overflow the stack"), `:118-119` and `:151-153`
- `census-reference-scan.md:72-73` (the same bound applies to reference source)
- `SECURITY.md:46-49` ("never a panic or crash … never unbounded … stack")

**Problem (E4).** `syn` recurses on constructs that open no delimiter: generic arguments `A<A<…>>` (angle brackets are not delimiters), prefix operators `!`/`-`/`*`/`&`, reference types `&&…T`, closure chains `|| || …`, and `else if` chains. A 1.5 KB file of 500 nested generics, with delimiter depth 0, aborts with SIGABRT on a 2 MiB stack in both debug and release. A stack overflow is not a panic. It kills the whole `cargo test` process, so the `Gate` produces no report. That contradicts the "typed error, never a crash" contract. A larger stack raises the threshold, but at 256 MiB `&`×20000 still aborted. A stack size alone is therefore not a bound.

**Fix.** Change the contract to a bound that actually covers syn's recursion. One acceptable shape:
- The pre-scan computes a recursion budget over the token stream, not only delimiters. It counts nested `<`, runs of prefix-operator tokens (`! - * &` and `&&`), closure-bar runs, and `else if` chain length, and refuses beyond a stated limit.
- Parsing also runs on a dedicated thread with a stated stack size.
- The Verification bullets add one fixture per shape: generics, unary, reference type, closure chain and else-if chain. Each sits just over the limit and fails with a typed error naming the file. Each runs on the default 2 MiB test-thread stack, in both debug and release.

Alternatively, parse in a child process and report its abort as a typed error, and say so. In either case, reword `source-index.md:47-48` and `SECURITY.md:48-49` so that they claim only what the mechanism guarantees.

### R-2. REQUIRED: the `cargo test` argv forms pass with zero tests, and the doc-test form can never select a doc test

**Where:** `docs/capabilities/workplan-complete-gate.md:63-85`, the invocation table at `:69-73`, and `:76-77` (the pattern). The `example-failing` row of `workplan-reconcile.md:76-77` reuses these forms.

**Problem (E2).**
- `cargo test … -- --exact <name>` exits 0 when nothing matches: it reports "0 passed" in every target. Resolution in the source index does not guarantee a match. The resolved item may not be a `#[test]`; it may be gated behind a `cfg(feature)` that is not enabled; or the libtest name may differ from the source path. In each case the "bound tests pass" condition is met vacuously, so `complete` deletes a change record that nothing tested.
- The doc-test row `cargo test --doc -- --exact <name>` never matches anything. libtest names doc tests `src/lib.rs - foo (line 4)`, and that form also cannot satisfy the identifier pattern `[A-Za-z_][A-Za-z0-9_]*(::…)*`. Every doc-test binding therefore passes without running.

**Fix.**
- Require each run to prove that it executed the bound test, for example with a `-- --exact <name> --list` pre-pass or by checking the summary. Exactly one test must match, or at least one per doc-test binding. Zero is an `unresolved-binding` failure, not a pass.
- For doc tests, list them (`cargo test --frozen --package=<pkg> --doc -- --list`), select the entries whose item part equals the bound path, and run each with `--exact '<full libtest name>'`. Give that name its own validation rule: it starts with the target's relative source path, and never with `-`.
- Add fixtures for:
  - a resolved non-test function bound as a test, which must refuse;
  - a doc-test binding that really runs, shown by a failing doc test that makes `complete` refuse;
  - a doc-test binding with no matching doc test, which must refuse.

### R-3. REQUIRED: SECURITY.md overclaims against the narrowed model and the specs

**Where:** `SECURITY.md:48` ("never a read outside the workspace root"), `:70` ("Every path is canonicalised and must lie inside the canonical root") and `:50-52` ("places `--` before any path or filter taken from a document").

**Problem.**
- The reference scan reads reference-package sources from the Cargo registry or git checkouts, which lie outside the workspace root by design. Its containment root is the package directory (`census-reference-scan.md:66-73`).
- The specs do not always use `--`. reconcile embeds the path in one `<commit>:<path>` argument (`workplan-reconcile.md:71-76`). entity-history embeds it in `-L<a>,<b>:<file>`. complete-gate relies on resolution plus a pattern, because libtest still parses options after cargo's `--` (E2). The policy states one mechanism, and the specs use three.

**Fix.**
- Line 48: "never a read outside the workspace root or, for the reference scan, outside the package directories that `cargo metadata` reports".
- Line 70: say the same.
- Lines 50-52: "a value taken from a document reaches a spawned process only after resolution against the loaded documents or the source index and a stated pattern, and then either after `--` or inside a single `<rev>:<path>` or `-L…:<path>` argument".

### Advisories (do not block)

- **A-1. libtest end of options.** libtest honours `--` (E2). Use `-- --exact -- <name>` as defence in depth behind the pattern, so that a dash-prefixed value is a filter even if validation regresses.
- **A-2. Target selection, joined forms, and example names.**
  - Pin the resolved target in the test argv with `--lib`, `--bin=<b>` or `--test=<t>`. Otherwise the same `--exact` name runs in every target: lib `it` and `tests/a.rs` `it` both match `it`.
  - Use joined forms: `--package=<pkg>` and `--example=<name>`.
  - The identifier pattern at `:76-77` is defined for item paths. Example target names commonly contain `-`, so give target names their own pattern, `[A-Za-z0-9_][A-Za-z0-9_-]*`. As written, hyphenated examples would be reported as unresolved.
  - Consider null stdin and a timeout for `cargo run --example`.
- **A-3. reference-scan containment wording (`census-reference-scan.md:66-73`).** Canonicalisation alone neither refuses a FIFO nor ends a walk through a directory symlink that points at an ancestor, because every canonical path stays inside the package. The fixtures at `:139-141` will force the right behaviour. Still, state the source-index `DiskTree` rules explicitly (no descent into directory symlinks, regular-file check) rather than attributing the effect to canonicalisation.
- **A-4. The 2 MiB per-file cap fails on popular crates (E5).** Naming `encoding_rs` or `chrono-tz` as a reference package, or vendoring Unicode tables first-party, would hard-fail the gate. Consider a configurable `[census]` cap, a larger default for reference packages, or both. Keep the error naming the file.
- **A-5. git details.**
  - Show where `--since=<date>` sits in the history argv: before `--`.
  - Consider `--no-show-signature` (a user's `log.showSignature` would add lines to stdout), and parse records with `splitn(3, '\x1f')`, because `%s` is attacker-chosen text.
  - In reconcile, list the command that finds the default branch name. The text says "the name that the repository reports" but gives no argv.
  - State a minimum Git version, given `rev-parse --end-of-options`, and exit 2 below it.
  - Bound subprocess output reads, as workplan-cli already does for metadata: `cat-file blob` to the 1 MiB document bound, and `git log`.
- **A-6. Which cargo binary.** The PATH-stub fixtures (`workplan-cli.md:127-130`, `workplan-complete-gate.md:184-186`) assume `cargo` is looked up on `PATH`. A cargo subcommand idiomatically uses `$CARGO`. State the rule (`$CARGO` if set, else `cargo` on `PATH`) so that the fixtures and the implementation agree.
- **A-7. Time-of-check versus time-of-use, and the current walker.**
  - Specify the regular-file check on the opened handle: open non-blocking and without following, then `fstat`. stat-then-open is racy. This is low priority under the trusted-input model.
  - The architecture text now says the walk "skips symlinks and non-regular files" (`okf-profile.md:141-142`). The current `tests/docs_contract.rs:1188-1200` skips symlinks but would `read_to_string` a FIFO named `*.md`. Align it in R7.
- **A-8. SECURITY.md small items.** Until the key is renewed, state its expiry (2026-12-24) next to the fingerprint. Restore the colon after "PGP key for encrypted reports" (`:121`). "Never unbounded … time" (`:49`) is stated by no spec. Either soften it or add a bound.

## Bounds sanity

| Bound | Judgement |
| --- | --- |
| 2 MiB per source file | Sane against hostile input. Too low for some real crates (A-4). |
| Delimiter nesting 128 | 127 nested parens parse on a 2 MiB stack in debug, so the number is fine. The mechanism is insufficient (R-1). |
| YAML 1 MiB, depth 32, aliases/anchors/merge keys refused | Sane. The closed-key schema never needs aliases, and refusing them removes billion-laughs outright. The "allocates ≤ input × constant" fixture (`okf-profile.md:158-160`) is hard to measure. Asserting fast refusal is enough. |
| Markdown block nesting 64 | Sane. |
| Ledger and migration TOML: 1 MiB, 32 levels | Sane. The toml crate's own limit is about 80 (E5), so 32 needs the stated own check. |
| Metadata output 64 MiB, 128 JSON levels | Sane. 128 equals serde_json's default recursion limit, and 64 MiB is generous for `cargo metadata` of large graphs. |

## Trust-model consistency

Every spawning spec now names the workspace as trusted build input, each with a link to the policy: `census-reference-scan.md:59-64`, `workplan-cli.md:37-43`, `workplan-complete-gate.md:82-84`, `workplan-entity-history.md:63-65` and `workplan-reconcile.md:77-80`. The parsing-hardened statements in okf-profile, census-ledger and source-index are consistent with "parsing stays hardened". The only contradictions found are R-3's overclaims and the R-1 stack claim.

## Recheck (fix round 1)

- Rechecked: the unstaged diff on `6f8891e` (20 files, now +426/−18). Only the changed text and my findings were in scope.
- Mode: read-only. Nothing was edited or staged; the worktree still shows 20 unstaged files and 0 staged. Every probe ran in a `mktemp -d` directory, which was removed afterwards.
- The superseded line `VERDICT: BLOCKED` above belongs to the first review. This section's verdict is the final line of the file.

### Gates and checks (all pass)

```
C1 1   C2 8   C3a 15   C3b 3   C4 source-index 8 / okf-profile 6 / census-ledger 3   C5 6   C6 5
C7 gpg fpr 52E54B131C297D1EAB143BAA0C54057C8F9A0090 == SECURITY.md fingerprint; block sha256 6bb700ee… unchanged
CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target cargo test --test docs_contract --locked -> 46 passed, rc=0
make issue-refs -> rc=0        uvx reuse@6.2.0 lint -> compliant, rc=0
```

### Status of my earlier findings

| Item | Status |
| --- | --- |
| R-1 Rust stack bound | **Not closed.** See RR-1. |
| R-2 zero-test pass | **Closed for tests** (E-R2). **Doc-test half broken:** it fails closed, and no doc-test binding can ever pass. See RR-2. |
| R-3 SECURITY.md overclaims | **Closed** in the Containment and Subprocesses bullets. The Bounds bullet inherits RR-1. A leftover single-mechanism sentence remains in `workplan-cli.md:104-105` (RR-4). |
| A-1 `--` backstop | Closed (and see RR-2 for doc tests). |
| A-2 pinned target, joined forms, name patterns, stdin and timeout | Closed. |
| A-3 reference-scan wording | Closed (`census-reference-scan.md:66-77`). |
| A-4 `max_file_bytes` | Closed. The 8 MiB default admits the 6.9 MiB chrono-tz file, and the value is configurable. |
| A-5 git | Mostly closed. The version gate is wrong (RR-3), and the `--short` ambiguity is a new advisory (RA-1). |
| A-6 `$CARGO` | Closed. |
| A-7 type checked on the opened handle | Closed (`source-index.md:41-46`). The docs_contract half moved to R7. |
| A-8 key expiry, colon, time-bound wording | Closed. |

### E-R1. The budgets against syn 2.0.119 on the specified 64 MiB parse stack

Each input was run through a model of the five stated budgets (delimiters 128, nested `<` 64, prefix-operator run 256 with `&&` as one token, `||` chain 128, `else if` 256). It was then parsed by `syn::parse_file` on a 64 MiB thread. rc=134 is a stack-overflow abort.

```
input                                   bytes   pre-scan model                              debug  release
C1 ('!'*256 + '(') x127                 32788   delim=128 prefix_run=256       -> admitted   134    0
C2 ('&'*256 + '(') x127, type           32779   delim=127 prefix_run=128       -> admitted   134    0
C3 ('&&'*256 + '(') x127, type          65291   delim=127 prefix_run=256       -> admitted   134    134
C4 ('A<' + '&'*256) x64, type           16589   lt=64 prefix_run=128           -> admitted   134    0
C5 ('|| '*128 + '(') x127               49044   delim=128 closure_run=128      -> admitted   134    0
U1 'return ' x20000                    140014   nothing counted               -> admitted   134    0
U2 '&mut ' x20000, type                100013   prefix_run=1 ('mut' resets)   -> admitted   134    0
U3 '*const ' x20000, type              140013   prefix_run=1                  -> admitted   134    0
U4 'fn() -> ' x20000, type             160013   nothing counted               -> admitted   134    0
U5 'a = ' x20000 (right-assoc)          80014   nothing counted               -> admitted   134    0
U7 'move || ' x20000                   160022   closure_run=1                 -> admitted   134    0
U8 "&'a " x20000, type                  80013   prefix_run=1                  -> admitted   134    0
U9 'break ' x20000                     120034   nothing counted               -> admitted   134    0
U6 'x @ ' x20000 (pattern)              80022   -> admitted                                  0      0
thresholds, debug, 64 MiB: '&mut ' x4000 (20 kB) and 'fn() -> ' x4000 (32 kB) abort; 'return ' x8000 parses
'&' type nesting, debug: parses at 3819 levels, aborts by 3968 -> about 16 KiB per level (the spec states about 13 KiB)
```

### E-R2. The list/run protocol (cargo 1.100 nightly, throwaway crate)

```
--lib -- --list --exact -- tests::it      -> "tests::it: test" / "1 test, 0 benchmarks"        (one entry: proceed)
--lib -- --list --exact -- not_a_test     -> "0 tests, 0 benchmarks"                           (refuse: correct)
--lib -- --list --exact -- tests::gated   -> "0 tests" (cfg(feature = "off"))                  (refuse: correct)
--lib -- --list --exact -- tests::ign     -> "tests::ign: test" (one entry)
--lib -- --exact -- tests::ign            -> "0 passed; 0 failed; 1 ignored"                   (refuse: correct)
--lib -- --exact -- tests::it             -> "1 passed; 0 failed; 0 ignored"                   (pass: correct)
--doc -- --list -> "src/lib.rs - (line 3)", "src/lib.rs - Wrapper<T>::get (line 17)", "src/lib.rs - bar (line 24)",
                   "src/lib.rs - bar (line 28)", "src/lib.rs - foo (line 8)", "src/lib.rs - inner::baz (line 35)"
--doc -- --exact -- 'src/lib.rs - inner::baz (line 35)'   -> running 0 tests, rc=0
--doc -- --exact 'src/lib.rs - inner::baz (line 35)'      -> running 0 tests, rc=0
--doc -- --exact inner::baz                               -> running 0 tests
--doc -- inner::baz                                       -> 1 passed ("test src/lib.rs - inner::baz (line 35) ... ok")
cargo -v shows: rustdoc ... --test-args --exact --test-args 'src/lib.rs - inner::baz (line 35)'
```

rustdoc splits each `--test-args` value on whitespace, so a full doc-test name can never act as one exact filter.

### E-R3. git

```
gh api repos/git/git/compare/3a1f91cfd963...v2.29.0 -> behind  ("rev-parse: handle --end-of-options" not in 2.29)
gh api repos/git/git/compare/3a1f91cfd963...v2.30.0 -> ahead   (first in 2.30)
gh api repos/git/git/compare/51b4594b4024...v2.24.0 -> ahead   (parse-options --end-of-options is in 2.24)
symbolic-ref --short refs/remotes/origin/HEAD: no remote -> rc=128; origin/main present, HEAD unset -> rc=128;
  origin/HEAD a plain ref -> rc=128; dangling origin/HEAD -> prints "origin/gone", then rev-parse rc=128
local branch literally named origin/main: rev-parse --verify --end-of-options 'origin/main^{commit}'
  -> "warning: refname 'origin/main' is ambiguous." and the LOCAL commit aa25c76 (refs/remotes/origin/main is 0c5d496), rc=0
```

The default-branch command fails hard when `origin/HEAD` is unset or dangling, as the spec states (`workplan-reconcile.md:74-76`, fixture `:173`).

### Recheck findings

**RR-1. REQUIRED: the in-process budgets are not a guarantee, so the stated guarantee is false.**

*Where:*
- `source-index.md:48-68` ("The budgets sum to 832 recursion levels … a file the pre-scan admits cannot overflow the parse stack") and `:136-139`
- `SECURITY.md:56-61` (the Bounds bullet)
- `census-reference-scan.md:76-77`

*Problem (E-R1).* There are two independent flaws.

1. **The budgets compose by multiplication, not addition.** A run of 256 prefix operators is legal at each of 127 delimiter levels, and likewise inside each of 64 `<` levels and between closure chains, so admitted recursion is in the tens of thousands of levels, not 832. Every composite input C1–C5 is admitted and aborts in debug. C3 also aborts in release.
2. **Many recursive shapes are counted by no budget:** `return`/`break` chains, `&mut T` and `&'a T` and `*const T` type chains (the interleaved keyword or lifetime resets the "run"), fn-pointer return chains, right-associative `=`, and `move ||` chains.

Admitted files of 20–160 kB abort the 64 MiB stack in debug, which is the profile `cargo test` uses for the dev-dependency Gate. The per-level cost measured in debug is about 16 KiB, above the stated 13 KiB.

The spec's honest residual clause ("a shape the budgets miss is a vulnerability to report") cannot rescue a guarantee that fails on the first shapes tried.

*Fix.* Choose one and state it accurately.

- **(a) Child-process isolation.** Parse in a subprocess, and report an abort as a typed error that names the file. This is the only design under which "never a crash" holds without a grammar-complete argument.
- **(b) One sound, global budget in place of five per-shape budgets.** syn is recursive descent with no left recursion, so each nested level consumes at least one token. Bound the recursion-carrying token count along the current path: at each token, sum over the open delimiter stack the tokens since the last `;` or `,` (or the group opener) at that level. Refuse above one budget B, where B × measured worst-case bytes per token (debug, with margin) is at most half the dedicated stack. For example, B = 2048 at 16 KiB is 32 MiB of 64 MiB. Back it with:
  - a fixture per shape in E-R1, plus the composites, just over and just under B, in debug and release;
  - a fuzz or property test that searches for admitted inputs that abort.

Under either option, rewrite `source-index.md:58-62` and the SECURITY.md Bounds bullet so the claim matches the mechanism. Drop the "sum to 832" argument. If (b) is chosen and the bound is still heuristic, say so.

**RR-2. REQUIRED: the doc-test run argv can never select a doc test, so every doc-test binding refuses.**

*Where:* `workplan-complete-gate.md:73`, the "Doc test, run pass" row (`cargo test --frozen --package=<pkg> --doc -- --exact -- <doctest-name>`), and `:100-104`.

*Problem (E-R2).* cargo hands each argument after `--` to rustdoc as `--test-args <arg>`, and rustdoc splits each value on whitespace. `src/lib.rs - inner::baz (line 35)` becomes five filters, so `--exact` matches nothing: "running 0 tests", rc=0. The protocol does fail closed ("each selected entry must report `1 passed`"), so no vacuous pass returns. But the contract cannot be met, and the spec's own fixture (`:207-209`, "runs that doc test by its full libtest name and passes") would fail.

*Fix.*
- Run doc tests with a whitespace-free substring filter equal to the bound item path, without `--exact`: `cargo test --frozen --package=<pkg> --doc -- <item-path>`. Here `<item-path>` matches the `<name>` pattern, so it never begins with `-`.
- Parse the per-test lines `test <full name> ... ok|FAILED|ignored`, and require every entry selected by the list pass to appear as `... ok`.
- Any other matched doc test (for example `inner::baz_more`) is either ignored or required to pass. State which.
- Replace the run-pass row and the `<doctest-name>` pattern accordingly. The pattern is then needed only to parse list output, not to build argv.

**RR-3. REQUIRED (one-token text fix): the Git minimum version is wrong for `rev-parse --end-of-options`.**

*Where:* `workplan-reconcile.md:63-65` ("exits 2 when Git is older than 2.24, which introduced `--end-of-options`").

*Problem (E-R3).* 2.24 introduced `--end-of-options` for parse-options commands. `rev-parse` gained it only in 2.30 (`3a1f91c`, "rev-parse: handle --end-of-options"). On 2.24–2.29 the version gate admits a Git on which the second command fails. That failure is closed, but the contract misstates the fact.

*Fix.* Change the gate to 2.30 and correct the sentence. `workplan-entity-history.md:50-51` uses no `--end-of-options`, so its 2.24 gate is harmless. Aligning it on 2.30 keeps one stated minimum.

**RR-4. REQUIRED (text): R-3 residue in workplan-cli.**

*Where:* `workplan-cli.md:104-105` ("uses `--` before any path or filter taken from a document").

*Problem.* This restates the single-mechanism claim that R-3 corrected in SECURITY.md. reconcile and entity-history embed paths in `<rev>:<path>` and `-L…:<path>` rather than placing them after `--`.

*Fix.* Point to the three forms in SECURITY.md's Subprocesses bullet, or restate them.

**RA-1. Advisory: `--short` makes the default ref ambiguous (E-R3).** A local branch literally named `origin/main` silently wins over `refs/remotes/origin/main`, with only a stderr warning and rc=0, so the merge base is wrong. Use the full ref: `git symbolic-ref refs/remotes/origin/HEAD` without `--short`, or resolve `refs/remotes/origin/HEAD^{commit}` directly. Also state that the remote name `origin` is assumed.

**RA-2. Advisory: generic doc-test names are unbindable.** rustdoc names impl items as `Wrapper<T>::get`. The `[A-Za-z0-9_:]*` item part rejects them, so such a binding refuses. This fails closed, but either widen the parse pattern for list output or state the limitation.

### R-1 design judgement (as requested)

In-process parsing with budgets can be adequate only if the bound is sound over the whole grammar and the guarantee is worded to match. The five-shape design is neither, as E-R1 shows. My child-process alternative is not the only acceptable fix. RR-1 option (b), one global path budget sized from measured per-token stack cost and backed by fuzzing, is a reasonable in-process design. Either (a) or (b) is required before this is honest.

VERDICT: BLOCKED

## Recheck 2 (fix round 2)

- Rechecked: the unstaged diff on `6f8891e` (20 files, +469/−18). Only my RR and RA findings and the changed text were in scope.
- The governing decision for RR-1 is "Fail-closed, honest (Recommended)".
- Mode: read-only. The worktree still shows 20 unstaged files and 0 staged. Probes ran in `mktemp -d` directories, which were removed afterwards.
- The earlier `VERDICT:` lines above are superseded. This section's verdict is the final line of the file.

### Gates and checks (all pass)

```
C1 1  C2 8  C3a 15  C3b 3  C4 source-index 9 / okf-profile 6 / census-ledger 3  C5 6  C6 5
C7 gpg fpr 52E54B131C297D1EAB143BAA0C54057C8F9A0090 == SECURITY.md; block sha256 6bb700ee… unchanged
CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target cargo test --test docs_contract --locked -> 46 passed, rc=0
make issue-refs -> rc=0     uvx reuse@6.2.0 lint -> compliant, rc=0
```

### New probe evidence

```
# RA-1, full-ref form, with a local branch literally named origin/main beside refs/remotes/origin/main
git symbolic-ref refs/remotes/origin/HEAD                         -> refs/remotes/origin/main
git rev-parse --verify --end-of-options 'refs/remotes/origin/main^{commit}' -> 3a9a98e (= remote-tracking; local branch is c9a2c34)

# RR-2, rustdoc behaviour (cargo 1.100 nightly)
--doc -- --list  -> "src/foo.rs - foo::bar (line 4)", "src/lib.rs - Pair<T,U>::first (line 8)", "src/lib.rs - nr (line 15)"
--doc -- nr      -> "test src/lib.rs - nr (line 15) - compile ... ok"   (no_run: compiled, not executed; body is panic!)
--doc -- foo     -> "test src/foo.rs - foo::bar (line 4) ... FAILED", rc=101  (substring also matches the source path)
```

rustdoc writes multi-parameter generics without spaces (`Pair<T,U>`).

The R-1 table in the first recheck (E-R1) still applies on the 64 MiB stack:
- **Debug:** C1–C5 and the six uncounted shapes (`return`, `&mut`, `*const`, `fn() ->`, `a =`, `move ||`) all abort.
- **Release:** C3 aborts. C1, C2, C4, C5 and all six uncounted shapes parse successfully (rc=0).

### Status

| Item | Status |
| --- | --- |
| RR-1 honest fail-closed wording | **Closed** in source-index (`:47-73`, `:141-145`), census-reference-scan (`:74-79`) and the SECURITY.md Bounds bullet (`:55-61`). The overflow claim is withdrawn, the residual case is stated as a failed run that never passes, and overflow is classed as a robustness bug under the trust model. The fail-closed property itself holds: a stack overflow is a SIGABRT of the test binary or the CLI, so `cargo test` and `cargo census check` exit non-zero. A new internal contradiction exists in the composite fixture (RR2-1). |
| RR-2 doc-test substring protocol | **Closed.** No false pass under the stated rule (see analysis). Advisory RA2-2. |
| RR-3 Git 2.30 | **Closed** (`workplan-reconcile.md:63-65`, `workplan-entity-history.md:50-51`). |
| RR-4 workplan-cli wording | **Closed** (`workplan-cli.md:103-107`, which now names the three forms). |
| RA-1 full ref | **Closed** (`workplan-reconcile.md:68`, `:74-76`, fixture `:176-177`). Verified above. |
| RA-2 generic item paths | **Closed.** The pattern is safe on argv (see analysis). Advisory RA2-3 covers its formatting. |

### RR-2: can the substring protocol produce a false pass?

No, under the rule as written.

- A binding passes only when every list-selected entry appears as its own `test <full name> ... ok` line.
- Any other doc test the substring matches is not counted. If one fails, the `cargo test` exit status fails, and the spec refuses the binding (`workplan-complete-gate.md:112-115`). E-probe: substring `foo` matched the failing `src/foo.rs - foo::bar`, rc=101.
- Extra matches can therefore only cause false refusals (fail closed), never a false pass.
- Each doc-test process's output is captured by the harness, so it cannot forge an `ok` line. The workspace is trusted build input in any case.

The one gap is RA2-2 (`no_run`).

### RA-2: is the item-path pattern dangerous on argv?

No.

- The value is a single argv element with no shell, so `'`, `&`, `,`, `:` and `<>` carry no meaning.
- The first character is `[A-Za-z_]`, so it never begins with `-`.
- rustdoc splits test arguments on whitespace, and the generic-argument class contains no `-`. Even if a space were admitted (see RA2-3), no split piece could begin with `-` and be read as a libtest option.
- The worst case is a broader OR filter, which only matches extra tests and so can only cause refusals.

### Recheck 2 findings

**RR2-1. REQUIRED (text): the composite fixture contradicts the stated budgets.**

*Where:* `source-index.md:191-199` ("each of which either is refused by the pre-scan … or aborts the run with a non-zero status … never success with an index"), against the five budgets at `:52-56` and `:142-143`.

*Problem.* With exactly the five stated budgets, the pre-scan admits C1, C2, C4, C5 and all six uncounted shapes, and a release build parses them successfully on the 64 MiB stack (E-R1 release rc=0). An implementation that follows the spec therefore fails its own fixture in release.

*Fix.* Choose one:
- **(a)** Permit a third outcome: exit 0 with the correct, complete index. That is not a false pass. The fail-closed property is "typed error, abort, or a correct full result; never success after an abort or with a partial index".
- **(b)** Add budgets for the six shapes and enumerate them at `:52-56` and `:142-143`, so they are refused in every profile.

**Advisories**

- **RA2-1. Overclaims left in SECURITY.md:**
  - The lead sentence at `:46-48` still says "a hostile file yields a typed error and not a panic" before the bullets qualify it. Reword it to "a refusal is a typed error and never a panic; see Bounds for the residual overflow".
  - The Bounds bullet describes a "committed fuzz target" in the present tense, but no `fuzz/` directory exists and no crate is implemented yet. Phrase it as design, as the "Once implemented" surface list does.
  - Consider replaying `fuzz/corpus/parse_source/` in `cargo test`, so the corpus is exercised without cargo-fuzz.
  - Name the parse thread after the file. Rust's overflow message is `thread '<name>' has overflowed its stack`, so the abort would then identify the culprit file.
- **RA2-2. `no_run` and `compile_fail` doc tests.** rustdoc reports a `no_run` doc test as `test <list name> - compile ... ok`: compiled, never executed (E-probe; the body was `panic!`). A strict full-line match refuses such a binding. A prefix match would pass a doc test that never ran, contradicting condition 5 ("built and run"). State the strict match, and say whether `no_run` and `compile_fail` entries count, with a fixture.
- **RA2-3. Formatting of the `<item-path>` bullet (`workplan-complete-gate.md:89-95`).**
  - The code span `<[A-Za-z0-9_,⏎':&]*>` wraps across a line, and CommonMark renders the line break as a space, so the rendered class contains a space although the text says "no whitespace". Keep the regex on one line.
  - " - the names in the list output …" is a list item fused into the paragraph, and the lead-in still says "Three patterns". Split it into its own bullet and correct the count.

VERDICT: BLOCKED

## Recheck 3 (fix round 3)

- Rechecked: the unstaged diff on `6f8891e` (20 files, +486/−18). Only RR2-1, RA2-1, RA2-2 and RA2-3 were in scope.
- Mode: read-only. The worktree still shows 20 unstaged files and 0 staged. Probes ran in `mktemp -d` directories, which were removed afterwards.
- The earlier `VERDICT:` lines are superseded. This section's verdict is the final line of the file.

### Gates and checks (all pass)

```
C1 1  C2 8  C3a 15  C3b 3  C4 source-index 9 / okf-profile 6 / census-ledger 3  C5 6  C6 5
C7 gpg fpr 52E54B131C297D1EAB143BAA0C54057C8F9A0090 == SECURITY.md; block sha256 6bb700ee… unchanged
CARGO_TARGET_DIR=/home/paudley/Active/.raf-stage2-target cargo test --test docs_contract --locked -> 46 passed, rc=0
make issue-refs -> rc=0     uvx reuse@6.2.0 lint -> compliant, rc=0
```

### Probe evidence

```
rustdoc (cargo 1.100 nightly):
  compile_fail -> "test src/lib.rs - cf (line 4) - compile fail ... ok"   (matches the spec's stated form)
  should_panic -> "test src/lib.rs - sp (line 10) ... ok"                (ran; correctly accepted by the strict match)
  no_run       -> "test src/lib.rs - nr (line 15) - compile ... ok"       (from Recheck 2)
thread named "parse:src/hostile.rs", 64 KiB stack, deep recursion:
  rc=134; stderr "thread 'parse:src/hostile.rs' (4134523) has overflowed its stack" / "fatal runtime error: stack overflow, aborting"
```

### Items

| Item | Status | Basis |
| --- | --- | --- |
| RR2-1 composite fixture vs budgets | **Closed** | `source-index.md:192-202`: three outcomes (refused naming file and budget; exit 0 with a correct, complete index compared with the expected symbols; non-zero abort). A wrong or partial index, or an index after an abort, fails. A release-build success on C1, C2, C4, C5 and the six uncounted shapes now satisfies the fixture. The same three outcomes appear in `census-reference-scan.md:79-81` and the SECURITY.md Bounds bullet (`:62-64`). `census-reference-scan.md:151-153` still says an over-budget file "does not abort". That is consistent, because a pre-scan refusal is deterministic. |
| RA2-1 SECURITY.md wording | **Closed** | The lead at `:46-50` now states an aim and the three outcomes, not a categorical typed-error guarantee. The Bounds bullet states that the pre-scan, stack and fuzz target "are specified in the source index and are not built yet, because RAF is pre-release". Corpus replay in a child process under `cargo test` (`source-index.md:206-208`) and a file-named parse thread with a fixture on the abort message (`:203-205`) are added. The thread-name mechanism is verified above: the runtime's overflow message carries the thread name. |
| RA2-2 strict match, `no_run` and `compile_fail` | **Closed** | `workplan-complete-gate.md:105-111`: a strict `test <name> ... ok` line, with nothing between the name and `...`. `no_run` (`- compile`) and `compile_fail` (`- compile fail`) are refused with a message asking for a runnable doc test, and the stated report forms match rustdoc exactly. Fixtures are at `:240-242`. `should_panic` still passes, correctly, because it runs. |
| RA2-3 formatting | **Closed** | "Four patterns" (`:84`). Each pattern is its own bullet. The `<item-path>` regex `<[A-Za-z0-9_,':&]*>` is on one line, so no space is rendered into it. The code spans wrapped at `:107-109` render a space only where the intended text has one ("`... ok`", "`- compile fail`"), so they are harmless. |

No new findings.

VERDICT: PASS
