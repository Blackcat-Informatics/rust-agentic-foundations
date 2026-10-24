<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

Configured Source consumers can now select binaries, tests, benches and examples with their ordinary package library, while development dependencies and filtered syntax stay in their declared target contexts. Missing companions, incompatible physical views and exhausted native resource grants fail with typed causes.

The shared code-query API also exposes actual external-library/public-export identities and bounded alias/member lookup for downstream provider enforcement. The Source capability states the supported compilation and inference boundaries.

Validation: full `make check` passed, including all five 5,000-run fuzz targets and all three cross-platform compilations. The 64 focused integration tests, exact structural-work regression, affected all-target Clippy, independent source review, native capability completion and normal signed commit hooks also passed. All eleven enabled hosted jobs passed, including Linux/macOS capability verification. CodeRabbit completed with no actionable source findings. Windows runtime remains skipped under the current explicit exception. Original qualification failures are retained with their corrections.

Closes #60. Broader provider enforcement, CLI/self-gate, adoption and resolver parents remain open.
