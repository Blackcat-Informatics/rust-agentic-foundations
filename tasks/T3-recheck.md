# T3 recheck (required findings of T3-review.md)

Scope: fix commit 7a3a5f2 (distinct claim keys, no separator claims, every paragraph end of the
claim rules), judged at 973d7ab in `scratch/t123-recheck`. No timing run. Mutations applied one
at a time in the clone to production code, the named test file run, then `git checkout -- .`.

| ID | Result | Evidence |
| --- | --- | --- |
| B1 claim keys not unique (`Foo`,`Foo`,`Foo 2`) | FIXED | `strict/claims.rs:689-709` `number_repeats` takes the first `-n` no unit of the section has. `tests/claim_coverage.rs:520` `a_repeated_key_takes_the_first_number_no_other_key_of_its_section_has_in_either_order` ok: `[foo, foo-3, foo-2]`, `[foo-2, foo, foo-3]`, `[foo, foo-2, foo-3]`. Mutant (`.find(\|_\| true)`): that test and `a_contract_is_cut_into_clauses...` FAILED. |
| B2 thematic break / empty item in Constraints is a claim | FIXED | `claims.rs:~570-577` ends the open unit and opens none; one shared `list_item`/`is_thematic_break`/`states_an_item` in `strict.rs:223-261` used by content.rs and claims.rs. `claim_coverage.rs:551` `a_thematic_break_and_an_empty_item_in_constraints_are_no_claim_and_an_item_beside_them_is_one` ok (also asserts the only strict finding is the neighbour `- The neighbour.`). Mutants (thematic break a claim; empty item a claim): that test FAILED each. |
| B3 comment line / any heading does not end a paragraph | FIXED | Constraints: `claims.rs:556-564` a comment-only line sets `after_blank` and closes unless in an item; Contract: `claims.rs:383-405` every heading sets the boundary. Tests `claim_coverage.rs:580` `a_comment_line_ends_a_constraints_paragraph_and_an_item_goes_on_only_with_its_indented_lines` and `:603` `a_bold_lead_directly_after_any_heading_starts_a_contract_clause` ok. Mutant (comment line neither sets `after_blank` nor closes): first test FAILED. Mutant (non-clause heading read as text, boundary false): second test FAILED. (Two other attempted mutants were equivalent because `after_blank` already stops continuation, or because the preceding marker line already left the boundary set; not defects.) |
| B4 report sort by capability and `--capability` filter unpinned (MR1, MR2) | FIXED | Report fixtures hold a second delivered capability `cap:alpha`. `claim_coverage.rs:459` `the_report_lists_every_claim_of_every_delivered_capability_sorted_and_the_same_each_run` ok; `crates/cargo-workplan/tests/claims.rs:176` `the_report_sorts_by_capability_first_and_capability_prints_the_one_it_names` ok. MR1 (sort `(section,line,capability)`): the library test FAILED. MR2 (`coverage.rs:91` `whole.capability(id)` to `whole`): the CLI test FAILED (3 passed, 1 failed). |

Commands: `cargo test --locked -p raf-workplan --test claim_coverage <name>` for each name, exit 0, 1 passed each (12 tests in the file all ok unmutated); `cargo test --locked -p cargo-workplan --test claims`: 4 passed, 0 failed.
Full-suite run: see the line at the end.

Advisory effects: A4 (duplicated `list_item`) fixed by sharing. Side effect recorded in validation.md: the content rule now refuses a ten-digit ordered marker as a list item (CommonMark limit), consistent, not a regression. No advisory made worse.


Full-suite: `cargo test --locked -p raf-okf -p raf-workplan -p cargo-workplan --no-fail-fast` at 973d7ab (clean clone): exit 0, 1509 passed, 0 failed (includes 9bfdaf6 rationale/roadmap tests).
VERDICT: PASS
