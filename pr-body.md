Closes #2

Adds the first two RAF crates, `raf-okf` and `raf-source`, and delivers `cap:okf-profile` and `cap:source-index`. Both capability documents are `state: delivered`, and their bindings resolve through `raf-source`.

## Changes

### `raf-okf`: reader, validator and writer for RAF's OKF profile

- A hand-written, bounded reader for the profile's YAML subset.
  - Flow mappings, duplicate keys, anchors, aliases, tags, merge keys, block scalars and unknown escapes are each refused with a positioned error.
  - Input is capped at 1 MiB, YAML nesting at 32 levels and Markdown nesting at 64. The billion-laughs input stays under 8× its size in allocation, measured with `cap`, with no `unsafe`.
- A deterministic writer.
  - Every tracked document round-trips byte-for-byte.
  - Key order comes from the same schema table the validator uses. `tags` render sorted.
  - Comments stay attached to their keys.
- `Body::parse` builds the body model:
  - fences, headings and anchors at every level;
  - H2 sections with line and byte spans;
  - links of every form, outside fences and code spans;
  - generated and PROPOSED blocks with content spans;
  - a prose view and `placeholder_lines`.
- `replace_spans` applies edits transactionally.
- `Index::parse` handles the capability index.
- `Profile::validate` returns a typed `RafDocument` or every violation. Each violation names the file, its location and the rule.

### `raf-source`: workspace source index

- **File access:**
  - Every read goes through the `Tree` seam, with `MemoryTree` and `DiskTree` implementations.
  - Containment is checked on the canonical path and verified again on the opened handle. On Linux that check goes through `/proc/self/fd`, which closes the rename race. Other platforms have portable fallbacks.
  - A FIFO is refused without blocking, and file size is capped at 8 MiB.
- **Parsing:**
  - A token pre-scan enforces five recursion budgets before parsing.
  - `parse_source` runs `syn` and the caller's visitor on a named thread with a 64 MiB stack. Syntax errors carry their line and column.
  - A child-process harness classifies each hostile input as a typed refusal, a complete index, or a genuine abort, and fails on anything else.
  - A cargo-fuzz target ships with a committed corpus, which is replayed under `cargo test` in debug and in release.
- **Discovery and modules:**
  - Package discovery takes explicit roots and parses manifests with `toml`.
  - Targets get crate paths in the style of rustc and Cargo: `bin_crate`, `test_crate`, `bench_crate`, `example_crate`, `build_script`. Each crate root is a module item.
  - The module tree follows rustc's rules: `#[path]`, inline modules, `name.rs` beside `name/mod.rs` is an error, and `cfg_attr` path alternatives are followed.
  - `cfg(test)` code is tagged, never dropped.
  - Module nesting and module walks are bounded.
- **Index and resolution:**
  - The symbol index records spans, namespaces, documentation flags and targets.
  - `resolve` applies rustc's shadowing order, deciding a glob only once every explicit import has settled, so the result does not depend on declaration order. There is no search cap. It returns a `Resolution` or a typed `NotFound`, `Ambiguous` or `Cyclic`.
  - Trait-impl members are keyed by the canonical trait path.
  - A workspace member inherits keys from its workspace the way Cargo does: membership covers the root package, the `members` globs minus `exclude`, and in-workspace path dependencies.

### Repository

- `tests/docs_contract.rs` reads every document through `raf-okf`, and its private YAML, link, marker and heading scanners are gone. It keeps only the cross-document and repository checks. It resolves every capability binding through `raf-source`, and requires each crate doctest to equal the Usage and Examples blocks of its capability document.
- New gates: `make test-release`, `make fuzz` and `make cross-check` (macOS and Windows targets).
- `make deny` now covers the whole workspace; until now it checked only the root package. CI has a job for each gate.
- AGENTS.md, README, SECURITY.md, CHANGELOG and the architecture page describe what is built. The workplan-document-model and census-rule-api capabilities gain one Contract sentence each, about macro-generated symbols and `Rule: Sync`.

## Decisions that differ from the issue text

1. **Base OKF keys.** The issue says `resource`, `tags` (re-emitted sorted) and `timestamp` "follow the base OKF v0.1 shape". The architecture page says "The closed key sets do not include the OKF base key `resource`, so a document that carries it is rejected." Decision: the profile is extended so `tags` (unique, rendered sorted) and `timestamp` (RFC 3339) are optional keys on every type, and `resource` stays rejected.
2. **Default roots.** The issue says package discovery has a "default `crates/`". Decision: `raf-source` takes explicit roots and has no default; each consumer owns its default. Consequence: an empty root list loads an empty workspace without error, so consumers must guard against that.
3. **Containment.** This is closed on Linux by verifying the opened handle. Other unix platforms and non-unix platforms use portable fallbacks, which keep a residual rename window. The Contract and SECURITY.md state the guarantee for each platform.
4. **Links split across lines.** These are refused as a profile rule, `Rule::SplitLink`, in every tracked Markdown file.

The port draws on every precedent implementation and takes the best part of each. One plan line is meant narrowly: the `cfg(not(unix))` skip in the existing `tests/support/mod.rs` is not carried into `DiskTree`, and that helper itself is unchanged.

## Validation

- `make check` on 6c744af (the last code change before this PR) exits 0 in 1m27s, with 963 tests passed and 0 failed. It runs these gates:
  - fmt, for the root and fuzz workspaces;
  - clippy with `-D warnings`;
  - test and test-release;
  - doc with `-Dwarnings`;
  - deny, now covering the whole workspace;
  - reuse;
  - msrv on 1.98.1, for the root and fuzz workspaces;
  - fuzz: 5000 seeded runs;
  - cross-check, licenses and issue-refs.
- The pre-commit hook (fmt, REUSE, `repo_gates` and `docs_contract` on the staged snapshot) ran in full on every commit.
- Each task had an independent review, and every blocking finding was fixed and rechecked before commit. Before implementation the plan had five separate reviews: completeness, constraints, SOLID/ETHOS design, enhancement, and a completion adversary.

## Residual risks

- **Abort on hostile shapes.** Composite and uncounted shapes that no budget counts can still abort the process with a stack overflow. An abort fails the run and is never a pass. The Contract states this.
- **Untested non-Linux paths.** The containment checks for other platforms are compiled for macOS and Windows but never run on those platforms, and the rename race remains there.
- **Loading time.** Loading a workspace has no time bound. Only work and size are bounded.
- **Unresolvable imports.** A `use` path that never resolves is not a load error. It shows up as `NotFound` only when something resolves through it.
- **Licence exceptions.**
  - The root `deny.toml` has a crate-scoped exception allowing Unicode-3.0 for `unicode-ident`, which `syn` requires. Its licence is `(MIT OR Apache-2.0) AND Unicode-3.0`. `deny.toml` says "an addition should require someone to look at the new crate and decide", so please confirm this.
  - `fuzz/deny.toml` allows NCSA for `libfuzzer-sys`.
- **Bounded fuzzing.** The fuzz gate runs a bounded 5000 inputs of at most 4096 bytes. The large hostile files are covered by the corpus replay instead.
- **Beyond the issue text.** Canonical trait keys and Cargo-accurate workspace membership go further than the issue asks. They were needed so that delivered bindings and real workspaces resolve.
