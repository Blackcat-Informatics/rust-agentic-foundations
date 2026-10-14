# R1 review: raf-okf body memory is bounded

## Reviewed scope
Uncommitted changes: crates/raf-okf/src/body/{heading,marker,mod,prose}.rs, crates/raf-okf/tests/bounds.rs,
fuzz/fuzz_targets/parse_document.rs, fuzz/Cargo.toml, fuzz/Cargo.lock, fuzz/corpus/parse-document/, Makefile.
Contract: remediation-plan.md "### R1", Mutation rule, Per-item gates. Changes under crates/raf-source, tests/, docs/ ignored.

## Evidence
- Falsification on original src: saved `git diff -- crates/raf-okf/src` to /tmp/r1src.patch, `git checkout -- crates/raf-okf/src`,
  `cargo test -p raf-okf --locked --test bounds` -> "memory allocation of 61440 bytes failed", SIGABRT (the 1 GiB ceiling
  returns null, handle_alloc_error), exit 101. Patch reapplied with `git apply -p0` (the repo diff uses no a/ b/ prefixes);
  `git diff --stat` identical before and after.
- `cargo test -p raf-okf --locked`: all suites pass, including bounds (1 test).
- Measured peaks (--nocapture): prose unit long 12,460,508 vs short 12,153,327 (delta 307,181);
  PROPOSED unit long 26,896,316 vs short 26,589,135 (delta 307,181). Allowance = 4*61440 + 65536 = 311,296.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean (one earlier run failed in raf-source test
  module_tree, a file of the other in-progress item; a re-run was clean).
- `make fuzz`: exit 0; fmt/clippy/deny pass; runs parse_source then parse-document (7 corpus files, 5000 runs, seed 1, no crash).
- `make reuse`: compliant, 210/210 files; fuzz/corpus/** is covered by the existing aggregate annotation in REUSE.toml.

## Assessment
- Arc<str> deviation closes the gap. Each section slug is one allocation, shared by Section, the Anchors table (first use),
  Heading.anchor, and every line/prose-line/PROPOSED block (pointer only). Heading.text is shared with Section.heading.
  Public accessors still return Option<&str>, so signatures are unchanged. The measured delta for a 60 KB heading is constant
  (~5 heading-sized copies: text, anchor, slug temporaries, input), independent of line count (about 65k lines here).
- Remaining heading-sized copies are per distinct heading or per violation, not per line: repeated-heading anchors
  (`base-N`, one per heading, each bounded by that heading's own input bytes), `visible_content` per section (sum <= input),
  `Location::Section(heading.to_string())` per violation. ProposedBlock.content and ProseLine.text are each the line's own text.
  No per-line input-sized or heading-sized copy remains.
- Test: both pairs (prose lines, PROPOSED blocks) with identical line counts asserted; 1 GiB ceiling in the allocator;
  formula matches the plan with sections = 1; falsifies on HEAD as shown above.
- Fuzz target calls Document::parse, reads headings/sections/links/prose_lines/proposed_blocks, and Profile.validate;
  registered as a [[bin]]; Makefile loops both targets. No lint, gate or deny setting weakened (the unsafe allow in the test
  carries a reason and SAFETY comments).

## Findings
1. ADVISORY: the allowance margin is thin. Observed delta 307,181 against an allowance of 311,296 (about 4 KB). The result
   is deterministic, but any added heading-sized temporary would trip it. That is the intended sensitivity; consider noting the
   five-copy baseline in a comment.
2. ADVISORY: the pair uses one section, so the `sections` factor of the formula is never exercised above 1. A multi-section
   variant (many long headings) would also guard Anchors and Section tables. Not required by R1.
3. ADVISORY: the plan says the HEAD abort run is "made in a child process and recorded"; the test itself has no child-process
   harness. This review recorded the run above, which meets the "recorded" part.

No REQUIRED findings.

VERDICT: PASS
