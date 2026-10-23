<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Package identity boundary

Read-only current-source review. No builds, product edits, or new worktrees were used. Graphify is absent. This addresses the locked package/source/symbol foundation; it does not complete the parent cfg, macro, or standard-library requirements.

## Smallest coherent representation

Introduce one opaque, load-local `PackageId`, following `TargetId`, and one admitted package identity table. Its exact identity record distinguishes package name, locked version, and locked source identity (including the selected revision where applicable). A package name or name/version pair alone is insufficient. A source catalog locates that identity's verified manifest and physical source authority; it is not interchangeable with Cargo source identity. Assign IDs deterministically from admitted catalog records, never hash iteration order.

Keep `Package.name`, `Target.package`, `Item.path`, crate paths, aliases, and diagnostic spellings unchanged. Do not inject versions or IDs into authored Rust paths. The minimal internal change is a package table plus a parallel `TargetId -> PackageId` vector, with identity accessors for packages, targets, and items through their target. `Item` needs no extra owned identity or string. Package records need an explicit association with their ID so clients never infer identity from vector/name searches. IDs are not persistent keys across loads; clients needing durable evidence use the exact identity record.

Use this same ID for package metadata and dependency destinations. Keep configuration/test eligibility on target/dependency edges rather than pretending an exact Cargo package identity alone determines every build context. Unsupported or ambiguous locked source selection must fail before an extern root is published; an unresolved destination must not be advertised as grounded dependency evidence.

## Concrete touched seams

| Current seam | Required change |
| --- | --- |
| `discovery.rs::Walk.names` and `Walk::manifest` reject duplicate names; `found` is manifest-path keyed | Preserve the first-party workspace duplicate-name refusal and same-manifest deduplication. Admit dependency packages by exact catalog identity separately. Do not remove the workspace validation globally to admit two locked versions. |
| `workspace.rs::load_with`: directory map yields names; `infos` is `BTreeMap<String, PackageInfo>` | Resolve admitted manifest/source locations to IDs; key metadata by ID. A directory is meaningful only in its source authority. Preserve workspace-inherited dependency manifest lookup. |
| `workspace.rs::dependency_info` and `index.rs::DependencyInfo.package: Option<String>` | Record the exact destination ID selected by the locked dependency edge. Preserve declared alias, rename flag, dependency kind, and source location. Present `None` means no discovered grounded destination, not proof of absence. |
| `index.rs::edition_of`, `Index.editions`, `Builder::preludes` | Key editions and library-root selection by ID. The current `libraries.insert(&target.package, ...)` overwrites a same-name library; replace it with exact ID selection. Preserve per-target edition and the existing normal/dev/build eligibility matrix. |
| `target.rs::check_crate_paths` | Compare package identity plus crate path, not package-name strings. Within-package duplicate targets still refuse; equal crate paths in different package identities are allowed. Carry identity through derivation rather than deducing it from display names. |
| `resolve.rs::package_of`, `resolve_in`, `through_unresolved`; `exports.rs::Surface::unbound_imports` | Internal package selection must use the actual target's ID. Add exact identity-qualified entry points for consumers. Existing `resolve_in_package(name, path)` retains unique-name behavior and must explicitly refuse an ambiguous name with actionable identities, rather than select or merge versions silently. |
| `Workspace` equality and package observation APIs | Include new identity/source/edge state in equality. Keep current public string fields as display data and document the distinction. |

`ExternName.roots` already stores actual numeric module roots and eligibility, and `target_roots`, root-scoped `module_by_path`, `(TargetId, module_path)` file contexts, and `Item.target` already separate crates. Reuse them. `crate_roots` intentionally maps one displayed crate path to several roots; unqualified canonical lookup must remain ambiguous when appropriate. Dependency alias lookup selects the destination root before the same ordinary resolver walks members. No second resolver is needed.

Preserve current custom library-name behavior: an unrenamed dependency currently uses the selected library crate name, while a renamed dependency uses its declared alias. Preserve own-library injection for non-library/non-build targets and the existing distinction between development roots available everywhere and only in test code.

## Less visible identity collisions

`impls.rs::walked_key` maps resolved trait items to displayed paths and deduplicates the strings. Two versions defining the same `api::Trait` can therefore collapse even after extern roots are fixed. `resolve.rs::conclude(..., from_keys=true)` deliberately omits the implementing target when grouping foreign trait members. Simply retaining the implementing target would break the existing grouping of one defining trait implemented by several consumers.

Retain the resolved defining-trait symbol identity internally alongside the public displayed `TraitKey`: a defining target plus canonical symbol group, including namespace/kind as needed for existing cfg alternatives. Key foreign-member fallback by that defining identity and member. Unresolved textual keys remain explicitly unresolved. This is required before claiming exact identity for all supported dependency symbols; displayed trait paths cannot serve as provider proof.

Public `SourceFile.package` is currently the first target's package display name; all semantic contexts are in `reached`. Public `TestFn.package` is documented for Cargo `-p`. Preserve unique-workspace behavior, but do not emit an ambiguous runnable package selector if dependency tests are exposed. An exact selector contract needs explicit support or actionable refusal, not guessed selector syntax.

## Physical source and grants

Current `modtree` caches physical files by `RelPath` and uses `FileId` for physical cycle detection, with separate target/module/test contexts. Two source authorities can both contain `src/lib.rs` and can generate equal memory-tree file keys. Namespace physical cache keys by authority, or supply one explicitly catalog-scoped tree that namespaces both paths and file identities. Retain parse deduplication for the same physical source and distinct semantic reach contexts. Do not widen the workspace filesystem containment root merely to read a cache directory.

New identity records, target relations, package maps, source names, and catalog edges must be admitted before retention through the original typed item/storage grants, with real work charged and failures propagated. Update type-based costs when replacing string keys with IDs; preserve limits and contextual file/origin errors. Do not hide lookup/storage failure as an unknown package or unresolved trait. The already-audited native APIs are the implementation selection reference; this report does not propose another parser, Cargo resolver, or version algorithm.

## Required fixtures

1. Two exact locked versions with the same package and library name, reached under distinct declared aliases: resolve distinct definitions and reexports through actual use paths; prove an unrelated or wrong version cannot satisfy a missing member. Retain both namespaces and target identity.
2. Same name/version with distinct locked sources remains distinct; duplicate exact identity deduplicates. Existing duplicate first-party package names still refuse, with a valid distinct-name neighbor.
3. Name-only package selection refuses ambiguity; identity-qualified selection succeeds for both versions; unknown identity refuses. Unqualified equal crate paths remain ambiguous.
4. Different consumers select different versions under the same alias. Preserve renamed/custom library names, own library selection, build dependencies, integration-target dev dependencies, and dev-only visibility in ordinary versus test code.
5. A trait with the same displayed path in two versions produces distinct defining identities and foreign-member results. The same defining trait implemented in two consumers still groups as intended. Preserve existing unresolved/multiple-trait evidence rather than canonicalizing strings prematurely.
6. Two authorities with equal relative paths/file keys remain separate; shared actual source reached by multiple targets still shares physical parsing and retains target-local roles. Verify imports, exported surface, privacy, and cycles through the real shared walker.
7. Exact and one-less original grants for admitted catalog/identity/edge storage, including typed failure location and a successful neighboring package. Preserve deterministic ordering, existing item-size bounds, and current unique-workspace outputs.

Existing anchors to extend include `two_libraries_with_one_crate_name_are_ambiguous_until_the_package_is_named`, `a_crate_name_in_an_import_is_the_dependency_its_target_declares`, `a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name`, `trait_impls_on_foreign_types_are_keyed_by_the_canonical_trait_path_in_every_spelling`, and the shared-file/test-function symbol-index fixtures. These are source inspection anchors, not execution claims.
