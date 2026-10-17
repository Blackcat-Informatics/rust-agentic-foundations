# Task 5 review: evidence and claims

## Scope

The staged diff (34 files) on top of 73662ab in the issue worktree. I read plan
D2-D7, D12, D13, D15, D17, Task 5, R2 and the edit list; the T1-T4 reviews;
SECURITY.md (diff); the reconcile and evidence-claims capability diffs; the
exec trusted variant (`Program::Trusted`, `Arg::Trusted`, `TrustedArgv`,
`TrustedWord`, `Limits::trusted`, `SystemRunner` changes); `evidence`,
`evidence/measure`, `obligation/trace`, `lifecycle/changed`,
`Lifecycle::process_obligations`; and the unit, integration and `complete`
fixtures.

Experiments ran in a scratch worktree (`git worktree add /tmp/t5rev HEAD`, staged
diff applied with `git apply --index`). It was removed afterwards. Nothing was
added, reset, stashed or committed in the issue worktree.

## Evidence

### Gates (scratch tree = HEAD + staged diff, CARGO_BUILD_JOBS=6)

All of these exited 0:

* `make fmt`, `make clippy` (`-D warnings`), `make test` (workspace, debug)
* `cargo test -p raf-workplan --release --locked`
* `make test-release`
* `make doc` (`-Dwarnings`)
* `cargo test --test docs_contract` (28 passed)
* `make issue-refs` (`repo_gates`, 57 passed)
* `make reuse`, `make deny`, `make msrv`
* `make docs-sync` twice: the tree was unchanged after the first run, and the
  second run was idempotent.

### Trusted process safety (1)

* **Constructors.** `TrustedArgv::from_command_line` takes only
  `std::env::ArgsOs`, which has no public constructor. `new` is `pub(crate)`. The
  `words` field is private, and the `TrustedWord(String)` field is private.
  * There is no `Default`, `From` or serde path. `Clone` copies only an existing
    value.
  * `Invocation::new` stays `pub(crate)`.
  * `Arg::Trusted` needs a `TrustedWord`, which only an existing `TrustedArgv`
    gives.
  * `EvidenceRequest`'s public `source` needs a `TrustedArgv`.
  * `FakeRunner::on(Program::Trusted, ..)` only answers a call and starts
    nothing.
* **Doctest mutants (all killed).** The three `compile_fail` doctests catch real
  widenings:
  * `new` made `pub`: the E0624 test compiled and failed.
  * `words` made `pub`: the E0451 test failed.
  * `from_command_line` widened to `impl IntoIterator<Item = OsString>`: E0308
    was not found and the test failed.
  * `from_command_line` widened to `String`: the E0308 test compiled and failed.
* **No shell.** The shell-metacharacter argv test passes (`$(...)`, backticks,
  `*`, `;`, the empty word). No `pwned` file appears.
* **stdin, cwd and environment.**
  * Standard input is null and the working directory is the root (both tested).
  * Without `LC_ALL` in the environment, the command sees no `LC_ALL`. The M7
    mutant (adding `LC_ALL` for trusted) is killed.
* **Bounds.**
  * 16 MiB and 1800 s, with no C locale, are pinned through the `FakeRunner`
    call.
  * Real kill at a 1 s bound: a grandchild `(sleep 3; touch late) &` never
    wrote `late`, so the process-group kill reaches grandchildren.
  * 4 MiB of NUL noise under the bound records normally.
* **Program lookup.** `./tool.sh` runs relative to the workspace root. A bare
  `tool.sh` is a `PATH` lookup (not the cwd), so a file in the workspace does not
  take over a bare name unless `PATH` holds `.`. That is caller trust, as
  documented.
* **Output edge cases.**
  * A command that prints only to stderr records `pass` with no values.
  * Binary noise and a non-UTF-8 name are discarded.
  * A non-UTF-8 byte inside a value is a `NotNumber` error (exit 2).

### Verdict semantics (2)

* **Refused values (each exit 2, nothing written).** `1e-3`, `1E3`, `+1`, `1 `,
  `1,5`, `--1`, `-.5`, `.5`, `NaN`, `status=ok`, a name printed twice, a limit
  never printed, output past the bound, and a command that cannot start.
* **Accepted.** `-0`, `00012`, a CRLF line, and `PEAK` beside `peak` (two
  distinct names).
* **Stored `command` never runs.** It is only written (`evidence.rs` sets
  `record.command`). No reconcile, verify or complete path reads it for a
  process. `Program::Trusted` is reached only through
  `Invocation::trusted(&TrustedArgv)`.

### from-binding (3) and record contents (4)

* **from-binding.**
  * Only `tests`/`examples` bindings of a `--for` capability are matched.
  * A non-matching path is `NoSuchBinding` before any `Executor`.
  * Limits with a binding are refused.
  * The run uses the existing `Executor` (`plan` then `run`).
* **Record contents.**
  * Inputs are `Digest::symbol` of every code binding of every `--for`
    capability, de-duplicated.
  * `--for` with no code binding, or with an unresolved one, is refused with a
    clear message (the Task 1 carried item is met).
  * `Create` uses `create_new` and is pre-checked.
  * `Replace` keeps nothing (tested).
  * The written-files test shows only the record is written.

### Obligations (5)

* **Fixtures.** Fixtures cover:
  * `claim-unsupported` for missing, failing, over-limit and removed records;
  * `evidence-stale` naming the symbol, and not stale after an unrelated edit,
    an inserted item or CRLF-only;
  * `evidence-changed` with real git: pass->fail, over limit, limit loosened
    (named against the base limit), limit removed, removed, committed-on-branch,
    no `origin/HEAD` (error), unreadable base blob (error, not a pass), one blob
    per record below a prefix;
  * the completion gate refusing each of the three kinds and completing once
    repaired.
* **Attachment.** `claim-unsupported` and `evidence-changed` attach per claim (at
  the marker line). `evidence-stale` attaches per record and input. This matches
  "each dependent claim receives an obligation".
* **Removed record with no claim.** It owes nothing. A record nobody cites has no
  dependent claim, so I judge this acceptable as written. Its `for` edge alone
  supports no statement.
* **`ObligationKind::ALL`.** It grows to 10. The const rank assert, the kind list
  test, the static/dynamic test and the evidence-column test are updated.
* **`process_obligations`.** It is the single helper for `reconcile` (examples
  on) and `complete` (examples off, judged under condition 5). The merge base is
  lazy and shared, and no claim means no git (tested).
* **`contract_effect`.** `contract_effect: none` keeps all three kinds.

### Documents (6)

* **Evidence-column test.** It still requires the parsed kinds to equal `ALL` in
  order and each evidence text to equal `contract_text()`. Stripping a leading
  `>` only adds the PROPOSED rows. A missing, reordered, duplicated or reworded
  row still fails, so it cannot be satisfied trivially.
* **States and records.**
  * `cap:workplan-reconcile` is `implementing`, and the index and the INVENTORY
    counts (4 implementing / 3 delivered) agree.
  * The `workplan-reconcile` record has 5 items, honest, with no outcomes yet.
  * The evidence-claims edits are inside PROPOSED blocks only.

### CLI seam (7)

`from_command_line(std::env::args_os())` is a sound seam. The binary integration
tests of Task 9 start `cargo-workplan` with a real argv, so the trusted path is
testable end to end through the binary. In-process clap unit tests cannot build
a `TrustedArgv`, by design.

Two cautions for Task 9:

* clap must treat the trailing words as `last = true`, so its view and the
  first-`--` scan agree.
* An option value that is itself `--` must be refused rather than shift the
  command.

### Mutation (8)

All 8 mutants were killed:

| Mutant | Change |
| --- | --- |
| M1 | exit-status mapping (`Some(_)` treated as success) |
| M2 | `>` changed to `>=` in the limit comparison |
| M3 | duplicate-name check removed |
| M4 | stale digest always equal (semantic form; the first form died on a lint) |
| M5 | merge-base direction swapped (`then`/`now`) |
| M6 | claim line dropped in trace |
| M6b | claim line dropped in changed |
| M7 | `LC_ALL` added to the trusted command |

### Hygiene (9)

* SPDX headers are on every new file.
* No tracker numbers, process wording, attribution or `unsafe` in the added
  lines.
* No manifest, lockfile or toolchain change.

## Findings

### REQUIRED

1. **The limit comparison is in `f64`, so over-limit values record `pass`.** This
   affects `evidence::measure::over_limits` (the recorded verdict),
   `obligation::trace::over_limit` (`claim-unsupported`) and
   `lifecycle::changed::compare` (`evidence-changed`). Each compares
   `Number::value()` (an `f64`). Reproduced through the real runner, each with
   `--limit`; every one recorded `verdict: pass`:

   | Printed value | Limit |
   | --- | --- |
   | `peak=9007199254740993` | `9007199254740992` |
   | `peak=0.30000000000000001` | `0.3` |
   | `peak=1.0000000000000000001` | `1` |
   | `peak=2` followed by 400 zeros (`inf`) | `1` followed by 400 zeros (`inf`) |

   Fix: compare the validated decimal texts exactly. Handle the sign, strip
   leading zeros, compare integer length then digits, then the fraction with
   trailing zeros stripped, and treat `-0` as `0`. Use one shared function in
   all three places, and add fixtures for each case above with an equal-value
   neighbour. Alternatively, `Number` must refuse a text that does not
   round-trip, but exact comparison is the fail-hard reading.

2. **A run killed at its time limit writes no record when any `--limit` is
   given.** On `ExecError::Timeout`, `run_command` discards stdout
   (`(None, Vec::new())`), so `over_limits` returns `LimitNotPrinted` and the
   command exits 2 with nothing written. Reproduced: a 1 s bound with
   `--limit peak=5` and a script that prints `peak=1` and then hangs gives
   "the limit `peak` names a value the command never printed ... nothing was
   recorded".

   This contradicts:
   * D12 ("a killed or failing run records `fail`");
   * R2;
   * SECURITY.md ("a kill there is recorded as `fail`");
   * the CHANGELOG ("a kill at the time limit record `fail`");
   * the PROPOSED Verification ("a kill at the time limit (recorded as fail)").

   The existing timeout fixture has no limit. Fix: a timed-out run records
   `fail` / `TimedOut` whatever the limits, without judging the limits against
   absent values. Add a fixture with a limit. Keep the documented and tested
   rule that a run that exited but did not print a limited value is an error.

3. **The second PROPOSED block of `docs/capabilities/workplan-reconcile.md`
   splits a delivered sentence.**
   * At HEAD, line 207 reads "...reads the file it means. A prefix that does" and
     continues "not match the relative-source-path pattern...".
   * Staged, the delivered fragment "A prefix that does" has moved inside the
     block (`> 1 MiB bound. A prefix that does`, line 248). The next line, "not
     match ...", follows the block with no blank line: a Markdown lazy
     continuation, so it renders inside the blockquote.
   * Resolving or rejecting the block would lose or mangle the delivered
     sentence.

   Fix: restore the delivered paragraph byte for byte and put the PROPOSED block
   after the whole paragraph (after "...a larger blob is an error. ..."),
   separated by blank lines.

### ADVISORY

1. **A grandchild holding stdout delays the verdict.** A command whose
   background grandchild keeps stdout open after the command exits 0 (a build
   daemon, for example) holds the run until the 1800 s bound and records
   `fail` / `TimedOut`. Reproduced at a 2 s bound. This is the existing runner
   behaviour and is conservative, but a reader sees "ran past its time limit"
   for a command that exited 0. Consider stating it in the Contract or
   SECURITY.md.
2. **Every value line is stored.** Every `name=number` line becomes a stored
   value, not only the limited ones. A command printing many such lines
   (bounded only by 16 MiB) produces a very large record. Consider a count bound,
   or say so in the PROPOSED text ("other output is discarded" is true, but
   values are unbounded in number).
3. **Any `key=value` text line fails the record.** A stray `key=value` line that
   is not a number (for example `status=ok`) is an exit-2 error for the whole
   recording. This is fail-hard by design (D12) and correct, but worth one
   sentence in the Usage so callers know to keep such lines off stdout.
4. **The cannot-start message gives cargo advice.** For a trusted program it says
   "install it, put it on PATH, or for cargo set $CARGO", and the `$CARGO` advice
   does not apply to the caller's command.
5. **`claim-unsupported` reports only one problem.** For a record that is both
   `fail` and over its limit, it reports only the over-limit problem (the order
   in `trace::raise`). This is acceptable: one obligation per claim and record.
6. **Task 9.** clap must take the trailing words with `last = true`, and must
   refuse an option value that is `--` so its view agrees with
   `TrustedArgv::from_command_line`. Test the record path through the binary.

Round-one verdict: BLOCKED (superseded by round two below).

## Round two

Scope: the restaged diff, now 40 files. It adds `raf_okf::Number::compare` and
`Number::exceeds`, the profile-rule relaxation, the timeout record and the
advisory fixes. It also edits the okf-profile capability, the architecture page
and the record. I rechecked it in a fresh scratch worktree (HEAD + staged diff),
removed afterwards. Nothing was touched in the issue worktree.

### R1 (exact comparison): resolved

* **No floating-point decision remains.** Every decision now goes through
  `Number::compare` or `Number::exceeds`:
  * `measure::over_limits`
  * `trace::over_limit`
  * `changed::compare` (including the choice of the tighter limit)
  * `Profile::validate` (`check_between`)

  `Number::value()` is documented as display-only. `PartialEq` compares text.
* **My own inputs.** I tested 21 ordered pairs, each in both directions, plus
  `exceeds`. All were correct:
  * leading zeros (`000`/`0`, `007`/`7.000`, `010`/`9.99`, 300 leading zeros);
  * `-0`, `-0.000` and `-0.0` against `0`;
  * `-0.0001` against `0`;
  * fractions: `0.5`/`0.49`, `0.05`/`0.5`, `0.10`/`0.1`, `1.01`/`1.1`;
  * negatives: `-1.01`/`-1.1`, `-10`/`-9`, `-0.5`/`-0.49`;
  * 5000-digit whole parts and 5000-digit fractions;
  * 400-digit negatives.
* **Still refused.** `1e5`, `1E5`, `+1`, `1.`, `.5`, `-`, `--1`, `1_000`, `0x1`,
  `inf`, `NaN` and a fullwidth digit. A printed `peak=1e5` is still exit 2.
* **Real runs.**
  * The four round-one cases now record `fail` with reason `OverLimit`.
  * `64` against a limit of `64.000`, `064` against `64`, and `-0` against `0`
    pass.
  * `0.0000001` against a limit of `-0` fails.
* **Mutants (all killed).**
  * M2 (`exceeds` changed to `!= Less`).
  * M2b (the fraction compared by length, in the form that compiles).
  * M2c (`-0` treated as negative).
  * M2d (negative order not reversed).

### R2 (timeout with limits): resolved

* **Behaviour.** A 1 s bound with `--limit peak=5` now records `fail` with reason
  `[TimedOut]`. The limits are kept, there are no values, and the record loads.
  A run that exited without printing the limited value is still exit 2. That is
  documented in the evidence-claims PROPOSED text, SECURITY.md and the CHANGELOG
  ("a command that exited").
* **Mutant M8 killed.** It applied the timeout rule only when there are no
  limits.
* **The relaxation has no loophole.** Only `verdict: fail` may carry a limit that
  names no value. A passing record with a limit and no value is still refused.
  * Mutant M9, applying the relaxation to every verdict, is killed by the
    raf-okf `records` test and the `kinds` test.
  * A failing record is always `claim-unsupported` (Fails) and, if it passed at
    the merge base, `evidence-changed` (Failed). So a limit without a value
    cannot make a record look supported.
  * A passing record whose limit was dropped is still `evidence-changed`
    (LimitRemoved).

### R3 (reconcile document): resolved

* The diff against HEAD removes only three lines: `state: delivered`, and the
  renamed test in the bindings and in the generated table.
* The delivered paragraph "...A prefix that does / not match..." is intact.
* The three PROPOSED blocks stand alone, with blank lines around them. The
  Verification block now follows the list instead of sitting inside it.

### Round-one advisories

* **2 (stored values):** `MAX_VALUES` is 1024. 1024 values record; 1025 is a
  `TooMany` error. M10 (an off-by-one) is killed.
* **3 (key=value lines):** a Usage/Constraints sentence was added.
* **4 (cannot-start message):** it now says "check the program named after
  `--`".
* **5 (one problem reported):** both reasons are now printed ("the command exited
  with 3; over the limit: peak").
* **1 and 6:** these stand as notes. Advisory 1 is now stated in the PROPOSED
  Constraints; advisory 6 is for Task 9.

### Mutation re-run

All 15 mutants were killed by failing tests: M1, M2, M2b, M2c, M2d and M3 to M10.
Round one's 8 are re-applied to the new code (M1 as "only a signal fails"), and
7 are new for the fixes.

### Gates

All of these exited 0:

* `make fmt`, `make clippy`, `make test` (workspace)
* `cargo test -p raf-workplan --release`
* `cargo test -p raf-okf --release`
* `make test-release`, `make doc`
* `docs_contract`
* `make issue-refs`, `make reuse`, `make deny`, `make msrv`
* `make docs-sync` twice: the tree was unchanged and the second run was
  idempotent.

### Hygiene

* SPDX headers are on every added file.
* No tracker numbers, process wording, attribution or `unsafe` in the added
  lines.
* No manifest, lockfile or toolchain change.

### Findings (round two)

REQUIRED: none.

ADVISORY:

* Round-one advisory 6 (Task 9 clap handling of `--`) carries to Task 9.
* `Number::value()` stays public as display-only. A later caller could compare
  with it again. Consider a lint-level note, or keep the doc sentence.

VERDICT: PASS
