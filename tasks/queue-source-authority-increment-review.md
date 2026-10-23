<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Source authority increment review

Read-only current-source review; no builds or product edits. Scope is RelPath, FileId and SourceTrees physical separation, not locked dependency grounding. The trusted caller/checkout source contract is sufficient for the intended functionality; the stronger immutable Git witness proposal is optional and is not an added acceptance requirement.

## Concrete finding

`tree/sources.rs::read_unseen` rescopes identities by calling `FileId::in_source`, which replaces the existing authority rather than retaining the complete backend identity. The public Tree/FileId APIs do not currently prohibit a custom rooted Tree from returning two distinct identities whose only difference is their original authority. For example, two actual files can have `from_key("same").in_source(a)` and `from_key("same").in_source(b)`. They are distinct before mounting but become equal after both are mounted beneath one SourceTrees authority. This can incorrectly answer Seen and merge parsed files.

For this bounded unqualified-backend design, the smallest repair is an explicit contract that supplied backend FileIds are first-party/unqualified, with a hard refusal before passing any foreign ID to the outer seen callback or returning it. Inspect both callback and returned Opened paths. Alternatively preserve the entire nested identity structurally. Do not silently flatten it. Native DiskTree and MemoryTree already return unqualified identities and are not affected by this demonstrated custom-backend gap. Add the actual two-identity refusal or preservation test with an ordinary valid neighbor.

Narrow repair recheck: closed in current source. SourceTrees now documents its unqualified backend-ID requirement, traps the first foreign callback ID without calling the consumer's seen function, and checks that trap before propagating a later backend error. Thus a subsequent TooLarge cannot mask the typed BackendAuthority refusal. Returned Seen and Read IDs are independently checked before rescoping, covering backends that omit the callback. The actual TaggedBackend wrapper exercises a nonzero tag and a zero-tag read/Seen neighbor. No further concrete defect found in this repair. The earlier 101-test execution predates this change; focused composition execution was still owner-running at this recheck request, and no build/test was performed by the reviewer.

## Source checks without further findings

- RelPath replaces depth with the authority handle, preserving its record footprint. join computes validated depth using native WorkList while retaining the old rule that lexical `..` remains for the backend; parent/source_root preserve the handle. Both actual tidy reconstructions now start at the original source_root, including prefix entry checks. Cross-mount dispatch is typed and cannot be selected or removed through textual path components.
- SourceTrees validates authority membership before dispatch, borrows its backend list and uses explicitly unqualified local paths. Read limits, UTF-8/regular-file/list/entry and backend errors flow through the selected backend; it does not copy an entire source tree. Its tests inspect equal paths/backend keys in distinct sources, the actual seen callback behavior, max-byte refusal, normalization, inside-root neighbor and unknown authority.
- Standalone DiskTree and MemoryTree reject nonzero tags before opening/reading/listing or calling seen. Their ordinary authority-zero paths retain existing filesystem containment and memory normalization. The explicit in_source operation is caller source selection, not proof or an implicit module traversal.
- Device/inode and original key identity remain intact for ordinary aliases; same-authority aliases therefore still deduplicate and participate in physical cycle detection. Key/native-path Arc payloads make the callback clone allocation-free. `heap_bytes` charges shared allocation headers/alignment through shared_text; retained map/record costs use actual size_of/FileId layouts. RelPath's shared text remains unchanged. No universal preallocation guarantee is inferred from public RelPath joining or caller-owned Trees.

## Verification limits

Inspected the new test bodies; no tests were executed by this reviewer. The owner reported workspace caller compilation passing and the source-library run still active at request time. Existing hardlink/symlink and platform runtime coverage is not newly executed proof here. Exact locked identities, catalog edge selection, trusted production Cargo checkout validation, actual pinned PurRDF symbol resolution, and first-party/external source API integration remain pending. Authority IDs are local handles, not content authentication or durable package identity.
