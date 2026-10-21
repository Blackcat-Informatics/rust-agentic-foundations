# C6 queue implementation review

Verdict: one medium-priority diagnostic contract finding remains; no additional production-wiring, identity, reference, or failure-propagation blocker found in the inspected current implementation. Review is read-only apart from this artifact. No builds or tests were started by the reviewer.

## Finding

R1 — `crates/raf-census/src/report/read.rs`, `finish`: locate noncanonical input using byte-preserving lines, rather than `str::lines()`.

The reader compares `body != wanted` strictly, then uses `body.lines().zip(wanted.lines())` to locate the first difference. `str::lines()` removes CR from CRLF. Converting a valid canonical fixture to CRLF therefore preserves its parsed graph and passes shape checking, but the difference locator sees every line as equal and reports the document line count plus one. The actual first differing source line is line 1. Removing the final required LF similarly reports the following line rather than the last source line. `ReportError::NotCanonical` explicitly promises the first differing source line.

Suggested verification: the existing canonical fixture succeeds; its CRLF spelling is `NotCanonical { line: 1 }`; removing its final LF identifies its final source line. Preserve line contents and terminators, or determine the first differing byte and count source newlines up to that byte. Keep the existing BASE-line mapping when translating back to original source lines. This is source-inspection evidence, not a newly executed reproducer.

## Inspected acceptance

- Actual public producers are `Census::baseline`, fallible `Census::index`, and the shipped baseline example through `tests/support/builtin::render`. Both render through `to_turtle`; rendering failures propagate to the example's nonzero exit. No serde/serde_json/JSON render path remains in raf-census.
- Unit start-byte identity is propagated from source facts into groups, job matches, reports and minted names. Percent encoding separates symbol/job delimiters and filenames; same-line alternatives retain distinct identities. Group/job references must resolve to index units; repeated rows/jobs fail rather than overwrite silently.
- Reader annotation recursion is bounded to one level. Syntax, shape, version, namespace, minted-name, reference, copy-count, orphan, canonical-spelling and content-digest refusals return typed errors. Writer validation uses the same shapes before rendering, and its graph construction propagates contradictions/dangling references.
- Report normal-form declarations are preserved for migration, while report-form layout is gated. The reader-version test covers an alternate normal form and the public normal-form gate refuses it afterward.
- Existing report acceptance logs show 24 executed integration tests, including classified jobs, large u128 constants, awkward names, same-line alternatives, a malformed caller-built report and disk/relative-path determinism. Independent RDF parser pins are present and checked. These are reviewed supplied results, not new reviewer executions or a full qualification claim.
- The report-reader fuzz target is wired into the actual Makefile fuzz loop, with valid baseline/index self-checks before fuzz input, both reader paths and accepted-byte round trips. The supplied live log ends with 5000 completed runs. Size is delegated explicitly to file-reading callers; the string reader's memory remains proportional to input-derived graph/string storage and no stronger bound is claimed.

Final reader/render/diagnostic edits were settling during review. R1 must be adjudicated against the final reader and its focused test output. Parent owns final full make-check qualification; the supplied focused results do not replace it.

## R1 source/evidence recheck

PASS: current finish() compares split_inclusive newline segments, preserving CRLF and a missing terminal LF during difference location; its fallback uses the same segment definition and retains the BASE-line mapping. The new canonical_line_diagnostics_preserve_crlf_and_the_final_newline test has the valid canonical neighbor, CRLF line-1 refusal, and final-LF refusal at the actual final source line. raw/queue-c6-canonical-lines.log shows that named test executed and passed (one passed, zero failed); supplied clippy output is clean. No reviewer build was run.

The final reader also renders the validated original graph before comparing canonical bytes, avoiding reconstruction loss while preserving shape validation and decoded report output. R1 is resolved; this review has no remaining blocker. Final full qualification remains the parent's responsibility and is not inferred from these focused results.
