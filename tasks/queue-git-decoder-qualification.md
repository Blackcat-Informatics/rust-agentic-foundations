<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Decoder dependency qualification

Disposition: the failed decoder choice has been replaced, with its exact source
and evidence preserved. The trusted committed-Git host adapter uses no gix
decoder, and unchanged offline deny now passes all categories. See
`queue-committed-git-implementation.md`. The original failure analysis below is
retained evidence, not the current dependency verdict.

Read the actual Makefile before execution. Unchanged `make deny` invokes
`cargo deny --offline --workspace check`; `make licenses` invokes the whole
repo_gates target. No dependency version, policy, exception or source was changed
in this initial qualification.

Actual raw receipts (basename plus `.log` and `.status`):

- `queue-git-decoder-deny`: make exit 2 before evaluation, missing exact offline
  platform cache material. No license verdict.
- `queue-git-decoder-licenses`: session 20778, exit 0, all 68 repo_gates tests pass.
- `queue-git-decoder-fetch`: `cargo fetch --locked`, exit 0, acquires only locked
  portable-atomic-util 0.2.8, jiff-tzdb 0.1.8 and jiff-tzdb-platform 0.1.3.
- `queue-git-decoder-deny-cached`: make exit 2; actual summary:
  `advisories ok, bans FAILED, licenses ok, sources ok`.

Six new duplicate bans: block-buffer 0.10.4/0.12.1, cpufeatures 0.2.17/0.3.1,
crypto-common 0.1.7/0.2.2, digest 0.10.7/0.11.3, getrandom 0.3.4/0.4.3 and
r-efi 5.3.0/6.0.0. The older crypto family comes through mandatory
gix-hash 0.26.2 → sha1-checked 0.10.0 → digest/sha1 0.10. The newer getrandom
comes through gix-utils 0.3.6's mandatory native-target dependency. Existing
pinned native RDF supplies the other versions. The current decoder selection is
not qualified; the source-identity runtime proofs remain preserved observations.

Both direct decoder dependencies already disable defaults and select only sha1.
A sha256-only workaround is unsound: gix-hash's Kind::Sha1 and ObjectId::Sha1 are
feature-gated, as is acceptance of a 20-byte borrowed oid. CommitRefIter requires
Kind, and TreeRefIter uses its length and typed oid validation. Passing Sha256
would decode 32-byte tree IDs and require 64-character commit references, changing
the actual Git grammar. Native SHA-1 verification does not restore those missing
public decoding types. No feature or grammar bypass was attempted.

Published source inspection after this failure found these concrete boundaries:

- Latest gix-object 0.66.1 uses gix-hash 0.28.1, whose sha1 backend is sha1dc
  0.1.3. That backend has no production dependencies, so the four older crypto
  duplicate families would leave this decoder closure. Its borrowed iterator
  constructors still expose the appropriate hash-kind-aware, fallible API.
- The same current decoder mandates gix-utils ^0.5.1, which still mandates
  native getrandom ^0.4. Thus it does not qualify against the current native
  evaluator's getrandom ^0.3. Its source uses utils only for integer parsing and
  progress; utils' getrandom use belongs to its always-enabled rng module.
- Registry metadata identifies gix-hash 0.16.0 as the last published hash-ID
  release without a SHA-1 implementation dependency. Its corresponding latest
  object release 0.47.0 requires winnow ^0.6.18, conflicting with the current
  graph's winnow 1.0.4, and older faster-hex. From object 0.48 onward the hash
  closure already requires the older crypto family. Random-free gix-utils
  releases end at 0.3.3; newer decoder constraints cannot select them.
- These inspected previous releases do not establish current maintenance or
  malformed-input equivalence, and were not adopted merely to remove one family
  while introducing another.

Exact `cargo info` source acquisitions are retained in raw
`queue-decoder-{candidate,current,identity,older}-info`, corresponding hash and
utils info logs, and `queue-decoder-sha1dc-info`. They downloaded published
source for inspection only, without changing Cargo.toml/locks or compiling it.
Registry metadata was decoded with jq and checked against downloaded manifests;
no alternate manifest/object grammar was introduced.

No compatible published gix profile was identified for the current complete
graph. A coherent maintained upstream remedy would make utils' rng/getrandom
optional for object parsing and hashing/progress consumers, then consume that
release. Alternatively the native evaluator's getrandom dependency would need
its own verified update and pin, which is not a decoder-only change. Neither is
implemented or silently assumed. No local registry patch, ban skip, source
replacement or handwritten decoder is an accepted repair. No repeated broad
gate, cold build, runtime suite or source-proof rerun was performed during this
diagnosis. The qualified route observations remain intact, with dependency
qualification explicitly failed.
