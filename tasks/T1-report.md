# T1 report

Status: DONE.

Files: Cargo.toml, Cargo.lock, crates/raf-census/{Cargo.toml,src/{lib,error,options,version}.rs},
work/changes/census-normal-form.md, work/changes/census-ledger.md,
docs/capabilities/census-normal-form.md (state implementing, six PROPOSED blocks),
docs/capabilities/index.md (docs-sync), tests/docs_contract.rs (INVENTORY, state counts 8/1,
package set gains raf-census), AGENTS.md, README.md.

Checks (exit): cargo test -p raf-census --locked 0 (8 unit + 1 doctest); docs_contract/self_workplan/repo_gates 0;
workplan check 0 (run as `cargo run -p cargo-workplan -- --root . check`); make fmt clippy doc deny reuse issue-refs all 0.

Decisions: Error has Source, Options, Collision, Version, Load(raf_source::LoadError). Setter min_tokens returns Result;
max_file_bytes is infallible. syn is a direct dependency with raf-source's version and features. Bindings are left empty
(allowed in the implementing state; T6 binds). Cargo prints three unused-dependency manifest warnings (blake3, serde,
serde_json) until later tasks use them; they are not rustc warnings and do not fail clippy. CONTRIBUTING.md needed no change
(its status line is still true). Job-matching item moved into the normal-form record; shim claim and units-by-source claim reworded.
