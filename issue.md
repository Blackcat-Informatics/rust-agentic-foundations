# Issue #64: cargo-census: run registered checks through a native RDF target protocol

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

## Outcome

Deliver a real registered `cargo census check` path through an explicitly selected repository example target, building on configured native provider execution. This is a bounded delivery from #6/#12/#53; those parents remain open for their full contracts.

Depends on #61 / PR #63. Implementation may proceed from the published parent head while qualification runs; landing remains sequential.

## Required behavior

- Invoke an explicitly validated package/example target using argv, `--locked`, workspace-root cwd and `--` before protocol arguments. Reuse the existing bounded host execution seam and frozen Invocation::example_run unchanged: a check-only target needs no target arguments and emits a fixed-version response. Do not fabricate TrustedArgv or duplicate a process runner. The frozen invocation is stricter than locked execution.
- Transport the actual Gate outcome and findings through a closed canonical N-Triples 1.2 check-response protocol, using the shared Census vocabulary and an IRI protocol version. Keep its native writer/reader in the Census library. Check-only v1 performs no debt writes and requires no request negotiation; reject malformed, partial or exit-inconsistent responses.
- Preserve registered Rules and LedgerExtensions, exact locked-source provider identity and all failures. Missing targets, build failures, malformed/oversized output, incompatible protocol versions and exit/outcome disagreement fail with actionable causes. Never retry through a built-in-only gate.
- State this explicit registered mode and source-acquisition boundary accurately. The repository target supplies real typed locked inputs; the CLI must not invent compiler-observed configuration from caller declarations.

## Acceptance

- A real fixture repository example registers a Rule and LedgerExtension. Actual CLI clean/refusal outcomes match the same Gate report, with appropriate exit codes, and leave source and debt unchanged. Invoke the shipped CLI from a different caller directory with an explicit workspace root and verify the target runs at that root.
- A real locked-source fixture reaches configured native provider execution through the CLI. Approved native delegation passes, and decorative native calls or unlisted competing code fail.
- Execute actual missing-target and build-failure controls. Pair strict version/shape/output/exit protocol refusals with valid neighbors; scripted runner tests may isolate transport errors but cannot replace actual target execution.
- Complete the capability lifecycle, independent focused review, required local gates, normal hooks and hosted checks; land through ghprsq.

## Preserved requirements

Registered debt writing remains required under #12, including explicit write authorization/request, exact no-growth/no-nondebt-write and actual locked evaluation. Never repurpose the read-only check-v1 target as an implicit writer. Other CLI commands, report vocabulary, automatic compilation-closure acquisition, RAF self-census/planted duplicate, read-only sibling trials and full native adoption stay under #6/#9/#12/#40/#53. This transport does not close them or remove existing refusals.

Timebox: at most four hours for this complete behavior unit. Split again before broadening if necessary; a serializer without a wired production caller is not completion.


## Comments (0)

