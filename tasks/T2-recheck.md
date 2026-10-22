# T2 recheck (required findings of T2-review.md)

Scope: fix commit a34db9a (test(raf-workplan): pin three rules of strict mode ...), judged at
973d7ab in `scratch/t123-recheck`. No timing run. Each fix was mutated in the clone (the
production line named by the review), the strict test file run, and the clone restored with
`git checkout -- .`.

| ID | Result | Evidence |
| --- | --- | --- |
| B1 generated-block exclusion of `strict-section-thin` unpinned | FIXED | `crates/raf-workplan/tests/strict.rs:192` `the_words_of_a_generated_block_are_not_prose_of_its_section`: ok (1 passed). Mutant `content.rs:127` `&& !line.in_generated()` replaced by `&& true`: that test FAILED (20 passed, 1 failed). |
| B2 "for each delivered capability" unpinned | FIXED | `strict.rs:225` `a_capability_that_is_not_delivered_has_no_finding_of_the_content_rules`: ok. Mutant `content.rs:26` `== State::Delivered || true`: that test FAILED. |
| B3 no gap while a capability document does not load unpinned | FIXED | `strict.rs:590` `while_a_capability_document_does_not_load_no_gap_of_a_bound_package_is_reported`: ok. Mutant `strict.rs:168` `if false && capability_unloaded`: that test FAILED. |

Also run (advisory fixes A1, A2 of a34db9a, no regression): `a_code_span_inside_a_comment_names_no_example_in_usage` ok, `a_thematic_break_and_an_empty_item_are_no_limitation` ok (each 1 passed, 0 failed).

Commands: `cargo test --locked -p raf-workplan --test strict <name>` for each name above, exit 0, 1 test each; the three mutant runs of `--test strict` each ended `FAILED. 20 passed; 1 failed` with exactly the named test failing; clone clean afterwards.
Full-suite run: see the line at the end.

Advisories: A1 and A2 fixed; A3-A6 unchanged as stated in validation.md; no advisory made worse.


Full-suite: `cargo test --locked -p raf-okf -p raf-workplan -p cargo-workplan --no-fail-fast` at 973d7ab (clean clone): exit 0, 1509 passed, 0 failed (includes 9bfdaf6 rationale/roadmap tests).
VERDICT: PASS
