<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Actual configured-native CLI acceptance

PASS: actual built cargo-census check --package registered-native-driver
--example registered --workspace ABSOLUTE/native-cli, launched outside that
workspace, completed clean with exit0. Raw registered-native-cli-first.txt and
registered-native-cli-final-clean.txt. Appending
`pub fn competitor()->u32{7}` to consumer/src/lib.rs completed exit1 and named
demo::competitor as a COPY; raw registered-native-cli-competitor.txt. Restoring
the exact valid source completed exit0 again. Every actual manifest, source,
lock, configuration, native-root locator and ledger byte was compared before
and after the planted/restored CLI calls; census-debt.ttl remained absent.

The owned native-cli fixture is a real virtual Cargo workspace: consumer is demo,
driver owns the registered example and SDK dependencies. The actual consumer
Cargo.toml and complete actual Cargo.lock are read unchanged through DiskTree.
Ledger root consumer makes the driver executable infrastructure rather than
consumer policy scope. No sanitized manifest or MemoryTree substitutes source.
Explicit LockedCatalog, local consumer manifest/edge, DeclaredConfiguration and
CommittedGitTree sealed at public PurRDF revision
5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43 reach Gate::check_locked. Native API homes
are the public Hasher new/update/finalize definitions. Both ordinary frozen
BLAKE3 outputs (empty and byte-zero input) are asserted in the actual target.

Replay: place native-cli at this same relative stage path in a checkout of this
result, update native-repository.txt to an available existing PurRDF Git checkout,
and inspect compiler-cfg.txt/declared target for the host. Prepare Cargo.lock with
offline resolution if dependencies differ, then run the frozen selected example
through the built CLI from outside this workspace. Driver SDK paths are relative
to the enclosing checkout. Existing archived sources/lock/ledger and both refused
and accepted output receipts are sufficient; target is only a build cache symlink.
The optional --emit-ledger target argument was used solely to create the initial
canonical ledger before assertions; production check sends no target arguments.

Limits: supplied configuration is explicit declared evidence, not authenticated
compiler-closure acquisition. The target is trusted code and is not sandboxed.
This proves bounded configured native execution through the actual registered
transport; full acquisition, wrappers/framing, self-census and native adoption
remain parent requirements. No Windows runtime acceptance is claimed.
