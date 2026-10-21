# T4 report

Status: DONE. Commit: see `git log` (feat(raf-census): grouping, hex tables, shared constants and job matching). Not pushed.

Files (crates/raf-census): src/group.rs (Member, GroupKind, Group, HexTable, SharedConstant, the builder and its in-crate tests), src/job.rs (Job, JobForbidden, JobMatch, MatchClass, JobReport, Census::match_jobs), src/census.rs (grouping computed in load; groups, body_groups, shim_groups, hex_tables, shared_constants), src/lib.rs (exports, docs), tests/groups.rs (14 tests).

API: Group { kind, fingerprint, tokens, members: Vec<Member> } with member_symbols(); Member { symbol, file, line, package } (ordered symbol, file, line); Job { id, home, entry_points, variants, forbidden: JobForbidden { fingerprints, constants } } with plain pub fields plus Job::new / entry_points / variants / forbidding; JobReport { id, home, matches: Vec<JobMatch { symbol, file, line, class }>, copies }; MatchClass { Home, Variant, Copy }.

Decisions:
- Grouping runs in Census::load, so a collision is a load error (Error::Collision naming both symbols). The builder keeps the full 32-byte digest for every candidate, singletons included; tested with forged prints in group.rs.
- A unit with a shim form is never body-grouped; it joins the shim group of its shim_key only when the key is Some (trait-impl and Self:: forwarders: no group). Body groups need print.body.
- groups() holds body and shim groups together, ordered count desc, tokens desc, fingerprint asc; body_groups/shim_groups filter it. Group tokens is the largest structural count among members.
- Hex tables: one HexTable per (unit, case), ordered by member then case. Shared constants: values above 0xffff held by units of two or more packages, ordered by value; members are every unit holding the value (decimal and hex spellings are one value).
- match_jobs: a unit matches by its body fingerprint, its shim fingerprint (the print's, not the key), a table: id of a table it holds, or a constant equal by value. rule: ids match nothing. Home = home or an entry point, then Variant, else Copy. Matches order by symbol, file, line; one report per job in input order.

Fixtures (tests/groups.rs): groups of one dropped; symbol order; min_tokens boundary 19/21; ordering over (3,21),(2,41),(2,31) and a fingerprint-tied pair; shim groups with the trait-impl method and Self:: forwarders excluded beside eligible neighbours; a 24-token forwarder in a shim group and no body group; all eight hex forms (str, byte str, [char;16], [u8;16], both cases); wrong digit, 15 entries, mixed case not reported; a function holding a table; shared constants (0x10000 in two packages, 65536 spelling, 0xffff not, one package not, value order); job acceptance (Home, Home via entry point, Variant, Copy, the 65536 const as Copy, copies 2); empty lists for no match, rule: ids and no forbidden items; shim and table matching.

Checks (exit): cargo test -p raf-census --locked 0 (21 unit, 19 forms, 4 golden, 14 groups, 20 units, 1 doctest); docs_contract+self_workplan+repo_gates 0; make fmt clippy doc 0; cargo workplan check 0. No timing, no mutation runs.

## Fix commit (review findings)

Commit: see `git log` (fix(raf-census): ...). Not pushed.
1. job.rs: the shim side of matching reads `unit.shim_key`, so a trait-impl method or `Self::` forwarder never matches a forbidden shim id; docs of `match_jobs` and `JobForbidden::fingerprints` say so. New test a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones: the three eligible forwarders match (Home plus 2 Copy, copies 2), the trait-impl method is absent, and a job forbidding the `Self::` forwarder's shim form matches nothing.
2. group.rs: `Candidate` carries a fingerprint and digest; shim units group by `shim_key` itself (digest from the print), so grouping and matching read one field.
Checks (exit): cargo test -p raf-census --locked 0 (21 unit, 19 forms, 4 golden, 15 groups, 20 units, 1 doctest); docs_contract+self_workplan+repo_gates 0; make fmt clippy doc 0; cargo workplan check 0. No timing, no mutation runs.
