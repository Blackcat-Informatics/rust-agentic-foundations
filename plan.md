# Plan: one home per foreign host import declaration (tracker issue 207)

Base: origin/main f7b87db8. Worktree: .worktrees/207-census-one-home-per-foreign-host-import. Build lock: /tmp/raf-build.lock.
Verbatim issue: `stagectl issue 207` (acceptance below is copied from it).

## Behavior
A built-in census detector, or a pattern matcher, for foreign-function import declarations inside `extern` blocks (the items an attribute macro turns into host bindings). It reports a second declaration importing the same host function (same attribute namespace and name) outside the job's home, one finding at the non-home site.

## Acceptance (verbatim from the issue)
- Two `extern` blocks in different modules, each importing the same host function (same attribute namespace and name), produce one finding at the non-home site.
- One import at the home produces none.
- Importing a different host function produces none.
- The check covers test targets and fixtures as well as shipping code.
- Positive and negative fixtures are registered.

## Exclusions
- No change to raf-workplan or the raf-okf profile schema.
- Choose the mechanism (a built-in detector in crates/raf-census/src/detectors/ or a pattern matcher) from the existing code: read crates/raf-census/src/detectors/mod.rs, rule.rs and selftest/fixtures.rs before choosing. Prefer the smallest that meets the acceptance.

## Documents
census-detectors.md and census-cli.md list the built-ins; update them. Open a change record with cargo workplan, complete it with `cargo workplan change complete`, renew stale assessments only after reading each section against the change, restore the two licence lines.

## User rulings
- Coordination (standing direction as Patrick's): this lane is census gap 207, in its own branch and reviewed pull request, parallel to 267 in raf-workplan and raf-okf (no shared files).
