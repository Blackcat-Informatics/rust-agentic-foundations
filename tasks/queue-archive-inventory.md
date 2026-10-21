<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census scratch archive inventory

Read-only inventory against issue HEAD `29df7f6` on 2026-10-20. No existing file
was removed or modified, no packaging was performed, and no builds were run.
Only this report was written. Paths below are relative to `scratch/` unless stated.

## Review clones

Every clone has **zero staged or unstaged tracked changes**. Its only ignored
path is a `target` symlink into the shared `/opt/.cargo/target/` cache. The exact
untracked source files are listed below; there are eight, all with distinct SHA-256
digests. These are unique evidence and must be retained as raw files before any
clone removal.

| Clone | Exact checked-out commit | Untracked source under `crates/raf-census/tests/` |
|---|---|---|
| audit | be93118c27c8c581c95761f9c2116c2c3375758f | audit_probe.rs |
| c1review | 31858ddfe93935f5d3234b6bdcc67cbb0bb6e413 | none |
| c2review | 5a3146f361e605949711b181350de7c2edb628dd | probe.rs; probe2.rs |
| gap-probe | be93118c27c8c581c95761f9c2116c2c3375758f | zz_gap_probe.rs |
| t2review | 3f8f46968ebd986fc902006c5692ac422440baab | probe.rs |
| t3review | e1073f94d5b2ea385e01458a6ba723d8f8a3865c | probe.rs |
| t4review | e02a4443b099ae4d47465c813f2786ef2035fbae | probe.rs |
| t5review | 2ec6a1e2aabbcdebf63e6817663064c0c5988f3a | none |
| t6review | be93118c27c8c581c95761f9c2116c2c3375758f | zz_scratch.rs |

All nine HEADs are ancestors of the current issue HEAD. Every commit reachable
from each clone's `--all --reflog HEAD` exists in the canonical repository's
object database. Each clone's `git fsck --full --no-reflogs --unreachable`
returned exit 0; each found 60 unreachable commits, and all 60 also exist in the
canonical object database. This is object availability, not a claim that
unreachable garbage is permanently retained by a branch.

All clone branch/remote refs also name commits available in the canonical store.
Some are deliberately **not** ancestors of the census HEAD: local main
`7976c1c03aee060a591dd3ff462c8c2aef236b56`, strict-mode snapshots
`1145af39612a7a00e93f055a0f8b63a7aaf89404`,
`fec6c212adf33da5b35a857571b34e868f0d1a1b`,
`dc1115b5ade6a884929bbb400f8df4ca2db07bc4`, and resolver
`ffd5bf28cae35724bdcacf34ed40ff8e9eb08e0b`. They remain reachable from the
canonical main, strict-mode, or resolver local branches, with remote branch refs
also covering strict-mode and resolver. Do not confuse their absence from census
ancestry with missing objects, or count these unrelated branch snapshots as
unique clone implementation work.

For a compact reconstruction record, retain the table above, each clone's exact
HEAD/refs text if historical ref labels matter, and the eight raw probes in
clone-qualified paths. A normal `git archive <checked-out-commit>` plus those
raw probes reconstructs each judged source tree without storing another `.git`
or tracked checkout. Re-run status immediately before any removal because this
inventory is a snapshot. A detached review checkout need not match its local
branch tip; the checked-out commits above are the reconstruction bases.

Probe SHA-256 identities:

```text
audit/audit_probe.rs fcef3f44c421afe4bfc66da6be5f77eb0ee1f0ec7ef6b2d96ce02c6aab770221
c2review/probe.rs fcf4da6dcf97ddbeb6bef07b83c3f7a73ad167537cde427e659d23e93f282016
c2review/probe2.rs 1e5cfaa9ce213a54ce9de106d6096c5ef6008305615b2d81e1dcd9f5876214db
gap-probe/zz_gap_probe.rs c86884f3c69bb1d1d74d7b2585997c408e7a6ea22fed9d419d5a035cbab15bd5
t2review/probe.rs b47ca6fec6169c75e671001f86d7cfb3de8d38c4c2135a8303256d43c104ad7f
t3review/probe.rs 803ca4a42948740167af7e9e97d33ceec59e1a0c786a3fd6cb6c451bac3e64a4
t4review/probe.rs f9613b3ef4356cc484890c4ffcb669e1b46086715ec54ac45c8bcb798581ec0e
t6review/zz_scratch.rs 7221252486a51ae9f4e87c2eda80b565bacc5bd893e5407a26ac5a3894538142
```

## RDF dependency comparison workspaces

The six `rdfprobe/ws*` directories each hold about 5.9 MiB. They are partial
workspace copies, not complete repository snapshots: **323 regular files** each.
The unmodified `rdfprobe/ws` matches byte-for-byte the following exact subset of
commit `be93118c27c8c581c95761f9c2116c2c3375758f`:

```text
Cargo.lock
Cargo.toml
deny.toml
rust-toolchain.toml
crates/** (all tracked files recursively)
src/** (all tracked files recursively)
tests/** (all tracked files recursively)
```

The file-name sets are identical, and every file in the base copy compared equal
to that commit's blob. This was checked against plausible later refs too;
`be93118` is the exact match, not a guessed approximate base. There are no source
edits hidden in the comparison variants. Exhaustive recursive comparisons against
`ws`, excluding only their `target` symlinks, found precisely:

| Variant | Raw files that must be retained as overlays | Observed dependency change |
|---|---|---|
| ws | none | exact historical base |
| ws-core | Cargo.lock; crates/raf-census/Cargo.toml | purrdf-core =3.0.1 |
| ws-rdf | Cargo.lock; crates/raf-census/Cargo.toml | purrdf-rdf and purrdf-core =3.0.1 |
| ws-lex | Cargo.lock; crates/raf-census/Cargo.toml | purrdf-lex and purrdf-iri =3.0.1 |
| ws-umbrella | Cargo.lock; crates/raf-census/Cargo.toml | purrdf =3.0.1 |
| ws-lexonly | Cargo.lock; crates/raf-census/Cargo.toml; deny.toml | removes serde/serde_json direct dependencies; adds purrdf-lex =3.0.1 and its Unicode-3.0 exception |

Keep each of these **11 variant files byte-for-byte**, including exact lockfiles.
The locks are evidence of the actual resolved dependency comparison, not something
to regenerate opportunistically. Reconstruction is the exact base subset above
plus the variant overlay. Neither whole-tree copies nor target links are required.

Keep `rdfprobe/base.txt`, `core.txt`, `rdf.txt`, `lex.txt`, `umbrella.txt`, and
`lexonly.txt` raw comparison outputs. Also keep all of `rdfprobe/samples/`,
including intermediate body/base/final forms and parser outputs, and
`rdfprobe/oracle/{Cargo.toml,Cargo.lock,src/main.rs}`. These are actual independent
parser programs, inputs, and outputs; the oracle's target symlink is only a cache
pointer. No inference of duplicate content justifies deleting these raw artifacts.

## Other scratch evidence and proposed packaging

Leave the remaining small material raw: `c6-fuzz/` generated corpus,
`portable-hash/` manifest/lock/source, both `selection-*.rs` measured-source copies,
`cap-before-identity.md`, and `identity-change-before-complete.md`. Existing top-level
stage logs, patches, measurements, contexts, and review reports remain in place.

The lossless compact form is therefore **historical commit/subset instructions +
eight raw clone-qualified probes + 11 raw RDF variant overlays + unchanged small
programs/inputs/logs**. It is a transparent source reconstruction, not a compressed
or encoded archive that obscures review. Before the primary authorizes removal,
copy and compare each unique file, verify reconstruction against the original
regular-file sets and bytes, and capture any desired historical ref labels. Remove
only redundant tracked clone trees and `.git` stores after that comparison; remove
target symlinks without traversing or deleting shared cache destinations. The 17
observed target links are not source evidence. This report itself performs none
of those mutations.
