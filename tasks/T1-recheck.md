# T1 recheck (required findings of T1-review.md)

Scope: 03edbca (reviewed) and the fix commits d2046a2 and 9bfdaf6, judged at 973d7ab in
`scratch/t123-recheck` (detached 973d7ab, clean at the end). No timing run.

| ID | Result | Evidence |
| --- | --- | --- |
| B1 CHANGELOG described strict mode as shipped | FIXED | CHANGELOG.md:10-15 at 973d7ab opens with "**docs:** strict mode ... is designed: its capability document states the contract, the change record ... lists the capabilities it changes, and the decision page `arch:debt-ratchet` ...". The other [Unreleased] lines are per-task present-tense lines for code that exists at 973d7ab (per the plan Addendum); the T1-time claims (`coverage lines`, `debt --write`, raf-source resolution) are not stated as shipped in the first line. d2046a2 changed CHANGELOG 37 lines to the designed wording. |
| B2 duplicate capability copied into the inventory table is accepted | FIXED | tests/docs_contract.rs: `change_record_violations` refuses a slug repeated in an `INVENTORY.changes` row ("inventory lists `cap:X` twice ...") and an unknown slug; `inventory_table_has_the_documented_shape` (:1262) asserts unique rows that are a subset of the inventory capabilities; new test `inventory_rejects_a_capability_listed_twice_even_when_the_record_lists_it_twice`. Mutation in the clone (`if false && !seen.insert(...)`): that test FAILED; restored. |

Commands (all in the clone):
* `cargo test --locked --test docs_contract` exit 0, 31 passed, 0 failed (includes `inventory_table_has_the_documented_shape`, `inventory_rejects_a_capability_listed_twice_even_when_the_record_lists_it_twice`).
* `cargo test --locked --test docs_contract inventory_rejects_a_capability_listed_twice` with the mutation: 0 passed, 1 failed (mutant killed).

Process finding closed too: 9bfdaf6 (rationale.rs architecture-page count 2 to 3; roadmap.rs guard "an open change that lists the capability") is covered by the full-suite run noted in T2-recheck.

Advisories: none made worse.


Full-suite: `cargo test --locked -p raf-okf -p raf-workplan -p cargo-workplan --no-fail-fast` at 973d7ab (clean clone): exit 0, 1509 passed, 0 failed (includes 9bfdaf6 rationale/roadmap tests).
VERDICT: PASS
