# Security review: PR 20 (issue 10)

## Scope and method

* HEAD 43c7e02, diff `origin/main...HEAD`.
* Read:
  * SECURITY.md (whole file) and AGENTS.md;
  * plan D4, D11, D12 and D13;
  * the T4, T5, T9 and T10 reviews;
  * `exec/{trusted,invocation,system,pattern}.rs`, `git/log.rs`, `history.rs`, `record.rs`, `files.rs` and `sync::stage`;
  * `evidence.rs` (raf-workplan and cargo-workplan), `evidence/measure.rs`, `assessment.rs` (`record_assessment`) and `digest.rs`;
  * `raf-okf` `Number`, the `cargo-workplan` render layer, and the history and coverage renderers;
  * the `exports.rs` header and its carry charging.
* All experiments ran in a detached copy (`git worktree add --detach /tmp/s10 HEAD`) with `CARGO_TARGET_DIR=/tmp/s10-target` and `CARGO_BUILD_JOBS=6`.
  * Probe tests were added only in that copy.
  * Git commits went into a separate full clone (`/tmp/s10c`).
  * The issue worktree was only read. No git add, stash or reset was run, and no toolchain was changed.
* The binary was `/tmp/s10-target/debug/cargo-workplan` built from HEAD.
* Tree snapshots: `/tmp/sec10s/snap.sh` lists every path with its type and a content hash, or the target of a link, excluding `.git`.
* The scratch copies are removed. The probe sources are kept in `/tmp/sec10s/probes/`: `sec_writes.rs`, `sec_gate.rs` (run it with `-- --nocapture -- true`, since it takes the command from the test binary's own argv), `sec_exports.rs`, and `unit-probes.patch` (the `exec::tests::sec_probe` and `digest::sec_probe` unit probes).

## Evidence

### 1. The trusted process (`Program::Trusted`, `TrustedArgv`, `evidence record -- <argv>`)

* **Construction.**
  * Production code has one call site: `Invocation::trusted`, inside `run_command`.
  * Its `TrustedArgv` comes only from `VerdictSource::Command`, which the binary fills from `TrustedArgv::from_command_line(std::env::args_os())` in `cargo-workplan/src/evidence.rs::source`.
  * `TrustedArgv::new` is `pub(crate)` (used by tests only). The `words` and `TrustedWord(String)` fields are private.
  * There is no `From`, `Default` or serde derive. `Invocation::new` and `Literal::new` are `pub(crate)`.
  * `Arg::Trusted` needs an existing `TrustedWord`, and no public constructor turns an `Arg` list into an `Invocation`.
  * The three `compile_fail` doctests stand.
  * The stored `command` and `revision` are only written and displayed.
* **Hostile words through the built binary.** I ran
  `evidence record ... -- /tmp/sec10s/argv.sh '$(touch pwned)' '' '--id' '--' "a'b" $'line1\nline2'`.
  * The probe received exactly those six words.
  * No `pwned*` file was created.
  * stdin was `/dev/null` and the cwd was the root.
  * `LC_ALL=` (set but empty) stayed set and empty. With `LC_ALL` unset, the child saw no `LC_ALL`.
  * The tree diff was exactly one new file, `work/evidence/sec-a2.md`.
  * The stored command quotes every word for a POSIX shell, and the newline is written `\n` inside the YAML string.
* **Option-looking and odd words.**
  * `-- --help` gives "cannot start `--help`", exit 2, and no record.
  * A non-UTF-8 word gives "word 1 of the command is not UTF-8 ... nothing was run", exit 2.
  * The `--` and option-value cases were covered by T9 and I did not repeat them.
* **Long argv.**
  * Nine words of 120 kB each are refused before the run: "the document is 1082316 bytes, over the 1048576 byte bound". The command `true` did not run, because the provisional record text is judged first.
  * 100000 words record normally (a 591 kB record).
* **Floods.**
  * `-- yes`: "wrote more than 16777216 bytes to standard output and was killed", exit 2, no record, no `yes` process left.
  * `-- sh -c 'yes >&2'`: the same for standard error.
* **A child that keeps stdout open.** I ran `sh -c '(sleep 4; echo late=1) & echo peak=1'`.
  * The run waited for the background child, which is the documented behaviour.
  * The record holds both `peak` and `late`.
* **Ignored signals and `setsid`** (library probe, timeout set to 1 s).
  * `trap '' TERM INT HUP; sleep 9` is killed at 1.0 s with `Timeout`. SIGKILL cannot be ignored.
  * A `setsid` grandchild survives the group kill: it created `/tmp/sec10s/escaped` 3 s after the run ended. `exec.rs` states this, SECURITY.md does not (A1).
  * A `setsid` grandchild holding stdout does not hold the run past its limit: `Timeout` at 2.0 s.
* **The CLI's own death** (A1). I signalled the process group of `cargo-workplan` while it ran `-- sh -c 'sleep 7; touch /tmp/sec10s/orphan'`.
  * `cargo-workplan` died. That group is the one a terminal Ctrl-C or hang-up reaches.
  * The child sits in its own process group from `process_group(0)`. It ran on with no supervisor and no limit, created `orphan`, and left no record.
* **Program lookup** (A2).
  * From `crates/sub`, `-- ./tool.sh` ran `<root>/tool.sh`, not `crates/sub/tool.sh`. The child's cwd is the root.
  * With `PATH=.:$PATH`, `-- true` ran `<root>/true`.
  * The record says `./tool.sh` (or `true`) either way.
* **Environment.** It is passed unchanged, as documented, including any secrets the caller's environment holds (a registry token in mine). This is by design. Noted only.

### 2. Git rows for history

* **Argv rows match SECURITY.md and the code.**
  * `--follow --cc --root --format=%x1e%H%x1f%aI%x1f%s --name-only -- <path>`.
  * `-L<s>,<e>:<file> --no-patch --format=... --`.
  * `--root -S<identifier> -n 1 --format=%H%x1f%P%x1f%aI%x1f%s -- <dir>...`.
  * There is no `--since` argument: the date is compared in process with `%aI`.
  * The version gate is `git::require_version`, and the shallow check comes before any log.
* **Values that reach git.**
  * The id never reaches git. It is matched against the loaded headers.
  * Document paths are `docs/<type>/<kebab>.md`.
  * Bound files come from the source index, under containment, through `TreePath`. `TreePath` refuses a leading `:` or `/`, control characters, `\ * ? [`, empty, `.` and `..` segments, and anything over 4096 bytes.
  * So `:(top)x`, globs and newlines become a `HistoryError::Pattern` before any process starts. `-rf.rs` and `--output=x.rs` are pinned by `tests/history.rs` with real git.
  * Crate directories go through the same pattern. The root package is `TreePath::top()`.
  * The pickaxe word must match `[A-Za-z_][A-Za-z0-9_]*`, after the generic arguments and the `r#` prefix are stripped.
  * The `%P` parents are each held to the commit-id pattern before use. No id read back from git reaches another process.
* **Hostile output with real git** (clone `/tmp/s10c`, commit subject `ESC\e[31mRED\e[0m bidi U+202E ... U+2066 x 1e\x1e 1f\x1f bell\a cr\r end | [link](x) <b>`).
  * `history arch:normal-form` parsed it. The table shows `\u{1b}`, `\u{202e}`, `\u{2066}`, `\u{1e}`, `\u{1f}` and `\u{7}`, with `|`, `[`, `]` and `<` escaped.
  * JSON gives `\u001b`, `\u001e`, `\u001f` and `\u0007`. Bidi characters stay raw, which is valid RFC 8259.
  * With `-c color.ui=always -c color.diff=always`, the name-only lines are not coloured.
  * A 0x1e or 0x1f inside a subject stays inside its record (one `strip_prefix`, then `splitn(3)`).
  * Unquoting follows git's `quote_c_style` (`\a\b\t\n\v\f\r\"\\` and 3-digit octal). A malformed quote, or a path that is not UTF-8, is an error.
  * The 16 MiB bound is `log_query` on stdout, and an overflow on stderr is an `ExecError`. Both bounds are pinned by the T4 tests.

### 3. File writes

* **Exactly one file per command** (tree snapshots of `/tmp/s10`).
  * `assess cap:workplan-cli#usage --verdict accepted` and then `--verdict rejected`: only `work/assessments/workplan-cli-usage.md` changed.
  * `change new sec-probe-change --capability cap:workplan-cli ...`: one new file.
  * `evidence record`: one new file.
* **Read-only commands write nothing.** These left the snapshot byte-identical: `history` (cap and arch), `evidence claims`, `evidence for`, `assess --context`, `assess --list`, `coverage code`, `coverage tests` and `--format json coverage code`.
* **The CLI with symlinks.** Every symlinked `work/evidence` is refused before anything runs, because default package discovery (`roots ["."]`) refuses any symlinked directory:
  * a link to an outside directory;
  * a link to an in-root directory;
  * dangling links, inside or outside.
* **The library (`record::create` / `record::replace`, probe `tests/sec_writes.rs`).** These are refused, with nothing touched outside the root and no temporary file left:
  * `work/coverage` linked to an outside directory, an in-root directory, or dangling outside;
  * `work` linked in root, or dangling outside;
  * replace where `default.md` is a link to an outside file, a link to an in-root file, or a FIFO;
  * replace where `work/coverage` links to an outside directory holding `default.md`;
  * create where `default.md` is a dangling link (`Exists`);
  * replace in a read-only directory.
* **Concurrency.**
  * 16 concurrent `create`s: 1 succeeds and 15 get `Exists`.
  * 16 concurrent `replace`s all succeed, and only `default.md` remains.
* **The exception (R1).** `replace` follows an in-root directory link: see the finding.

### 4. raf-okf parsing

* **Hostile records under `work/evidence` through `check`.**
  * Each of these is a typed finding: an anchor or alias, a `!!map` tag, a duplicate key, 5000-deep flow nesting (`okf-parse-depth-exceeded`), and files over 1 MiB (`unreadable-file`, "larger than the limit").
  * 1024, 1025 and 60000 values load clean (A5).
* **`Number::compare`** is linear: one pass over the sign, then the whole part stripped of leading zeros (compared by length, then digits), then the fraction stripped of trailing zeros. `f64` is parsed only for display. The T5 cases still hold by code reading.
* **Claim markers** (debug build, document at the 1 MiB bound; a plain `check` takes 4.1 s):

  | Case | Result |
  | --- | --- |
  | 12679 claims | rc 0, 10.9 to 14.2 s |
  | 32535 unknown `raf:` comments | `okf-parse-unknown-marker` findings, 6.7 s |
  | One marker with about 29k ids | `okf-parse-malformed-claim` |
  | One 1 MB id | one finding, 12.9 s |
  | 20769 claims of a missing id | 20764 `claim-evidence-missing` lines, 11.3 s |

  Scaling from a quarter to the full size is roughly linear (the runs were noisy because a release probe ran beside them).

### 5. `Workspace::exported_items`

Release probe (`raf-source/tests/sec_exports.rs`, MemoryTree, one file). There were no panics or aborts, and every refusal was a typed `ExportError` or `LoadError`.

| Case | Result |
| --- | --- |
| 20k named re-exports | 37 ms |
| 20k external re-exports | 69 ms |
| Glob chain of 200 modules | 20300 items, 45 ms |
| Glob chain of 1000 modules | `Budget` (index records) after 1.2 s, 328 MB peak |
| Glob chain of 3000 modules | the load is refused by the glob-table budget (1.3 s) |
| Glob cycles of 2, 50 and 500 | 500 gives 250500 items in 0.8 s |
| 400 modules glob the root, which globs each | 320800 items in 1.0 s |
| `pub use self::*` | 1 item |
| `pub use b as a; pub use a as b;` | cycle error |
| Mutual globs through an external glob | correct |
| Glob chain of n modules, each with 40 external re-exports: n = 50 | 51000 re-exports in 1.4 s |
| Same, n = 100 | 202000 re-exports in 7.9 s |
| Same, n = 200 | `Budget` after **75 s**, 138 MB peak |
| Same, n = 500 (about 0.5 MB of source) | no answer after 4 min of CPU at 53 MB; stopped (A4) |

### 6. Digest

* The in-crate SHA-256 equals `sha256sum` on 313 pseudo-random inputs:
  * every length from 0 to 300 bytes;
  * 447, 448, 511, 512, 513, 1000, 4095, 4096, 4097 and 65536 bytes;
  * 1000003 and 5000000 bytes.
* Feeds of 3, 63, 64 and 65 pieces equal the one-shot hash.
* **The encoding** is `kind NUL name NUL len NUL bytes`.
  * Kinds are fixed words, and names (section slugs and resolved paths) hold no NUL. The length ends each field, so the encoding is unambiguous.
  * The file path of a module file is not hashed. Only its text and the order of the source key are. That is not a security issue.
* **What `Digest::symbol` reads.** It reads only the source index (`source.source(&item.file)`, `module_files` over `source.files()`) and opens no file of its own. So `#[path]` and symlink containment is the loader's, which refuses outside paths.

### 7. Output escaping

* The history and coverage tables escape every control character (`char::is_control`) and the bidi formatting characters (U+061C, U+200E and U+200F, U+202A to U+202E, U+2066 to U+2069). Their JSON escapes C0.
* The record-based text output does not escape (A3). I put `ESC[2J ESC]0;PWNED BEL` and U+202E into the claimed statement of `docs/capabilities/workplan-complete-gate.md`.
  * `check` exits 0.
  * `evidence claims gate-kills-a-test-past-its-timeout` and `assess cap:workplan-complete-gate#contract --context` print the bytes raw (`cat -v` shows `^[[2J^[]0;PWNED^G`). That clears the screen and sets the terminal title.
  * The JSON form escapes them.

### 8. SECURITY.md against behaviour

These statements contradict the code:

1. "`evidence record` writes one record, `work/evidence/<id>.md` ... and never a document".
2. "recording an assessment (`Workplan::record_assessment`) ... put in place of the assessment of the same section through a temporary file and a rename, refusing a link or any other file in the way, ... and never edits a document".

   Both fail through the public library path when `work/evidence` (or `work/assessments`) is a link to an in-root directory (R1).

Everything else I checked holds:

* argv form, no shell, stdin null, root cwd;
* environment unchanged with `LC_ALL` only for parsed git and cargo;
* 16 MiB and "no record";
* "a kill there is recorded as `fail`";
* "a background process that keeps its output open ... holds the run";
* `name=number` only, at most 1024;
* `command` and `revision` reach no process;
* the four value forms and the `TreePath` rules;
* refusal before root discovery (T9's M3 fixture);
* the read-only commands write nothing;
* the history rows, `%P` validation, shallow refusal, `--since` compared in process, and the 16 MiB logs;
* "in a partial clone Git itself may fetch".

"A 1800-second limit" holds only while `cargo-workplan` lives (A1). That is an omission, not a contradiction.

## Findings

### REQUIRED

**R1. `record::replace` follows an in-root directory link, so a replacement can overwrite a document.**

`files::create_new` judges every directory on the way (`work`, `work/<type>`) with `DiskTree::entry` and refuses a link. `record::replace` judges only the final path: `tree.entry(&path)` follows in-root links. It then renames onto `canonicalize(root).join(path)`.

The pre-checks have the same gap: `evidence::target_fits` and the replace test in `record_assessment` both use `symlink_metadata(root.join(path))`, which follows the parent directories.

Reproduction 1, library (`tests/sec_writes.rs` case 9):

* `root/work/coverage -> ../docs`, and `root/docs/default.md` exists.
* `record::replace(&root, &Record::Coverage(..), &RecordOptions::new())` returns `Ok("work/coverage/default.md")`.
* `docs/default.md` now holds `type: Coverage`.

Reproduction 2, through the documented `Gate` path (`tests/sec_gate.rs`):

* The workspace has the member `pkg`, `docs/capabilities/fixture.md` binding `fx_ev::add`, and the link `work/evidence -> ../docs/capabilities`.
* `Gate::workspace(root).source_roots(&["pkg"]).tracked_paths(vec![]).open()` succeeds. Its findings already list `work/evidence ... unreadable-directory`.
* Then `workplan().record_evidence(root, &SystemRunner::new(Programs::new(root)), &EvidenceRequest::new("fixture", vec!["cap:fixture"], Actor::Agent, VerdictSource::Command(argv)).mode(WriteMode::Replace))` returns `Ok`.
* `docs/capabilities/fixture.md` now starts `---\ntype: Evidence\nid: evidence:fixture`.
* The capability document was replaced by an evidence record.

`record_assessment` reaches the same `replace`. I found this by code reading and did not reproduce it separately.

The binary escapes this only because the default source roots happen to refuse every symlinked directory during package discovery. The write path itself does not hold to its contract.

Fix:

* In `replace`, judge each directory of `record.doc_type().directory()` exactly as `files::create_new` does. Factor the loop out and share it.
* Then judge the target. Make `target_fits` and `record_assessment` decide through the same judgement.
* Add negative fixtures, each with a passing neighbour of real directories:
  * `work/evidence` and `work/assessments` linked to an in-root directory that holds a file of the record's name;
  * `work` linked in root.

### ADVISORY

**A1. The trusted command outlives `cargo-workplan`.**

* The child leads its own process group, so a terminal Ctrl-C or hang-up kills only the CLI.
* Reproduction: `setsid` the CLI on `-- sh -c 'sleep 7; touch /tmp/sec10s/orphan'` and signal its group 5 s in. The CLI dies, `orphan` appears 7 s later, and no record is written.
* A `setsid` or `setpgid` descendant also escapes the timeout kill (reproduced). `exec.rs` states this, SECURITY.md does not.
* Either kill the child's group when the CLI receives SIGINT, SIGTERM or SIGHUP, or state both limits in SECURITY.md next to the 1800 s bound.

**A2. A relative program resolves against the workspace root, not the caller's directory.**

* `-- ./tool.sh` typed in `crates/sub` ran `<root>/tool.sh`.
* With `.` on `PATH`, a bare name resolves in the root.
* The record stores the text as typed, so it names a program other than the one the caller meant. Rust documents this resolution as platform-specific.
* Either refuse a program that contains a separator but is not absolute, resolve it against the caller's cwd first, or state the rule in SECURITY.md and the evidence-claims usage.

**A3. Document text reaches the terminal unescaped in the text output of `evidence claims` and `assess --context`.**

* Reproduction is in section 7: `check` exits 0, and ESC, OSC, BEL and U+202E are printed raw.
* The history and coverage renderers already escape these characters.
* Apply the same escape (`history::render::hidden`) to the record-based text of the new commands, or refuse C0 controls other than tab and LF in document bodies.

**A4. Re-export carry time grows super-linearly before the budget refuses it.**

* A glob chain of 200 modules, each with 40 external re-exports, is refused only after 75 s. 500 modules (about 0.5 MB of source) had no answer after 4 min of CPU, at about 50 to 140 MB.
* The result stays a typed error. Correctness is unaffected.
* SECURITY.md's Bounds paragraph states the time cost of settling, but not of `exported_items`, which `check` runs for the coverage scope.
* State it there, or charge the carry to a tighter work budget of its own.

**A5. A hand-written record is not held to the 1024-value bound.** A 60000-value record loads with no finding, so the loader accepts records the writer cannot produce. Consider a profile rule.

**A6. The `Display` of a trusted `Invocation` is ambiguous in error messages.** It joins the words unquoted, for example "`sh -c yes >&2` wrote more than ...". Use `TrustedArgv::text()` there.

**A7. The write checks are path-based, not descriptor-based.** A concurrent local writer could swap a directory for a link between the judgement and the `open` or `rename`. That is within the trust model, so this is a note only.

VERDICT: BLOCKED
