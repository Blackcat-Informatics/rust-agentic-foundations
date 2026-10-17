---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:okf-profile
title: "A claim marker split over lines"
description: "A multi-line HTML comment that starts as a workplan marker is a marker error, never a plain comment."
changes: [cap:okf-profile]
affects:
  - cap:okf-profile#contract
  - cap:okf-profile#verification
contract_effect: changes
items:
  - claim: "A claim marker split over lines is an error: a multi-line HTML comment whose first text after its opening is raf:claim in any letter case is okf-parse-malformed-claim."
  - claim: "A multi-line HTML comment whose first text is another raf: marker, other than raf:generated:, is okf-parse-unknown-marker, while multi-line raf:generated: comments, SPDX headers and ordinary multi-line comments stay plain."
---

## Summary

A claim marker written over three lines was read as a plain comment, so its
claim left the graph with no finding. A multi-line comment that starts as a
workplan marker is now an error of its own kind, and every other multi-line
comment stays plain.
