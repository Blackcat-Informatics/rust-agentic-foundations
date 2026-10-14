# Current status

The first review below blocked; the findings were fixed and the later recheck sections (at the end of this file) passed. Earlier sections are kept as written.

VERDICT: PASS

---

# Task 4 review: raf-source tree and containment

## Scope
Staged changes on top of 1a67fa3 (22 files). Read in full: src/{error,path,options,lib}.rs, src/tree/{mod,memory,disk}.rs,
src/tree/disk/tests.rs, tests/support/mod.rs, tests/{disk_tree_containment,memory_tree,read_cap,fs_confinement}.rs (read or
skimmed by name), and the staged diffs of AGENTS.md, SECURITY.md, source-index.md. Plan: user decision 3, section A, C1/C2,
D rows Tree/Memory, Disk handle check, Containment, the "Handle containment per platform" row, "## Task 4".

## Evidence
- `cargo test -p raf-source --locked`: all 9 binaries ok (re-run here).
- Greps over crates/raf-source/src for `.ok()`, `unwrap_or*`, `else { continue`, `_ => {}`, `.unwrap()`, `.expect(`, `unsafe`: only hit is
  `path.parent().unwrap_or_else(RelPath::root)` (disk.rs:159, see F8). No unsafe; the `allow`s (disk.rs:247-253, 269-275) carry
  reasons. libc is used for constants only. No plan labels, precedent names, "ported", task numbers or tracker numbers in
  staged crate files or the three docs. `std::fs` appears only under src/tree/disk*; the scanner has a negative fixture
  (fs_confinement.rs:133-166).
- Not run: non-unix builds (known gap), make reuse/msrv/deny (taken from the orchestrator), fuzz/long loops.

## Adjudication of the listed questions
1. Duplicate key: silent last-wins. BLOCKING (F1).
2. Infallible From/FromIterator with bad key: acceptable (F2).
3. `entry` NotADirectory -> absent, `list` of symlink -> NotRegular: correct (F3).
4. FIFO sentinel: partly demonstrated; see F4.
5. Check order: no hole found (F5).
6. Swap seam: genuine, and the Linux verifier is what catches it (F6).
7. C1 fixtures: all present (F7).
8. Docs: accurate, with two advisory notes (F9).
9. Cross-cutting: clean except F8 and F10.

## Findings

### F1 (blocking) memory.rs:80 - duplicate normalised key is silently replaced
`files.insert(collapse(&path), text.into());` discards the previous value. `[("a.rs","1"),("a.rs","2")]` and
`[("src/../a.rs","1"),("a.rs","2")]` (same key after `collapse`) build a tree holding "2" and no error. Section A bans first-wins
maps and silent skips; last-wins is the same silent data loss, and in a fixture builder it hides a typo that then reads as a
passing test. The Contract is "Only the listed errors; nothing silently absorbed". Fix: add a `Fault::Duplicate { key }` raised when
`insert` returns `Some`, surfaced by `try_from_iter` and, like the bad-key fault, from every call on the infallible
conversions. Since C1/C2 fix the `TreeError` variant set, map it to a typed existing variant (for example `Io` with
`ErrorKind::AlreadyExists`, path = the key) or get the variant added to C2 deliberately. Add a refused case (exact duplicate and
collapsed duplicate) beside the existing accepted case (distinct keys) in tests/memory_tree.rs.

### F2 (non-blocking) memory.rs:112-121, 139-148 - deferred construction fault
Construction cannot fail through `From`/`FromIterator` (the traits are infallible, and C1 requires them). Recording the fault and
returning it from `read`, `list` and `entry` (all three call `check()`: memory.rs:152, 167, 191) never serves a partial tree and
never reads as absence, so it satisfies fail-hard; `try_from_iter` is the construction-time path and is documented
(memory.rs:23-27, Contract). Test memory_tree.rs:130-146 covers all three calls and `read.path() == key`. Acceptable. Only note:
the fault is not visible to `Debug`/`PartialEq` consumers other than via the call; no change required.

### F3 (non-blocking) disk.rs:147-177, 117-123 and error.rs:80-86 - NotADirectory and symlink semantics
`entry` returning `None` when a parent component is a regular file (`lib.rs/x`) is correct: nothing is at that path. It is not
a swallowed error; `OutsideRoot`, loops and `Io` still propagate (disk.rs:153-176). `list` on a symlink -> `NotRegular` matches C1
("never descends into a directory symlink") and the trait doc, and happens after the containment check, so an outside link is
`OutsideRoot` first. One inconsistency worth knowing for Task 6: a dangling symlink is `Some(Symlink)` from `entry` but `NotFound`
from `read`/`list` (tests/disk_tree_containment.rs:223-225 pins the `read` side). The walker must test `entry` first, as C5 already
says; no change needed here.

### F4 (non-blocking) FIFO sentinel proof
- In-root FIFO as a module file: tested, under the 10 s watchdog, `read` -> `NotRegular`, `entry` -> `Other`, regular neighbour ok
  (disk_tree_containment.rs:150-169). That is the real non-blocking proof: the FIFO is opened (verify passes, root prefix holds)
  and refused by `ensure_regular` at disk.rs:378.
- Escape cases (symlink out, chain out, `vendored/..`, symlinked root): refused at the canonical prefix check (disk.rs:94) before any
  open, so the outside FIFO is never opened and the watchdog cannot fire in them. "Sentinel unread" there rests on the typed
  `OutsideRoot` result (a read would return `Ok`/block), not on an observation of the sentinel. That is sound but the watchdog is
  belt-and-braces only.
- The one case where the outside FIFO is opened and then refused is the swap test (disk/tests.rs:182-199). It is not under the
  watchdog, so a regression to a blocking open would hang the test run instead of failing at 10 s. Advisory: run that read through a
  watchdog (the helper lives in tests/support, the unit test module has its own scratch copy).

### F5 (non-blocking) disk.rs:369-394 - handle containment before regular-file check
Linux order is fstat, readlink prefix, then `ensure_regular`; no byte is read before both pass (read_capped is called after,
disk.rs:113-114). A device or FIFO outside the root is already `OutsideRoot` at the canonical prefix check, and one swapped in
afterwards is refused by the readlink first. No hole found. Residual points, advisory and not covered by the plan: (a) a hard link
inside the root to an outside inode passes by name and cannot be told from an in-root file; (b) `/proc` not mounted makes
`read_link` fail with ENOENT, which `from_io` maps to `NotFound` (error.rs:80-86), so every read would report "absent" instead of
`Io`. (b) misreports a failure as absence, which the Contract forbids; make `verify_linux_link` map a failed `read_link` to
`TreeError::Io` (not through `from_io`'s NotFound arm) and add a fixture with a nonexistent link path. Non-blocking because the
read still fails and the walker checks `entry` before `read`; fix in this task if cheap.
(c) opening an in-root device node without `O_NOCTTY` could attach a terminal; negligible.

### F6 (non-blocking, confirmed) swap seam
disk.rs:105-115 runs `before_open` after `contain` and before `open_handle`. The test (disk/tests.rs:182-199) renames `root/dir`
away and plants `root/dir -> outside/dir` in that window; `open(root/dir/file.rs)` then follows the intermediate symlink
(`O_NOFOLLOW` covers the final component only) and opens the outside FIFO. The only remaining check is the Linux readlink verifier,
which returns `OutsideRoot`; without it the result would be `NotRegular` (FIFO) so the assertion discriminates. The no-swap read
beside it is accepted (disk/tests.rs:201-210). Verifier unit tests: Linux outside/inside on symlinks and on real descriptors,
other-unix differing and equal ids plus outside path, portable non-regular and outside beside accepted (disk/tests.rs:62-153).
All three verifiers are plain fns compiled everywhere, as decision 3 requires. The swap test is `#[cfg(unix)]` but not
`target_os = "linux"`; on macOS the swap would be caught by the other-unix verifier (equal prefix/identity check on the second
canonicalisation), so the expectation still holds.

### F7 (non-blocking, confirmed) C1 fixtures
Present with behaviour names: symlinked file (+in-root alias, relative alias) :37; symlink-to-symlink out and in-root chain :59;
dir symlink with `..` (`vendored/../x.rs`, also `entry`) :77; dir-symlink loop incl. `list` and never-entered listing :106; FIFO :150;
directory as file :172; missing root :186; symlinked root :202; NotFound vs OutsideRoot :217; 0xFF content :238; non-UTF-8 name
:248; entry kinds :273; sorted listing :295. MemoryTree: `../a`, `/a`, `x/../../a` (memory_tree.rs:117), file-as-directory
:148; cap at 8 MiB+1, lowered cap, exactly 8 MiB for both backends, zero cap (read_cap.rs). std::fs scanner with negative fixtures.
RelPath and TreeError have their own test files. Gap: no fixture for F1 (duplicate key), which does not exist yet.

### F8 (non-blocking) disk.rs:159
`path.parent().unwrap_or_else(RelPath::root)` is unreachable (`file_name()` returned `Some` just above, so the path is not the
root) but it is a default over an `Option`. Replace with a `let Some(parent) = path.parent() else { return Ok(Some(EntryKind::Dir)) }`
shared with the root branch, or restructure so the root case is one branch. Style/silent-path hygiene, not behaviour.

### F9 (non-blocking) docs
Accurate and in present tense: source-index Contract "Tree" and "Path containment" match the code (three calls, error set, sorted
`list`, symlink listed but not entered, per-platform handle check, residual windows only off Linux); Usage has
`DiskTree::new(".")?`; the Examples fixture list names every Task 4 fixture; SECURITY.md Containment bullet states the three
platform guarantees consistent with disk.rs:360-367; AGENTS.md table and layout say `raf-source` is implementing and a member.
Advisory: (a) "Every path the crate opens is checked ... and no check separates from the open it guards" is stated for files; `list`
canonicalises then `read_dir(&canonical)` (disk.rs:120-126) with no handle verification, so a swap in that window can list an outside
directory's names. State the scope ("file reads") or verify the directory handle too (Task 6's walker relies on `list`). (b) The
Contract does not say that `entry` can return `OutsideRoot` for a parent that leaves the root (the trait doc does). Also the
Contract says `MemoryTree` refuses `../a` with `OutsideRoot`; after F1, add the duplicate-key sentence.

### F10 (non-blocking) design carry-over for Task 6
`RelPath::join` keeps `..` and uses a lexical depth counter, so `s/../../x` is refused as `AboveRoot` even when `s` is a symlink to a
deeper in-root directory (conservative, typed, fine). The converse (`s -> root`, `s/..`) is caught on disk as `OutsideRoot`
(test :77). Both outcomes are typed and fit the walker's "join before touching the tree" order and the Workspace::load re-run of
the containment cases. `Tree` is object-safe, takes `RelPath` only, and has `entry` for existence checks, so nothing blocks Tasks
5-7. `tests/disk_tree_containment.rs` is `#![cfg(unix)]` as a whole because its fixtures need symlinks and FIFOs; non-unix coverage
rests on the portable verifier unit test and disk_tree_root.rs, and the cross-check gate arrives with Task 5 (as stated).

## Summary
One required fix (F1: silent last-wins on duplicate MemoryTree keys). Containment, ordering, the swap seam, fixtures and docs
check out. Recommended with F1: F5(b) (`/proc` failure must not read as `NotFound`), F4 watchdog on the swap test, F8, F9(a).

VERDICT: BLOCKED

## Recheck

Read the new text in memory.rs, disk.rs, disk/tests.rs, memory_tree.rs, SECURITY.md; re-ran `cargo test -p raf-source --locked`
(all binaries ok).

- F1 DISCHARGED. memory.rs:93-97 raises `Fault::Duplicate` when `insert` returns `Some`, mapped to the typed
  `TreeError::DuplicatePath` (error.rs:63, in `path()` and Display). `try_from_iter` returns it; the infallible conversions record
  it and every call returns it. Test memory_tree.rs:~160-187 covers literal, `src/../a.rs`+`a.rs` and `./a.rs`+`a.rs`, asserts
  `read`/`list`/`entry` all fail, and has the distinct-keys neighbour. tree_error.rs includes the new variant.
- F5(b) DISCHARGED. `verify_linux_link` maps a failed `read_link` through `link_read_error` to `TreeError::Io` (disk.rs, no
  `from_io` NotFound arm). Tested directly on a nonexistent link path and on a NotFound io error
  (disk/tests.rs `a_missing_descriptor_link_is_an_io_failure_and_never_absence`).
- disk.rs `unwrap_or_else` DISCHARGED. `entry` now uses `let (Some(name), Some(parent)) = (...) else { return Ok(Some(Dir)) }`;
  no default remains (grep of src finds no `unwrap_or*`).
- F9(a) DISCHARGED. `list` now opens the canonical directory with `O_DIRECTORY|O_NOFOLLOW|O_NONBLOCK`, verifies the handle
  (Linux readlink prefix; other unix identity; non-unix `verify_portable_directory`), requires a directory, and reads entries
  through `/proc/self/fd/<fd>` while holding the handle, so the names are those of the verified directory. The seam fires in `list`
  (hook before `open_verified_directory`); the swap test plants an outside symlink for an intermediate directory and gets
  `OutsideRoot` with no entries returned, beside a no-swap listing that returns `inside.rs`. SECURITY.md now says "for a file read and
  for a directory listing alike". Only cosmetic: the SECURITY.md sentence has a line over the wrapped width (one long line).
- Watchdog advisory DISCHARGED. Both swap tests (read, list) run through `within_watchdog` (disk/tests.rs:206, 242).

No new issues found in the new text. The earlier non-blocking notes (hard links, dangling symlink read vs entry, whole-file
`cfg(unix)` on the containment suite) are unchanged and remain advisory.

VERDICT: PASS
