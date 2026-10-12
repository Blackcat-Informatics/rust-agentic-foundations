### Stage 2 fix R1 (`f711aa2`): workplan and architecture docs

**Gaps closed**
- **G1 (CRITICAL), essay content lost.** The OKF profile now has a "Section | Holds" table with the essay's seven definitions, and the document model links to it. The Grounding Contract is cited at its pinned commit, and Scenario B lists "an accurate capability explanation".
- **G4, examples at completion.** Reconcile has a new `example-failing` obligation. Complete-gate condition 5 now requires the bound tests *and examples* to pass, with a refusal example. "criterion" is removed.
- **G5, workplan `depends` edges.** prose-assessment now depends on evidence-claims. The OKF reader no longer names a `raf_source` type. The profile defines what a `depends` edge means.
- **G6.** All five M2 change records now list `#constraints` in `affects`.
- **G8.** The profile states that binding resolution waits for the source index.
- **G12.** The delivered binding minimum is in the normative table, with identical wording in all three docs.
- **A4.** The reconcile example sentence now matches what the example shows.

**Checks run**
- Every R1 grep and `yq` check in the plan passes.
- `cargo test --test docs_contract --locked` 46/46, so the graph stays acyclic.
- `make issue-refs` rc 0.
- `uvx reuse@6.2.0 lint` 77/77.
- The pre-commit hook passed on the staged snapshot.

**Review:** `G-R1-review`. The first pass BLOCKED on two stale "bound tests" / "criteria" phrases; both were fixed, and the recheck gave PASS.

**Hosted CI:** still blocked by the organisation billing block, so it is unverified.
