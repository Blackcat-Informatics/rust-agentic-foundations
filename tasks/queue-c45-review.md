# C4/C5 substantive review

VERDICT: NEEDS-FIX for the missing caller-span warning below. No production-code
defect or fuzz-path bypass found in the reviewed delta.

Scope: current dirty manifest/lockfile, census-normalize harness and its three
named seeds, Makefile fuzz loop, SECURITY/AGENTS/CONTRIBUTING, census capability
dependency and ledger/migration designed-scope repairs. Read remediation-plan.md
through Rev 5. Generated hash-named corpus products are disposable execution
output, not reviewed source. No builds or source mutations performed.

## Required correction

Rev 5 expressly requires a census-normal-form Constraint stating that collection
invalidates proc_macro2 spans held by its caller on the same thread. The existing
Constraint (lines 642-644 at review) only says the thread's span table is cleared
when the load ends. That does not tell callers that their existing spans become
invalid. Add the explicit warning and refresh its assessment. This documents an
observable API hazard already present in the C2 implementation; it is not a new
permission or a proposed scope expansion.

## Reviewed behavior and wiring

- The new fuzz binary is registered under the kebab-case name census-normalize;
  Makefile includes that exact name in the real bounded loop. Existing failure
  propagation and run/memory/allocation limits remain intact; no ignored target,
  swallowed panic, catch_unwind or feature-dependent exclusion was introduced.
- The target calls public Normalizer::print on arbitrary UTF-8 text and public
  Census::load on a one-file MemoryTree. Invalid UTF-8 is correctly excluded from
  APIs taking str. Ordinary syntax/budget Results are discarded as expected fuzz
  outcomes; panics and overflows still fail the harness. Span invalidation happens
  after temporary print results are dropped. Census returns owned unit data, so no
  live proc_macro2 tokens escape those calls.
- The Once self-check requires valid source to load and produce exactly one unit.
  That traverses source discovery, parameter fact collection, body slicing,
  normalization and unit assembly, preventing a build that refuses all source from
  silently passing. It currently does not assert parameter contents or positions.
- Named seeds exercise a simple body, closure/local bindings and the deeply nested
  signature reproducer. They are plain source data and covered by the existing
  REUSE corpus annotation. No filesystem escape, command or external dependency
  is introduced by the seed text.
- blake3 uses the required 1.8.7 caret floor; the stale exact-pin comment is gone.
  The fuzz lock includes raf-census and its production dependencies. No arrayref
  package appears in the inspected root/fuzz lockfiles; deny execution is still
  required for actual dependency qualification.
- SECURITY accurately describes the two public input paths and shared source
  parse. AGENTS/CONTRIBUTING now distinguish delivered census normalization from
  the census Gate that remains to be built. CONTRIBUTING names the actual target.
- Census normal-form now depends on source-index. Ledger capability remains
  designed; its text and change items preserve forbidden-name/pattern/rule matching,
  once-per-unit classification and re-export resolution as undelivered obligations.
  Migration's change record preserves the required index-reader size-bound work.
  These repairs do not claim the downstream features are implemented.

Advisory: make the fuzz self-check assert params == ["value"], print.is_some(),
symbol/file and nonzero source line. Add a constant initializer to a named seed or
self-check if direct protection of the initializer-fact path is wanted. These are
small useful checks, not grounds for claiming unrun production acceptance.

## Evidence and limits

The coordinator reports the fuzz target cargo check passed and docs-sync plus
assessment listing succeeded with none stale. At inspection the live
raw/queue-c4-fuzz-run.log still ended in compilation and had no terminal fuzz
result; this review cannot certify the 5000-run campaign. No builds, fuzz runs,
timing tests, mutation passes, deny/reuse, hooks or broad gates were started by
this review. Full census delivery and make check remain pending.
