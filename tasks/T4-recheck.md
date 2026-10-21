VERDICT: PASS

# T4 recheck (fix commit e02a444)

* Finding 1 (shim matching by print): fixed. `job.rs` now matches `unit.shim_key`; docs of `match_jobs` and
  `JobForbidden::fingerprints` state that trait-impl methods and `Self::` forwarders never match a shim id.
  Probe rerun in the clone at e02a444 (same source as before): `GROUP Shim [demo::p, demo::q]`, matches are
  `demo::p Home`, `demo::q Copy`; `S::make` no longer appears. New test
  `a_shim_id_matches_the_units_that_group_under_it_and_not_the_excluded_ones` covers the excluded neighbours.
* Finding 2 (shim_key used as boolean): fixed. `Candidate` carries fingerprint and digest; shim units group by
  `shim_key` itself, so grouping and matching read one field. Collision tests updated and still cover forged digests.
* `cargo test -p raf-census --locked`: exit 0 (21 unit, 19 forms, 4 golden, 15 groups, 20 units, 1 doctest).
* No timing or mutation runs.

Notes (non-blocking): the T4-report.md "Decisions" bullet for match_jobs still says "its shim fingerprint (the
print's, not the key)", which the appended fix section supersedes; one doc line in `match_jobs` exceeds the usual
wrap width (rustfmt does not touch comments).
