<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Portable names for replay metadata

The first ghprsq publication was rejected before its atomic push because each
of twenty normalized clone records had both head.txt (revision) and HEAD.txt
(symbolic reference). Remote main remained 6cfa0b8, PR 38 remained open at
7311ed0, and no remote audit refs were published. The complete failed result is
retained in raw/recovery-first-publication.log. Source and passing CI did not
change.

All forty metadata paths were renamed without rewriting their bytes: head.txt
became head-revision.txt, and HEAD.txt became head-symbolic-ref.txt. The complete
old/new path mapping is raw/archive-reconstruction/portable-path-renames.tsv.
The original executed normalization script remains unchanged as historical
evidence; apply this explicit name mapping to its output when replaying it.
Commit identities, symbolic-reference values, exact indexes, used refs, diffs,
ordinary clone sources and earlier preservation receipts retain their meaning.

The installed hygiene guard's actual check_paths accepted every one of the
16,875 then-current archive paths. The installed attribution guard's streaming
Objects reader scanned all 4,281 distinct blobs from the first captured tree,
including unchanged contents of the forty renamed files, and reported zero
credential or attribution findings. Session 43848 exited 0. This is packaging
qualification, not project behavior or a replacement for the normal push guard.
The retry must perform its own final capture and every normal guard. No hook,
guard, attribution rule, signature or CI requirement was changed or bypassed.
