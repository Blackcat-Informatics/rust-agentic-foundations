<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Staged gate cost

The repository's tracked-document test and its production Workplan test share one source index. The tracked Markdown, diagram assets, symlink rules and inventory have their own check. The production check evaluates the disk documents and requires every assessment to be current and accepted.

## Measured work

| Complete staged hook | Independent source indexes | Shared source index |
| --- | ---: | ---: |
| Workspace index builds | 67 | 66 |
| Physical Rust files analyzed by the module walker, counting repeated loads | 8,082 | 7,400 |
| Native whole-file parser entries in Source | 11,448 | 10,766 |
| Additional parses in the repository runtime-boundary rule | 389 | 389 |
| User instructions, including descendants | 5,448,761,196,322 | 5,384,900,616,686 |
| Successful glibc allocation calls at the observed interfaces | 511,344,172 | 494,584,679 |
| Peak cgroup memory in bytes | 584,114,176 | 605,732,864 |

Sharing the index saves a walk and parse of 682 physical files. Both configurations pass the same 116 named tests, including the two distinct document judgments. The shared configuration executes about 1.2% fewer instructions and makes about 3.3% fewer allocation calls. Its observed peak memory is about 3.7% higher. Both document tests can run concurrently in one process. The observations come from one complete run of each configuration and include the measurement overhead.

A two-module crate with a re-export gives equal source workspaces and resource usage through the direct loader and the production gate. The ordered export and resolution report has ten lines and digest `5ab34bd7e1e6d9f8`. When both modules exceed the delimiter budget, both loads refuse `src/first.rs` at line 1 with `Budget::Delimiters`, limit 128, reached by `mod first;`. Comparing the public-surface report and the complete refusal text checks ordering as well as the error kind.

## Inputs and measurement

The native Linux sample uses Git tree `566c505001750b48746d0a1f37afe73ad260372f` from commit `b784a526bab523c493062bca78862d95be06322f`. Both hook implementations check that tree. The shared-index implementation is at commit `789a6184`. Both measurements cover formatting, licensing and the repository test binaries. The current hook also runs production Clippy on the staged libraries and binaries. The host has an AMD Ryzen AI Max+ 395 processor, Linux `7.3.4-1-cachyos`, glibc 2.44 and Rust nightly `5df8c02af` with LLVM 23.1.5. The test binaries use the hook's default unoptimized test profile.

To reproduce the comparison, retain the same Git index and change the hook and test implementations in the working tree. Check `git write-tree` before and after each run. Warm the hook's selected test executables with `cargo test --locked --no-fail-fast --no-run` and the hook's exact list of test targets. Execute `.githooks/pre-commit` once per configuration. Preserve its exit status, the sorted list of test names and their outcomes, stdout and stderr.

* Compile the `raf_source` library and the `repo_gates` test with `-C instrument-coverage -C llvm-args=-instrprof-atomic-counter-update-all`. Link the matching LLVM profile runtime into every test binary with instrumented library code. Start each with `--list` in a separate profile directory and verify it writes a profile there. Set `LLVM_PROFILE_FILE` to a fresh directory with the filename pattern `%p-%m.profraw` for the measured run. Combine those profiles with `llvm-profdata merge --sparse` and export line counts with `llvm-cov export --format=lcov` using the measured executables.
* Count a workspace index build at the package-directory indexing call in `Workspace::build_discovered_with_syntax`, a physical source analysis at `meter.read(path, &text)` in the module walker's new-file arm, and a native parse at `syn::parse2(original)` in `raf-source`. Count the repository runtime-boundary rule's `syn::parse_file(text)` separately. One parsed file can serve multiple target reaches. These counters describe RAF analysis. Compiler and formatter parses and work performed inside the native parser have separate scopes.
* Run `perf stat --no-scale -x, -e instructions:u` with child inheritance and require full reported event coverage. In a fresh systemd user scope, read `memory.peak` from the current cgroup after the hook exits. The kernel high-water mark covers the full process tree, observer processes, charged file pages and kernel memory.
* Interpose glibc `malloc`, `calloc` and `realloc`. Forward each call to the native allocator and count successful results in one shared mapping across threads, forks and executable replacements. Count registrations after startup and compare their PID multiset with successful workload execs from `strace -f`. The dynamic loader's standalone information commands and the static cache query have no dynamic allocation imports. The counts describe these three entry points. Custom arena suballocations, aligned-only APIs and private allocator entry points are outside this metric.

The independent serial and two-worker measurement used the release-profile `raf-source` fixture at commit `55be9e5d62fcf545098a014960fb873546b150ad` with 64 sibling modules. Both modes returned the same 8,320-line export digest, `61b7161fb620fcad`. The serial load used 207,568,118 instructions and 285,482 allocations. The parallel load used 207,715,145 instructions and 285,513 allocations. These two allocation counts come from the fixture's Rust global allocator. Sharing the existing source workspace eliminates one full traversal while keeping the ordinary parser and its resource limits. [Parallel measurement](../crates/raf-source/examples/parallel_cost.rs) · [Workspace report](../crates/raf-source/examples/ws_cost.rs)
