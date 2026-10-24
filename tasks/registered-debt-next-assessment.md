<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Registered debt next-unit assessment

Read-only source assessment; issue body and both current comments fetched with
stagectl issue 12 and captured below. No issue/branch/worktree creation, product
edits, compilers or process lifecycle actions. Parent scope remains open.

## Smallest complete useful unit

Deliver explicitly selected registered debt --write through the existing bounded
Cargo example runner, a fixed write request, a distinct native RDF write-response
protocol, and a real locked Gate writer sharing the existing exact debt policy.
Do not deliver a transport-only writer that calls the structural Gate on a
preferred-provider ledger. Check-v1 remains unchanged and grants no write.

The latest issue comment explicitly requires write authorization and actual
locked writing. The older body names census-debt.json and TOML extension syntax;
current authoritative APIs use native census-debt.ttl and RDF Extension nodes.
Do not restore the superseded carrier while satisfying the remaining behavior.

## Exact execution/request seam

- exec/invocation.rs::Invocation::example_run emits only frozen Cargo package/
  example selection. Invocation::new and Literal::new are crate-private;
  args() is borrowed. There is no public arbitrary target-argument builder.
- TrustedArgv::from_command_line requires the process's actual ArgsOs and has
  private from-words constructors. It must not be fabricated or repurposed.
- Add one narrowly typed invocation constructor in the existing exec crate for
  the registered write operation, with fixed `--` followed by a fixed explicit
  operation/write token. No generic Vec<String> escape hatch is needed. Keep
  Programs::new(root), null stdin, C locale and existing per-stream/build limits.
- A target-side typed operation decoder must require the exact write token and
  reject missing/duplicate/unknown arguments before calling any writer. No args
  may remain check-only or refuse; they must never select writing. CLI write
  selection requires --write plus explicit validated package/example. A separate
  fixed operation IRI/version and response class prevents accepting check-v1 as
  a write receipt. A protocol mismatch stays failure with no fallback.

For this single operation a fixed argv token is a complete explicit request;
request negotiation and an RDF document on stdin add no necessary behavior.
SystemRunner currently fixes stdin to null, so such a document would require an
additional execution API that is not otherwise needed. The response must use
raf_rdf::{Builder,Graph,Format,TermValue}, backed by native storage/reader/writer,
as current RegisteredCheck does. Retain shared native integer/IRI lexical
primitives and Census vocabulary; add only RAF-specific closed write schema.
Current PurRDF host-runner absence was already source-verified in the portfolio
assessment, so reuse RAF exec rather than introducing another subprocess layer.

## Actual locked-write gap

Gate::write_debt (gate/mod.rs:257) always calls evaluate_tree, the ordinary
Workspace::load_with path. Gate::check_locked (198) builds exact configured
locked first-party/provider identities, evaluates with registered rules and
extensions, then calls check_debt. No public locked writer exists. Passing the
checked report to adoption is unsound: check_debt has already appended debt
growth/shrink differences to findings, obscuring non-debt-versus-ratchet policy.

Smallest coherent extraction: private evaluate_locked(tree,catalog,local,sources)
contains the existing check_locked evaluation before check_debt. Keep
check_locked calling it followed by check_debt. Add write_debt_locked taking
typed catalog/local/source inputs and constructing its actual DiskTree at
self.root(); evaluate through the exact same locked path, then reuse one private
write-evaluated helper with the existing write_debt policy. Do not expose a
public arbitrary/stale GateReport publishing function or evaluate against one
tree and write into an unrelated root. The target supplies the real acquired
catalog and sealed source/configuration inputs; the CLI invents none.

Reuse unchanged existing policy in write_debt: refuse any non-debt findings;
read existing native DebtRecord; same-normal-form compare rejects growth;
absent/different-form adoption remains explicit; unchanged content avoids a
write; canonical debt serialization and replace_record provide the one actual
publication path. Preserve all storage/I/O/temporary-record failures. The write
response can project Result<DebtWrite,Error> with distinct success/error status,
fixed record path and counts, strict cardinality/version/exit agreement. It is
a trusted target receipt, not proof against a dishonest/malicious executable.
Host refusal after target execution cannot promise to roll back a write that
the explicitly authorized target already performed.

## Required concrete qualification for this unit

Use the real shared Rule/Extension fixture plus a real configured native
provider job. Exercise direct locked Gate and shipped CLI writing from identical
fixture states and compare exact census-debt.ttl bytes. Include initial adoption,
unchanged byte identity, no-growth refusal, strict shrink, non-debt finding
refusal and malformed prior-record refusal. Snapshot all source/manifest/lock/
ledger inputs and debt around every refusal; check absence of temporary output
left by failures where the existing writer contract guarantees cleanup.

Pair missing/wrong write token and check-only invocation with valid write mode
to prove authorization is not inferred from target selection. Verify missing
target/build/version/shape/exit failures, retaining bounded causes. Retain actual
empty/nonempty pinned native output witness and decorative/competing provider
refusals while proving the writer uses that locked evaluation rather than
structural fallback. Existing provider and check fixtures may be reused when
still applicable, but no existing result proves a new writer was executed.

Keep automatic source acquisition, compiler authentication, remaining CLI/query
commands, broader native adoption and self-census obligations under their current
parents. If the locked writer plus authorization/response cannot qualify within
the delivery bound, do not claim registered debt completion from an adapter.

## Captured issue and comments

# Issue #12: cargo-census: run repository-registered rules and ledger extensions from the CLI

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github
Labels: crate:cargo-census, dogfood

## Body

## Context
The census specs make the Rust `Gate` the authority for every ledger. `cargo census check` evaluates only ledgers that need no repository-registered code. When a ledger has a non-empty `[job.ext]`, or a `rule:` id that is not a built-in detector or pattern, the CLI exits 2 and names the Gate path (the repository's census test). Only the Gate writes `census-debt.json` for those ledgers. See `cap:census-cli`, `cap:census-rule-api`, `cap:census-ledger` and `cap:census-debt-ratchet`.

This issue adds a second mode: `cargo census` runs the repository's own registered rules and ledger extensions. A dialect ledger, such as one migrated from the purrdf or decodes formats, can then be checked and its debt record written from the CLI without a hand-run test.

## Scope
- [ ] Design and specify, as a change record with PROPOSED blocks in the affected capability docs, how `cargo census` reaches registered rules and extensions. The candidate approach is a mode that builds and invokes a repository test or example target, with a defined report and exit-code protocol between `cargo-census` and that target.
- [ ] `check` gives the same findings as `Gate::assert_clean` for ledgers that use `[job.ext]` and custom `rule:` ids. The mode never silently drops a registered rule.
- [ ] `debt --write` works for those ledgers through the same path, and keeps the existing refusal rules (no recorded growth, no write while non-debt findings exist).
- [ ] Hard failure when the repository target is missing, fails to build, or speaks a different protocol version. Never fall back to the built-in-only check.
- [ ] Spawned cargo follows the census subprocess rules: no shell, argv only, `--` before document-derived arguments, the workspace treated as trusted build input.
- [ ] Verification: fixtures for a dialect ledger checked through the CLI, a custom `rule:` id enforced through the CLI, and each protocol failure case.

## Acceptance criteria
- [ ] For a fixture repository with a registered `LedgerExtension` and a custom `Rule`, `cargo census check` and `Gate::assert_clean` report identical findings.
- [ ] `cargo census debt --write` produces the same `census-debt.json` as the Gate write path for that fixture.
- [ ] Every failure mode above exits non-zero with a message naming the cause.

## Depends on
The census CLI (`cargo-census`) and the census Rule, Gate and ledger crates.


## Comments (2)

### paudley — 2026-10-20T23:05:22Z

The repository's standing goals (RDF 1.2 only, wasmable core, greenfield, no build optionality, maximal utility and performance, Rust-first with a Python surface) bear on this issue.

**Report protocol format (goal: RDF 1.2 only, no JSON).**
- Finding: the Rust `Gate` and `cargo-census` are to speak "a defined report and exit-code protocol" between the CLI and a repository test target; the format is unspecified and the natural choice is JSON. Evidence: this issue's body ("report and exit-code protocol", "protocol version").
- Direction: specify the CLI-to-target report as RDF 1.2, using the same vocabulary as the census reports (PR #39, #40) and versioned by an IRI rather than a number in a JSON object. A protocol mismatch stays a hard error, as the issue already says.


### paudley — 2026-10-24T05:52:31Z

<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

The bounded read-only registered check is tracked in #64, dependent on #61 / PR #63. It wires the shipped CLI to a real repository-owned Rule/Extension target using a fixed-version canonical N-Triples1.2 response, bounded frozen Cargo execution and strict protocol/exit refusal. The target runs in the explicitly selected workspace root. No invocation API extension is needed for the check-only operation.

This parent remains open for registered debt writing, including explicit write authorization, the exact no-growth/no-nondebt-write rules and an actual locked write path. Check-v1 cannot become an implicit writer. All remaining CLI/report/self-census/native-adoption requirements stay under #6/#9/#40/#53. No parent acceptance is closed by creating the child. The current CLI has migration/debt entry points; registered check is new production behavior, not an already delivered command.


