# Security review: PR 13 (issue 2), raf-okf and raf-source

VERDICT: NEEDS-FIX

- Branch `paudley/2-shared-crates-raf-okf-raf-source` at 6c744af. Scope is `git diff origin/main...HEAD`.
- Platform: Linux 7.2.9 (x86_64), nightly 1.100.0 (2026-09-13). Probes ran in release unless marked debug.
- The security contract is the Contract and Constraints sections of `docs/capabilities/source-index.md` and `docs/capabilities/okf-profile.md`, plus the SECURITY.md Containment bullet.

## Reviewed scope

- `crates/raf-source/src/tree/{mod,disk,memory}.rs`: canonical containment, symlinks, FIFOs and other non-regular files, the per-platform handle verifier, listing, `entry`.
- `crates/raf-source/src/path.rs`: `RelPath` validation, including Windows-style input.
- `crates/raf-source/src/modtree.rs`: `#[path]`, `cfg_attr` paths, `tidy`, the visited-file key, the cycle and depth checks.
- `crates/raf-source/src/{discovery,manifest,members,glob,workspace,target}.rs`: toml parsing, member and exclude globs, path dependencies, the ancestor-manifest walk, target `path` keys.
- `crates/raf-source/src/{parse,prescan,budget}.rs`: the dedicated 64 MiB parse thread, the pre-scan counters, abort behaviour, the `Spawn` path.
- `crates/raf-source/src/settle.rs` (glob fixed point), from the resource angle only.
- `crates/raf-okf/src/{frontmatter,body}/*`: YAML subset bounds (anchors, aliases, merge keys, tags, depth, size) and body model allocation.
- `deny.toml`, `fuzz/deny.toml`, `fuzz/Cargo.toml`, `fuzz/fuzz_targets/parse_source.rs`, the Makefile `fuzz` and `cross-check` targets, and the CI jobs.

**Method.** I used a scratch crate at `/tmp/raf-sec`, with path dependencies on both crates and the worktree's `rust-toolchain.toml`. Each binary there takes the shape and size on the command line. Memory figures are `getrusage` max RSS. Runs that could exhaust the host were capped with `ulimit -v` or a `systemd-run --user --scope -p MemoryMax=...` cgroup. No tracked file was edited.

`performance-review.md` (same stage) measured several of the same costs from the performance side. Where a finding overlaps, I cross-reference it and state the security consequence against the contract.

---

## Findings

| ID | Severity | Area | Summary |
|---|---|---|---|
| S-1 | High | raf-okf body | Quadratic allocation from the section slug. A 200 KB document aborts the process. Breaks "never a panic, abort or unbounded allocation" |
| S-2 | High | raf-source mod-tree | In-root directory symlinks give one file unbounded spellings. It is read, parsed and kept up to 262144 times. A 1.2 MB tree is OOM-killed at a 3 GB cap |
| S-3 | Medium | raf-source resolver | The glob fixed point uses about 6.6 KiB per (module, name) pair. A 140 KB file aborts on allocation failure |
| S-4 | Medium | raf-source discovery | Discovery recursion on the caller's stack has no bound. Deep directories abort in debug on a 2 MiB thread, and `MemoryTree` aborts in release |
| S-5 | Medium | raf-source pre-scan | `async`/`const` between closure heads gets past the closure-bar budget. Several more documented-as-absent shapes abort (`break`, `yield`, `become`, `\|\| return`, `impl Fn() ->`, `extern "C" fn() ->`, `unsafe fn() ->`, `for<'a> fn() ->`) |
| S-6 | Low | DiskTree read (Linux) | When a directory is swapped during the read, the outside file is opened before it is refused. Contradicts "neither opened nor read" and "no check is separated from the open it guards" |
| S-7 | Low | DiskTree entry | `entry` reports existence and kind of an outside path when a directory is swapped during the call (all platforms) |
| S-8 | Low | DiskTree list (non-Linux) | Per-platform statement: non-Linux listing re-opens by path after verification. The Contract states listing safety without condition |
| S-9 | Low | members/exclude | `members` × `exclude` matching is quadratic with no budget. An 880 KB manifest takes 118 s, and the 8 MiB cap allows hours |
| S-10 | Low | errors | `LoadError` Display embeds attacker-chosen control characters (terminal escape injection) |
| S-11 | Low | parser | `box` patterns parse in quadratic time: 80 KB takes 11 s, and the 8 MiB cap allows about 30 h |
| S-12 | Info | fuzz | The fuzz target cannot find stack overflows (`-max_len=4096`). It covers only `parse_source`, not raf-okf, manifests or the walk, and sets no RSS limit |
| S-13 | Info | deny | Licence exceptions are crate-scoped and correct, but they are not version-pinned. `fuzz/deny.toml` is a full copy that can drift |
| S-14 | Info | RelPath | Windows-ambiguous segments (`.. `, `...`, `con`, `nul.rs`, `1:`) are accepted. Only the canonical check guards them on Windows, which was not run |
| S-15 | Info | diagnostics | A non-literal `#[path]` is reported as a cfg parse error "reached by manifest key `lib.path`" |

---

### S-1 (High): quadratic allocation from the section slug in the body model

**Where.**
- `crates/raf-okf/src/body/mod.rs:131-137` clones the H2 slug into every line of its section: `*slot = Some(section.slug().to_string())`.
- `crates/raf-okf/src/body/mod.rs:173` and `crates/raf-okf/src/body/marker.rs:191` then clone that slug again for every prose line and every PROPOSED block.
- `crates/raf-okf/src/body/heading.rs:85-111` and `:192-217` put no bound on heading or slug length.
- Memory = lines in the section × slug bytes. Both factors are bounded only by the 1 MiB text.

**Reproduction.** `/tmp/raf-sec/src/bin/okf_slug.rs` builds `"## " + "a"*H + "\n" + "\n"*L` and calls `raf_okf::Body::parse`. `okf_shapes slug N` does the same through `Document::parse`, with frontmatter and `x\n` lines.

```
$ (ulimit -v 6000000; target/release/okf_slug 10000 10000)
input bytes = 20004
ok = true in 29.1ms maxrss = 100540 KiB
$ (ulimit -v 6000000; target/release/okf_slug 30000 30000)
input bytes = 60004
ok = true in 244.7ms maxrss = 884428 KiB
$ (ulimit -v 6000000; target/release/okf_slug 100000 100000)
input bytes = 200004
memory allocation of 100000 bytes failed
Aborted (core dumped)           exit=134
$ (ulimit -v 4000000; target/release/okf_shapes slug 200000)    # Document::parse
Aborted (core dumped)           exit=134
```

A 1 MiB document would need about 2^18 lines × 2^19 bytes, which is about 128 GiB.

**Consequence against the contract.**
- `okf-profile.md:96-98` says `Document::parse` "fails with a positioned error and never a panic, abort or unbounded allocation", for "sources the reader may not control". This input is accepted (`ok = true`), so no bound is checked at all. Allocation is quadratic and the process aborts.
- `docs_contract` and any workplan or census consumer parse every tracked Markdown file. A pull request adding one such file kills the gate or the tool.
- The alloc-bound test (`crates/raf-okf/tests/alloc_bound.rs`) covers only refused documents, so it cannot see this.
- Same root cause as performance-review F1.

**Required fix.**
1. Keep the per-line section as an index into `sections` (`Option<usize>`) or as a shared `Rc<str>`/`Arc<str>`, so that memory is O(lines + sections × slug). The public accessors `ProseLine::section` and `ProposedBlock::section` can still return `Option<&str>`.
2. Bound heading text, for example at 4096 bytes like link parts, with a positioned error. This keeps the slug-per-block cost linear.
3. Add an accepted-document case to `alloc_bound.rs`, a long heading over many prose lines and many PROPOSED blocks, held to the same 8× + 16 KiB bound.

---

### S-2 (High): one file reached under unbounded spellings through in-root directory symlinks

**Where.**
- `crates/raf-source/src/modtree.rs:183` keys visited files by `RelPath` spelling.
- `tidy` (`modtree.rs:536-549`) collapses `dir/..` only when `dir` is a real directory, and never resolves a symlinked directory component.
- A file reached as `src/l/x.rs`, `src/m/x.rs`, `src/l/m/x.rs` and so on, where `l -> .` and `m -> .` are in-root symlinks, is a new key each time. Each spelling gets a fresh `Tree::read`, a fresh parse, and its own retained `Parsed { text, facts }`.
- Containment is not breached, because every spelling canonicalises inside the root. The protection that fails is the resource bound.

**Reproduction.** The tree is `p/src/{l -> ., m -> .}` plus `lib.rs`: `#[path = "x0.rs"] mod a;`. Each `x{k}.rs` holds `#[path = "l/x{k+1}.rs"] mod a; #[path = "m/x{k+1}.rs"] mod b;`, then `pub fn f() {}` and a comment of P bytes. The depth is 18, so the arrivals reach exactly 2^18 and the ModulePaths budget is not passed. The load runs through `DiskTree` and `Workspace::load(&tree, &["p"])` (`/tmp/raf-sec/src/bin/src_load.rs`).

```
pad=0     levels=18 tree_bytes=1341     -> ok files=262144 in 24.0s  maxrss=2086164KiB
pad=4096  levels=16 tree_bytes=66729    -> ok files=65536  in 6.9s   maxrss=1033872KiB
pad=65536 levels=18 tree_bytes=1180989:
  (ulimit -v 8000000)        -> err: p/src/l/l/m/m/m/l/l/m/m/l/m/l/m/m/m/l/m/x17.rs: cannot start the parse thread: Resource temporarily unavailable (os error 11)
  systemd-run MemoryMax=3G   -> exit=137 (OOM-killed)
```

Memory grows as 2^18 × `max_file_bytes`. At the default 8 MiB cap that is up to 2 TiB.

**Consequence against the contract.**
- `source-index.md:288-290` says a file reached several ways "is parsed once, listed once ... so one file has one spelling".
- `source-index.md:192-198` says the module-path limit's cost is "a fraction of a second and a few tens of megabytes" and "cannot exhaust a machine".
- A hostile checkout needs nothing but symlinks to an in-root directory, which git stores and restores. Two such symlinks and 18 small files cost 24 s and 2 GB. Padding the files ends in an OOM kill.
- Related to performance-review F4 (items × arrivals). This finding is the stronger form: the file text and the parse are duplicated too, not only the item records.

**Required fix.**
1. Key visited files by identity, not spelling. On `DiskTree`, use the canonical path or the `(dev, ino)` of the verified handle, for example through a `Tree::identity(path)` call (or `read` returning it). `MemoryTree` would use its collapsed key. Record each spelling as a `Reach` of the one parsed file.
2. Also add a per-load budget on total bytes read, or total files parsed, with a typed `Budget` error. Cost is then bounded even when identity is unavailable.
3. Add a `DiskTree` fixture with two `-> .` symlinks that asserts one parse.

---

### S-3 (Medium): glob fixed point allocates about 6.6 KiB per (module, visible name) and aborts

**Where.** In `crates/raf-source/src/settle.rs`:
- `visible` (`:331-355`) clones a `Bound` for every name.
- `glob_pass` (`:371-404`) builds a whole new scope table per pass, with an `Origin` holding a `Vis` clone and `String`s per binding.
- `rebuild_globs` (`:510-533`) repeats until nothing changes.
- No budget applies.

**Reproduction.** `/tmp/raf-sec/src/bin/resolve_cost.rs globs N` creates one `lib.rs` of N copies of `pub use m{i}::*; pub mod m{i} { pub use crate::*; pub fn g{i}() {} }` and loads it through a `MemoryTree`.

```
globs n=100  input=6570B  load=0.11s maxrss=69820KiB
globs n=200  input=13470B load=0.51s maxrss=268852KiB
globs n=400  input=27270B load=1.74s maxrss=1058684KiB
globs n=1000 input=68670B load=10.99s maxrss=6586816KiB
globs n=2000 (ulimit -v 16000000): memory allocation of 1864 bytes failed / core dumped (exit 134)
```

**Consequence.**
- A 140 KB source file aborts the consuming process. Rust semantics make the K² visible names real, but the constant (KiB per pair, plus a whole table rebuilt per pass) turns a small input into tens of GB.
- The Contract budgets module walks so that a load "cannot exhaust a machine" (`source-index.md:196-198`). Use settlement has no such bound, and `source-index.md:224` disclaims only time.
- Overlaps performance-review F5 and F7.

**Required fix.**
1. Add a per-load budget on glob-imported bindings, for example 2^22, with a typed `Budget` error naming the file and line of the glob that crosses it.
2. Cut the per-binding cost: intern names, share `Vis` behind `Rc`, and propagate only new names from changed modules instead of rebuilding every table per pass.
3. State the bound in the Resource bounds paragraph.

---

### S-4 (Medium): discovery (and member expansion) recurse on the caller's stack with no depth bound

**Where.**
- `crates/raf-source/src/discovery.rs:133-156`: `visit` calls `self.visit(&child)` at line 150 for every directory level.
- `crates/raf-source/src/members.rs:83-142`: `expand` recurses once per pattern segment, bounded only by the 262144 meter.
- Neither runs on the 64 MiB parse thread.

**Reproduction.** A chain of 2100 nested directories `a/a/a/...` under the tree root, with no manifest, loaded with roots `["."]`. `src_load_thread STACK DIR ROOTS` runs the load on a thread with the given stack. `mem_deep N` uses one `MemoryTree` key `a/`×N + `x.txt`.

```
$ target/debug/src_load_thread 2097152 /tmp/raf-sec/deep .
thread 'caller' (3886651) has overflowed its stack
fatal runtime error: stack overflow, aborting                 exit=134
$ target/release/src_load_thread 8388608 /tmp/raf-sec/deep .
err in 90.5s: /<a/ x N>Cargo.toml: File name too long (os error 36) (reached by package discovery)
$ target/release/mem_deep 5000
depth 5000: ok files=0 in 497.8ms
$ target/release/mem_deep 20000          # 40 KB key, 8 MiB main-thread stack
fatal runtime error: stack overflow, aborting                 exit=134
```

**Consequence.**
- On `DiskTree`, depth is capped only by PATH_MAX, at about 2048 levels. That is enough to abort a debug build on the default 2 MiB test thread, and a debug test thread is how `docs_contract` and every `cargo test` consumer load the repository.
- On `MemoryTree`, depth has no cap, and release builds abort too. The crate documents `MemoryTree` as the in-memory backend, and a consumer that builds trees from git objects would use it.
- `source-index.md:187-189` bounds module nesting at 128 "because the walk recurses on the caller's stack". Discovery applies the same reasoning nowhere.
- The release run also shows quadratic syscall cost per level (90 s for one chain), which performance-review F8 covers.

**Required fix.**
1. Bound directory depth in discovery and in `expand`, for example 128 directory levels below a root, with a typed `Budget` error. Alternatively, make both walks iterative with an explicit stack.
2. Add a test that runs the deep-directory case on a 2 MiB debug thread and expects the typed error.

---

### S-5 (Medium): closure-bar budget evasion and further overflow shapes the Contract does not list

**Where.**
- `crates/raf-source/src/prescan.rs:24-26` and `:204` let only `move` sit between chained closure heads.
- `async`, `async move` and `const` before `||` are accepted by `syn` and reset the chain.
- Several other right-recursive shapes are counted by no budget.

**Reproduction.** `/tmp/raf-sec/src/bin/shapes.rs SHAPE N` calls `raf_source::parse_source` on the generated text, on its own 64 MiB thread.

```
async_closure n=129    PARSED            # 129 chained `async ||` heads; plain `||` x129 is refused at 128
async_closure n=200    PARSED
release, N=20000, every line below: thread 'x.rs' (...) has overflowed its stack, exit=134
  async_closure        fn f() { let _ = async || async || ... 0; }
  async_move_closure   async move || ...
  const_closure        const || ...
  break_value          fn f() { loop { break break ... 0; } }
  yield / become       yield yield ... 0;  become become ... 0;
  return_closure       || return || return ... 0
  impl_fn_ret          fn f() -> impl Fn() -> impl Fn() -> ... u8
  extern_fn_ptr        type T = extern "C" fn() -> ... u8;
  unsafe_fn_ptr        type T = unsafe fn() -> ... u8;
  for_fn_ptr           type T = for<'a> fn() -> ... u8;
debug, N=20000, additionally: a += a += ..., *mut *mut ..., &raw mut *... (all exit=134)
```

**Consequence.**
- `source-index.md:218-220` says shapes that try to slip past a counter "are counted by the definitions above and refused". The `async ||` chain is such a shape for the closure-bar counter, and it is admitted far above its limit.
- `source-index.md:214-217` enumerates the uncounted shapes as `return`, `&mut`, `*const`, `fn() ->` and `a =`. The list is incomplete.
- Any consumer loading untrusted source aborts instead of getting a typed error. The abort fails closed, which the Contract allows for residual overflows. The evasion of a defined counter is not allowed.
- Performance-review F11 separately measured postfix and binary chains at about 100k terms. This finding is the counter evasion plus the keyword shapes.

**Required fix.**
1. Count `async`, `async move`, `const` and `static` between closure heads, as `move` is counted. Add the evasion to `shapes::evasions()` and the corpus.
2. Either add the additive "open recursion" budget proposed in performance-review F11, or widen the Contract's list of uncounted shapes to the ones above and add them to `shapes::uncounted()` and `fuzz/corpus/parse_source/`. The Contract says an input that aborts "is added to the fuzz corpus and to the budgets".

---

### S-6 (Low): on Linux, a directory swap makes `DiskTree::read` open an outside file before refusing it

**Where.**
- `crates/raf-source/src/tree/disk.rs:109-118`: `contain` canonicalises and checks the prefix, then `open_handle(&canonical)` (line 116) opens by path with `O_NONBLOCK|O_NOFOLLOW`.
- Only after that does `verify_open_handle` (line 117) read `/proc/self/fd/N`.
- `O_NOFOLLOW` covers only the final component, so an intermediate directory swapped for a symlink to outside is followed by the open.

**Reproduction.** `/tmp/raf-sec/src/bin/race_open.rs`:
- Root `root/d/f.rs` holds "inside". Outside, `out/f.rs` is a FIFO; the FIFO run uses `out/f.rs` = "OUTSIDE-SECRET" as its file variant.
- One thread swaps `root/d` with `root/swap -> out`, using `renameat2(RENAME_EXCHANGE)`.
- A writer thread blocks in `open(out/f.rs, O_WRONLY)`. That open returns only when some reader opens the FIFO.
- The main loop runs `read("d/f.rs")`, `entry("d/only_outside")` and `list("d")` 200000 times.

```
$ target/release/race_open /tmp/raf-sec/race 200000 fifo
mode=fifo iters=200000 read_ok_inside=45300 read_ok_OUTSIDE_CONTENT=0 outside_root=154700 other={}
outside FIFO opened by a reader (writer open returned): 80007
entry() reported an outside-only name: 63691; list() listed an outside-only name: 0
$ target/release/race_open /tmp/raf-sec/race2 200000 file
mode=file iters=200000 read_ok_inside=42508 read_ok_OUTSIDE_CONTENT=0 outside_root=157492 other={}
entry() reported an outside-only name: 62902; list() listed an outside-only name: 0
```

**Consequence.**
- Content never leaks: 0 of 400000 reads returned outside content, and the Linux verifier refuses every one.
- The outside file is opened, though. `source-index.md:123` says a candidate outside the root "is neither opened nor read". `SECURITY.md:59` and `source-index.md:140` say "no check is separated from the open it guards", but the prefix check is separated from the open, and the handle check runs after it.
- Opening has side effects:
  - it releases a writer blocked on an outside FIFO;
  - it runs a device's open handler (for example a watchdog or tape device the user can open);
  - it can trigger an automount.
- Exploiting this needs a local attacker who can rename inside the tree during a load.

**Required fix.**
- Resolve beneath a held root descriptor so the kernel refuses the escape before any open: `openat2(rootfd, rel, RESOLVE_BENEATH | RESOLVE_NO_MAGICLINKS)`, with `O_NOFOLLOW`/`RESOLVE_NO_SYMLINKS` for the final component where needed.
- Or open with `O_PATH|O_NOFOLLOW` first, which runs no device open and no FIFO rendezvous. Then verify `/proc/self/fd/N` and `fstat` the type, and only then reopen `/proc/self/fd/N` with `O_RDONLY`.
- Otherwise reword the Contract and SECURITY.md to "never read", stating that a racing swap can cause an open of the outside file.
- Add the FIFO-writer race as a Linux test.

---

### S-7 (Low): `entry` is an existence and kind oracle for outside paths under a race

**Where.** `crates/raf-source/src/tree/disk.rs:158-187`. `contain(parent)` canonicalises the parent directory, then `fs::symlink_metadata(directory.join(name))` (line 175) runs a path-based `lstat` that follows intermediate symlinks. Nothing ties the second call to the first check.

**Reproduction.** Same run as S-6: `entry("d/only_outside")` returned `Some(kind)` for a name that exists only outside the root, 63691 times out of 200000 (FIFO mode) and 62902 (file mode).

**Consequence.**
- `source-index.md:112-113` says "`entry` returns `OutsideRoot` when the directory that holds the path leads outside the root".
- Under a race it reports the existence and kind of an attacker-chosen outside path. The mod-tree walk then picks between `name.rs` and `name/mod.rs` from that answer.
- No content leaks. The gap holds on every platform, Linux included.

**Required fix.** On Linux, open the parent with `O_DIRECTORY|O_NOFOLLOW`, verify it like `list` does, and `fstatat(dirfd, name, AT_SYMLINK_NOFOLLOW)`. Elsewhere, re-verify the parent after the `lstat`. Or state the race in the per-platform bullets.

---

### S-8 (Low): the listing guarantee is stated without condition but holds only on Linux

**Where.**
- `crates/raf-source/src/tree/disk.rs:257-291`. On other Unix, the handle is verified by `(dev, ino)` against a second canonicalisation (lines 272-280), but `listing = canonical.to_path_buf()` (line 281). `fs::read_dir(&listing)` (line 137) then opens the directory again by path.
- On non-Unix no directory handle is opened at all: lines 286-289 use path `fs::metadata` and return `None`.

**Consequence.**
- `source-index.md:133-136` says the listing "reads the entries through the handle, so a swap between the check and the listing cannot list a directory outside the root". `SECURITY.md:57` says the handle "for a file read and for a directory listing alike, is verified".
- On macOS and the BSDs, a swap between verification and `read_dir` lists outside entries. On Windows there is no handle check at all.
- Linux held: 0 outside listings in 200000 (S-6 run).
- Stage 2 recorded this as SC-13 PARTIAL. I confirm it from the code; it cannot run here.

**Required fix.**
- On other Unix, list through the verified handle. Rust std has no `fdopendir`, so either keep the `(dev, ino)` check and re-check it after `read_dir` by comparing the `.` entry or a second `fstat` of a re-opened path, or use `libc::fdopendir`.
- On Windows, open the directory with `FILE_FLAG_BACKUP_SEMANTICS` and verify with `GetFinalPathNameByHandle`.
- Or qualify the Contract sentence and the SECURITY.md bullet per platform, as reads already are.

---

### S-9 (Low): quadratic member × exclude matching with no budget

**Where.** In `crates/raf-source/src/members.rs`:
- Lines 231-242 test every literal member against every exclude pattern before any meter spend.
- `is_excluded` (lines 219-224, 245, 287) runs every exclude pattern per found directory, collecting a `Vec<char>` per match.
- Duplicate patterns are not removed.

**Reproduction.** A workspace manifest with M copies of `"p"` in `members` and E copies of `"q"` in `exclude`, plus one package `p` with `edition.workspace = true`, so `members()` runs.

```
members=1000  excludes=10000  manifest_bytes=44073  -> ok in 0.35s
members=10000 excludes=100000 manifest_bytes=440073 -> ok in 21.5s
members=20000 excludes=200000 manifest_bytes=880073 -> ok in 117.7s
```

Inside the 8 MiB cap, the meter allows 131072 members against about 1.9M excludes. That is about 2.5×10^11 matches, roughly 1.5 h or more.

**Consequence.** One manifest stalls every load. `source-index.md:224` disclaims a time bound, but the `Meter` doc (`members.rs:30-31`) presents itself as bounding "what a walk of the workspace directory may cost", and pattern work is not metered.

**Required fix.** Deduplicate patterns, put literal excludes in a `BTreeSet`, cap `members` and `exclude` lengths (for example 4096 each, as a typed `Manifest` error naming the key), and charge pattern matches to the meter.

---

### S-10 (Low): error messages carry raw control characters from the input

**Where.**
- `crates/raf-source/src/error.rs:126-134` turns a `PathError` into `OutsideRoot { path: input }`.
- The `Display` impls (`error.rs:137`, `:200`, `:356`, `:381`) write paths, attribute values and names with `{}`, unescaped.

**Reproduction.** `/tmp/raf-sec/src/bin/esc_term.rs`:

```
lib.rs: #[path = "../../../\u{1b}]0;pwned\u{7}\u{1b}[2J\u{1b}[31mfake.rs"] mod m;
"p/src/lib.rs:1: ../../../\u{1b}]0;pwned\u{7}\u{1b}[2J\u{1b}[31mfake.rs leads outside the root (reached by the `path` attribute of module `m`)"
raw ESC bytes in Display: 3
```

The NUL case in the escape battery printed a raw NUL as well.

**Consequence.** Any CLI or CI log that prints a `LoadError` runs the input's terminal escapes (CWE-150): a window-title set, a screen clear, or faked "success" lines. raf-okf refuses control characters in frontmatter, so it is not affected.

**Required fix.** Escape untrusted strings in `Display`, with `escape_debug` or a helper used for every path, attribute value and name field. Add a test with an ESC byte.

---

### S-11 (Low): quadratic parse time for `box` patterns

**Where.** This is in `syn` (verbatim `box` pattern handling), reached through `parse_source`. No budget counts it.

**Reproduction.** `shapes pat_box N` builds `fn f() { let box box ... x = 0; }`:

```
pat_box n=2500  PARSED 0.127s
pat_box n=5000  PARSED 0.503s
pat_box n=10000 PARSED 1.983s
pat_box n=20000 PARSED 10.87s       (debug: 43.1s)
```

**Consequence.** An 80 KB file costs 11 s. The 8 MiB cap allows about 2M repetitions, which is on the order of a day of CPU per file. The crate disclaims time bounds (`source-index.md:224`), so this is Low. Generally, though, a single source file can hold a load for a day.

**Fix.** Count `box` runs in the prefix-run counter, since `box` is a prefix pattern operator. Or add the per-level token budget proposed in performance-review F11.

---

### S-12 (Info): fuzzing cannot find the bugs it exists to catch

**Where.** `Makefile` `fuzz` target (`-runs=5000 -max_len=4096`), `fuzz/fuzz_targets/parse_source.rs`.

**Issues.**
- Every overflow shape needs well over 4096 bytes: `async ||` overflows between 8000 and 20000 heads, which is 72-180 KB. A 5000-run, 4096-byte campaign can never find a new abort.
- Only `parse_source` is fuzzed. There is no target for `raf_okf::Document::parse`, for manifests, or for `Workspace::load` over a `MemoryTree`.
- No `-rss_limit_mb` or `-malloc_limit_mb` is set. With a raf-okf target and a memory limit, S-1 would have been found in seconds.

**Fix.**
- Add `fuzz_targets/okf_document.rs` and a `MemoryTree` workspace target, built from structured input.
- Run all targets with `-rss_limit_mb=2048 -malloc_limit_mb=512`.
- Keep the short gate run, and document that long-input overflow discovery happens in a separate longer job, not the gate.

### S-13 (Info): licence exceptions

- `deny.toml:24-26` (`unicode-ident`: Unicode-3.0) and `fuzz/deny.toml` (`unicode-ident`, plus `libfuzzer-sys`: NCSA) are crate-scoped and match the crates' declared licences. NCSA is limited to the unshipped fuzz graph. Both are correct.
- They are not version-pinned, for example `crate = "unicode-ident@1"`, so a later major version with different terms would inherit the exception.
- `fuzz/deny.toml` is a full copy of the root policy, so later changes to the root (bans, sources, advisories) will not reach the fuzz graph.
- The owner still has to approve the exceptions (handoff).

### S-14 (Info): RelPath accepts Windows-ambiguous segments

`RelPath::new` accepts all of the following:
- `".. "`, `"a/.. /.. /x"`, `"..."`;
- `"con"` and `"nul.rs"` (DOS device names);
- `"1:/x"`;
- `"a/\u{202e}b"` (bidi override);
- `"．．/x"` (fullwidth dots).

It refuses backslashes, `C:`, `a:b`, `//server` and NUL. On Windows, the canonical prefix check in `DiskTree` (`disk.rs:96-105`, and `verify_portable`) is the only guard against Win32 normalisation of trailing dots and spaces and against device names. That check should hold, since a resolved device or a climbed path is not under the root, but it was never run on Windows (SC-16 UNVERIFIED). For portability and defence in depth, consider refusing segments that end in `.` or space, reserved device names, and bidi controls.

### S-15 (Info): misleading diagnostic for a non-literal `#[path]`

`#[path = concat!("../../../", "outside/plain.rs")] mod m;` yields `cannot read a cfg predicate: `path` is not a string literal (reached by manifest key `lib.path`)`. The kind (cfg parse) and the origin (manifest key) are both wrong; the error comes from the attribute in `lib.rs`. Nothing is opened, so this is diagnostic only.

---

## Attacks tried that held

**Path containment.** I ran `/tmp/raf-sec/src/bin/escapes.rs` on DiskTree, with an outside FIFO whose blocked writer detects any open. Every case gave a typed error, the outside FIFO was opened 0 times, and no outside content was indexed. Cases:
- `#[path]` values:
  - absolute path, `../../../outside`;
  - backslash path, `C:/x.rs`, `//server/share`, NUL;
  - raw string, `cfg_attr`, nested `cfg_attr`;
  - inline module `#[path]` to an outside directory, nested inline `..` chain.
- Symlinks:
  - symlinked module file to outside, symlinked FIFO;
  - directory symlink then `..` (`ld/../plain.rs`), directory symlink to outside;
  - default `mod plain;` resolving through a file symlink and through a directory symlink;
  - links to `/proc/self` and `/dev/zero`.
- Non-regular files: FIFO inside the root (`NotRegular`, no hang), directory named as a module.
- Macros: `concat!` path (refused); `#[doc = include_str!("/outside/secret")]` (not read).
- Manifests: `[lib] path` absolute or `../..`, `package.build` to an outside FIFO, symlinked `Cargo.toml` to outside.

**Race.** I ran the directory-swap race with `renameat2(RENAME_EXCHANGE)` 400000 times. `read` never returned outside content, and on Linux `list` never listed an outside entry (0/200000).

**Bounds and typed errors.**
- `Spawn` error (SC-19b): reached for real under `RLIMIT_AS` during the S-2 run. The output was the typed `cannot start the parse thread: Resource temporarily unavailable (os error 11)`, with no panic.
- `toml` nesting: `[`×200000, `{a=`×200000 and mixed nesting in `[package.metadata]` each gave a typed `Manifest` error on a 2 MiB debug thread.
- Pre-scan lexer on large lexical inputs: 4M nested `/*`, 2M lifetimes, 1.5M char literals, 3M escaped quotes in a byte string, an 8 MB identifier, 700000 items, 800000 inline modules and raw strings with many hashes. Each gave a typed error or parsed in 1.2 s or less.
- Counted shapes: prefix runs (`-`, `&` patterns), qualified-self `<`, and match nesting were refused with `Budget` as documented.
- Re-export chain of 50000 named imports in reverse order: resolved in 0.24 s with 157 MB.
- Member glob pattern `*a*a*...b`: not exponential (also measured by the performance review).

**raf-okf frontmatter** (`/tmp/raf-sec/src/bin/okf_yaml.rs`, inputs about 1 MiB). All cases parsed within 57 MB RSS and 31 ms, or were refused with a positioned error. Cases:
- volume: 100k keys, 170k list items, a 1 MiB flow list, 1 MiB of comments, 1 MiB of `''` escapes, block maps inside lists;
- nesting: flow nesting of 1M, block nesting past 32;
- refused constructs: `- - -` lists, `? complex` keys, `!!str` tags, `<<` merge keys, anchors and aliases (billion laughs is covered by the existing alloc test);
- refused text: duplicate keys at the end, BOM, mixed CRLF, NUL, tab indentation, non-ASCII keys, `\u` escapes, an empty flow entry.
- Round trip was byte-identical for every accepted case.

**raf-okf body** (`okf_shapes.rs`, 22 shapes of 1 MiB). The shapes covered unclosed brackets, link openers, backtick runs, comment openers, split links, 60-deep quotes, list markers, 260k fences, 208k headings, PROPOSED pairs, reference definitions and uses, and nested brackets and parens. All were linear in time, under 125 ms, with at most 126 MB RSS. That is about 120× the input for headings and quotes, which is bounded but well above the 8× the alloc test asserts for refused documents. Link parts over 4096 bytes were refused with a position.

**Licences and CI.**
- CI pins cargo-deny 0.20.2 and cargo-fuzz 0.13.2 with `--locked`, and actions are pinned by SHA.
- `make deny` now uses `--workspace`.
- `libc` is a unix-only dependency and is used for constants only.

## Required before merge

1. S-1: bound the body-model allocation, and test it on an accepted document.
2. S-2: dedupe visited files by identity or canonical path, and add a per-load read budget. The Contract's "parsed once / one spelling / cannot exhaust a machine" is false today.
3. S-3, S-4, S-5: add the missing budgets (glob bindings, directory depth, closure-bar evasion), or narrow the Contract text to what is enforced. Each is a reachable abort from small hostile input that the current Contract implies is bounded or refused.
4. S-6, S-7, S-8: harden the code, or bring the Contract and SECURITY.md sentences ("neither opened", "no check is separated from the open", `entry` `OutsideRoot`, listing through the handle) into line with the per-platform behaviour.
