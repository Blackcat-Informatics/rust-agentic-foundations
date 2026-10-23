<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Indexed owner-cache delta review

Read-only review of current paths/cache.rs, its visitor/path integration and Error routing, with the exact pinned native OrderedTable/Memory implementation. No builds/tests or product edits. No Source/provider/dependency API review was reopened.

## Concrete finding

IndexAdmission::resize computes its permitted upper bound with checked `unit * limits.states * 3` and reports SizeOverflow if that bound overflows. A permissive states=usize::MAX therefore refuses a tiny first insertion even though its actual native layout is small and valid. This is overflow of an allowance, not of a physical request. Saturate/compare that upper bound without overflow and let native Memory retain checked Layout validation for actual requested arrays. A small successful lookup with an effectively unlimited state grant is the valid neighbor; do not classify derived-budget overflow as impossible storage.

## Inspected mechanics

No concrete identity/live-account defect found otherwise. The cache is constructed per parsed file, so byte-only keys cannot cross files in current production callers. Native key order is numeric byte order, not source display/path identity. Cached owner lists retain all actual source reaches and the existing real-item preference over root pseudoitems. The in-place partition preserves relative order of the kept real items; cache lookup/tree order does not replace result processing order.

Each growth resumes Memory with the same cache's saved exact arena bytes and initializes IndexAdmission.live to that value. The first positive native layout establishes the slot-layout unit. Pinned OrderedTable grows one slot array geometrically, so three slot layouts per admitted state conservatively cover old plus replacement arrays. Native reserve releases the old layout only after movement; the saved admitted_bytes remains the surviving capacity. Failed work admission retains the domain failure; allocator/layout failures without that first domain failure retain the original StorageError through PatternIndexStorage and Error::source. The owner is adding an explicit unexpected-replacement invariant; no replacement is expected after the preceding miss under this exclusive mutable borrow.

Both get calls are logarithmically precharged; insertion has a conservative AVL height/link-edit precharge and actual native replacement-layout charge, avoiding a constant-cost fiction. Cache misses charge the actual Source ownership scan; repeated byte queries reuse it. The preflight bounds logical retained entries and owner references. self.bytes covers the native slot arena only: values are Vecs returned by existing Source item_at and may retain spare capacity. Do not describe logical reference lengths as exact physical admission of those separate payload buffers. The source-owner API boundary remains distinct; this review does not request a duplicate owner algorithm.

Owner reports all 60 Gate cases and 22 library cases passed, including the real 4,000-binding positive/clean neighbor and low-grant refusal; additional focused admission/error coverage and final lint were pending at this checkpoint. These are owner receipts, not tests executed by this reviewer. No complete hosted gate/publication result is inferred.

## Narrow repair disposition

Confirmed the reported upper-bound overflow defect is closed. The derived allowance now saturates while native Memory still checks actual requested Layouts. The small states=usize::MAX lookup is a success fixture. A separate actual impossible native Vec layout produces SizeOverflow, whose original typed cause is inspected through PatternIndexStorage, followed by a valid one-byte reservation.

Payload retention now counts owners.capacity(), including spare reference cells, before publishing the cache entry; checked-subtraction admission prevents state-count overflow. The initial candidate preflight still bounds invocation of the existing Source owner query, while its internal temporary allocations remain that API's explicit boundary. Exact IndexAdmission bytes continue to describe only the native OrderedTable arena, not a falsely universal ownership grant. The new unexpected-replacement check preserves an actionable invariant failure rather than silently swapping cached owners.

The same-original-Memory growth fixture checks real old+replacement admission, a refused next growth preserving value/capacity/live bytes, then resumed successful growth under a larger state allowance. No further concrete defect found in the exact re-read corrections. Owner reports all 24 library and all 60 Gate cases passed at raw/indexed-cache-runtime-final.log; workspace lint was active. No build/test or product mutation was performed by this reviewer, and no lint/hosted completion is inferred.
