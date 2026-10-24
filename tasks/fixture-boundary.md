<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Actual Cargo fixture dependency boundary

Current ordinary Source loading accepts a declared path dependency that is
absolute or lexically above its tree root as an external, nonmember dependency.
It does not try to open that external source or fail containment merely because
the dependency declaration exists. This is explicit policy, not a fallback:
workspace.rs::dependency_info (1328-1335) maps AboveRoot/Absolute to no local
package, members.rs (327-332) excludes such paths from inferred member closure,
and dependency_declarations retains the original written specification.

Existing source test
trait_keys::a_path_dependency_with_an_unusable_path_is_an_error_beside_one_that_is_outside_the_tree
has positive controls for ../../up and /absolute/elsewhere and negative controls
for unusable path forms. The neighboring
a_path_dependency_outside_the_workspace_directory_or_excluded_is_not_a_member
also establishes the explicit membership boundary. This differs from an actual
in-tree symlink/module/read that escapes the root, which remains a hard refusal.

Smallest correct actual target fixture: use one real temporary package with
explicit name/version/edition, its own [workspace], actual source/example and
ledger, and absolute path dependencies on the current checkout's raf-census,
raf-source and raf-rdf as needed. Do not edit/sanitize the manifest for Gate.
Gate::workspace(".").check() then reads that exact root; the support libraries
remain external to its ordinary consumer Source inventory. The example's Cargo
build still uses the actual current libraries and their real dependency graph.
No workspace.exclude entry is necessary for physically outside dependencies.

Use native testkit TempDir as in cargo-census/tests/delivery.rs; prefer its
target-root convention for the new fixture. Prepare a real dependency-complete
Cargo.lock with offline Cargo resolution before the assertion phase, fail if
required cached dependencies are absent, then invoke the shipped check through
the unchanged frozen example invocation. Do not fabricate a truncated lock or
regenerate it during registered check. Snapshot source, manifest, lock and debt
around the asserted read-only command, and invoke from a different caller cwd
with explicit fixture workspace root.

This ordinary loading policy does not create locked/provider authority for
external libraries. The later native witness still needs real explicitly
supplied LockedCatalog/LockedManifest/configured sources and the pinned native
identity; an external dependency spelling alone proves none of those facts.
Static inspection only: no builds, Cargo resolution, source edits or test runs.
