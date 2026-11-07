<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Supported targets

Through the 0.9 release, RAF supports and qualifies the primary native Linux
target used by its standard repository gates.

WebAssembly and WASI, other operating systems, secondary Linux architectures
and ABIs, foreign execution runtimes and cross-compilation are outside this
release scope. Additional targets will be reassessed after the 0.9 release.
That reassessment determines any additional support.

Native libraries, binaries, tests, benchmarks, examples and build scripts remain
covered. The stable minimum supported Rust version and the development nightly
are compiler versions for the same primary target. Static documentation output
and browser verification on Linux remain covered.

Source analysis may read target-conditional Rust declarations on Linux. Those
declarations describe input syntax and configuration; they do not establish
execution support or qualification for another target. Existing platform
adapters may remain in source without additional support claims.

Published workspace members configure docs.rs with
`default-target = "x86_64-unknown-linux-gnu"` and `targets = []`. The empty list
restricts that host to the default target; omitting the metadata would build
multiple targets. See the [docs.rs metadata contract](https://docs.rs/about/metadata).

Docs.rs [permits documentation JavaScript without guaranteeing its compatibility](https://docs.rs/about).
Interactive hosting therefore requires its own evidence. Its [downloaded Rustdoc archives](https://docs.rs/about/download)
omit shared toolchain assets and retain host-specific asset paths, so they require
processing before offline use. RAF's self-contained standalone diagram bundle
has a separate offline publication contract.

The [development guide](../CONTRIBUTING.md) states the required native checks.
