# Issue #61: Census: enforce configured native delegation through the locked Gate

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

## Outcome

Land the existing configured preferred-provider backend through `Gate::check_locked` as a bounded delivery from #5/#53, after Source prerequisite #60. The current provider worktree is the donor; preserve its unique implementation and evidence during transfer.

## Required behavior

- Resolve provider APIs through actual locked package/source identities, admitted Rust library names and public exports. Renames and aliases preserve original definition identity and ambiguity.
- Require supported adapters to return actual native computation. Explicit mutable state operations must contribute to the returned native result; discarded calls, replaced values, dead state and stale snapshots refuse.
- Audit selected declarations, aliases and ancestors under their actual target/configuration and every physical consumer reach. Homes and entry-point exemptions contain only proved definitions, and consumers agree on one provider home.
- Detect unlisted competing implementations without granting package-wide exemptions. Unsupported proof syntax fails with an actionable refusal; imports or decorative native calls never establish delegation.
- State the finite supported adapter grammar accurately. Checked/wrapped-result validation, receiver inference, domain framing and arbitrary control-flow proof are not silently accepted or removed from the parent adoption contract.

## Acceptance

- Reuse the donor implementation and qualifying evidence; integrate Source #60 deliberately.
- Exercise the actual locked Gate in Bin, Test, Bench and Example contexts with ordinary own-library companions, declared test modes, custom library names and renamed dependencies. Pair development eligibility with a production sibling where that route is absent.
- Replay the pinned public PurRDF Hasher `new/update/finalize` adapter with both empty and nonempty input against frozen output bytes. The nonempty case must independently demonstrate update contribution.
- Preserve all existing return/state/header/alias/home/competitor refusal controls and valid neighbors. Complete the Census-only capability lifecycle, focused independent review, required gates and normal hooks, then publish and land through the normal merge path.

## Parent requirements retained

This issue does not close #5/#53. All remaining detector/default/reference/Rule requirements remain in #5. Actual CLI registration, acquiring the real locked compilation closure, RAF self-gate, migrated-repository acceptance and planted-competitor refusal remain in #6/#12/#9/#53. Full resolver scope remains in #48.

The donor audit identifies concrete subsequent adoption proof gaps in `raf-okf` native Option-to-Boolean projection, `raf-workplan` native payload/error wrapping, `raf-rdf` closed-profile validation and Census canonical token framing. Those real contracts remain required; no arbitrary helper-purity assumption, blanket exemption, or deleted validation is permitted.

Implementation timebox: at most four hours for this behavior unit. Split again if new required work cannot fit, preserving all parent acceptance.


## Comments (0)

