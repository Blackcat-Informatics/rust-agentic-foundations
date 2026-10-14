# R8 review: containment on every platform

**Current status (Recheck 3):** PASS. R2-1 is resolved: the source-index and SECURITY.md text matches the walk, and a test pins each strategy's answer. No REQUIRED findings are open. R2-A1 (running the walk on Linux too) remains a user-decision candidate, not a blocker. See `## Recheck 3`.

Independent security review of the uncommitted R8 change on top of cd7d4fb.

## Scope

- Contract: `### R8` of `remediation-plan.md`, the Mutation rule and the Per-item gates. Background: S-6, S-7, S-8 and S-14 in `reviews/security-review.md`, and G3 and G7 in `gap-analysis.md`.
- Files reviewed:
  - `crates/raf-source/src/tree/disk.rs`
  - `crates/raf-source/src/tree/disk/{linux,other_unix,portable}.rs` (new)
  - `crates/raf-source/src/tree/disk/tests.rs`
  - `crates/raf-source/tests/workspace_containment.rs`
  - `docs/capabilities/source-index.md`, `SECURITY.md`, `AGENTS.md`
  - `Makefile` (`cross-check`) and `.github/workflows/ci.yaml` (`test-macos`, `test-windows`)
- Method:
  - Old and new copies were made outside the worktree:
    - `/tmp/r8rev/old` = `git archive cd7d4fb`;
    - `/tmp/r8rev/new` = cd7d4fb + `git diff | git apply -p0` + the three untracked files.
  - Every probe, mutation and port ran only in those copies. The worktree was not edited; `git status` is unchanged.

## Evidence

### Gates (worktree, R8 applied)

| Command | Result |
|---|---|
| `cargo test -p raf-source --locked` | exit 0; 309 passed, 0 failed, 1 ignored (23 suites) |
| `cargo test -p raf-source --locked --release` | exit 0; 309 passed, 0 failed, 1 ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `make cross-check` (now `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets`, Apple and Windows) | exit 0 |
| `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-pc-windows-gnu` | exit 0 |
| `RUSTFLAGS=-Dwarnings cargo check -p raf-source --locked --all-targets --target x86_64-apple-darwin` | exit 0 |
| `make msrv` (rust-validate 1.98.1, workspace and fuzz) | exit 0 |
| `cargo test --locked --test repo_gates --test docs_contract` | exit 0; 139 + 55 passed |
| `make reuse` | exit 0; 223/223 files compliant |
| `actionlint .github/workflows/ci.yaml` | exit 0 |

Further checks:
- `grep unsafe|#![feature` over `crates/raf-source/src` and `tests`: no `unsafe` and no `#![feature]`. The only `#[allow]` hits are pre-existing, in `index.rs`, `facts.rs` and `tests/symbol_index.rs`.
- Process-wording scan of the diff and the new files: none found. There are no item IDs, issue numbers or stage wording.

### Native race probe (Linux strategy, 200k iterations)

`race_open_counted` was rebuilt in `/tmp/r8rev/sec` against the worktree.

```
mode=fifo iters=200000 read_ok_inside=90850 read_ok_OUTSIDE_CONTENT=0 outside_root=109150
outside FIFO opened by a reader before the final unblock: 0
entry() reported an outside-only name: 0; list() listed an outside-only name: 0
mode=file iters=200000 read_ok_inside=98985 read_ok_OUTSIDE_CONTENT=0 outside_root=101015
entry() reported an outside-only name: 0; list() listed an outside-only name: 0
```

This reproduces the implementer's figures: 0 outside FIFO opens and 0 outside entry answers.

### Strategy race probe (all three strategies on Linux through the test override, 200k iterations each)

- **Setup:** a probe test was added in the `/tmp/r8rev/new` copy only (`tests.rs`, `mod probe`). A thread runs `renameat2(RENAME_EXCHANGE)` between `root/d` and `root/swap -> out`.
- **Outside files:**
  - `out/f.rs` = "OUTSIDE-SECRET";
  - `out/p.rs` is a FIFO, with a writer looping on a blocking open;
  - `out/only_outside` and `out/outsub/` exist only outside.

```
PROBE Linux     read_in=37013 READ_OUTSIDE_CONTENT=0    UNSEEN_OUTSIDE_CONTENT=0    FIFO_OPENED=0      ENTRY_OUTSIDE_SOME=0    LIST_OUTSIDE_NAMES=0  entry_outsub_OutsideRoot=69333 entry_nosuch_OutsideRoot=0
PROBE OtherUnix read_in=881   READ_OUTSIDE_CONTENT=3101 UNSEEN_OUTSIDE_CONTENT=2908 FIFO_OPENED=240289 ENTRY_OUTSIDE_SOME=992  LIST_OUTSIDE_NAMES=92 entry_outsub_OutsideRoot=69853 entry_nosuch_OutsideRoot=0
PROBE Portable  read_in=25393 READ_OUTSIDE_CONTENT=2241 UNSEEN_OUTSIDE_CONTENT=2514 FIFO_OPENED=259982 ENTRY_OUTSIDE_SOME=1392 LIST_OUTSIDE_NAMES=86 entry_outsub_OutsideRoot=76287 entry_nosuch_OutsideRoot=0
```

### Deterministic interleavings (copy only; a test-only `mid()` hook added at fixed points)

| Case | Interleaving | Result |
|---|---|---|
| Portable | swap before the open (existing `before_open` hook), undo after `open_handle` and before the re-canonicalisation | `tree.read("dir/file.rs") = Ok("OUTSIDE-SECRET")` |
| OtherUnix | flip at `before_open`, after `open_handle`, between `canonicalize` and `metadata` in `recheck`, at `after_verify`, and again between `canonicalize` and `metadata` | `Ok("OUTSIDE-SECRET")` |
| Linux, final component | `before_open` replaces `dir/file.rs` with a symlink to an outside FIFO | `Err(NotRegular)`. The `O_PATH` reference is fstat'ed as a symlink and refused before any reopen; nothing blocks. |
| Linux, read | `after_verify` renames `file.rs` and writes a replacement at the old name | `Ok("inside")`. The `/proc/self/fd/N` reopen follows the referenced inode, not the name. |
| Linux, list | `after_verify` renames `sub` and creates a new `sub` holding `new.rs` | `Ok(["orig.rs"])`. `read_dir("/proc/self/fd/N")` lists the referenced directory after the rename. |

### `/proc` absent (Linux)

`unshare --user --map-root-user --mount --pid --fork`, with tmpfs mounted over `/proc`, then the worktree's unit test binary:
- `read: Io { path: "dir/file.rs", source: NotFound }`
- `list: Io { path: "sub", ... }`

Both fail hard as `Io`. There is no fallback and no `NotFound`. `hidepid` does not apply to `/proc/self`.

### Single-open property (`read_unseen`, Linux strategy)

`strace -f -e openat,readlink,statx,...` of `src_load` over this repository (`ok files=102`):
- **One `O_PATH` reference per file:** each of the 102 `.rs` files is opened exactly once with `O_RDONLY|O_NOFOLLOW|O_CLOEXEC|O_PATH`.
- **No by-name read open:** in-tree paths were opened without `O_PATH` 0 times. All 136 read and listing opens go through `/proc/self/fd/N` (113 `O_RDONLY`, 23 `O_DIRECTORY`).
- **Per-file sequence:** `openat(path, O_PATH|O_NOFOLLOW) = 3` → `readlink(/proc/self/fd/3)` → `statx(3, "", AT_EMPTY_PATH)` → `openat(/proc/self/fd/3, O_RDONLY) = 4` → `readlink(/proc/self/fd/3)`.
- **Identity and text agree:** the identity comes from the `statx` of the reference and the text from the reopen of the same inode.
- **Other strategies:** OtherUnix and Portable take the identity and the text from the same handle, as before. The non-Unix identity is the canonical path; see finding 1.

### Falsifiability

- **FIFO test against the old code:** `an_outside_fifo_reached_by_a_swap_before_the_open_is_never_opened` was ported to `/tmp/r8rev/old` (cd7d4fb, which already has `before_open`). It fails there: `the outside FIFO was opened, releasing its writer: Ok(Ok(File { ... outside/dir/file.rs, write: true }))`.
- **FIFO test is stable:** on the new code it passes 30/30. The `tree::disk` suite passes 10/10.
- **Entry swap test against the old code:** the `before_open` variant of `an_entry_whose_directory_is_swapped_for_an_outside_symlink_is_refused` was ported with a hook placed after `contain(parent)` in the old `entry`. It fails there with `Ok(Some(File))`.
- **Mutation 1** (copy): `O_PATH` replaced by `O_NONBLOCK` in `linux::open`. Only the FIFO test fails, 8/9 swap tests pass. The FIFO test is the one that pins the `O_PATH` property. Reverted.
- **Mutation 2** (copy): the `recheck` after `read_entries` in `list` removed. `a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` fails. Reverted.

### Windows code, by reading

- **Held handle:** `share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)` withholds delete sharing. `custom_flags(FILE_FLAG_BACKUP_SEMANTICS = 0x0200_0000)` is set. `handle_metadata` (the former `on_handle`) is the handle's metadata.
- **Reserved names:** the list is CON, PRN, AUX, NUL, COM0-9, LPT0-9, COM¹²³, LPT¹²³, CONIN$ and CONOUT$. It is case-insensitive and takes the stem before the first `.` or `:` with trailing spaces trimmed. A trailing dot or space is refused, and `..` is exempt. This matches the Win32 naming rules and over-refuses only where that is harmless.
- **Where the refusal runs:** before canonicalising, in `read`, `read_unseen`, `identity`, `list` and `entry`.
- **`..` join note:** the Windows statement about it matches std's verbatim-prefix `push`.
- **The defect:** the comparison is path against path and never involves the held handle. See finding 1.

### CI

- `test-macos` and `test-windows` use the same pinned checkout and `dtolnay/rust-toolchain` SHAs as the other jobs, with `toolchain: nightly` and `rust-toolchain.toml` honoured.
- `test-windows` sets `core.autocrlf false` before the checkout.
- `actionlint` is clean.
- Neither job has run (hosted CI is blocked; see G6). Runtime on macOS and Windows is **unverified**.

## Findings

### 1. REQUIRED: on other Unix and Windows a swap made and undone across the checks returns an outside file's contents, and every doc says "never"

- **Defect**
  - **OtherUnix:** `recheck` runs `fs::canonicalize(canonical)` and then `fs::metadata(&after)`. These are two independent path walks. When the walk inside `realpath` sees the real directory and the `stat` walk sees the swapped link, the result is an in-root string whose `(dev, ino)` is the outside file the handle holds. The check passes.
  - **Portable:** this is worse. `verify_portable_path` compares the re-canonicalised path to the opened path and never consults the handle. One swap before the open, undone before the check, is accepted. On Windows the held handle without delete sharing pins only the opened (outside) file. It does not pin the junction or the parked in-root directory, so the undo is possible there too.
- **Evidence**
  - The deterministic interleavings above return `Ok("OUTSIDE-SECRET")` for both strategies.
  - In the 200k race, OtherUnix read outside contents 3101 times (`read_unseen` 2908) and Portable 2241 (2514). The identity returned with that text is the in-root identity.
- **False statements**
  - `SECURITY.md:66-71` ("refused before any read", "never their contents")
  - `AGENTS.md:104-108`
  - `docs/capabilities/source-index.md:223-230, 235-236`
  - `crates/raf-source/src/tree/disk/other_unix.rs:9-10`
  - `crates/raf-source/src/tree/disk/portable.rs:10-11`
  - `crates/raf-source/src/tree/disk.rs:7-8` ("none of them trusts a check that the next one could outlive")
  - The plan prescribed the "refused before any read" sentence, but the code does not support it.
- **Fix**, one of:
  - **(a)** Restate the residual window per platform. On other Unix and Windows, a swap made before the open and undone around the checks can make a read return the contents of an outside regular file the process can read; listings and entries can likewise return names and kinds. Scoping this to non-Linux is within user decision 3. Pin it with a deterministic test-only hook between the open and the first check, so the documented window is exercised under OtherUnix and Portable.
  - **(b)** Close the window with a handle-derived path or a beneath-root walk: `F_GETPATH` on macOS, an `openat` component walk on the BSDs, `GetFinalPathNameByHandleW` on Windows. Each needs FFI, which means a new dependency or `unsafe`, so it is a user decision.

### 2. ADVISORY: canonicalisation still answers whether an outside path exists, on Linux too

- **Behaviour:** `contain` maps a failed `canonicalize` to `NotFound` and an outside result to `OutsideRoot`. `entry` turns `NotFound` into `Ok(None)`.
- **Measurement:** under a swap of `d`, Linux `entry("d/outsub/x")` returned `OutsideRoot` 69,333 times, while `entry("d/nosuch/x")` never did. So the call answers whether `out/outsub` exists. The same holds for `read` (`NotFound` vs `OutsideRoot`) and for `list`'s path `symlink_metadata(candidate)` pre-check (`NotFound`/`NotRegular` vs proceeding).
- **Severity:** it is one bit per call, needs a local renamer, and was present before R8. The probe's "0 outside entry answers" counts only `Some(kind)`.
- **Fix:** SECURITY.md contrasts Linux with "the kind of an outside path" elsewhere. Add one sentence that, on every platform, a racing swap can reveal whether an outside path exists through the error returned.

### 3. ADVISORY: macOS listing through `/dev/fd/<fd>` is unverified and may fail

- `opendir` opens with `O_DIRECTORY`. If XNU checks `O_DIRECTORY` against the fdesc vnode type before fdesc dups the descriptor, every macOS `list` would fail with an `Io` `ENOTDIR`. That would fail hard, not leak, but it would break loading on macOS.
- On Linux the `cfg!(target_os = "macos")` branch never runs.
- The docs (`SECURITY.md:68-69`, `AGENTS.md:105-106`, `source-index.md:225-226`) state the macOS listing as fact. Mark it unverified until `test-macos` has run green.

### 4. ADVISORY: a Windows `:` stream segment is not refused

- `RelPath` accepts `lib.rs:payload` (only a drive-letter colon is refused), and on Windows that reads an alternate data stream of an in-root file.
- This is not an escape, but Win32 does not read it "as the name it spells". The same holds for `x::$DATA`.
- **Fix:** have `is_ambiguous_on_windows` refuse any segment containing `:` (NTFS forbids it in names), to match the stated rule.

### 5. ADVISORY: cosmetic reflow

- `AGENTS.md:143-144` leaves a dangling "`make`" at a line end.
- `source-index.md:147-148` has an over-long line.

## Result

- **Linux:** the implementation does what the docs claim.
  - An outside candidate gets only an `O_PATH` reference.
  - Non-regular and symlink finals are refused before any reopen.
  - The `/proc` reopen and `read_dir` follow the referenced inode.
  - A missing `/proc` fails hard.
  - The single-open property holds.
  - The FIFO and entry tests fail on the old code.
- **Other Unix and Windows:** the security statements for these platforms are false. Their verifiers are defeated by a swap made and undone around the checks, and outside file contents are returned. Finding 1 must be resolved, by restating the window (with a pinning test) or by closing it, before R8 can pass.

VERDICT (first pass): BLOCKED

## Recheck

### Scope of the recheck

- **The rework under review** (uncommitted on cd7d4fb), applied by the user decision "Add rustix + windows-sys" in `validation.md`:
  - `tree/disk.rs` holds the shared `Handle` type.
  - `tree/disk/linux.rs`: the Linux strategy, unchanged in behaviour.
  - `tree/disk/other_unix.rs`: the rustix `openat` walk from a root descriptor.
  - `tree/disk/handle_path.rs`: the Windows strategy, run on Linux in tests with `/proc/self/fd`.
  - `tree/disk/final_path.rs`: the single FFI call.
  - `tree/mod.rs` (`FileId::Native` is now Windows only), the new tests, `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`, and the three docs.
- **Copy used for attacks:** `/tmp/r8rev/new2` (cd7d4fb + the diff + untracked files; identical to the worktree when copied).
  - It adds a test-only `mid()` hook at 6 points of the walk: before and after each `openat`, before `statat`, before `readlinkat`, each `climb` step, and before the recheck `statat`.
  - It adds an `attack` test module.
- The worktree was not edited.

### Gates (worktree)

| Command | Result |
|---|---|
| `cargo test -p raf-source --locked` | exit 0; 313 passed, 0 failed, 1 ignored |
| `cargo test -p raf-source --locked --release` | exit 0; 313 passed, 0 failed, 1 ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy -p raf-source --all-targets --locked --target x86_64-pc-windows-gnu -- -D warnings` | exit 0 (lints the `final_path.rs` FFI and the Windows tests) |
| `cargo clippy -p raf-source --all-targets --locked --target x86_64-apple-darwin -- -D warnings` | exit 0 |
| `make cross-check` | exit 0 |
| `make msrv` | exit 0 |
| `make deny` | exit 0 (advisories, bans, licenses and sources ok) |
| `make reuse` | exit 0 |
| `cargo test --locked --test repo_gates --test docs_contract` | exit 0; 139 + 55 passed |
| `cargo metadata --locked` and `cargo check --locked` on `fuzz/Cargo.toml` | exit 0 |
| `make check` | exit 0; 1,108 tests passed, 0 failed |

### Original REQUIRED finding 1: RESOLVED

**Fixed-order attack** (the interleaving that returned `Ok("OUTSIDE-SECRET")` in the first pass):
- The committed `a_swap_before_the_open_undone_before_the_check_is_refused` makes the swap before the open and undoes it after the open, for `read`, `read_unseen`, `list` and `entry`. `a_swap_flipped_at_every_hook_point_is_refused` covers the flip at every point. Both pass under Linux, OtherUnix and Windows.
- **Exhaustive flip schedules** (copy, `attack::exhaustive_schedules`):
  - Every subset of flips (`renameat2` exchange of `root/d` with `root/swap -> out`) over the first 16 interleaving points: 65,536 schedules × 5 calls per strategy.
  - Points include `before_open`, `after_open`, `after_verify` and the 6 walk points. OtherUnix reaches 23 points per call.

```
SCHED Linux     schedules=65536 FIFO_OPENED=0     read_out=0 unseen_out=0 entry_out=0     list_out=0
SCHED OtherUnix schedules=65536 FIFO_OPENED=0     read_out=0 unseen_out=0 entry_out=0     list_out=0
SCHED Windows   schedules=65536 FIFO_OPENED=49654 read_out=0 unseen_out=0 entry_out=16384 list_out=16384
```

**200k race** (copy, `attack::race_probe2`, with a continuous exchange thread):

```
RACE Linux     FIFO_OPENED=0     read_out=0 unseen_out=0 entry_out=0    list_out=0
RACE OtherUnix FIFO_OPENED=0     read_out=0 unseen_out=0 entry_out=0    list_out=0
RACE Windows   FIFO_OPENED=51340 read_out=0 unseen_out=0 entry_out=4856 list_out=258
```

- **Before and after:** outside contents went from 3,101 (OtherUnix) and 2,241 (Portable) in the first pass to 0 under every strategy.
- **Residual in the Windows model:** outside names and kinds through `list` and `entry`, and the open of an outside file. The docs state this residual.
- **Native probe** (`race_open_counted`, 200k, FIFO mode): 0 outside contents, 0 FIFO opens, 0 outside entry answers, 0 outside listings.

### Attacks on the openat walk (OtherUnix; Linux and Windows models alongside)

**Symlinks to `..`, absolute links, and the hop limit** (copy, `attack::links_dotdot_limits`). All three strategies agree:
- `up -> ..`: `up/secret.rs` is `OutsideRoot`; `list("up")` and `entry("up/secret.rs")` are `OutsideRoot`.
- `sub/up -> ..`: `sub/up/real.rs` reads.
- `sub/up2 -> ../..`: `OutsideRoot`.
- `slashes -> /../../..` and `fsroot -> /`: `…/etc/hostname` is `OutsideRoot`.
- `viaup.rs -> sub/up/real.rs` reads.
- **Hop limit:** chains of 39 and 40 links read, and 41 links is `SymlinkLoop`, as with the Linux kernel's 40.

**Links to the root through other paths:**
- The committed `every_strategy_follows_an_absolute_in_root_link_spelled_through_a_linked_parent` passes. The walk re-enters at the root's device and inode, and `spelled_root/..` is `OutsideRoot`.

**An intermediate directory renamed out of the root mid-walk** (copy, `attack::rename_mid_walk`):
- `a` was renamed out of the root at each of the first 8 walk points. A rename before `a` is opened gives `NotFound`, the in-root answer. A rename after it gives `OutsideRoot`.
- In the latter case the walk continues inside the moved directory, and the post-use `climb` refuses it. What was read is the moved directory's own file, which is discarded.

**A directory moved out during a listing:**
- The committed `a_directory_swapped_after_verification_before_the_entries_are_read_is_refused` passes under OtherUnix. The listing goes through `Dir::read_from` on the handle, and `climb` refuses it.

**An entry that flaps between a symlink and a directory** (copy, `attack::flapping_entry_terminates`, 100k reads):
- Result: `{Ok(inside): 45472, Ok(inside2): 53670, NotFound: 858}`. Every walk terminates and no outside answer is returned.
- The `NotFound` answers are advisory R-A1.

**FIFOs, devices and outside finals:**
- An outside final component is refused before any open (`if last && !self.inside`). Outside, only `O_DIRECTORY|O_NOFOLLOW|O_NONBLOCK` directory opens occur.
- On Linux, `O_DIRECTORY` fails `ENOTDIR` before any FIFO or device open.
- The committed OtherUnix FIFO race test passes.

**Mutations** (copy, each reverted):
- Removing the `last && !inside` refusal fails 5 tests: the FIFO race, the outside-symlink fixtures, `..` after an outside link, the listing refusal, and the swap after the check.
- Making the post-use recheck skip `climb` fails 3 tests: the listing moved out, the file moved out, and the entry swap.

**Single-open property:**
- Linux strategy: `strace` of a full load of this repository (103 files) shows exactly one `O_PATH` open per `.rs` file, 0 by-name read opens, and every read and listing goes through `/proc/self/fd`.
- OtherUnix: the identity is the `fstat` of the walked descriptor and the text is read from that same descriptor.
- Windows: the identity is the verified final path of the held handle and the text is read from that handle. This also closes the first pass's note that the non-Unix identity came from a path.

### unsafe

- **Where:** `grep -rn unsafe crates fuzz` finds library `unsafe` only in `crates/raf-source/src/tree/disk/final_path.rs`. It is one function, `of`, under an item-scoped `#[allow(unsafe_code, reason = ...)]`, with one `unsafe` block. The only other `unsafe` in the repository is the pre-existing counting allocator in `crates/raf-okf/tests/bounds.rs` (a test).
- **Lints:** the workspace `unsafe_code = "deny"` is unchanged. `allow_attributes_without_reason` is satisfied, and the Windows-target clippy run is clean.
- **SAFETY argument: correct.**
  - The handle is borrowed from a live `File` for the whole call.
  - `capacity` is the exact length of the live, writable `Vec<u16>`.
- **Return handling: correct.**
  - 0 → `last_os_error`.
  - `written < len` → success, truncated to `written` (which excludes the NUL).
  - Otherwise → resize to the required size (which includes the NUL) and retry, which also covers a path that grows between calls.
- **Edge case:** see advisory R-A3.

### Dependencies

- **Versions:** `rustix 1.1.5` (`fs` and `std`, no default features) is a normal dependency on non-Linux Unix and a dev-dependency on Linux, so production Linux does not link it. `windows-sys 0.61.2` (`Win32_Foundation` and `Win32_Storage_FileSystem`) is used on Windows.
- **New lock entries:** `bitflags 2.13.2`, `errno 0.3.14`, `linux-raw-sys 0.12.1`, `rustix 1.1.5`, `windows-link 0.2.1` and `windows-sys 0.61.2`, identical in `Cargo.lock` and `fuzz/Cargo.lock`.
- **Duplicates:** `cargo tree -d --locked -e normal,dev,build` shows none for `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `x86_64-pc-windows-gnu` and `aarch64-unknown-freebsd`.
- `make deny` passes.

### Earlier advisories

- **2** (canonicalisation existence oracle): the docs now state it for Linux and Windows. It is superseded by REQUIRED finding R-1 below for the other-Unix sentence.
- **3** (macOS `/dev/fd` listing): **resolved.** `/dev/fd` is gone, and the listing uses `rustix::fs::Dir::read_from` on the walked descriptor (`openat(fd, ".")` + `fdopendir`). Runtime on macOS is still unverified until the `test-macos` job runs.
- **4** (Windows `:` segments): **resolved.** `is_ambiguous_on_windows` refuses any segment holding `:`. The test covers `lib.rs:payload`, `x::$DATA` and `:`, and the docs say so.
- **5** (reflow): accepted. The remaining wraps split inline code across a line break, which renders correctly. The long lines are binding lists.

### Findings of the recheck

#### R-1. REQUIRED: the other-Unix walk does tell an outside path that exists from one that does not

- **Repro** (copy, `attack::oracle_out_and_back`). The tree holds `via_there.rs -> <scratch>/outside/there/../../root/real.rs` and `via_missing.rs -> <scratch>/outside/missing/../../root/real.rs`. No race is needed.

```
ORACLE Linux     via_there=Ok("real") via_missing=Err(NotFound)
ORACLE OtherUnix via_there=Ok("real") via_missing=Err(OutsideRoot)
ORACLE Windows   via_there=Ok("real") via_missing=Err(NotFound)
```

- **Cause:** while outside the root, the walk takes `..` lexically back through the directories it opened. A link that leaves the root and returns therefore succeeds exactly when the outside directories on the way exist.
- **False statements:**
  - `SECURITY.md`: "The other-Unix walk answers `OutsideRoot` for both".
  - `docs/capabilities/source-index.md`: the same sentence after the platform bullets, and the Verification bullet "the other-Unix walk answers `OutsideRoot` for both".
  - `other_unix.rs` module docs: "the walk does not tell an outside path that exists from one that does not".
  - The committed test name `whether_an_outside_path_exists_is_told_apart_only_by_canonicalisation`, which only uses links that never return.
- **Incomplete statement:** the Linux and Windows sentence says the call fails with `NotFound` vs `OutsideRoot`. It can also succeed (`Ok` vs `NotFound`).
- **Fix**, one of:
  - Refuse a `..` taken while the walk is outside the root (`Step::Up` with `!inside` → `OutsideRoot`). The other-Unix claim then holds, except through an outside symlink that itself points back into the root. Say so, and add the out-and-back fixture.
  - Or restate the docs and the test to admit the out-and-back oracle on every platform.

  In both cases, extend the Linux and Windows sentence to the `Ok` vs `NotFound` case.

#### R-A1. ADVISORY: a racing symlink↔directory flip yields a spurious `NotFound` in the walk on Linux

- **Cause:** on Linux, `openat(O_NOFOLLOW|O_DIRECTORY)` on a symlink fails `ENOTDIR`, not `ELOOP` (checked with `os.open`). When the entry is a directory again by the time of `statat`, the error is neither `LOOP` nor `MLINK`, so the step is not retaken and `ENOTDIR` maps to `NotFound`.
- **Measurement:** 858 of 100k reads in the flap test.
- **Severity:** fail-safe; no outside answer. It affects only the Linux test model and any BSD that reports `ENOTDIR`.
- **Fix:** retake whenever `statat` now reports a type the open would have accepted (for example `NOTDIR` with a directory now present).

#### R-A2. ADVISORY: the walk holds one descriptor per component

- `ancestors` keeps every directory descriptor until the walk ends.
- **Measurement:** with `RLIMIT_NOFILE` = 128, a 200-deep path fails OtherUnix with `Io` "Too many open files", while Linux and Windows read it.
- **Impact:** macOS's default soft limit is 256, so paths about 250 components deep fail hard there.
- **Fix:** drop ancestors that no `..` left in the queue can reach, or document the limit.

#### R-A3. ADVISORY: `final_path::of` can loop forever on a buffer-size return of exactly `len`

- If `GetFinalPathNameByHandleW` ever returned exactly `buffer.len()`, `resize(written)` would keep the same size and retry forever. The documentation says this cannot happen, and std's own helper guards the equivalent case.
- **Fix:** grow strictly (`written.max(buffer.len() + 1)`) or cap the iterations.

#### R-A4. ADVISORY: two wording overclaims

- **"outside the root only directories are opened, on the way back to it"** (SECURITY.md, source-index.md): outside directories along any absolute link target are opened whether or not the walk returns.
- **"no rename can redirect the walk out of the root"**: a directory renamed out mid-walk is still walked; it is the post-use `climb` that refuses (rename_mid_walk above).
- Reword both, for example: "an outside non-directory is never opened; a walk redirected by a rename is refused by the check after the use".

#### R-A5. ADVISORY: runtime on macOS and Windows is unverified, and BSDs are not compiled

- macOS and Windows runtime stays unverified until hosted CI runs.
- `make cross-check` compiles macOS and Windows only. "The BSDs" (and the `EMLINK` branch) are not compiled anywhere, because the FreeBSD target is not installed here.

VERDICT (Recheck 1): BLOCKED

## Recheck 2

### Scope of Recheck 2

- **Round 3 of R8** (uncommitted):
  - **R-1:** the walk refuses any `..` taken outside the root, and re-enters the root only by descending into its device and inode.
  - **R-A1:** a changed entry is retaken on `ENOTDIR`, `ELOOP`, `EMLINK` and `ENOENT`, with a `mid_step` hook for the test.
  - **R-A2:** the walk holds one current descriptor plus the recorded `(dev, ino)` of its ancestors.
  - **R-A3:** the final-path buffer grows strictly.
  - **R-A4:** reworded.
  - **R-A5:** FreeBSD is added to `make cross-check`, CI and AGENTS.md.
- **Copy used for attacks:** `/tmp/r8rev/new3` (cd7d4fb + the diff + untracked files; identical to the worktree when copied).
  - It adds the test-only `mid()` hook at 7 points of the walk: before and after `openat`, before `statat`, before `readlinkat`, at `up`, in `climb`, and at the recheck `statat`.
  - It holds the round-2 `attack` module and a new `round3::oracle_matrix`.
- The worktree was not edited.

### Gates (spot-checked on the worktree)

| Command | Result |
|---|---|
| `cargo test -p raf-source --locked` | exit 0; 315 passed, 0 failed |
| `cargo test -p raf-source --locked --release` | exit 0; 315 passed, 0 failed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| `cargo clippy -p raf-source --all-targets --locked --target x86_64-pc-windows-gnu -- -D warnings` | exit 0 |
| `make cross-check` (`CROSS_TARGETS` = apple-darwin, unknown-freebsd, pc-windows-gnu; all three installed) | exit 0 |
| `make deny`, `make msrv`, `make reuse` | exit 0 |
| `cargo test --locked --test repo_gates --test docs_contract` | exit 0 |
| `cargo metadata --locked` on `fuzz/Cargo.toml` | exit 0 |
| `actionlint` | exit 0 |

- **`unsafe`:** library `unsafe` is still the single item in `final_path.rs`. Its growth is now strict: `written.max(buffer.len() + 1)`.

### R-1: out-and-back attacks (copy, `round3::oracle_matrix`)

Each row is a pair of in-root links whose targets differ only in whether an outside directory exists. Each cell gives the `read` of the link, then `entry(link/x)`.

| Pair | Linux / Windows (exists vs missing) | Other Unix (exists vs missing) |
|---|---|---|
| absolute out-and-back `/…/outside/there/../../root/real.rs` | `Ok`, `None` vs `NotFound`, `None` | `OutsideRoot` vs `OutsideRoot` (both calls) |
| relative out-and-back from the root `../outside/there/../../root/real.rs` | `Ok` vs `NotFound` | `OutsideRoot` vs `OutsideRoot` |
| relative out-and-back from `sub/` `../../outside/…` | `Ok` vs `NotFound` | `OutsideRoot` vs `OutsideRoot` |
| absolute, ends outside `/…/outside/there/deeper` | `OutsideRoot` vs `NotFound`; entry `OutsideRoot` vs `None` | `OutsideRoot` vs `OutsideRoot` |
| `..` straight after re-entering `/…/root/../root/real.rs` | `Ok` vs `NotFound` | `OutsideRoot` vs `OutsideRoot` |
| re-enter, then an in-root `..`: `/…/root/sub/../real.rs` vs `/…/root/nosub/../real.rs` | `Ok` vs `NotFound` | `Ok` vs `NotFound` (an in-root fact only) |
| descend through an outside absolute link to the root: `/…/outside/there/to_root/real.rs` | `Ok` vs `NotFound` | `Ok` vs `OutsideRoot` (the documented exception) |
| descend through an outside relative link `to_root -> ../../root` | `Ok` vs `NotFound` | `OutsideRoot` vs `OutsideRoot` (the `..` is taken outside) |
| root spelled through another path: `/…/alias/root/real.rs` | `Ok` vs `NotFound` | `Ok` vs `OutsideRoot` (the documented exception) |
| in-root target spelled by climbing out by name: `../root/real.rs` | `Ok` vs `NotFound` | `OutsideRoot` vs `OutsideRoot` |

- **Resolved:** the out-and-back oracle on the walk, whether the link is absolute, relative or mixed, and whether `..` comes straight after re-entering.
- **What the walk still reveals:** only whether an outside path exists that *descends* into the root. That means a spelling of the root's own ancestors, a bind mount, or an outside link whose resolution descends.
  - The attacker cannot place such a path inside a victim's directory.
  - So the walk's answer reveals nothing about outside paths in general, only whether a given path leads into the root.
- **Docs:** this exception is stated accurately in `SECURITY.md`, in `source-index.md` (the paragraph after the platform bullets) and in the `other_unix.rs` module docs. The Linux and Windows pairs are stated exactly: ends outside gives `OutsideRoot` vs `NotFound`/`None`, and out-and-back gives the in-root file vs `NotFound`. The matrix and the committed `links_into_the_outside_are_answered_alike_only_by_the_walk` confirm them.
- **Falsifiability:** the committed test fails against round 2. In Recheck 1, `oracle_out_and_back` gave `via_there=Ok("real")` under OtherUnix.
- **Redundant guard:** the `!self.inside` guard in `up()` is redundant. The walk records no ancestors outside the root, so `..` there is refused by the empty stack anyway. Removing the guard is an equivalent mutant, not a gap.

### Is the remaining Linux and Windows answer acceptable under the user's decisions?

- **Linux:** Stage 1 decision 3 (`plan.md`) fixes Linux as canonicalise, open, and verify the handle's kernel path. `issue.md` declined `openat2(RESOLVE_BENEATH)` via `rustix` for Linux. The Stage 2 decision (`validation.md`) authorises `rustix` "for handle-relative opens and listing on non-Linux Unix". So keeping canonicalisation on Linux, with its existence answer documented, is within the recorded decisions.
- **Windows:** there is no handle-relative open there without more FFI (`NtCreateFile` with a root directory), which would exceed the one authorised `unsafe` item.
- **Severity of what remains:** the information is one bit per link: whether a path the loading process can resolve exists. The attacker needs to plant a link in the tree and observe the load's error. It never yields contents.
- **Judgement:** acceptable as documented. It is flagged below (R2-A1) as a user-decision candidate, not a blocker.

### R-A1 and R-A2: the tests fail when the fix is removed (copy, each reverted)

- **R-A1 mutation:** the `Errno::NOTDIR` arm of `changed()` made `false`. `a_symlink_that_becomes_a_directory_during_a_step_is_stepped_again` FAILS, and the other 10 `strategy` tests pass.
  - Under the continuous flip, the round-2 spurious answers (858 `NotFound` in 100k) are now 0: `{Ok(inside): 22456, Ok(inside2): 77544}`.
- **R-A2 mutation:** `descend` leaks the previous directory descriptor (`mem::forget`), which is equivalent to holding one per component. `a_path_three_hundred_deep_is_walked_under_a_limit_of_sixty_four_descriptors` FAILS with `Io … Too many open files`. It passes once reverted.
  - The child process asserts that the limit is in force before it walks.

### Earlier attacks re-run on round 3 (copy)

- **Exhaustive flip schedules:** 16 points × 65,536 schedules × 5 calls (OtherUnix reaches 26 points).
  - Linux and OtherUnix: 0 outside contents, names or kinds, and 0 FIFO opens.
  - Windows model: 0 contents; the documented name and kind residual remains.
- **200k race:**
  - Linux and OtherUnix: 0 of every kind.
  - Windows model: 0 contents; residual of 10,179 kinds and 548 listings, as documented.
- **Links, `..`, hop limit and mid-walk renames:** unchanged from Recheck 1. Results are identical across all three strategies, chains of 40 links read, 41 links is `SymlinkLoop`, and a directory renamed out mid-walk is `OutsideRoot`.

### Earlier advisories

- **R-A1 to R-A3:** resolved, with tests that fail when the fix is removed, or the strict-growth change.
- **R-A4:** resolved. The wording now says "an outside non-directory is never opened" and "a directory renamed out … is still walked, and the check after the use refuses it".
- **R-A5:** resolved for compilation. FreeBSD is in `make cross-check` (passes here) and in the CI `cross-check` targets. Runtime on macOS, FreeBSD and Windows stays unverified until hosted CI runs, and the docs say FreeBSD is "compiled and not run".

### Findings of Recheck 2

#### R2-1. REQUIRED: a source-index Contract sentence overclaims for other Unix

- **The sentence:** `docs/capabilities/source-index.md:206-207` says "A symlink whose target is inside the root is followed, whether the target is relative or absolute and however an absolute target spells the root."
- **Why it is false:** under the other-Unix walk, the in-root targets `../root/real.rs` (relative, climbing out by the root's own name) and `/…/root/../root/real.rs` (absolute, with a `..` at the root) are `OutsideRoot`, while Linux and Windows read them (matrix rows above). The walk also refuses an outside link to the root whose own target uses `..`.
- **Consequence:** a tree holding such a link loads on Linux and fails on macOS and the BSDs. The fail-safe direction is intended, but the Contract sentence denies it, and the platform bullet ("a link cannot leave the root and climb back into it") is the only hint.
- **Fix:**
  - Scope the sentence, for example: "…followed; on other Unix systems a target that leaves the root on the way, even to come back into it (`../root/x`, or an absolute target with a `..` taken outside the root), is `OutsideRoot`."
  - Add the plain `../root/real.rs` pair to `links_into_the_outside_are_answered_alike_only_by_the_walk`, or to a neighbour test, so the divergence is pinned.

#### R2-A1. ADVISORY (user-decision candidate): run the walk on Linux too

- **Benefits:** running the `openat` walk on Linux (already compiled there as a dev-dependency) would:
  - remove the Linux existence answer and the `/proc` dependency;
  - remove the Linux/macOS divergence in R2-1;
  - give one containment semantics on every Unix.
- **Why it needs the user:** it exceeds the Stage 2 authorisation, which scoped `rustix` to non-Linux Unix, and it reverses the `issue.md` "declined" note. The current Linux design is sound for contents and opens: 0 outside answers of any kind under race and schedule.

VERDICT (Recheck 2): BLOCKED

## Recheck 3

### Scope of Recheck 3

R2-1 only, in the uncommitted worktree:
- `docs/capabilities/source-index.md:207-214`: the Contract sentence on following symlinks.
- `:1017-1021`: the new Verification bullet.
- `:90`: the binding.
- `SECURITY.md:66-73`: the other-Unix bullet.
- The new test `strategy::a_link_that_climbs_back_into_the_root_is_read_only_by_canonicalisation` in `crates/raf-source/src/tree/disk/tests.rs`.

### Text against the code

- **"A symlink whose target is inside the root is followed, whether the target is relative or absolute. On Linux and Windows the target is followed however it spells the root."**
  - Matches canonicalisation. In Recheck 2's `round3::oracle_matrix`, Linux and Windows read every in-root spelling: `../root/real.rs`, `/…/root/../root/real.rs`, an outside link `to_root -> ../../root`, and `/…/alias/root/real.rs`.
- **"On other Unix systems … followed only when its resolution never leaves the root and climbs back through `..`: an absolute target that descends into the root through any spelling of it is followed, but `../root/real.rs` from the root, `/…/root/../root/real.rs`, and an outside link to the root whose own target climbs through `..`, are `OutsideRoot`."**
  - Matches `other_unix.rs`. `up()` refuses every `..` at the root or outside it. Re-entry happens only in `descend` on the root's `(dev, ino)`.
  - The matrix gave `OutsideRoot` for each listed example, and `Ok` for `/…/alias/root/real.rs` and for an outside absolute link to the root.
- **"A tree that holds such links loads on Linux and Windows and not on macOS or the BSDs."**
  - True. This is the divergence R2-1 asked to be stated.
- **SECURITY.md** states the same examples and the same divergence in its other-Unix bullet.
- **The Verification bullet** matches what the test asserts.

### The test asserts each strategy's answer

- **What it covers:** for every strategy in `STRATEGIES` (Linux, OtherUnix and Windows on a Linux host), it reads `dotdot.rs -> ../root/real.rs`, `absolute.rs -> /…/root/../root/real.rs` and `through_link.rs -> /…/outside/to_root/real.rs`, where `outside/to_root -> there/../../root`.
  - Under OtherUnix it asserts `OutsideRoot` for each.
  - Under the others it asserts the text `"real"`.
  - Beside them, `spelled.rs -> /…/alias/root/real.rs` must read `"real"` under every strategy.
- **It passes** in the worktree.
- **It fails against round 2:** I ported it verbatim into the round-2 copy (`/tmp/r8rev/new2`, lexical `..` outside the root). It fails there with `OtherUnix through_link.rs: Ok("real")`.

### Gates (spot check)

| Command | Result |
|---|---|
| `cargo test -p raf-source --locked` | exit 0; 316 passed, 0 failed |
| `cargo test --locked --test docs_contract --test repo_gates` | exit 0; 139 + 55 passed (the new binding resolves) |
| `cargo clippy -p raf-source --all-targets --locked -- -D warnings` | exit 0 |

### Findings of Recheck 3

- **R2-1: resolved.**
- **R3-A1 (ADVISORY, wording only):** the rule could be stated exactly, as "a `..` taken at the root or outside it is `OutsideRoot`". An absolute target such as `/…/other/../root/real.rs` (a `..` taken outside, never inside the root) is also refused. The current phrase "never leaves the root and climbs back through `..`" covers it only by reading "leaves" as including a start outside the root. The examples make the intent clear, so this does not block.
- **R2-A1** (a user-decision candidate: run the walk on Linux too) is unchanged.

VERDICT: PASS
