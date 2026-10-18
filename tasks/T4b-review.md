# T4b review (rustc differential test: order check, cycle direction 2, corpus)

VERDICT: PASS

Scope: tests/rustc_oracle.rs, tests/support/rustc.rs, tests/support/program.rs and the three new corpus files. Nothing tracked was modified; scratch under /tmp/t4b.

## Gates
* cargo fmt --all --check: clean.
* cargo clippy -p raf-source --locked --all-targets -- -D warnings: clean.
* cargo test -p raf-source --locked --test rustc_oracle (debug): 5 passed, 1 ignored (write_corpus), including the corpus replay with the three new entries. No RAF_ORACLE_PROGRAMS run, no whole-crate run.

## (a) Per-space verdict
`space_verdicts` writes `type __t = <path>;` and `fn __v() { let _ = <path>; }` at the end of the reading module (render_by_space), finds both lines by exact text, and reads errors whose primary-span line equals that line. No error on the line = Resolves; E0659 = Ambiguous; any other error (E0573/E0412/E0423/E0425/E0433/E0432/E0603, or an error with no code) = Unresolved. A diagnostic on another line is filtered out by the line equality, so it cannot be attributed to the probe. Correct. Observed with rustc 1.101.0-nightly: a type probe of a fn gives E0573, which is covered by the catch-all. Limitation (non-blocking): an error elsewhere that makes rustc skip a probe-line diagnostic would read as Resolves; I found none in the corpus programs.

## (b) Shuffles
`shuffled` parses the program into nodes (lines and modules), Fisher-Yates shuffles each sibling list, and recurses into every module, so root declarations and in-module declarations (including module blocks relative to `use` lines) are permuted. Probe lines are emitted after the shuffle at the end of their module, so they stay put. Seeded: `Draw::seeded(0x0de2_0001)`; the as-written order is noted first, then 12 shuffles. Comparison is of each disagreeing space against the as-written answer. It detects variance of the verdict only, not of which item is named (under-detects, never over-detects). Correct.

## (c) classify ordering
Non-forward: documented is forward-only; then order, then undeterminable cycle, then not-visible gives unclassified, then rule1/rule2. Forward (AcceptedButRefused): documented, cycle (needs a Cyclic answer, no ambiguity, other space agreeing), order, unclassified.
* N1 (non-blocking, doc precision): the module doc says order is asked "right after `documented`" for every kind; in the forward direction `cycle` is tested before `order`. Harmless (both are non-failing) but the sentence is slightly off.
* The trade-off for applying order to permissive divergences is stated honestly in the module doc ("A rule bug on a probe that `rustc` answers order-dependently cannot be told from the order dependence"). It is limited to the disagreeing spaces, which keeps it narrow.
* `when_visible()` is now evaluated eagerly before the order check (cost only).

## (d) UNDETERMINABLE and the graph check
`code_of` maps a code-less error whose combined text starts with the message to UNDETERMINABLE; the message is the first text pushed by `texts`, so this is the diagnostic's own message (verified: message is exactly "cannot determine resolution for the import", no code, no children). It is a prefix match, not equality (nit).
`import_cycle_reaches` does what the code doc says: edges from `use` declarations (module to the module the path reads, followed only as far as modules go), from the probe, and parent to child; true if a module reachable from the probe's module (or the probe's module itself) reaches itself. N2 (non-blocking, doc honesty): because of the parent-to-child edges, any nested module that globs or names an ancestor (`use super::*`, `use crate::*`) is a cycle, which is very common in generated programs, so the check is close to "the program has an upward import". The program.rs doc says "over-approximates"; the module doc in rustc_oracle.rs says only "an import graph with a cycle among the modules the probe's module reaches". Since rustc only emits this diagnostic in a stuck import fixpoint, the real filter is the diagnostic; saying so would be more honest.

## (e) Corpus
* import_cycle_rustc_cannot_determine_beside_a_glob_that_resolves_it (`cycle`): with the probe as `use crate::s0::B as probe0;` rustc gives "cannot determine resolution for the import" on the probe line; type-space split probe gives E0573, the value probe resolves. Header right.
* import_cycle_rustc_cannot_determine_through_a_named_import_and_two_globs (`cycle`): probe in demo::m1::s1 gives the same diagnostic on the probe line. Header right.
* glob_order_decides_whether_a_type_a_glob_cycle_carries_is_ambiguous (`order`): 60 random orders (python reimplementation of the shuffle, probe in m1): (type, value) = (resolves, resolves) in 38 and (E0573 not found, resolves) in 22. So the type-space verdict really varies and the value space does not. N3 (non-blocking, naming): rustc never gave E0659 for rustc here; the file name says "is ambiguous", which describes the index's answer, not rustc's. A name such as "...whether_a_type_a_glob_cycle_carries_is_found" would match the evidence. The replay test accepts the entry as `order`.

## (f) Rules
Added lines and the three corpus files grep clean for Instant, sleep, timeout, Duration, elapsed, issue or PR numbers, milestone, revision, task, review, stage. No attribution. Comments match the code (checked the Class::Cycle, Facts, rustc_depends_on_order, space_verdicts and import_cycle_reaches docs). SPDX not needed in the corpus (REUSE.toml). The doc line "These divergences ... the corpus records each known shape" has no open-ended "until".

## Findings
No blocking finding. Non-blocking: N1 (doc: cycle precedes order in the forward direction), N2 (doc: graph check is loose; the diagnostic is the real filter), N3 (rename the third corpus file; rustc's type verdict varies between found and not found, never ambiguous).
