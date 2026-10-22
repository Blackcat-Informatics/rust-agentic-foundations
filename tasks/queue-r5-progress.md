Integrated the delivered census/source-root foundation and the maintainer's local policy commits through ordinary merges, preserving the existing strict implementation and unfinished adoption work.

Fixed a reproduced reachability bug in `07169f5`: packages of different editions sharing one physical source file reused the first edition's keyword-filtered runs. The cache now uses file and edition. Tests cover combined versus separate roots, reversed root order, swapped editions, raw-identifier neighbors, the real workplan line-coverage caller, and exact/one-byte-short retained-memory limits.

Focused verification passed: 24 reachability integration tests, 15 reachability unit tests, 19 line-coverage tests, 5 touched-surface tests, denied-warning clippy and workspace all-target compilation. The normal commit hook passed formatting, REUSE, 31 document-contract tests, 68 repository gates and the self-workplan gate. No bypass was used.

This is an intermediate repair, not completion of the revised contract. Shared RDF results, removal of the alternative coverage policy, actual zero-debt repository adoption and final full qualification remain in the same delivery. Parent RDF scope stays open for census debt and CLI consumers.
