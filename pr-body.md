<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Census baseline and index reports now use PurRDF's native datasets, terms, parser
and writer through the shared `raf-rdf` kernel. This removes the private RDF graph,
parser and rendering implementations while retaining closed Census shape checks.

Report form 2 explicitly migrates the canonical Turtle and N-Triples 1.2 layouts
to one normalized-body snapshot identity. Normal-form 1 fingerprint bytes remain
unchanged. The kernel's supported profile and resource limits are documented.

Validation: production reader/writer and refusal fixtures, immutable NF1 goldens,
both whole-repository comparisons, a 5,000-input sanitizer run, eight actual parser
revision replays, real workplan completion, current section judgments and
independent substantive review passed. Recovery repository-contract and coverage
checks passed. Full `make check` passed, including Rust 1.98.1, release tests,
all five 5,000-input fuzz targets and cross-platform compilation.

Refs #40 and #53. Both remain open: Workplan machine-output migration, strict/default
adoption, Census debt/CLI/protocol and grounded provider enforcement retain their
existing delivery requirements.
