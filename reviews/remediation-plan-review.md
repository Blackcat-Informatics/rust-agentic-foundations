# Review: Stage 2 remediation plan, raf-census core (normal form 1), pass 4 (final recheck)

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/remediation-plan.md`, section Rev 4 (lines 119-128). Rev 4 is authoritative over Rev 3 and over `rdf-reports-design.md` where they differ.

Pass history:

| Pass | Verdict | Findings |
|---|---|---|
| 1 | NEEDS-REVISION | B1-B3 |
| 2 | PASS | advisories N1-N4 |
| 3 | NEEDS-REVISION | B4, B5; advisories P1-P7 |

This is the plan's last recheck under the three-pass limit. Anything still open is listed below as a
concrete choice.

## Coverage

All 17 gaps remain mapped. GAP-13 now closes in C6 instead of C3. R10 and R14 are carried by C6, under the
user's verbatim decision "RDF 1.2 reports (Recommended)". No new scope cut is introduced.

## Pass 3 findings

| Id | Status | Evidence |
|---|---|---|
| B4 exact `purrdf-lex` pin | **DISCHARGED** | Rev 4 line 120 changes the pin to `purrdf-lex = "3.0.1"`, a caret floor. The page states the escaping as RDF 1.2 canonical N-Triples form, and an awkward-names golden pins its exact bytes. The consumer-lockfile conflict is removed. A spelling drift fails RAF's golden instead of silently moving the output. |
| B5 reader parse surface | **DISCHARGED** | Rev 4 line 121 adds a fuzz target that reads arbitrary text with the strict reader. Any accepted text must re-render byte for byte. It is seeded with each refused construct and one valid report per kind. The same commit updates SECURITY.md's census section to the Turtle reader, CONTRIBUTING's target list and `fuzz/Cargo.lock`. |
| P1 reifier type | **DISCHARGED** | Rev 4 line 122 types the reifier `census:Match`, so the rule "every subject has exactly one `rdf:type`" now holds. |
| P2 unit IRI collision | **DISCHARGED** (choice C1 below) | Rev 4 line 123 makes the key symbol, file, line and start byte, with a one-line cfg-alternatives fixture. |
| P3 digest wording | **DISCHARGED** | Rev 4 line 124 describes the digest as an unkeyed integrity check and drops the claim of tamper-evidence. |
| P4 span map | **DISCHARGED** (choice C3 below) | Rev 4 line 125 calls `invalidate_current_thread_spans()` after collecting the units. |
| P5 JSON work in C3 | **DISCHARGED** | Rev 4 line 126 closes GAP-13 by removing `serde_json` in C6, and carries GAP-12's check to the RDF reader test. |
| P6 scratch oracle | **DISCHARGED** (choice C2 below) | Rev 4 line 127 replaces the scratch parser run with a committed test that pins the N-Triples. |
| P7 stale details | **DISCHARGED** | Rev 4 line 128: the `blake3` caret floor supersedes the design text, the Usage assessment is re-accepted, and GAP-16 uses the `max_file_bytes` bound. |

## Remaining open items (all ADVISORY; concrete choices for the implementer of C6 and C1)

### C1. The start byte must be a property of the unit, or the reader cannot re-mint the IRI

Rev 4 line 123 says "a unit IRI key is symbol, file, line and start byte". Design 2.2 says "The reader
never parses a fragment. It re-mints the fragment from the node's properties and requires equality". The
shape table in design 2.3 gives a unit node only `symbol`, `file`, `line` and, in the index, `package`,
`shipping`, `tokens`, `body` and `shim`. With only those properties, the reader cannot rebuild a key that
contains the start byte.

Choose one:
* (a) Add a `census:startByte` row (integer, exactly 1) to every `census:Unit` shape in `SHAPES`, the page
  table and `tests/page.rs`. The typed rows (`MemberRow`, `IndexUnit`, the match rows) gain the field.
* (b) Key the unit by symbol, file and line, plus an ordinal among the units that share that triple, and
  add a `census:ordinal` row in the same way.

Option (a) is the simpler one. Either way, the cfg fixture must read back through `from_turtle`.

### C2. State where the pinned N-Triples come from

Rev 4 line 127 reads: "a committed test pinning the N-Triples rendering of the example reports". If the
pinned text is produced by RAF's own reader, the test only shows that the reader agrees with itself. Design
6.2 defines the text as "the expansion an independent RDF 1.2 parser (purrdf-rdf) produced from the same
bytes".

Keep that definition. The test's doc comment should state it as a fact. The run of the scratch oracle on
the final bytes, after C1, should be recorded in validation.md. Whenever the example output changes, the
pin is regenerated from that oracle.

### C3. Give the span-map call a check

Rev 4 line 125 has no check of its own. Choose one:
* load one tree twice on one thread in a `units.rs` test, and read both loads;
* accept that every suite exercises it, because a span used after the call would panic.

A test is the stronger option and costs one function.

### C4. Fuzz target naming

Rev 4 line 121 names the target `census_report_read`. The existing targets are named in kebab case
(`parse-source`, `parse-document`, `load-workspace`, as CONTRIBUTING.md:70-72 lists them), with snake-case
file names. Use `census-report-read` as the target name and `fuzz_targets/census_report_read.rs` as the
file, so the list and `fuzz/corpus/<target>` match. The C4 target should follow the same pattern.

## Notes

* No timing runs and no mutation passes are planned anywhere in Rev 1-4.
* GAP-07 rests on the user's verbatim decision (validation.md:33-34). CLAUDE.md's wording is left for the
  user to edit.

VERDICT: PASS
