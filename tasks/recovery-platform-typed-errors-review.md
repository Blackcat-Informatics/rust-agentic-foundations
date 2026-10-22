<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Typed compiler-platform error delta

Scoped pass by read-only inspection of `recovery-platform-typed-errors.patch`
and applied Source, Workplan caller and source capability documentation. No
builds, tests or product edits performed. Parent reports actual strict completion
session 11246 exited 0; adoption completion, final candidate equality, full gates
and CI remain pending. This review does not replace session 9892 qualification.

The parser's decisions are unchanged (`platform.rs:93–140`): same nonempty-line
filter, `syn::Meta` parser, independent bare atoms, string-value requirement,
single OS requirement, family accumulation and rejection of lists. Only error
representation changes. Each former string refusal maps to its corresponding
non-exhaustive public enum variant. Display preserves the former messages.
`Syntax` owns the original line and `syn::Error`, exposed through `Error::source`
(`platform.rs:31–76`); semantic failures have no underlying parser cause to lose.
The public reexport and capability binding/documentation agree with that API.

Workplan's single production call (`bound/run.rs:161–176`) renders this typed
error into the existing string-valued failed binding diagnostic, just as before.
The Source error chain survives at the Source API boundary; Workplan receipts do
not expose that chain structurally and this patch does not claim they do. Invalid
UTF-8 or malformed compiler output still calls `run.fail` and exits the platform
decision without caching a platform or setting `NotApplicable`. Successfully
parsed configuration alone is cached. Exclusion still requires nonempty guards
and every guard evaluating to `Some(false)`; unknown/live alternatives execute
normally. Rustdoc's separate listing requirement is unchanged.

No new lifetime or ownership path is introduced: the enum owns only error-line
text and the parser error. The added invariant comment describes the existing
inline-module selection; it changes neither traversal nor guards. The expanded
negative test asserts every distinct variant and syntax cause; the neighboring
valid `target_os="none"` input is retained. Runtime success remains owned by the
pending focused checks, not inferred here.
