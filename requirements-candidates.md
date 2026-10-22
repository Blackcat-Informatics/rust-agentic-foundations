# Requirement candidates — issue #16

Lines from the issue body and comments that matched a lexical rule, copied
verbatim with the rule that fired. This is a CANDIDATE list: rules both
over-match (a "should" in passing) and under-match (a requirement stated
without a keyword). The issue-analyzer keeps or drops each row, adds what
the rules missed, and accounts for every silent comment. Code fences are
skipped.

| ID | Source | Line | Rules | Text |
|---|---|---|---|---|
| B1 | body | 7 | checkbox | - [ ] A `strict` setting. Library: `Gate::strict()` or a `Strictness` option. CLI: a `--strict` flag plus a repo-level setting (for example a field in a config file next to `docs/capabilities/index.md`, or `Cargo.toml` metadata). Propose one location and justify it in the capability document. |
| B2 | body | 8 | checkbox | - [ ] Every `delivered` capability binds at least one runnable example or doc test and has a generated verification-matrix block. |
| B3 | body | 9 | checkbox | - [ ] Coverage: every Contract and Constraints claim is covered by a bound test or example. Define the claim unit (a bullet, a bold-lead paragraph, or an explicit claim marker) and the mapping from matrix rows to claims. Report unsupported claims as findings with fix text. |
| B4 | body | 10 | checkbox | - [ ] Every public item of a bound crate path is explained by some capability (orphan-symbol finding), reusing the two-way coverage direction. |
| B5 | body | 11 | checkbox | - [ ] `change complete` additionally requires `verify` to pass for ALL bound items of the listed capabilities and zero tense-mismatch findings on those capabilities, and forbids `contract_effect: none` for changes that touch a bound public item. |
| B6 | body | 12 | checkbox | - [ ] Minimum content rules per section for delivered capabilities, reported as findings: non-trivial length, a runnable Usage example, a Limitations statement. |
| B7 | body | 13 | checkbox | - [ ] No `dropped` or `moved` item outcome without a stated reason field. |
| B8 | body | 14 | checkbox | - [ ] Stricter obligations: `section-untouched` also applies to Usage and Examples for `contract_effect: none`. |
| B9 | body | 15 | checkbox | - [ ] A baseline/ratchet option so adopting strict on an existing repository can start from today's findings and fail only on new ones. Coordinate with the census debt-ratchet design (#4) instead of inventing a second ledger. |
| B10 | body | 16 | checkbox | - [ ] Reverse coverage mode (`cargo workplan coverage --reverse`, also enforced under strict with a threshold): report the source lines of a crate that map to no deliverable or specification, for example "crate x has 100 lines that no capability binding or contract explains". A line is covered when it lies inside the span of an item that a capability binds (code, test or example) or inside an item reachable from one through the source index; everything else is an orphan line. Report per crate, per module and per item with counts and the list of orphan items and line ranges, sorted for stable output, as a text table and as JSON. |
| B11 | body | 17 | checkbox | - [ ] Reverse coverage gate: a configurable ceiling (maximum orphan lines or percentage per crate), a way to mark deliberate exceptions with a reason (generated code, test support, vendored code), and the baseline/ratchet option above so the count can only go down. |
| B12 | body | 18 | checkbox | - [ ] Reverse coverage fixtures: a crate with a fully bound public surface reports zero orphan lines; an unbound private helper reports exactly its line range; a bound item covers its nested items and impl blocks; macro-generated items and `#[cfg(test)]` modules are handled and documented; the report is identical across runs and platforms. |
| B13 | body | 21 | checkbox | - [ ] Each strict rule has a fixture that triggers it and a passing neighbour. |
| B14 | body | 22 | checkbox | - [ ] Default (non-strict) behaviour and every existing fixture are unchanged. |
| B15 | body | 23 | checkbox | - [ ] Strict findings carry the same `Finding {kind, file, location, message, fix}` shape, with the new kinds added to the closed `KINDS` list and covered by the both-directions coverage test. |
| B16 | body | 24 | checkbox | - [ ] CLI integration tests cover `--strict` exit codes 0, 1 and 2. |
| B17 | body | 25 | checkbox | - [ ] Documented in the capability documents as a new designed capability (for example `cap:workplan-strict`), following the capability-document workflow in AGENTS.md. |
| B18 | body | 26 | checkbox | - [ ] Once the rules pass, the repository enables strict on its own docs. |
| B19 | body | 30 | modal | - Overlaps with #10: the two-way coverage and evidence-claims items there cover code-to-docs and tests-to-claims, which rules 3 and 4 above enforce under strict. Scope should be reconciled between the two so neither duplicates the other. |

## Comments that matched no rule

None.
