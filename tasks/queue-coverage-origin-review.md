<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Coverage reexport origin review

Bounded read-only review of coverage.rs surface_paths/named_paths and the current
edition/super/module-input owners. No builds, probes or product edits.

The new known-prefix path uses actual `Workspace::resolve_from` at each declaring
module, preserving its edition and import viewer rather than duplicating bare
path rules. It resolves the longest prefix before appending only the unknown
tail, then tries the actual head if the longer prefix is NotFound. Repeated
super components remain together in that head, so multi-super climbs are not
mistaken for repeated child names. All non-NotFound errors propagate as
ExportError::Resolve. `surface_paths` obtains every origin before Judge counts
any item of the target: a late origin Budget/Storage/ambiguity refusal therefore
cannot leave partial target item counts. Known external heads that cannot be
resolved produce no local coverage reexport, as intended by the stated limit.

The paired-edition test asserts exact exported path, declaration file/line and
different named origins: root gen in 2015 versus api::gen in 2021. The corrected
super fixture uses a genuine edition-valid `self::deep::*` rather than the old
2015 root-relative bare deep path. It retains exact module-file super origin,
covering-module success versus declaring-module failure, inline double-super
identity and the self-path neighbor. The separate rustc paired-origin witness
is supplied owner execution evidence, not something executed by this reviewer.

The Module input identity owners inspect actual current evidence/assessment
currency: own-file and child-file edits name the bound module as changed, while
an inline/file sibling edit remains current. Their clause owns real module
input identity rather than borrowing a coverage-binding assertion to prove
digests. No arbitrary prose-truth judgment is claimed.

## Open leading-absolute boundary

The current unconditional `written.starts_with("::")` early return is wrong
for local Rust 2015 paths such as `::gen::Made`. Source exports preserves the
leading colon in Reexport.written, so this is a real current omission rather
than lost source metadata. It also cannot generally assume a leading-absolute
head is outside the loaded workspace in newer editions. The owner independently
identified this existing boundary and is preparing native routing plus an
actual 2015 absolute-origin fixture; it is not repaired or qualified here.
Merely deleting the early return is insufficient: the current first-segment
fallback would see an empty head, so retain the leading colon while identifying
the actual first head for resolve_from. Non-NotFound failures must still refuse
the whole target.

No other concrete defect found in this bounded slice. Release the read freeze
for the leading-absolute repair and its focused assertions. Owner reports the
selected edition repair and independent native paired-origin witness passed;
family/Clippy 33757 was active at request. Earlier invalid-2015 fixture and
unbound-owner failures are not passes. No full family, candidate count or
adoption acceptance follows from this review. Recheck the actual absolute-path
repair before closing the open item.

## Leading-absolute repair recheck

Current named_paths removes the unconditional early skip. It strips the leading
colon only to identify the first textual segment, then slices the native head
from the original written path, preserving `::` for both longest-prefix and
head-fallback calls. Thus `::gen::Made` sends `::gen` to the existing source
resolver, leaving edition policy with that API. No independent Workplan edition
whitelist, local-root guess or swallowed error was introduced. The same repeated
super handling and non-NotFound whole-target error propagation remain.

The actual edition fixture now includes 2015 bare gen, 2015 absolute ::gen and
2021 bare gen, requiring exact named origins, kind/exported path and declaration
location. The corrected inline-super expected gaps also retain root
`made::Made -> made::gen::Made` when only the outer module is bound: an outer
binding must not mask a reexport whose actual origin is outside it. This is an
assertion correction preserving uncovered behavior, not a new exclusion.

The identified leading-absolute boundary is closed in inspected source and
assertion ownership. Owner reports independent real 2015 absolute rustc witness
exit 0 in queue-coverage-reexport-absolute-rustc.log. The prior full family was
29 pass/one stale expected root-origin gap failure; it is not an all-pass
receipt. Final family/Clippy 84197 remained pending at request. No reviewer
builds or product edits, and no full family/adoption claim. This bounded read
is released.
