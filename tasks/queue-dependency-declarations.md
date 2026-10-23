<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Dependency declaration prerequisite

Subsequent declaration-qualified outgoing selection is now implemented; see
`queue-declaration-edge-selection.md` for that checkpoint. The grounding list
below records this declaration-retention checkpoint's original limits; owning
package/source verification and active configuration remain unresolved.

Implemented in the existing manifest parser, with no new dependencies or alternate
manifest path. Public observations are available through
`Workspace::dependency_declarations(PackageId) -> Option<&[DependencyDeclaration]>`.
Unknown package IDs return `None`. Ordinary discovery and resolution still use
the existing package/path dependency interpretation.

Public types, reexported by `raf_source`:

- `DependencyKind::{Normal, Dev, Build}`.
- `DependencyDeclaration { key, kind, name, condition, specification, inherited }`.
  `condition` retains the written target triple or `cfg(...)` table key.
- `DependencySpec { package, version, path, registry, registry_index, git, rev,
  branch, tag, optional, default_features, features, workspace }`. String fields
  retain their spelling; flags preserve omission versus explicit false. Features
  preserve their written order and duplicates.
- `InheritedDependency { manifest, specification }` retains the exact declaring
  workspace manifest, including its source authority, independently of the local
  optional/default-feature/features declaration. This preserves the workspace
  path base without pretending to have computed an effective configuration.

Both ordinary and workspace entries use one dependency specification decoder over
the existing TOML implementation. The existing `semver::VersionReq` validates
requirements. The pinned PurRDF lex public API has allocation, scan and text
facilities but no semantic-version parser; no competing version parser was added.
Malformed types, malformed requirements, Git selector conflicts, source conflicts
and invalid inheritance overrides fail through the existing manifest error path.
Retained outer/inner vectors use native `Memory` and `ItemAdmission`; retained
strings and feature buffers are charged to the original ItemBytes allowance
before their clones are constructed. This does not introduce a bound on the
existing TOML parser's temporary allocations.

Edited production files are `manifest.rs`, `workspace.rs`, `lib.rs`, the new
focused `tests/dependency_declarations.rs`, and the source-index capability's
bindings/current observation contract. All prior dirty work remains intact.

Actual focused evidence:

- `cargo check --workspace --all-targets --locked --offline`: exit 0, session
  20920, `raw/queue-dependency-declarations-callers.{log,status}`. This compiled
  affected real callers early.
- `cargo test -p raf-source --test dependency_declarations --locked --offline`:
  exit 0, session 57586, all four tests passed, none ignored or filtered;
  `raw/queue-dependency-declarations-test-final.{log,status}`. Cases cover all
  retained fields and target kinds, workspace path inheritance, malformed and
  valid declarations, and exact ItemBytes / one-byte-short refusal.
- `cargo clippy -p raf-source --all-targets --locked --offline -- -D warnings`:
  exit 0, session 59017,
  `raw/queue-dependency-declarations-clippy-final.{log,status}`. A preceding lint
  found only load_from length; reusing its identical Reader removed the repetition
  without a waiver. Earlier failed test receipts retain the fixture construction
  corrections (MemoryTree takes borrowed text; inherited paths need real manifests).
- `cargo fmt --all` and `git diff --check`: exit 0. No full suites, measurements,
  full gate, commits or publication were performed.

The explicit lock-to-source-to-PackageId join remains unimplemented. Required
grounding includes the owning package's version/source identity, outgoing lock
edge selection against these declarations, explicit Cargo registry/configuration
and feature/target eligibility, independently supplied source-tree correspondence
and relevant revision/state verification, and installation of those verified
edges into the existing resolver's extern prelude. A declaration, a matching
crate name, a local lock row without a source, or a source authority alone is not
that proof. No PurRDF definition/source binding is claimed. Original issue 48 cfg,
macro/unsupported-marker, external and differential acceptance remains open.
