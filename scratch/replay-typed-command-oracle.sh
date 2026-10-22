# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
set -euo pipefail
stage=.stage/raf-workplan-strict-mode-force-coverage
fixtures=crates/cargo-workplan/tests/fixtures/reports
inputs=()
for turtle in "$fixtures"/*.ttl; do
    inputs+=("$turtle" "${turtle%.ttl}.nt")
done
cargo run --offline --manifest-path "$stage/scratch/rdfprobe/oracle/Cargo.toml" -- "$stage/raw/typed-command-oracle-final" "${inputs[@]}"
for turtle in "$fixtures"/*.ttl; do
    stem=${turtle##*/}
    stem=${stem%.ttl}
    cmp "$stage/raw/typed-command-oracle-final/$stem.ttl.nt" "$stage/raw/typed-command-oracle-final/$stem.nt.nt"
    cmp "$stage/raw/typed-command-oracle-final/$stem.ttl.nt" "$fixtures/$stem.oracle.nt"
done
