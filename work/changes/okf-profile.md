---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Change
id: change:okf-profile
title: "Record types and claim markers in the OKF profile"
description: "Adds the Evidence, Assessment and Coverage record types and the claim marker to the profile that raf-okf reads and validates."
changes: [cap:okf-profile]
affects:
  - cap:okf-profile#contract
  - cap:okf-profile#verification
contract_effect: changes
items:
  - claim: "Profile::validate accepts the record types Evidence, Assessment and Coverage with closed key sets, returns their typed values, and reports every violation of their keys, values and empty bodies."
    outcome: implemented
  - claim: "The Coverage record is a singleton, coverage:default at work/coverage/default.md, and refuses a crate-root exclusion and an exclusion with no reason."
    outcome: implemented
  - claim: "Body::claims reads each claim marker with the evidence it names and the paragraph or list item it supports, outside fenced code, code spans and PROPOSED blocks."
    outcome: implemented
  - claim: "A malformed or unattached claim marker, and any other raf: comment that is not a raf:generated: comment, is a parse error with its line, and a claim outside a capability document is a violation."
    outcome: implemented
  - claim: "Each record type is built with its constructor and written by to_document as a document whose rendering Profile::validate reads back as an equal record."
    outcome: implemented
  - claim: "Number::compare orders two numbers exactly on their decimal texts, and Profile::validate judges the limits of a passing evidence record with it; a limit of a failing record may name no value."
    outcome: implemented
---

## Summary

The profile gains three record types, which state a result in their frontmatter
and have no body: evidence of an execution, an assessment of an authored section,
and the coverage record of a repository. Evidence and an assessment share one trace
shape (a subject, the inputs with their digests, a verdict and the kind of author),
so a later edit of an input can be detected. Capability prose names the evidence
a statement rests on with a claim marker, which the body model reads and checks.
