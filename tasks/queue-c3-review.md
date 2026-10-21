# C3 preserved delta review

VERDICT: NEEDS-FIX (assessment SPDX headers). No functional defect found in the reviewed production delta.

Scope: current uncommitted changes to census.rs, units/mod.rs, tests/units.rs,
census-normal-form.md and its six assessment records. Read remediation-plan.md
through Rev 5 and its plan review. GAP-12's reader-version acceptance and GAP-13's
serde_json removal remain mapped to C6; this review does not declare them complete.

## Required fix

All six changed work/assessments/census-normal-form-*.md records remove their two
YAML SPDX comment lines. Restore the copyright and CC-BY-4.0 identifiers. This is
a concrete regression against AGENTS.md licensing requirements; qualification
must include the normal reuse gate. Assessment freshness does not replace licensing.

## Production and requirements review

- GAP-14: Census::load calls units::collect directly. Every ItemId in each source
  group is resolved with an error-returning lookup; an unresolved id and an empty
  group return Error::Source naming the group's file. Neither can silently discard
  a unit. The remaining unit_kind continue is the intentional non-unit-kind filter
  explicitly preserved by the plan amendment. Source-error line calculation runs
  only on the error path; absent text/invalid offset supplies line 1 while retaining
  the hard failure and explanatory reason.
- GAP-17: Census::load traverses every SourceFile.reached entry, resolves its
  TargetId and collects TargetKind into a BTreeSet. It includes reaches through test
  code, deduplicates kinds and preserves independent shipping classification.
  An invalid target returns Error::Source rather than dropping its reach.
  Census::files and Census::file expose the same production FileCtx values.
- Confirmed raf-source's SourceFile/Reach/TargetKind contracts against current
  implementation. Workspace construction already validates file reaches;
  items_by_source exposes index-owned groups. The new inconsistency branches are
  defensive checks, not malformed-user-input rules reachable through the current
  public Workspace construction API. No execution evidence exercises those branches.
- The capability adds the target-kind field and hard-failure contract and binds
  the new test. The test checks Lib, Test and Example through Census::load. Existing
  tests retain shipping and multi-reach unit behavior.

Advisory: extend the existing shared-file fixture with a {Lib, Test} target_kinds
assertion. It would directly pin the newly documented aggregation behavior, which
the single-kind fixtures do not exercise. No architectural abstraction is needed.

## Validation limits

No builds or tests were started by this review. The coordinator-owned
raw/queue-c3-units.log currently records 25 passed, zero failed/ignored/filtered.
That is observed focused evidence only; it does not establish reuse, current
assessment acceptance, document-contract gates, normal commit hooks or make check.
No timing runs, mutation passes or source edits were performed.
