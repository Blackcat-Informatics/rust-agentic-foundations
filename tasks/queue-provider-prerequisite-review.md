<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Preferred-provider prerequisites

Read-only analysis of the captured portfolio issues and current source. No
product edit, build, source checkout mutation, upstream change or fresh intake
was performed. Graphify is absent; no rebuild was attempted. Repository-memory
recall confirmed the user requirement, not an implemented provider gate.

## The portability chain is not a prerequisite

Full issues 43 and 44 do not have to precede the dependency foundation needed by
provider enforcement. Their original acceptance concerns inline parsing/host
stack strategy, removal of OS dependencies, WASM compilation and runtime/corpus
equivalence. Existing native Census/reference consumers can use raf-source's
Tree, verified DiskTree and MemoryTree while those requirements remain open.
Captured issue 48 lists resolution/cfg/macro work and related issues; it does
not declare completion of 43/44 as a prerequisite. The 43/44-then-48 ordering in
the umbrella is a scheduling choice, not a demonstrated implementation edge.

Extract a coherent locked dependency/source/symbol behavior from 48 with explicit
ownership. Do not close or silently narrow 48: declared cfg inputs/results,
path-dependency resolution, external namespace oracle including std re-exports,
Unexpanded markers, per-item unsupported handling and differential acceptance
all remain required. Neither that extraction nor host-only provider enforcement
establishes the original portability acceptance of 43/44/46.

## What actually exists and can be reused

| Current code | Reusable behavior and actual limit |
| --- | --- |
| raf-source tree/mod.rs: Tree::read, read_unseen, FileId, RelPath | Bounded, containment-aware source/manifest reads and physical identity; MemoryTree fixtures use the same seam. This does not discover locked registry/git sources by itself. |
| manifest.rs: dependency_entries/dependencies, PathSource, WorkspaceDependency | Existing TOML parse, package rename, normal/dev/build kinds and workspace inheritance. These structures are private; version/source/features are not retained, and target predicates survive only in diagnostic key text. There is no Cargo.lock reader. |
| workspace.rs: Reader, workspace_of, package_info, dependency_info | Existing manifest caches, workspace membership and inherited edition/dependency logic. dependency_info links a path only when discovery already found its directory. An absolute/above-root or undiscovered dependency becomes an external name, not an indexed package. Provider analysis must not mistake that for grounded identity. |
| index.rs: PackageInfo/DependencyInfo, ExternName::roots | Shared per-target extern-prelude and actual alias/re-export resolution. A non-workspace dependency has no roots to walk. Do not create a parallel provider path resolver. |
| discovery.rs: Walk::names and DuplicatePackage refusal | Package-name uniqueness is a current invariant. Simply adding two locked versions as more Workspace roots cannot implement distinct package identities. |
| use_resolution.rs: SHARED, a_crate_name_in_an_import_is_the_dependency_its_target_declares, a_dev_dependency_is_named_only_by_test_code_even_under_a_normal_dependencys_name | Existing real neighbors for package renames, custom lib names, inherited dependencies and target-local dev edges. Reuse these laws when extending actual dependency edges. |
| Census::load and units::collect | Already reuse one raf-source Workspace for source item spans, target contexts and normal forms. The Workspace is dropped after collection; FileCtx/package names and Unit.symbol alone cannot recover a dependency callee or source identity. Preserve/derive provider observations while the shared Workspace is alive. |
| Normalizer, Unit fingerprints and JobForbidden | Actual structural/reference-copy evidence can reuse the existing normal form. Rule fingerprints currently match no unit; Gate/rules/reference scan and policy are not implemented by adding a declaration. |

The APIs on origin/main and the new adoption checkout have these limits. The
preserved resolver checkout has newer metered/native query internals; a dependency
extension must preserve those when integrated. That is a code-integration
dependency, not a requirement to finish the unrelated instruction optimization
before a smaller separately qualified source feature can exist.

## Smallest complete dependency foundation

One independently shippable source behavior is: from a locked consumer package,
its manifest/lock input and supplied contained sources, resolve an actual direct
or re-exported provider symbol to its exact package identity and source site,
or return an actionable hard failure. It must include the following complete
boundary, rather than return a crate-name guess:

1. Read manifests and Cargo.lock through raf-source/Tree with existing byte/work
   grants. Retain exact package name, version and source identity; path packages
   also need their canonical manifest identity. Resolve declared aliases and
   normal/dev/build edges against the lock, including disambiguated versions.
   Do not implement a Cargo version solver or invoke Cargo metadata/JSON.
2. Bind each selected locked identity to the actual supplied package source.
   Missing, stale, mismatched or ambiguous source is an error. Name/version
   agreement alone does not prove a git revision or registry content. Source
   acquisition/integrity must be explicit; Tree containment is not provenance.
   For RAF's real git-pinned PurRDF, this binding is required before claiming a
   production provider identity. A contained path-dependency fixture is a
   complete first source foundation, but alone is not production PurRDF proof.
3. Give library roots distinct package identities and install the exact edges
   in the shared source resolver. Reuse module walking, named aliases, glob
   ambiguity, visibility, selected-import determinacy and typed resource errors.
   Preserve per-target/test contexts. Do not key two dependency versions solely
   by package name or expose a second independent alias walker.
4. Establish completeness for each selected route: unresolved macros, relevant
   unsupported items or unselected cfg/feature alternatives cannot silently
   become a successful provider binding. Implement the necessary selected
   dependency/configuration portion of 48, or report incomplete evidence. A
   complete, explicitly supported host-source contract is meaningful; it is
   not a claim of full cfg/macro/std resolution or all of 48.

This is new work, not a public API already available. Public identity/source
observations should be added at raf-source's existing manifest/load/resolution
boundary; Census must not independently parse Cargo metadata or duplicate that
resolver. Explicit source catalogs/host acquisition are still needed for locked
git/registry trees; no existing automatic cache lookup was found in inspected
source. External content must retain its own verified Tree authority rather
than widening the first-party root to admit arbitrary filesystem reads.

## First meaningful enforcement unit

After that foundation, the smallest grounded policy can target one real,
declared functionality and a supported adapter form. A concrete existing example
is raf_okf::profile::records::is_digest(text): its returned value is
purrdf_hash::hex::decode_32_canonical(text).is_some(). Resolve the actual callee
through the exact dependency edge, prove that the consumer input reaches that
callee and that its result determines the returned predicate. A bounded AST
adapter recognizer can establish this form without a general Rust dataflow
engine. Unsupported forms must report incomplete policy evidence, not pass or
claim algorithm duplication.

Pair it with the actual former/known competing structure or reference normal-form
equality for this functionality. An imported package, reachable call, ignored
native result or dummy native call beside a local hex recognizer must fail.
RAF-specific checks consuming the native result remain legitimate adapters.
Crate prefixes select a configured project candidate set; they prove neither
the package's provenance nor semantic delegation.

Policy ownership remains ledger identity/schema in 4; shared reference/rule/Gate
and provider checks in 5; actual CLI registration in 6/12 and RAF self-gate plus
both original flagship scenarios in 9. The callable library checker can be an
independently complete engine capability, but the umbrella's actual CLI/self-gate
requirement is not delivered until those callers run it. Preserve all fourteen
built-ins, all pattern kinds and original external-reference requirements; a
single digest obligation must not replace them or stand for complete PurRDF
adoption across all functionality/targets.

## Missing executable evidence and required neighbors

No provider foundation or policy tests were run here. At minimum its delivered
behavior needs these real invalid/valid pairs:

- Locked renamed provider plus named/glob re-export resolves to the exact source
  item; same-spelled local module/callee, stale symbol and unresolved/cyclic
  route fail. A second locked version selects its own edge; ambiguous/mismatched
  package/source fails instead of selecting the first name match.
- Existing workspace/path fixture passes, including inherited aliases and custom
  lib names; an outside/missing/symlink-swapped source refuses through Tree.
  Exact-budget success and one-less refusal retain the actual file/key context.
- The same symbol in shipping, test, example, bench and build contexts resolves
  only through eligible edges. A dev-only provider cannot satisfy shipping code.
  A physical file shared across contexts is checked in each, then deduplicated
  for reporting. Relevant cfg/feature uncertainty fails evidence completeness;
  the adjacent fully selected route passes.
- Actual input/result delegation passes; unused import, unreachable/dummy call,
  ignored result, native call on unrelated input and local-return substitution
  fail. Local shadowing does not authenticate a native symbol. Valid native-result
  validation passes. Planted copied implementation matches the reference normal
  form; unrelated neighboring domain code is not labeled an algorithm copy.
- Missing required policy/rule registration, stale consumer bindings and
  unsupported delegation forms are actionable hard failures. No debt baseline
  or sanctioned variant may suppress provider violations.
- The real locked PurRDF source and an actual migrated RAF consumer are checked,
  not just an invented fixture crate named purrdf. Production CLI and self-gate
  then fail a planted competing implementation and accept the real adapter.

These prerequisites let the highest-priority provider work advance independently
of the portability delivery while keeping every original acceptance visible.
