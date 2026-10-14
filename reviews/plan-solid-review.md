# SOLID / ETHOS design review (independent, single concern) — condensed record

Run read-only during plan mode; full report returned inline. VERDICT: NEEDS-REVISION (14 required, 11 advisory).

Required: 1 parse entry takes a visitor run on the parse thread, `pub use syn`, panic re-raised; 2 owned FileFacts cross the thread, walk on caller thread, Tree needs no Send/Sync; 3 bin crate naming and katamari target silences; 4 namespaces, multi-site symbols, Resolution carries via_glob; 5 cfg_attr path alternatives as sites, not errors; 6 file reached by several paths indexed under each, one identity; 7 impl self-type rules (inherent vs trait impl); 8 rustc shadowing precedence, search to completion; 9 RelPath and a precise Tree contract (cap in both, sorted list, entry kinds); 10 layered LoadError with origin; 11 one schema table for writer and validator; 12 writer and reader share plain predicate and escape set, GMO style choice only for constructed values; 13 splice on Body, content spans, transactional SpliceError; 14 census payload excluded from ported ranges.
Advisory: 15 module layouts; 16 partial Task 1 code fixes (MAX_DEPTH placement, unwrap_or_default, Item name collision, trailing comments, non_exhaustive constructors); 17 shipping derived from kind; 18 pre-scan budgets table; 19 Violation Location enum; 20 discovery skip set named in Contract, duplicate crate names; 21 Workspace::source; 22 stop at first LoadError; 23 decide cap; 24 PROPOSED block section; 25 load delegates, Profile unit struct.

Dispositions: plan.md section "F. Review dispositions".
