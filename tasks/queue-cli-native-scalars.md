<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native scalar assertion callers

The CLI RDF assertion helper now uses native XSD typed nonnegative Integer and
boolean readers instead of Rust lexical parsing or a local Boolean parser.
Datatype checks and exactly-one scalar cardinality remain; canonical native
lexicals, an unsigned 128-bit preparse width guard and checked usize conversion
preserve its count domain. Native Boolean term equality keeps the canonical
true/false subset rather than admitting the native reader's 0/1 aliases. Fixture
failures name the property and node. Direct pinned XSD is a dev dependency; the
root lock records it, and locked offline fuzz metadata remains valid.

The completion projection's optional item number uses presence plus the same
native reader. Claim ordering is checked from typed RDF capability/section/line
keys after exact text-table equivalence, with no reparsed table or swallowed
conversion. Ordinal sorting caches each native scalar key once per row. Stale
JSON local names in those touched typed-output tests are removed.

Actual all-target CLI compilation passed with exit 0 in
`raw/queue-cli-native-scalar-callers.log`. The complete/history/RDF/registry batch
passed all 12, 6, 5 and 1 tests with terminal exit 0 in
`raw/queue-cli-native-scalar-qualified.log`; the registry test checks all command
families and exact pins. After the neighboring conversion repairs, all 4 claims
and 12 completion tests passed with terminal exit 0 in
`raw/queue-cli-native-scalar-current-neighbors.log`. Current all-target CLI Clippy
passed with exit 0 in `raw/queue-cli-native-scalar-clippy.log`. No new parser-test
mirror was introduced, and no full gate or adoption completion is claimed.
