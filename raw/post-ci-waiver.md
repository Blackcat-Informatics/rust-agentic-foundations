### Owner waiver: hosted CI half of the acceptance criterion

**Shortfall.** Issue 1's acceptance criterion reads "`make check` passes on the empty workspace, and so does CI". Its **CI half is unverified**:
- GitHub refused every Actions run on this branch for organisation billing ("The job was not started because recent account payments have failed or your spending limit needs to be increased").
- The latest refused run is 38229506850, on head `80c14d7`.
- No step of `.github/workflows/ci.yaml` has ever executed.

**Owner decision, quoted verbatim.**
- Question: "Hosted CI has never executed on PR 11 … Issue #1's acceptance criterion says "make check passes … and so does CI". Local `make check` passes on the exact merge candidate (base unchanged, candidate tree == PR head). How should Stage 3 proceed?"
- Answer: **"Waive CI, merge now"**. The owner chose it on 2026-10-12.

**What stands instead.**
- Local `make check` returns rc 0 on `80c14d7`, which is the exact integration candidate: the base is unchanged and the merge tree equals the head tree. It covered:
  - fmt;
  - clippy `-D warnings`;
  - 114 + 42 tests;
  - doc;
  - `cargo deny --offline`;
  - reuse;
  - MSRV 1.98.1;
  - licences and issue-refs.
- `actionlint` and zizmor pass on the workflow.
- Every action pin was verified against upstream.

**Still unproven until CI runs.**
- The workflow's behaviour on a GitHub runner, including toolchain installation from the pinned action and the msrv job's `RUSTUP_TOOLCHAIN` override.
- The *fetching* `cargo deny` advisory check. Locally only the offline check has run.

**To close it.** Clear billing, then run CI on `main` after merge (or re-run on this head) and fix anything its first execution surfaces.

This is a waiver of a validation step, not a cut in shipped code. Under the repository's ledger rules it is recorded here, not in `.deficiencies`.
