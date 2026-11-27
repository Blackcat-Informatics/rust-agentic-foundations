<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Use Census in a Rust repository

Census groups Rust helpers with the same normalized body or forwarding pattern. Use a baseline to see what already matches, a ledger to declare the repository's policy, and a check in CI to catch new copies. Start with structural duplication. Add named jobs when you need to enforce a particular implementation as the home of a helper.

## Inspect the workspace

On native Linux with Rust 1.98.1 or newer, install Census from crates.io. Use the same exact RAF version for the command and the dev-dependency on developer machines and in CI.

```sh
RAF_VERSION=0.9.0
cargo install --locked --version "=$RAF_VERSION" cargo-raf-census
```

From the consumer workspace, run the self-test and inspect its baseline.

```sh
cargo raf-census self-test
cargo raf-census baseline --format turtle
```

Make sure Cargo's install directory (normally `$HOME/.cargo/bin`) is on `PATH`. Run the baseline in the consumer workspace, or select it with `cargo raf-census --workspace /path/to/workspace baseline`. Baseline discovery needs a Cargo manifest and readable sources. It prints an RDF 1.2 report using the default Census options and makes no policy decision. To save a report, add `--out baseline.ttl`. The self-test checks the installed detectors against seeded cases independently of your workspace.

## Capture the production lint result

Use the Cargo lint tables below for a standalone package. In a workspace, put them under `[workspace.lints.rust]` and `[workspace.lints.clippy]` in the root manifest. Each member inherits them with `workspace = true` under `[lints]`.

```toml
[lints.rust]
unused = { level = "deny", priority = -1 }
dead_code = "deny"
unreachable_pub = "deny"
unused_crate_dependencies = "warn"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
used_underscore_binding = "deny"
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
```

If the workspace has no lockfile yet, run `cargo generate-lockfile` from its root. Run the production check from the trusted build or hook with a fresh receipt path outside the inspected repository.

```sh
evidence_dir=$(mktemp -d)
export RAF_CENSUS_LINT_EVIDENCE="$evidence_dir/production.ttl"
cargo raf-census lint-capture --trust-lints --out "$RAF_CENSUS_LINT_EVIDENCE"
```

The command runs Clippy on the locked workspace libraries and binaries with
the required denied lints and `--target host-tuple`. The explicit host
selection takes precedence over Cargo's `build.target` setting.
`--trust-lints` authorizes Cargo to run build scripts and procedural macros
with your permissions for that invocation. It provides no sandbox. A private
helper called only from a test can be dead in that build. A passing
`--all-targets` result cannot satisfy the check. A dev-dependency used only
in tests can remain in `[dev-dependencies]`.

Use a fixed checkout and a trusted compiler and build environment. The command
calls the installed `rustc` and `cargo-clippy` executables directly. The receipt
covers member manifests, the lockfile, workspace compiler configuration,
Clippy configuration from each member up to the workspace root, shipping
modules and directly named literal file inclusions. Standalone tests, examples
and benches do not supply source roots for the production pass.

Place literal file inclusions in the consuming source. Census refuses a file-inclusion name in a declarative macro definition or one forwarded as a macro argument because the expansion can resolve the path at another call site. The scan skips directly `#[cfg(test)]`-guarded items and statements where it can prove they are inactive. Inputs to an unknown attribute macro still count because the macro can rewrite a nested declaration. Census also reports first-party build scripts, unverified target feature requirements, renamed or dynamic inclusions, and local configuration that changes the compiler invocation. Keep dependency sources and toolchain fixed when producing and consuming a receipt.

Keep the receipt in the build-owned directory. The CLI accepts its path through the exported variable or `check --lint-evidence /absolute/path/to/production.ttl`. Missing evidence, changed inputs, a weakened manifest or a failed compiler run fails the default check. Repeat the capture with a fresh destination after changing the sources or policy. Keep the same Cargo target directory, compiler and effective production flags to reuse the warm Clippy artifacts while capturing a fresh receipt. Cargo reports the compiler run time in its diagnostics. Use `check --no-production-lints` for an explicitly structural check. Baseline inspection and debt recording need no compiler receipt.

On the RAF 0.8.1 workspace, one direct invocation of the built command with
warm host-target artifacts on x86-64 Linux and compiler revision
`5df8c02af` took about 17 seconds to publish a fresh receipt. Cargo reported
3.44 seconds for its Clippy step. The full capture includes both source-input
checks and the receipt write. It excludes building the CLI and fetching
archives.

## Install the ledger

From the workspace or one of its members, initialize the structural policy, inspect the resulting ledger, then check the source.

```sh
cargo raf-census init
cargo raf-census dump-ledger --format turtle
cargo raf-census check
```

`init` creates canonical `helpers-ledger.ttl` at the workspace root. Its sole scan root is `.` and it declares no jobs or allowances. An existing ledger path makes initialization fail and preserves the existing entry. Initialization leaves any debt record untouched.

`dump-ledger` validates and prints the ledger. `check` evaluates it against the source and debt record. An absent `census-debt.ttl` counts as empty. A clean workspace passes immediately. A finding includes the group or policy violation to inspect. For `check`, exit status 0 means clean. Status 1 signals findings or an evaluation error. Status 2 signals invalid command usage or a ledger that requires Rust registrations the CLI lacks.

The ledger reader requires canonical bytes. Even a whitespace edit can invalidate the file, and a policy edit changes the content-derived `BASE`. For a custom policy, construct the graph with `raf_rdf::Builder` and serialize it with `Graph::to_text(Format::Turtle)`. The example constructs a job, names its home and attaches an extension using that writer. [Ledger construction example](crates/raf-census/examples/support/ledger.rs) · [Ledger fields](docs/capabilities/census-ledger.md)

## Adopt existing duplication

Inspect the reported occurrences together. Consolidate copies that should share one implementation. When you deliberately accept some existing duplication as the starting point, publish and review the exact record.

```sh
cargo raf-census debt --write
cargo raf-census check
git add helpers-ledger.ttl census-debt.ttl
```

Commit the ledger and any adopted debt. In CI, capture the production Clippy result, then run the read-only `cargo raf-census check`. A new duplicate group or a count above its record fails. Removing a copy can also produce a finding because the record now grants more than the code uses. Rerun `debt --write` to lower it, review the diff and run `check` again. The writer refuses growth against an existing record of the same normal form. Hard policy errors need a code or policy repair before debt can be written. [Debt ratchet](docs/capabilities/census-debt-ratchet.md)

To exercise structural matching, use this `src/lib.rs` in a scratch library with the starter ledger and an empty debt record. Run `cargo raf-census check --no-production-lints`. It finds the two bodies in one group. Rename the locals and the match persists. Remove the second helper and the structural check passes. For a full check, capture fresh production evidence after each source edit.

```rust
pub fn scaled_total(samples: &[u64]) -> u64 {
    let mut sum = 0;
    for sample in samples {
        sum += sample * 3;
    }
    sum
}

pub fn batch_total(values: &[u64]) -> u64 {
    let mut total = 0;
    for value in values {
        total += value * 3;
    }
    total
}
```

## Find the source of a finding

| Question | Command |
| --- | --- |
| Which sources, groups and classifications did the gate see? | `cargo raf-census index` |
| Which sites match a pinned Git reference? | `cargo raf-census reference-matches --reference-repo ../upstream --reference-commit COMMIT` |
| Which occurrences and fingerprints belong to a symbol? | `cargo raf-census fingerprint demo::scaled_total` (use your crate and item name) |
| Which occurrences matched a declared job? | `cargo raf-census job JOB_ID` |
| How many copies did that job classify? | `cargo raf-census count JOB_ID` |

Use `--max-index-bytes` to cap the selected job, count, fingerprint, or
pinned match report in its chosen format. The complete index has its own
N-Triples size check.

The starter ledger declares no job IDs. For policy beyond structural grouping, a job can identify a home symbol and forbidden competitors. A reference scan can compare local code with sources sealed from the Cargo lockfile and cached registry archives. Supply consumer-owned Rust rules and extension validators through the library gate when the policy requires them. The native CLI never launches the consumer to discover registrations. [Rule registration](docs/capabilities/census-rule-api.md) · [Reference scans](docs/capabilities/census-reference-scan.md)

## Run the gate from a Rust test

To use the same structural policy from your test suite, add the library at the version selected above. In a virtual workspace, run this command in the member that owns the integration test. Census discovers the workspace root from the member directory.

```sh
cargo add --dev "raf-census@=$RAF_VERSION"
```

```rust
// tests/census.rs
fn checked_gate() -> Result<raf_census::Gate, Box<dyn std::error::Error>> {
    let gate = raf_census::Gate::workspace(env!("CARGO_MANIFEST_DIR"));
    let receipt = std::env::var_os("RAF_CENSUS_LINT_EVIDENCE")
        .ok_or("capture production Clippy before running this test")?;
    let run = raf_census::ProductionLintRun::read_external(
        &gate.root()?,
        std::path::Path::new(&receipt),
    )?;
    Ok(gate.with_production_lint_run(run))
}

#[test]
fn duplicate_helpers_match_the_record() -> Result<(), Box<dyn std::error::Error>> {
    checked_gate()?.assert_clean();
    Ok(())
}
```

Resolve the new dependency, commit the updated lockfile and capture fresh production Clippy evidence. Run `cargo test --locked --test census` from the owning package with the receipt in the environment. For a policy with custom rules or required providers, share the gate configuration across the enforcing entry points. RAF's own test shows a locked registry check over the workspace and its separate fuzz root. [RAF's Census gate](tests/self_census.rs) · [CLI reference](docs/capabilities/census-cli.md)

## Prove a preferred provider

A job that requires delegation names one preferred provider. Put the package and the APIs
the consumer must delegate to in that row. Only the named supplier satisfies
the proof. For example, a digest job can name `purrdf-hash` with
`purrdf_hash::hex::encode` as the home and the three APIs used by its
local adapter below. Set the consumer path to `demo::digest_hex`.
[Preferred provider contract](docs/capabilities/census-ledger.md)

```rust
pub fn digest_hex(bytes: &[u8]) -> String {
    let digest = purrdf_hash::blake3::hash(bytes);
    purrdf_hash::hex::encode(purrdf_hash::blake3::Hash::as_bytes(&digest))
}
```

For `purrdf-hash = "=5"` under `[dependencies]`, prepare a
[build-owned registry archive directory](docs/capabilities/census-reference-scan.md#registry-dependencies-sealed-by-the-lock-file).
Set `CENSUS_ARCHIVES` to its `registry/cache` directory. In `tests/census.rs`,
keep `checked_gate` and replace the structural test with this one.
`seal` admits the package for provider proof. Declare its complete unified
features, including defaults and what they imply. This package has none.

```rust
#[test]
fn registry_provider_policy_holds() -> Result<(), Box<dyn std::error::Error>> {
    use raf_census::{ArchiveCache, LockedRegistry};

    let archives = std::env::var_os("CENSUS_ARCHIVES")
        .ok_or("set CENSUS_ARCHIVES to the build-owned registry/cache")?;
    let registry = LockedRegistry::new(ArchiveCache::at(archives))
        .seal("purrdf-hash")
        .features("purrdf-hash", std::iter::empty::<&str>())
        .require_declared_features();
    let report = checked_gate()?.check_registry(&registry)?;
    assert!(report.is_clean(), "{}", report.findings().join("\n"));
    Ok(())
}
```

For a pinned Git supplier, add `raf-source` at the same RAF version to
`[dev-dependencies]`. In the example below, the consumer is `demo` at `1.0.0`
with a normal dependency named `provider` on the Git package `kit` at
`1.0.0`. Declare its actual URL and full revision in `Cargo.toml` and lock it.
The job names package `kit` and the actual library API `kit::encode`.
Its consumer calls that function through the alias `provider::encode`.
Set `PROVIDER_URL` and `PROVIDER_REVISION` to the URL and full commit in the
lock row and `PROVIDER_OBJECTS` to an existing local repository holding the
commit. The test reads its committed objects.

```rust
#[test]
fn git_provider_policy_holds() -> Result<(), Box<dyn std::error::Error>> {
    use raf_source::{
        CfgAtom, CommittedGitTree, DeclaredCfg, DeclaredConfiguration, DiskTree,
        LoadOptions, LockedCatalog, LockedEdge, LockedManifest, LockedPackageRoots,
        LockedSourceKind, LockedTree, RelPath, TargetKind,
    };
    use std::path::Path;

    let gate = checked_gate()?;
    let tree = DiskTree::new(gate.root()?)?;
    let options = LoadOptions::default();
    let manifest = RelPath::new("Cargo.toml")?;
    let catalog = LockedCatalog::load(&tree, &RelPath::new("Cargo.lock")?, &options)?;
    let app = catalog.packages()
        .find(|row| row.name() == "demo" && row.version() == "1.0.0" && row.source().is_none())
        .ok_or("missing local demo row")?;
    let url = std::env::var("PROVIDER_URL")?;
    let revision = std::env::var("PROVIDER_REVISION")?;
    let mut suppliers = catalog.packages().filter(|row| {
        row.name() == "kit" && row.version() == "1.0.0" && row.source().is_some_and(|source| {
            source.kind() == LockedSourceKind::Git
                && source.repository() == url
                && source.revision() == Some(revision.as_str())
        })
    });
    let kit = suppliers.next().ok_or("missing exact Git provider row")?;
    assert!(suppliers.next().is_none(), "ambiguous Git provider row");
    let objects = std::env::var_os("PROVIDER_OBJECTS").ok_or("set PROVIDER_OBJECTS")?;
    let git = CommittedGitTree::open(&catalog, kit.id(), Path::new(&objects), &options)?;
    let atoms = [
        CfgAtom::Name("unix"),
        CfgAtom::Value("target_arch", "x86_64"),
        CfgAtom::Value("target_os", "linux"),
        CfgAtom::Value("target_family", "unix"),
        CfgAtom::Value("target_endian", "little"),
        CfgAtom::Value("target_pointer_width", "64"),
    ];
    let library = |name: &'static str| DeclaredConfiguration::new(
        DeclaredCfg::complete("x86_64-unknown-linux-gnu", &atoms),
        &[], TargetKind::Lib, name,
    );
    let edges = [LockedEdge::new("dependencies.provider", None)];
    let local = [
        LockedManifest::new(app.id(), manifest.clone(), &edges)
            .with_configuration(library("demo")),
    ];
    let provider = [
        LockedManifest::new(kit.id(), manifest, &[]).with_configuration(library("kit")),
    ];
    let sources = [LockedPackageRoots::new(LockedTree::Git(&git), &provider)];
    let report = gate.check_locked(&tree, &catalog, &local, &sources)?;
    assert!(report.is_clean(), "{}", report.findings().join("\n"));
    Ok(())
}
```

The Git example checks two libraries with no features or extra cfg
requirements on native x86-64 Linux. Use the manifest's actual library names,
source-relative manifest paths, feature sets and target cfg for a different
workspace. Pass the entire selected context list to `with_configurations`
when the policy covers several targets. The gate verifies the lock row and
the consuming manifest edge and never fetches the commit.

## Share jobs across repositories

A shared Rust dev-dependency or canonical Turtle file can supply one or more
jobs and their patterns. For example, put the PurRDF digest job above in one
shared job set. Include its provider and any other owned nodes, and use
distinct fragments such as `#purrdf/digest` and `#purrdf/digest-provider`.
Keep scan roots, reference selectors and scope in the consumer's base ledger.

Use the native RDF builder to serialize these assertions as a canonical job
set. A shared Rust authoring function can take the consumer symbol as an input.

```turtle
@prefix raf: <https://blackcatinformatics.ca/raf/report/1#> .
<> a raf:Ledger ; raf:normalForm 1 ; raf:reportForm 6 ;
   raf:job <#purrdf/digest> .
<#purrdf/digest> a raf:LedgerJob ; raf:jobId "purrdf-digest" ;
   raf:summary "Digest and hexadecimal encoding come from PurRDF" ;
   raf:homeSymbol "purrdf_hash::hex::encode" ; raf:enforced true ;
   raf:forbiddenFingerprint "rule:hex-table", "rule:std-default-hasher" ;
   raf:preferredProvider <#purrdf/digest-provider> .
<#purrdf/digest-provider> a raf:PreferredProvider ;
   raf:package "purrdf-hash" ; raf:consumer "demo::digest_hex" ;
   raf:entryPoint "purrdf_hash::blake3::hash",
                  "purrdf_hash::blake3::Hash::as_bytes",
                  "purrdf_hash::hex::encode" .
```

```rust
use raf_census::Ledger;

fn combined_ledger(base: &str, shared: &str) -> Result<String, raf_census::Error> {
    let base = Ledger::from_turtle(base)?;
    let shared = Ledger::from_turtle(shared)?;
    base.compose_jobs(&[&shared])
}
```

Review the canonical output and install it as `helpers-ledger.ttl`. Apply the
same provider checks to the combined ledger. Gate evaluation reads only that
file.

## Use the gate in PurRDF

PurRDF can take the registry RAF release as a dev-dependency. Cargo identifies
the workspace and registry copies of PurRDF as separate packages, even where
the name and version agree. Keep app dependencies on workspace members and
use the package and source in the lock row when a proof names a provider.
[Cargo package identities](https://doc.rust-lang.org/cargo/reference/pkgid-spec.html)

With `multiple-versions = "deny"`, cargo-deny's default
`multiple-versions-include-dev = false` excludes copies reached only through
the RAF dev-dependency from the duplicate check. That route needs zero skips.
If the policy includes dev dependencies, use the actual locked graph to
identify reported duplicate packages and add a `bans.skip` entry for each
version to exclude. For example, `purrdf-hash@=5.0.0` selects by name and
version and matches both copies when both have that version. Keep the advisory,
license and source checks over the complete graph.
[cargo-deny bans configuration](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html)
