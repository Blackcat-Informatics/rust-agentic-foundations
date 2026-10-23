<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Declaration-qualified outgoing lock selection

Implemented `LockedCatalog::select_dependency(owner: LockedPackageId,
declaration: &DependencyDeclaration, registry: Option<RegistryIndex<'_>>,
options: &LoadOptions) -> Result<LockedPackageId, LoadError>`.

`RegistryIndex::new(name: Option<&str>, source: &str)` supplies an explicit observed
registry-name mapping, with `None` naming the default registry. Its source must
include the `registry+` or `sparse+` family. Missing default/custom configuration
and mismatching names refuse; no crates.io identity or custom index is guessed.
A directly written `registry-index` supplies its explicit index instead and
conflicting configuration refuses.

Selection searches only the owning row's already uniquely resolved outgoing
edges. It applies the package rename, maintained semantic version requirement,
source family, exact lexical repository and Git reference kind/value. A Git match
also requires the catalog's complete selected revision. A requested revision
expression remains distinct from that full selected commit; branches and tags
are not interchangeable. Repository spellings are not heuristically normalized.
Path declarations can select local lock rows, but this alone proves neither the
path's contents nor its correspondence to the row. A second compatible row
refuses rather than taking the first one; unrelated global catalog rows cannot
satisfy an owner's missing edge.

Workspace inheritance uses the separately retained workspace package/version/
source constraints and rejects missing or inconsistent inherited entries. Local
features, optional flags and unevaluated target conditions do not decide
activation. The observed owning row is an explicit caller input: this method does
not prove that a particular Workspace package or declaration belongs to that row.

The original manifest validator is shared rather than duplicated. Existing native
LockedSource parsing supplies IRI validation, percent-decoded Git references and
source framing. The existing maintained semver dependency supplies requirements
and versions; no new dependency or competing parser was added. Native temporary
storage is admitted on top of the retained catalog's ItemBytes usage under the
supplied allowance. Selection work includes input text and every outgoing
comparison's text lengths. Native storage and typed budget failures propagate
through the existing LoadError paths. As before, maintained SemVer internals have
no native admission callback; this is not a claim to have replaced those internals.

Changed files for this step: new `src/locked/select.rs`, `src/locked/mod.rs`,
`src/manifest.rs` (shared validator extraction), `src/lib.rs` (RegistryIndex
reexport), new `tests/locked_selection.rs`, and source-index capability bindings
and its precise observation contract. Existing dirty/stash work is preserved.

Actual terminal evidence, under this exact stage's `raw` directory:

- Caller compile: `cargo check --workspace --all-targets --locked --offline`,
  session 79228, exit 0; `queue-declaration-edge-callers-fixed.{log,status}`.
  The initial compile's explicit Option annotation finding remains in the earlier
  receipt and was corrected without a fallback.
- Affected shared validation and selector: `cargo test -p raf-source --test
  locked_selection --test dependency_declarations --locked --offline`, session
  11516, exit 0; all four declaration cases and the initial five selection cases
  passed. `queue-declaration-edge-focused-fixed.{log,status}`. The preceding
  fixture failure was an ambiguous omitted-source local lock edge; the fixture
  now gives that local row its own unambiguous version and a real path manifest.
- Final selector: `cargo test -p raf-source --test locked_selection --locked
  --offline`, session 71100, exit 0; all six tests passed, none ignored or
  filtered. `queue-declaration-edge-selection-final.{log,status}`. Cases cover
  both row orders, same-name versions and registry sources/families, unrelated
  global rows, package renames, decoded Git branch/tag/revision/default references,
  local-family selection, inherited/unevaluated declarations, missing/wrong/
  malformed configuration, malformed observations, unknown owner and work/item
  refusal with valid neighbors.
- Final affected lint: `cargo clippy -p raf-source --all-targets --locked
  --offline -- -D warnings`, same sequential session 71100, exit 0;
  `queue-declaration-edge-clippy-final.{log,status}`.
- Formatting and `git diff --check`: exit 0. No full suites, measurements,
  full gate, commit, push or publication were performed.

This closes declaration-qualified outgoing selection only. The explicit owning
Workspace package-to-lock identity, independently verified source Tree-to-lock
row correspondence, precise row-to-loaded-PackageId association, active cfg/
feature policy, and installation of verified edges into the existing resolver
remain required. This selection does not ground PurRDF usage or definition
ownership. Original issue 48 cfg, macro/unsupported-marker, external namespace and
expanded differential acceptance remains open.
