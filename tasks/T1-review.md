# T1 review: workspace manifest, toolchain, lints, Cargo.lock

## Reviewed scope
Cargo.toml, Cargo.lock, src/lib.rs, rust-toolchain.toml, rustfmt.toml, clippy.toml, deny.toml, .gitignore (all untracked) against plan Task 1, A1, A13, A19, adv. 8, .gitignore row, and R1-R4, R40-R42 (MSRV 1.98.1).

## Evidence (all run in the worktree)
- R1 + R2 + R3 tomllib assertions (resolver 3, edition 2024, rust-version 1.98.1, authors, license, repository; lints deny/warn with priority -1; nightly + clippy/rustfmt): rc 0
- awk reason check (every "allow" preceded by a # comment): rc 0
- `cargo metadata --locked`: rc 0, 0 `warning` lines; one package, 0 dependencies, rust_version 1.98.1, license correct
- `cargo fmt --all --check`: rc 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: rc 0
- `cargo test --workspace --locked`: rc 0 (0 tests; expected for the stub)
- `cargo doc --workspace --no-deps --locked`: rc 0
- `cargo deny --offline check`: rc 0 (advisories, bans, licenses, sources ok)
- `rust-validate 1.98.1 cargo check --workspace --all-targets --locked`: rc 0
- `git check-ignore -v .goals .baseline .stage/x target`: rc 0, matches .gitignore lines 6, 7, 5, 4
- clippy.toml has no msrv/1.98 key; no `#![feature]`; no `crates/` dir; no purrdf/katamari/logit/sparrow/1.97 strings in the files; no .cargo/config.toml, no rustflags key.
- SPDX header present in all 8 files (checked by reading each). Cargo.lock is generated and has no header (REUSE coverage is T2's REUSE.toml; plan R41 says REUSE.toml covers it).
- Root package: publish = false, no deps, no raf-* crate, members = []; does not pre-empt shared crates. A1, A13 (code licence on code/config), A19 (logit-loom set + unused_qualifications + allow_attributes_without_reason), adv. 8 (allow list, unused-allowed-license) all satisfied.

## Findings
REQUIRED
1. Cargo.toml line 33 comment contains the word `rustflags` (twice). The contract's R2 check `git grep -nE 'RUSTFLAGS|rustflags' -- Makefile .github .githooks Cargo.toml` must produce no output, and it will once Cargo.toml is tracked (confirmed: `grep -nE 'RUSTFLAGS|rustflags' Cargo.toml` prints line 33). The behaviour is right (no rustflags exist) but the acceptance check fails. Fix: reword the comment (for example "Declared as workspace lints rather than compiler flags, which would also reach path dependencies we do not own").

ADVISORY
- Cargo.toml `[workspace.lints.rust]` comment sits above all four lints but explains only unreachable_pub; harmless.
- `authors` uses "Blackcat Informatics Inc." (matches R1 exactly) while SPDX headers use the registered-mark form; intentional per R1.

Initial verdict (superseded by the Recheck below): BLOCKED
(One REQUIRED finding: reword the `rustflags` comment, then re-run the R2 grep; everything else passes.)

## Recheck
- `grep -nE 'RUSTFLAGS|rustflags' Cargo.toml`: no output (rc 1), so finding 1 is resolved.
- `cargo metadata --format-version 1 --locked`: rc 0, 0 warning lines.
- `cargo fmt --all --check` rc 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` finished clean; awk allow-reason check rc 0.

VERDICT: PASS
