<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Locked catalog adapter

Read-only design against the current manifest/load seams, existing identity reviews, plan, and cached cargo-lock 11.1.0 source. No build, dependency change, product edit, upstream change, or source verification was performed. The selected PurRDF APIs remain the existing audit's authority for native storage; no new native inventory is needed.

## Maintained parser boundary

Use `cargo-lock = { version = "=11.1.0", default-features = false }` for its public in-memory `str::parse::<cargo_lock::Lockfile>()`; do not call filesystem `Lockfile::load`, optional dependency-tree/petgraph, or Cargo metadata JSON. Its inspected manifest declares Rust 1.85, MIT OR Apache-2.0, TOML 1, serde, semver, and url; exact lock/licence/advisory/MSRV qualification remains required. TOML 1 already reads RAF manifests. Use maintained semver `Version`/`VersionReq` for versions and requirements, never string ordering or a new requirement evaluator. `cargo_lock::Version` is reexported; `VersionReq` requires an explicit compatible semver dependency.

Keep the original Tree read and total-byte boundary. Decode a small structural raw lock view with the existing TOML decoder before the cargo-lock call: top-level version and each package's name/version/source/checksum, raw dependency strings, and replacement evidence. This is field extraction, not a copied lock parser. Keep raw row ordinal/origin to pair complete package records with the maintained parsed result, and validate agreement rather than matching rows through `Package`/`SourceId` Eq.

Before invoking cargo-lock, guard every checksum location it will decode: a checksum must be an ASCII hexadecimal string of exactly 64 bytes. The cached `Checksum::from_str` checks byte length then slices pairs with string byte indices; a 64-byte non-ASCII value can panic. Prefer native `purrdf_hash::hex::decode_to_slice` into `[u8; 32]` for the guard/retained checksum, or ASCII+length guard followed by maintained checksum parsing. Do not use catch_unwind as input validation. If supporting legacy metadata checksums, guard those too; otherwise refuse that format explicitly before invoking its decoder. The smallest current Cargo profile is explicit version 3/4, with older formats reported as unsupported rather than partly interpreted.

## Exact data and matching

The minimal domain records are:

```text
LockedPackage { id: PackageId, name: Name, version: Version,
                source: ExactSource, checksum: Option<[u8;32]>, origin }
ExactSource = Local { catalog_location }
            | Cargo { kind: SourceKind, canonical_url: String,
                      precise: Option<String> }
LockedEdge { from: PackageId, to: PackageId, origin }
DeclaredEdge { from: PackageId, alias, actual_name, requirement,
               source_constraint, kind, configuration, origin }
SourceBinding { package: PackageId, authority, manifest, verification }
```

For Cargo sources, obtain `kind()`, `url()` and `precise()` from the maintained `SourceId` parser. Retain the `SourceKind::Git(GitReference)` value, including reference kind/value, and the precise revision separately. Compare the complete tuple with normal exact option equality; never use `SourceId` Eq/Ord/Hash or `Dependency::matches` for identity. Git package records require a full precise revision suitable for the supported host adapter. Registry/sparse kinds remain distinct. Absent lock source is not a universal source identity: bind a local package to its actual catalog manifest location. Keep PackageId load-local; the tuple is durable evidence, not the numeric ID.

The public API does not expose abbreviated `EncodableDependency`; do not copy that private resolver. A thin raw-reference envelope adapter can recognize one name token, optional version token, and an optional parenthesized source token, rejecting extra/malformed framing, then delegate each token to public `Name`, `Version`, and `SourceId` parsers. Use whitespace tokenization and safe prefix/suffix stripping; no URL/SemVer implementation or byte-index slicing. Preserve which qualifiers were actually present. cargo-lock remains the whole-lock format validator; this adapter exists only to prevent its lossy abbreviation resolution becoming identity proof. Canonical names/versions and diagnostic rendering use the maintained Display implementations, not custom encoders. Test encoded Git reference round trips in version 4: public SourceId Display uses the older unencoded reference spelling, so do not require raw byte equality with that renderer or normalize by rewriting lock input.

For each raw dependency reference, query the catalog by maintained name; apply an explicitly present exact Version, then an explicitly present exact source tuple. Require exactly one candidate. Zero is stale/missing evidence; more than one is ambiguous, including same-name/same-version packages from different sources and name-only references to multiple versions. An omitted source does not mean local/path and does not authorize first-match selection. Cross-check the maintained parsed edge against the chosen complete identity; never accept the parser's selected candidate as the uniqueness test. Duplicate complete package records and replacements/patches need explicit handling: refuse an unsupported replacement path rather than silently publishing the pre-replacement target. Unused patch records do not establish active dependency edges.

## Manifest edges and existing resolver

Current `Dependency`/`WorkspaceDependency` retain alias, renamed package and path, but discard version requirements, Git/registry constraints, optional/features/default-features, and target-table conditions. Preserve those fields through the existing TOML parser before grounding dependencies. Preserve workspace inheritance and its actual base directory. The lock edge lists package names, not Rust aliases: join a declaration only against the importing exact package's locked outgoing edges, filtered by its actual package name (`package` or declaration alias), maintained requirement matching, and declared source constraint. A package existing elsewhere in the lockfile is not an edge. Path dependencies additionally bind the exact supplied local manifest; do not choose a remote same-name/version package.

Several declarations may legitimately select one locked package, under distinct aliases or kinds. Publish each alias with the existing renamed/custom-library-name behavior and normal/dev/build target eligibility matrix. Two versions under two aliases select distinct PackageIds before ordinary member resolution. No alias should be reconstructed from lock edge text.

Configuration is separate from locked package selection. The existing manifest walker unions target dependency tables, which is not proof that an edge is active. Preserve conditions/features and either validate a supported explicit configuration or return incomplete-evidence for a relevant unresolved conditional/optional/feature/macro route. A lockfile is not a feature-resolution witness. Keep raw syntax-index behavior distinct from the new grounded-provider result; do not silently claim all current unioned candidates are active. Full parent cfg, macro and standard-library acceptance remains open.

After matching, feed exact destination IDs and actual library roots into current `PackageInfo`/`DependencyInfo`/prelude construction and the shared resolver. Preserve defining-trait identity, target/test contexts, visibility, ambiguity and typed Budget/Storage failures. There is no second symbol resolver or new Git object database.

## Source and storage ownership

Sources are trusted independent Trees with explicit package-to-authority bindings. The ordinary checkout adapter must actually verify the locked full revision and relevant checkout state, including additions visible to package/module discovery, before reading real definitions; repeat the relevant verification around the load as required by the existing containment model. A caller-supplied name/version label is insufficient. Use the existing host command/error seam for ordinary Git observations, not a new Git protocol/subsystem. No signature, archive attestation, hostile-host or cryptographic-content assurance is added. Registry content correspondence needs its own real supported binding; do not pretend the lock checksum authenticates an arbitrary unpacked Tree.

Carry authority-qualified paths and physical FileIds as described by the package-boundary review. Refuse traversal across authorities. Keep same physical source parse sharing distinct from target semantic contexts. A missing binding, wrong revision, dirty/discovery-affecting source mismatch, invalid ID, absent library needed by the route, or unsupported relevant configuration must produce actionable typed evidence failure.

Native Memory reserves catalog rows, indexes, edge/reference buffers and transient matching scratch under the original cumulative ItemBytes owner before population; use native WorkList/OrderedTable only for compatible actual operations. Admit key text and source tuples, not only numeric slots. Count actual scans/comparisons through the existing work owner. External TOML/cargo-lock/semver/url parsing allocates internally and exposes no RAF admission callback: retain the existing byte/parse boundary honestly, and do not claim those parser internals have native physical pre-admission. Verify bounded retained conversion and avoid claiming a refund from cumulative ItemBytes when temporary buffers die.

## Smallest complete qualification packet

- Real lock version 3/4 parsing; malformed types/version/source; 64-byte Unicode checksum refusal beside valid ASCII checksum; no panic or lossy parse recovery.
- Name-only and name+version abbreviations: unique succeeds; multiple versions/sources refuse regardless of package row order; explicit source selects the right revision; missing versus present precise never compares equal. Encoded Git references remain supported through maintained parsing.
- Correct alias/reexport and same-package multiple versions; missing outgoing lock edge, stale requirement, wrong source, workspace inheritance, normal/dev/build neighbors; inactive/unknown relevant cfg/features explicitly incomplete.
- Actual trusted source Trees and ordinary checkout adapter: right full revision succeeds, wrong revision and discovery-visible changes refuse, equal relative files across authorities remain separate. A real pinned PurRDF dependency resolves an actual definition; a migrated production consumer uses that result.
- Exact/one-less original retained catalog/scratch grants and typed errors; shared resolver privacy/ambiguity/trait tests remain unchanged. Parser tests or fictional package labels alone do not complete provider grounding.

The foundation remains planned until those actual production paths, fixtures, documentation, and required gates execute. This design neither weakens the parent acceptance nor expands it into cryptographic attestation.
