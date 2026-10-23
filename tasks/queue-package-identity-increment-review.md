<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Package identity increment review

Bounded read-only review of the current uncommitted package-ID and native-selector delta, including the final replacement of the temporary directory map with a borrowed-key sorted ID vector. No build, test, or product mutation was performed. This is not dependency-foundation acceptance.

## Repaired concrete finding

`meter::tests::native_storage_capacity_failure_has_its_original_error_source` repeatedly creates `Memory::new` around an already allocated `values` vector. The native contract requires a nonempty buffer's existing layout to belong to that Memory's live total. After the first `reserve(1)`, the final `reserve(2)` starts Memory at zero despite the adapter retaining the old grant. For `u64`, admission can therefore reach only 16 bytes when the replacement requires 8 old plus 16 new bytes concurrently. This test does not correctly exercise the ownership protocol it claims to validate.

Keep one Memory across the reserves, capturing its native error before releasing the borrow to inspect the adapter; or explicitly resume its exact admitted total before each later operation. Pin the impossible-layout result to `StorageError::SizeOverflow`, and assert the replacement's real coexisting-buffer charge as an adjacent success/refusal boundary. The three new production allocation sites start with empty vectors and reserve once; I found no equivalent misuse in those callers.

Owner repair rechecked: the test now retains one Memory across all three operations and only maps the native error after that borrow ends. This closes the concrete ownership defect. Exact SizeOverflow and coexistence-usage assertions would strengthen the test, but the corrected source no longer restarts a live buffer's native account. No test execution was performed by this reviewer.

## Reviewed behavior

- `Target.package_id` is assigned from deterministic discovered-manifest order. Package metadata, editions, dependency destinations, own-library injection, and dependency-root selection now use IDs. `check_crate_paths` compares IDs, preserving duplicate paths within one package. Display names and crate/symbol paths remain unchanged.
- `resolve_in_package_id` validates membership before calling the same Index resolver. Root filtering, unresolved-self filtering, foreign-candidate filtering, and exported unbound-import queries use the actual target's package ID. The ordinary resolver has not been duplicated.
- The name selector sorts by borrowed name then ID and equal-range searches only the name, preserving all equal-name candidates. The directory selector uses the same native sort/search operations over borrowed discovered records. Current discovery supplies unique manifest directories; selecting the first equal directory is not a catalog ambiguity policy for future source authorities.
- Native `Memory::reserve` precedes `infos`, `libraries`, and selector-vector population. The original cumulative ItemBytes meter remains authoritative; adapter shrink updates live ownership without refunding cumulative usage. Budget failures retain their original typed LoadError; native layout/allocation failures use distinct `LoadErrorKind::Storage` and preserve Error::source. New table backing replaces the old metadata/library map costs; the existing LOAD_BASE header model and the added selector-header charge remain explicit.
- The target-kind normal/dev/build eligibility matrix and renamed/custom library-name choice are unchanged. Numeric crate/module roots still distinguish actual crates; displayed path ambiguity remains intentional.
- Workspace equality includes the retained name selector, and PackageId rustdoc explicitly disclaims durable version/source evidence. Cross-load IDs remain caller-scoped positional handles, like existing target IDs; an in-range ID from a different load cannot itself prove membership provenance.

I found no definite compile error in the changed call signatures and inspected direct production callers, but compilation and warning-denied qualification remain owner-run obligations. Native API observations were checked against the selected PurRDF allocation/sort interfaces; no upstream change is proposed.

## Incremental limits and required next evidence

The new integration assertions exercise two distinct package names sharing a crate name, exact ID selection, an absent ID, and deterministic manifest ordering. They do not execute the new `AmbiguousPackageName` branch: normal discovery still forbids duplicate package names. Add a bounded internal real-Workspace test with duplicate display names and distinct target IDs, or exercise it through the actual catalog once admitted; retain a unique-name valid neighbor.

An invalid `Some(PackageId)` dependency destination currently falls through `libraries.get` to an ungrounded root, just as an absent library does. Current construction obtains IDs only from the admitted discovered list, so this is not a demonstrated reachable regression. The exact catalog must distinguish an invalid ID invariant from a valid package without a library, and must not silently turn a broken locked edge into unknown extern evidence.

Catalog source/version provenance, authority-qualified physical file caches, dependency selection, trait defining-identity preservation, cfg/configuration, macro and standard-library behavior remain pending. In particular the existing displayed-path trait-key/foreign-member collapse is not fixed by changing package filters. The present changes must not be described as resolving two locked versions end to end. Original budgets, duplicate-workspace-name refusal, surface/privacy behavior, and parent acceptance remain in force.

## Invalid destination follow-through

Rechecked the subsequent `Builder::preludes` delta: eligible `Some(PackageId)` destinations now use checked lookup and return contextual `LoadErrorKind::Internal` naming the dependency, importing package, and invalid numeric identity when out of range. A valid slot containing `None` still means a known package without a library; an absent dependency destination retains its existing ungrounded-extern behavior. No new allocation/admission owner or alternate resolution path is introduced. This closes the previously recorded invalid-ID concern in the reviewed path.

The actual `Index::build` unit `an_unknown_dependency_identity_is_not_an_external_or_libraryless_package` constructs all three controls: out-of-range ID refuses with the specific Internal message, existing library ID builds successfully, and a newly admitted metadata slot without a library builds successfully. These controls exercise the production prelude builder, not just a selector helper. Their successful branches assert construction, not resolved root contents; existing alias/resolution fixtures remain the root-selection coverage. No execution was performed by this reviewer, and no new concrete defect was found in this bounded follow-through.

## Defining-trait identity increment

Reviewed `Item.trait_definition_target`, all three Item construction sites, `push_member` propagation, the `trait_key`/`walked_key` tuple, native in-place candidate sort/dedup, foreign fallback grouping, and `foreign_impl_members_keep_defining_traits_distinct_under_equal_crate_paths`. No concrete semantic or signature defect was found in this bounded source review; no build was run.

`walked_key` now requires both one displayed path and one defining TargetId before returning resolved target evidence. Multiple cfg alternatives at the same path in the same target still canonicalize together; equal paths in distinct targets become explicitly unresolved when encountered in one walk. Distinct paths in one target remain unresolved. Empty walks return no key. Text-only external and unresolved fallbacks retain `None` target evidence, so this does not falsely give them indexed provenance.

For foreign impl-member fallback, `conclude(from_keys=true)` groups by defining target plus member path/kind, rather than implementing target. This distinguishes two indexed defining traits under an equal crate path while preserving one defining trait's implementations in multiple consumer targets. Ordinary non-key resolution continues grouping by the actual item's target. The public displayed `TraitKey` stays unchanged. The additional field is covered by type-derived `ITEM_BYTES`; actual Item layout/size-bound and all caller compilation remain qualification obligations.

The new fixture provides a substantive pair: separate indexed trait targets yield two ambiguous member sites with unequal defining targets; directing both consumers to the same provider yields one resolution containing both sites, all pointing at the actual provider trait target. A missing-member neighbor remains refused. The local generic parameter addresses the foreign-type orphan boundary. Its macro-generated method declaration intentionally leaves foreign fallback observable without an indexed declaration method; this proves neither macro expansion nor compiler/provenance parity. Existing same-target cfg-alternative trait fixtures should remain passing, but the new fixture itself does not directly execute that cfg boundary. No native compiler run or owner test result is inferred here.

Remaining limitations are explicit: `None` is not grounded defining identity, and equal textual unresolved keys can retain the existing fallback grouping. Exact dependency-source identity and configuration admission must be implemented before describing all foreign trait fallback results as provider-grounded.

### Kind qualification finding

The new target evidence currently accepts every indexed Type-namespace hit. Ordinary walk lookup is not a Rust trait checker: `impl Struct for Vec<u8>` can name a Struct item, and a terminal TypeAlias/module is not automatically a defining trait. Restrict attached defining-target evidence to a nonempty selected set whose items are all `SymbolKind::Trait` and share one target. Preserve the prior canonical/unresolved textual classification independently; a same-path nontrait or mixed cfg Trait/Struct set should retain its text with `None` evidence instead of introducing a new source-index type-checking refusal.

Named import aliases/reexports preserve the actual bound definition IDs through scope lookup, so an alias to an actual Trait remains eligible. Type-alias following is explicit in nominal-type analysis and is not justification to label a terminal TypeAlias as a trait here. A syntactically indexed struct-as-trait fixture and an actual aliased-trait valid neighbor should pin the evidence boundary. This is a concrete evidence-labeling issue in the newly added field, not a claim that the earlier syntax-index contract type-checked impls. No builds or edits were performed by this reviewer.
