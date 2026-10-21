<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Reconstruct historical source evidence

The archival inventory is `../../tasks/queue-archive-inventory.md`. Nine redundant review clones and six RDF comparison workspace copies were replaced by exact Git bases and raw overlays. All eight unique probe files and eleven RDF variant files were copied with their original modes and compared byte-for-byte before removal. No source logs, measurements, independent oracle programs, samples, fuzz inputs or measured-source patches were removed. Shared build caches were preserved; target symlinks were disposable pointers.

`clones.tsv` maps each historical review directory to its exact checked-out commit. These commits are ancestors of the census delivery and remain reconstructible from its archived head. For a selected clone, create an empty destination beneath the canonical checkout's `.worktrees/` and extract `git archive <base>` there, with shell pipefail enabled. Copy `clones/<name>/` over that destination when present. The result is the judged tracked tree plus every unique untracked probe. Historical reports still cite their original directory names; this is their reconstruction mapping.

For the RDF dependency comparisons, the base is `be93118c27c8c581c95761f9c2116c2c3375758f`. Extract only `Cargo.lock Cargo.toml deny.toml rust-toolchain.toml crates src tests` from its Git archive into an empty destination beneath `.worktrees/`. This is the exact 323-file subset used by `rdfprobe/ws`. The other five variants are that subset plus the corresponding `rdf-overlays/<variant>/` raw files. Locks must be copied unchanged, rather than regenerated. The inventory records each dependency choice and the exhaustive equality comparison against the historical base.

Reconstruction restores source, manifests and inputs. Build products and Cargo target-directory symlinks are not acceptance inputs and must be provisioned normally if replaying a check. This packaging does not turn historical test results into current qualification or preserve unreachable Git garbage as project evidence.
