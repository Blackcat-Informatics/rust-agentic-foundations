VERDICT: PASS

# T1 review (commit cb08b0c, base origin/main)

## Reviewed scope
Crate scaffold (Cargo.toml workspace member, crates/raf-census/Cargo.toml, lib/error/options/version),
Cargo.lock, six PROPOSED blocks in docs/capabilities/census-normal-form.md, docs/capabilities/index.md,
tests/docs_contract.rs (INVENTORY and counts, package set), work/changes/census-normal-form.md and
census-ledger.md, status prose in AGENTS.md, README.md, CONTRIBUTING.md. Judged against plan Rev 2 over Rev 1 over the original.

## Commands
- `cargo test -p raf-census --locked`: exit 0 (8 unit tests, 1 doctest).
- `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: exit 0 (68 + 1 + repo_gates all passed).
- `cargo run --locked -q -p cargo-workplan -- --root . check`: exit 0, no output.
No timing runs, no mutation passes.

## Checklist
- Crate and deps: workspace member added; raf-source (path), syn (same version/features as raf-source), proc-macro2 with
  `span-locations` and default features off, `blake3 = "=1.8.7"` (Cargo.lock holds 1.8.7), serde, serde_json. Lints inherited. SPDX headers present.
- Error (Source, Options, Collision, Version, Load with `source()` and `From<LoadError>`), CensusOptions (`#[non_exhaustive]`,
  Default 20 / 8 MiB, `min_tokens` refuses 0, `max_file_bytes` setter), NORMAL_FORM = 1, check_normal_form: present.
- Refusal tests have valid neighbours: min_tokens 0 refused / 1 accepted; versions 0 and 2 refused with both versions and
  `cargo census migrate` / 1 accepted; Load is the only variant with a source. Fail hard holds: no fallback anywhere.
- Change records: census-normal-form has (a) golden-pins item and (b) the version-gate item, the job-matching item moved in,
  the combined old item reworded away; census-ledger lost the job-matching item and keeps "A normal_form mismatch fails before any job is read". Shim and units-by-source claims reworded per Rev 1.
- PROPOSED blocks exist in Purpose, Contract, Usage, Examples, Constraints, Verification; Contract carries job matching, version
  gate, shim predicate and key, shim body-print rule, report members and ordering, all consistent with Rev 1/Rev 2.
- INVENTORY `implementing`, counts 8/1/12, package set gains raf-census, index.md regenerated (docs-sync clean; docs_contract passes).
- Status prose in AGENTS.md, README.md and CONTRIBUTING.md is true (CONTRIBUTING needs no change; the census gate genuinely does not exist).
- No tracker numbers or process wording found in added lines (repo_gates issue-refs scan passes).
- Outside PROPOSED blocks the page's text is unchanged except state; the PROPOSED blocks describe designed behaviour, which is their purpose.

## Findings
None blocking and none required. Observations only, no action needed:
1. crates/raf-census/Cargo.toml:15-26: cargo prints three unused-manifest-dependency warnings (blake3, serde, serde_json). Not rustc
   warnings, do not fail clippy; they disappear when T2/T5 use them. Expected per the plan.
2. docs/capabilities/census-normal-form.md:60-61: the original "6 to 29 tokens also has a shim form" bullet still sits beside the
   PROPOSED shim-predicate bullet; the PROPOSED block supersedes it at completion. Expected for the PROPOSED mechanism.
