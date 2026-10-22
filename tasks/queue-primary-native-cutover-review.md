<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native cutover source review

Primary source inspection complements the earlier independent native reviews.
Inspected current raf-rdf Graph, term roles, parse and render paths, census
scalar/report adapters, Number native value storage, exact line-ceiling products
and digest framing. Compared supplied scalar APIs with current PurRDF source.
This review ran no additional suites and makes no delivery or adoption claim.

The graph owns one Nodes store with native TermValue objects. Census transfers
the same store rather than constructing a converted graph. Empty stored rows
are refused before becoming invisible to serialization. Native parser results
refuse named graphs before flat default traversal; normalization only rewrites
the selected root and its fragment boundary. Native term traversal/output owns
atomic and triple-term escaping, with RAF's relative snapshot adapter retained.
Closed roles, dangling references, root identity and exact canonical bytes remain
checked through actual public read/render callers. Resident is not a claimed
output allocation ceiling, and depth-one validation is not a preparse bound.

The scalar unsigned adapter preserves the full u128 range above the native
signed term constructor. Native exact Decimal storage preserves the narrower
authoring grammar and authored negative-zero display. One concrete remaining
reparse was found in line-ceiling products: they parsed Number::text even though
Number already retains the exact native value. The owner is replacing it with
a borrowed native-value accessor and correcting implementation-specific rustdoc.
Additional supplied Integer::from(u128), Boolean lexical reading and full-width
integer parse/conversion APIs were identified for the final scalar sweep, with
canonical lexical and width refusals retained. These findings require the owner's
focused recheck; earlier scalar results do not automatically cover those edits.

Digest framing still feeds the original NUL/name/decimal-length fields. Native
one-shot SHA-256 and hex replace available helpers; sha2 streaming remains an
explicit public-API gap rather than buffering complete module preimages or
retaining a copied compressor. Fingerprint goldens and supplied focused results
are recorded separately in the current native checkpoint.

No additional model/parser/writer correctness blocker was found in this bounded
inspection. Full mandatory gates, current assessments, delivered-state adoption,
lifecycle completion and integration acceptance remain open. The current passing
implementing-state default diagnostic does not prove those remaining contracts.
