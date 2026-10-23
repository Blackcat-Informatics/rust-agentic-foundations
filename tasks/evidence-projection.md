<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Review evidence representation

The first atomic merge push was rejected by the repository attribution guard on
an automated authorship HTML marker in archived review text. Remote main stayed
at d061457 and PR 59 remained open; no merge or audit ref was published.

Review JSON and extracted review text are policy-compatible projections. Only
HTML comments rejected by the installed attribution policy were removed.
Technical findings, supporting analysis, commands, recommendations, reviewer
identity, comment/review identifiers, URLs, state and commit metadata remain.
JSON was parsed and serialized without changing other field values. The same
live guard checked every transformed payload. Original review records remain
available through their retained forge identifiers and URLs.

The current installed stage capture helper excludes files ending in `.log`.
Execution logs remain local, while the acceptance index, validation report,
review dispositions and machine metadata are captured. This is the helper's
current capture contract; raw logs are not claimed archived. Preserve the local
execution evidence before removing the completed checkout.
