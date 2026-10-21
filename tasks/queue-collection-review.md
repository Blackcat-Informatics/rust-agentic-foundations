<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Owned collection execution review

Independent review of the transferred correction source, before final contract
and broad qualification. The existing capability explicitly disclosed destructive
caller-span invalidation; this change removes that restriction. The old cleanup
test's file-relative offset did not establish its named storage property.

The corrected public path loads the Tree on its caller, then borrows the completed
Workspace/options into a fresh scoped collection worker. It adds no Send/Sync
requirement to Tree, copies no workspace, and returns only plain owned units/errors.
The global SpanMap reset is removed entirely. Worker teardown owns the TLS source
map, including both token lexing and literal decoding, on success/error/unwinding.
The helper joins explicitly and resumes the original panic payload; spawn failure
is an actionable CollectionSpawn with its io source. No fallback/partial result
path was introduced. The new diagnostic escapes control characters.

Actual source tests cover live caller spans around public successful/refused
loads; repeated loads preserve unit results and add no caller source-file entries;
normalization at MAX_NESTING and one above runs through the real worker; TLS Drop
witnesses cover success/error/panic and the panic's u32 payload survives unchanged.
The file ordinal observes the locked proc_macro2 source-map entries, not retained
byte counts or RSS. Final inspected neighbors load a Cell-backed !Sync Tree
without copying it, and retain standalone Tower forms/prints across valid and
refused public loads. All139 census tests passed, including unchanged normal-form
and report goldens. No new normal-form token/hash algorithm is changed.

Remaining review: current public docs/closed error bindings, actual lifecycle and
assessment currency, focused final tests and all normal commit/full gates. Native
worker execution does not qualify wasm; standalone Normalizer/Tower calls retain
their existing caller-thread lexical storage behavior. Those portability/session
contracts remain visible under the parent and source/core work. This review does
not claim that broad qualification or final acceptance has already passed.
