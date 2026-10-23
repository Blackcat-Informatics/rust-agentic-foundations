<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Locked dependency and provider identity foundation

Highest-priority adoption/enforcement work under the portfolio outcome. This
checkout now includes landed native RDF at 6cfa0b8, preserving its unique source
work through exact stash/normal fast-forward/restore. This is the sole active
implementation lane; strict/resolver donors remain frozen. No Stage panel.

The selected complete foundation must resolve an actual dependency symbol to its
exact locked package and supplied verified source, or refuse missing, stale,
ambiguous or incomplete evidence. Names/imports alone are never provider proof.
Full parent cfg/macro/std and portability requirements remain mapped and open;
this foundation must state its actual supported configuration boundary explicitly.

1. Introduce load-local package IDs and exact target/package observations without
   changing authored names/paths. Route metadata, library selection and dependency
   destinations by IDs. Preserve first-party duplicate-name refusal and existing
   target/dev eligibility. Keep defining-trait identity distinct across versions.
2. Decode the explicit Cargo.lock profile once through the existing Tree/byte and
   full TOML boundary. Use maintained SemVer, native IRI/percent/hex operations and
   admitted native storage/traversal. The inspected cargo-lock candidate adds 30
   packages, including a competing URL/Unicode stack, and fails the license gate;
   reject it without changing allowances. Its partial-source equality and lossy
   first-match graph also cannot supply exact identity. Keep Cargo-specific
   profile/edge constraints as the thin domain adapter; do not copy a generic
   TOML, URL, SemVer parser or Cargo resolver. Only unique complete locked edges
   become dependency evidence; unsupported profiles/conditions refuse explicitly.
3. Bind exact aliases/version/source/edge identities to real supplied source
   authorities under an explicit trusted-source contract. No widening the
   first-party filesystem root. Namespace physical
   paths and FileIds, preserve parse sharing and semantic contexts, and refuse
   module traversal crossing an authority boundary. Name/version agreement cannot
   establish correspondence to a Git checkout or registry archive. The production
   host adapter must check the full locked revision and relevant source state,
   including discovery-affecting additions, then read actual definitions. A hostile
   filesystem threat model, signatures, raw Git-object witnesses and a host object
   database are not required by the functionality. The reviewed object design is an
   optional stronger design, not added acceptance. Actual pinned PurRDF source
   verification and a migrated RAF consumer are mandatory before claiming production
   provider grounding; path-only fixtures are insufficient.
4. Publish exact roots to the existing resolver and add identity-qualified queries.
   Name-only ambiguous queries refuse. Native Memory/WorkList/OrderedTable and
   existing interning own reusable operations; no parallel resolver or Cargo JSON.
   Relevant unsupported cfg/features/macros produce incomplete-evidence refusal.
5. Execute actual alias/reexport/multiple-version/source/target/trait/authority
   fixtures and exact/one-less original grants, including semantic refusal/valid
   neighbors and a real locked PurRDF route. Compile production callers early.
   Reconcile current capability contracts, individually judge changed inputs,
   complete the real lifecycle and required gates/hooks before publication.

Current architecture/reference reviews: tasks/queue-package-identity-boundary.md
and refs/ghprsq/pr/54/stage:tasks/queue-provider-prerequisite-review.md and
queue-provider-native-api-audit.md. Native pin 5db00aa supplies mutable WorkList
operations; no public compatible Cargo/TOML/SemVer parser was found. Existing
Cargo parser reference: https://docs.rs/cargo-lock/11.1.0/cargo_lock/.

Source provenance/backend admission remains a concrete implementation design task,
not a completed API or waiver. Generic source catalog labels and fictional native
packages must never stand for verified production delegation. No provider rule,
CLI/self-gate or complete umbrella adoption is claimed by this source foundation.

Bounded refinement: inputs are trusted independently rooted Trees whose callers
guarantee correspondence to declared exact identities. The real checkout adapter
must verify ordinary correspondence instead of accepting a name/version label.
State this trust limit honestly; do not add a hostile-source assurance project as
a prerequisite to reuse. Carrying opaque source authority in RelPath is the smaller
reviewed direction for preserving keys and parent/join normalization. Ordinary
backends must reject foreign tags, and scoped FileIds must retain their underlying
hardlink/symlink identity. Physical authority is now implemented and incrementally
reviewed; locked catalog correspondence and the production checkout adapter remain
unimplemented. The shared loader's source-qualified roots and native source-local
name table are implemented but await caller/runtime checks.

Parser inspection found a concrete identity trap in cargo-lock 11.1.0:
package/source.rs implements SourceId ordering for dependency matching, and a
missing Git precise revision can compare equal to a present one. Do not use that
wildcard-like equality/ordering as an exact provider identity key. Retain complete
locked source/revision fields in the domain identity and require unique actual
edge selection; ambiguous partial dependency references refuse. A valid parsed
SourceId is not authenticated source content.

The same exact release has another trap: lockfile/encoding.rs
EncodableDependency::resolve returns the first name/version match for a
source-omitted abbreviation. Parsing does not establish uniqueness. Preserve raw
dependency evidence through the existing TOML decoder and require exactly one
admissible complete package match before publishing an edge. Do not inherit the
parser's first-match choice as provider proof. Inspect a maintained alternative
if its public seam can preserve this evidence more directly; do not invent a
second TOML, URL or SemVer parser.
