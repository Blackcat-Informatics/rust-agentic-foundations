<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native scalar adapter review

Read-only primary inspection of actual CLI assertion helpers and the subsequent
OKF/workplan lowercase-hex and debt-count replacements. No reviewer builds or
product edits. Owner remains the sole implementation mutator.

CLI RDF assertions retain datatype and exactly-one cardinality checks, use native
typed nonnegative Integer parsing and native boolean reading, and check native
canonical representations. Checked usize conversion retains the scalar domain.
Ordinal sorting caches each native key once per row. Claim comparison uses typed
RDF ordering after exact text table equivalence rather than swallowing failed
numeric parsing. No competing generic scalar parser remains in these helpers.

OKF digest validation delegates exact 32-byte decoding to native canonical hex.
The OKF and workplan Git commit adapters retain 40/64 lexical widths and delegate
each digit to native canonical classification. This preserves uppercase and
malformed refusal without allocating a decoded Vec for a boolean question.

OKF debt counts preserve digit-only syntax, including accepted leading zeroes.
The adapter strips zero padding before native arbitrary-precision parsing,
bounds significant decimal width, and uses checked native-value conversion.
This avoids allocating in proportion to accepted zero padding. Long padding and
one-past-usize remain relevant caller controls; the owner has been asked to verify
them. Native decimal Number parsing under RAF's narrower grammar was already
coherent and is not a competing implementation.

The subsequent bounded source review also covers the implemented Git version,
libtest count, YAML float-style and census hex-table replacements. Workplan's
shared unsigned adapter preserves optional plus and zero padding under a
significant-width bound, then delegates interpretation to native Integer and
checked conversion. Git's whole components retain their stricter digit guard.
An overflowing numeric patch now produces the existing actionable version-answer
error instead of a fabricated zero; absent or nonnumeric patches retain zero.
Actual maximum-u32 and one-past refusal controls were inspected.

Libtest counts retain their required/unique fields and use the same native count
adapter; inspected controls cover host maximum, overflow and malformed spellings.
The YAML style adapter maps existing case-insensitive infinity/NaN aliases to native
XSD spellings, then uses native double reading. The native implementation delegates
ordinary numeric forms to the same Rust float primitive after its XSD lexical guard;
the adapter restores the aliases that guard excludes. Actual renderer controls
include numeric and keyword aliases plus adjacent strings that remain plain.

Census TableKind classification now encodes an eight-byte fixture through native
lower/upper hex writers into stack storage and compares the resulting sixteen
digits. It retains RAF's classification and avoids recreating the generic alphabet
generator or allocating temporary Strings. The short native destination guarantees
justify its explicit invariant expects. Planted hexadecimal tables in detector
fixtures remain input evidence rather than implementation helpers.

No concrete defect was found in this bounded final source recheck. Owner reports
terminal all-target Clippy and actual OKF/workplan/census caller results; those are
owner execution, not reviewer execution. Exact logs remain indexed by the owner.

Owner qualification is indexed in queue-cli-native-scalars.md. This disposition
does not establish full gates, current repository adoption, lifecycle completion
or assessment currency.

## Extended test and tooling audit

Primary's next exact source search found the repository-gate copied SHA-256
kernel, manual recorded-digest validation and remaining registry/test scalar
readers. Owner replaced these through ContentDigest, native canonical/nibble
hex and bounded native integer adapters. Primary inspected the resulting root
gate, registry version/count callers, process readers and Census literal adapter.
The root helper now delegates its entire one-shot hash; no compressor remains.
The recorded-digest validator preserves uppercase hexadecimal boundary rejection
while requiring lowercase canonical payloads, with non-ASCII boundary neighbors.
Version components preserve their existing digit-only guard and u64 width;
prerelease ordering preserves numeric-versus-text treatment at overflow. Host
counts retain optional plus, zero padding and checked narrowing.

The Census adapter retains Rust suffix/separator/radix grammar. It scans every
digit even after overflow, bounds significant storage before native decimal
reading or hexadecimal decoding, and preserves syntactically valid oversized
integers as value-less integers. Zero and leading-zero paths remain correct.
No public compatible native binary/octal Rust literal reader was found; that
small grammar-specific accumulation remains explicit. This is not a generic
normalization replacement or authority to change normal-form bytes.

No substantive defect found in this bounded source review. Owner reports 72
root gates passed; the sequential immutable normal-form and caller qualification
is still running. Primary did not duplicate its execution. Full gates remain.

The subsequent native bidi predicate is wired into the actual hidden-character
decision, retaining RAF's control-character and line-formatting policy. Primary
inspected the public native predicate and its pinned Unicode table contract,
the exact revision/dependency declarations and owner's actual closure/license
record. Only the text package is newly added to the existing closure; the fuzz
graph does not depend on workplan. Its mirrored policy's unmatched-exception
warning is disclosed, not a claimed skip. Owner's final sequential session
returned zero for workspace all-target Clippy, root/fuzz deny, 72 repository
gates and docs sync. Census immutable goldens and actual native scalar callers
passed their focused runs. The zero-test private-helper selection is explicitly
excluded from evidence; the actual FIFO process-family caller executed instead.
No remaining substantive finding in this bounded replacement slice. Owner may
continue claim ownership; full gates, adoption and lifecycle remain required.
