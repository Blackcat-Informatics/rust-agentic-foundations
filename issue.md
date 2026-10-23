# Issue #4: raf-census ledger: unified schema, staleness, debt ratchet, migration

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: enhancement, crate:raf-census

## Body

## Context
The ledger records which helper is the **home** for each job ("use X instead of Y") and which duplicates are sanctioned. The copies have drifted into several dialects:
- purrdf's own strict TOML subset parser;
- lillith_decodes' `disposition`/`owner`;
- katamari's `external` flag;

This issue unifies them into one strict schema and makes the ratchet native.

Capabilities delivered: `cap:census-ledger`, `cap:census-debt-ratchet`, `cap:census-migration`.

## Scope
### Ledger (`helpers-ledger.toml`)
- [ ] `toml` + serde with `deny_unknown_fields`
- [ ] `[census]`: `normal_form` (a mismatch is a hard error with migration instructions), `min_tokens` (default 20), `roots`, `reference` (used by the detectors issue)
- [ ] `[[job]]`: `id`, `summary`, `home`, `entry_points`, `enforced`, `forbidden{names (regex), constants, fingerprints, patterns}`; `[[job.variant]]`: `symbol`, `file`, `detector`, `criterion`, `anchor`, `reason`
- [ ] `[[distinct]]`: `members`, `kind` (`body`|`shim`), `criterion`, `anchor`, `reason`. Only an **exact** member-set match is sanctioned, so one new clone breaks it.
- [ ] `[job.ext]` table validated by a registered `LedgerExtension` trait. This carries the repo dialects (purrdf `spec`/`vectors`/`bench`/`replaces_external`, decodes `disposition`/`owner`) without loosening the core.
- [ ] Job matching: HOME / VARIANT / COPY classification; copy counts
- [ ] Staleness failures: a member that does not exist, a fingerprint that matches nothing, an unenforced job with zero copies, a variant whose symbol is gone

### Debt ratchet
- [ ] `census-debt.json`: `{groups:[{kind,members}], job_copies:{job:count}}`
- [ ] Fails on a new group or a rising count. **Also fails** on a disappearing group or a falling count until `debt --write`, so the recorded debt never carries slack (gmeow semantics).

### Migration
- [ ] `migrate` reads a legacy ledger plus a legacy `clone-baseline.json` and maps old fingerprint → member symbols → v1 fingerprint
- [ ] Rewrites the ledger in the unified schema, moving dialect fields into `[job.ext]`, and lists every row it could not map

## Acceptance criteria
- [ ] Fixtures for each staleness failure, each schema error and each ratchet direction
- [ ] Migration fixture built from real-shaped rows of the purrdf, katamari and lillith_decodes ledgers
- [ ] Capability docs reconciled

## Depends on
- #3

## Comments (2)

### paudley — 2026-10-20T23:05:13Z

The repository's standing goals (RDF 1.2 only, wasmable core, greenfield, no build optionality, maximal utility and performance, Rust-first with a Python surface) bear on this issue.

**Debt record format (goal: RDF 1.2 only).**
- Finding: the designed ratchet record `census-debt.json` and the migration input `clone-baseline.json` are JSON. Evidence: `docs/capabilities/census-debt-ratchet.md:7,27-40,125`, `census-migration.md:28,67,77`.
- Direction: make the debt record RDF 1.2 (canonical N-Triples 1.2 gives the byte-stable order a ratchet diff needs). Accept legacy JSON baselines only as a one-way import of foreign data. The vocabulary work is tracked in #40.

**Ledger format (goals: RDF 1.2 only, greenfield).**
- Finding: the census ledger is TOML with `deny_unknown_fields` and dialect `[job.ext]` tables; it is the authority for jobs, variants and sanctioned duplicates and is not RDF. Evidence: `docs/capabilities/census-ledger.md:41-72,143`; `crates/raf-census` on the branch.
- Direction: decide explicitly whether the ledger is an authoring format with a canonical RDF 1.2 projection (jobs, variants and distinct sets as triples, the reason as a literal, `anchor` as an IRI) or is replaced by RDF. Re-examine the `[job.ext]` dialect system, which exists to host earlier projects' fields, against the greenfield goal. The projection design is shared with #41.

**Dialects imported, not supported (goal: greenfield).**
- Finding: the ledger, ratchet and migration are designed around the dialects of seven earlier copies (`disposition`/`owner`, `external`, `spec`/`vectors`/`bench`, a TOML subset parser, the Python debt file) and ship `[job.ext]`, `LedgerExtension`, `migrate` and a read-only trial as first-order features. Evidence: `docs/capabilities/census-ledger.md:72,196`, `census-migration.md:28,67`.
- Direction: keep the core schema from bending to carry every dialect. Confine dialect fields to the `migrate` import path and one validated `[job.ext]` extension, make the unified schema the only thing the gate reads, and state in the capability document that dialects are imported, not supported.


### paudley — 2026-10-21T01:28:18Z

The Python references in her are not actionable - Python is banned from the repo at this point.

