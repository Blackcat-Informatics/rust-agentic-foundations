# G-R2 review: census docs remediation

## Scope
Unstaged R2 diff: docs/capabilities/census-{cli,debt-ratchet,detectors,ledger,migration,patterns,reference-scan,rule-api}.md and work/changes/census-{cli,debt-ratchet,detectors,ledger,migration,reference-scan,rule-api}.md. Read-only review.

## Evidence
- R2 Check commands re-run: job.ext (census-cli:52,141) and "exit 2" present; write_debt 16 lines across rule-api/debt-ratchet/cli; "same function" appears once, qualified (cli:113); migration Usage names `cargo test --test census` and `cargo xtask census-debt`; ledger line 81 "unknown rule: id" hard error; depends: ledger = normal-form+source-index, cli includes census-rule-api, rule-api = detectors, ledger, patterns, debt-ratchet, reference-scan; no `--locked` in reference-scan; no `"1"` in rule-api (now "0.1", matches README); normal-form has no raf-source/source index text, so no edge there (correct).
- `cargo test --test docs_contract --locked`: 46 passed (graph acyclic). `make issue-refs`: rc 0. `uvx reuse@6.2.0 lint`: compliant. No R2 text references the CLI-runs-repo-rules enhancement, an issue number or process wording. No new hedging (remaining "may/could" are pre-existing, descriptive). No contradiction with README (raf-census 0.1) or AGENTS.md (xtask permitted).
- G3 consistency: exit 2 plus message naming Gate::write_debt/assert_clean (cli, ledger, migration fixture); Gate::write_debt refuses on non-debt findings (rule-api, debt-ratchet); never from `#[test]` (rule-api, debt-ratchet; cli says binary/xtask); unknown `rule:` id is a hard ledger error in CLI and Gate (ledger, rule-api, cli, with Verification bullets in all three). Change-record items match docs (cli, debt-ratchet, ledger, migration, reference-scan, rule-api, detectors).
- G13a, b, d, e, f and G15a delivered (--frozen plus bullet; migrate --ledger has no default; patterns/ledger staleness all say fail; "every key except [job.ext]"; empty positive set refused with four-test bullet; raf-census = "0.1").

## Adjudication 1: G13c shipping-only set (WRONG)
Source: purrdf/crates/helper-census/src/{rules,layout,structure,source}.rs. There:
- Shipping-only (skip #[test]/#[cfg(test)] items and tests/benches/examples dirs): hex-digit-radix, json-pointer-escape (lex_rules, rules.rs:972-990), and the five layout rules length-prefix-frame, le-slice-int, align-up-mask, div-ceil-by-hand, xor-first-mismatch (layout.rs header "Test code is out of scope"). vocabulary-literal and home-literal are documented "shipping source" (rules.rs:42,49).
- Read everything, tests included: std-default-hasher (rules.rs:5 "tests, benches and examples included"; structure.rs:13 "test modules included"), hex-format-loop, hex-pair-radix, hex-table (hex_rules, no test filter).
The doc marks std-default-hasher, hex-format-loop and hex-pair-radix as shipping and json-pointer-escape and the five layout detectors as `all`: nearly inverted. Only hex-digit-radix is right. The Verification bullet and change-record claim repeat the error.

## Adjudication 2: G13g normal_form bump path (ACCEPTED)
Consistent with census-normal-form (a mismatch fails hard and names `migrate`; the check keeps failing hard until migration and the write run; no fallback or auto-upgrade). Replacing the record is necessary because group identity changes with fingerprints, so growth cannot be compared across versions. The "cannot admit growth" claim is correctly narrowed to "into an existing record", with the creation/bump exception stated and review of creation and `normal_form` diffs named as the control (debt-ratchet Rationale and Usage), and a Verification bullet. Advisory only: census-migration describes legacy-to-new migration, not an N to N+1 bump; "the ledger is migrated" could say which command.

## Findings
Required:
1. census-detectors.md table, prose, Verification bullet, and work/changes/census-detectors.md claim: the shipping-only set contradicts the source the census is ported from. Correct set (of the 14): shipping = hex-digit-radix, json-pointer-escape, length-prefix-frame, le-slice-int, align-up-mask, div-ceil-by-hand, xor-first-mismatch, home-literal, vocabulary-literal (sole-name: verify in source.rs units, which are shipping-only); all = std-default-hasher, hex-format-loop, hex-pair-radix, hex-table. Update the prose list ("The four shipping detectors ..."), the bullet and the claim accordingly. If the owner wants a different set, it must be stated as a deliberate change from the source, but no source supports the current set.
Advisory:
2. census-debt-ratchet.md and census-reference-scan.md have newly added over-long lines (single-line sentences); rewrap for consistency.
3. census-migration could name the command used at a normal_form bump (see above).

VERDICT: BLOCKED

## Recheck
- G13c corrected: the table, prose list, Verification bullet and change-record claim now agree on 9 shipping (hex-digit-radix, json-pointer-escape, length-prefix-frame, le-slice-int, align-up-mask, div-ceil-by-hand, xor-first-mismatch, home-literal, vocabulary-literal) and 5 all (std-default-hasher, hex-format-loop, hex-pair-radix, hex-table, sole-name). This matches the purrdf source I cited in the first review (lex_rules, layout.rs, rules.rs headers).
- sole-name=all holds: census.rs `matches` iterates every `workspace.units` entry (including non-shipping units, which carry a `shipping` flag) with no filter, and `second_home` (census.rs:449-466) has none; by contrast the vocabulary-literal (census.rs:361) and home-literal (census.rs:420) loops both skip `!unit.shipping`. The choice is source-backed.
- Advisories done: bump commands named in census-migration (`cargo census migrate`, then `cargo xtask census-debt`); lines rewrapped.
- Re-run: docs_contract passes (graph acyclic), `make issue-refs` rc 0, reuse lint compliant. No untracked or staged files introduced.

VERDICT: PASS
