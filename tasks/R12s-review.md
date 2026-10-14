# R12s review

## Scope
Uncommitted change in /tmp/raf-par/p4-r12s (on e472918) plus raw/R12s-bom-span.patch. The p4 tree does not contain the BOM patch (facts.rs unmodified there), so the full state was reviewed in a copy, /tmp/raf-par/r12s-rev, with the BOM patch applied cleanly.
Contract: R12s bullets (G5, G11, Q-6, Q-7, G13), Mutation rule, per-item gates.

## Evidence
- `cargo test -p raf-source --locked`: all binaries pass, 38 s total including build. `--release`: all pass, 59 s. load_cap 0.5 s debug, 0.04 s release; source_text and spawn_limit under 0.1 s.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean. docs_contract (139) and repo_gates (57): pass. `make reuse`: compliant.
- Mutations re-run by me, each failed and was reverted:
  - G5: `max_bytes = u64::MAX` at modtree.rs:440. 3 of 5 load_cap tests fail.
  - Q-7/BOM: facts.rs `Positions::new` back to `vec![0]`. `a_byte_order_mark_on_the_first_line_starts_the_span_of_its_item_after_the_mark` fails (start_byte 0, expected 3).
- G5: tests cover the library root and a module file at exactly 8 MiB (loads) and +1 (TooLarge with the cap as limit, naming the file, declaring file and the `mod` line), a lowered cap of 1000 on a child (1000 admitted, 1001 refused), and a raised cap. The files are comment-padded and valid Rust. Both profiles are fast.
- G11: the 64 MiB parse stack is fixed in `PARSE_STACK_BYTES`; the limits tried are 16, 24, 32, 40 and 48 MiB (`mib = 1024` KiB). I probed the child directly: 8 to 64 MiB all give `Spawn ... cannot start the parse thread` (EAGAIN), 4 MiB cannot start the binary (no line), and 90 MiB and above load. The plateau is wide. 25 repeat runs all passed. The test cannot pass vacuously: a limit with no result line is skipped, but any line that is not a Spawn refusal naming src/lib.rs panics, and `refused > 0` is required, so at least one real Spawn result is needed. An unlimited control child must load.
- Q-6: `lints_the_root_denies_missing_docs_unsafe_code_and_unreachable_pub` reads the real root Cargo.toml via `gate_root()` (package dir unless RAF_GATE_ROOT is set), through a parser that has its own unit test (`lints_reads_the_level_of_a_lint_and_sees_a_weakened_one`). The mutation log records failure under "warn".
- BOM fix: syn 3.0.6 `parse_file` strips exactly one leading U+FEFF, and `Positions::new` moves line 1's start past exactly one, so spans, lines and columns stay consistent and nothing is double-stripped. `starts` is only used in `byte()`, so only line 1 is affected; later lines are unchanged, and the long-line mark index starts from the shifted `own`. The test checks line, start_byte, covered text and `item_at`.
- G13: the stale "unknown-key path rejects" comment is gone (grep empty) and the new text matches the rule in raf-okf violation.rs ("`resource` is not a key of a ...").
- Bindings: all new tests are listed in source-index.md with matching names, and the BOM test's binding and Verification sentence are added by the BOM patch. No process wording or issue numbers in the new files.

## Findings
REQUIRED: none.

ADVISORY:
1. R12s-mutations.txt names `a_byte_order_mark_leaves_lines_and_byte_spans_where_the_text_has_them`, which no longer exists. The final test is `a_byte_order_mark_leaves_the_lines_of_the_text_where_they_are` plus the new first-line test. Only the log is stale; the "no BOM strip" mutation (`parse_str`) is caught only by the first-line span test, as the log notes.
2. The BOM span test checks only a column-0 item on line 1. A second item on line 1 (non-zero column after the mark) would assert the column arithmetic directly. The fix is correct by inspection and syn's behaviour.
3. The G11 test accepts a single refusal; asserting at least two of the five limits would guard a narrower plateau. It is not needed today.
4. The BOM patch must be applied before commit (the p4 tree lacks it). The orchestrator should commit the BOM fix as `fix(raf-source)` separately from the tests.

VERDICT: PASS
