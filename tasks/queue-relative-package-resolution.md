<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Exact reading-package relative resolution

Implemented one prerequisite for Census provider consumers, without binding
locked packages to sources or claiming the full cfg/dependency contract.

New public methods:

```rust
Workspace::resolve_from_package_id(package: PackageId, module: &str, path: &str)
    -> Result<Resolution, ResolveError>
Workspace::resolve_from_package_id_in(
    package: PackageId, module: &str, path: &str, namespace: Namespace,
) -> Result<Resolution, ResolveError>
```

Workspace validates the exact load-local PackageId first. The existing
`Index::module_named` filters crate roots by the existing `package_of` identity
before its ordinary ambiguity/module lookup. Both new methods then use the
unchanged relative import walk, namespace order, source authority, visibility,
alias/reexport traversal and error mapping. Ordinary methods pass no package
filter and retain their previous ambiguity behavior. Package selection applies
to the reading module; it does not restrict a legitimate result to that package.

An ID absent from this Workspace returns UnknownPackageId. A module absent from
the selected package returns UnknownModule even if another package has it.
Unreachable/private definitions return ordinary NotFound; there is no path-only
fallback. IDs remain local positions, so an in-range number originating in some
other Workspace is not persistent provider identity.

Product files changed for this delta: `src/resolve.rs`, `src/workspace.rs`,
`tests/discovery_targets.rs` under raf-source, and
`docs/capabilities/source-index.md`. The capability removes its obsolete claim
that no relative-query variant can select a package, binds the two methods and
the new decisive fixture, and states the lock/configuration proof limit. Prior
dirty/untracked work and the earlier four documentation fixes remain preserved.

Actual focused receipts, using the ordinary separate managed cache:

- Workspace all-target caller compile, locked/offline: exit 0, session 8642,
  `raw/queue-relative-package-callers.{log,status}`.
- Initial all-target Source Clippy: exit 101 for the new fixture's 109-line
  function only. Extracted its repeated route assertions into a private helper,
  retaining every assertion; no waiver. Final warnings-denied Clippy: exit 0,
  `raw/queue-relative-package-clippy-fixed.{log,status}`.
- `cargo test -p raf-source --test discovery_targets --locked --offline`:
  exit 0, session 8240, all 31 tests passed, zero filtered or ignored,
  `raw/queue-relative-package-discovery.{log,status}`.
- `git diff --check`: exit 0.

The new fixture loads two packages named demo from independently rooted Trees,
with the same displayed reader/provider module paths and different definitions.
It checks both unqualified APIs still refuse ambiguity; exact IDs select each
renamed reexport and its Type/Value identity; actual defining target and source
authority match selection; foreign/private paths refuse; wrong-package module
and absent package ID refuse beside successful selected-module/package neighbors.

Census can now resolve a declared alias from an exact reading package with this
API, then inspect Resolution.sites and actual defining Item/Target identities.
It must still require the missing verified lock-row/source/manifest/PackageId
mapping and eligible declaration evidence before granting preferred-provider
status. No such mapping was added. Full original issue acceptance, broader
runtime/differential qualification, generated docs sync/assessments/lifecycle,
mandatory gates and publication remain open. No commit, push, PR or merge.
