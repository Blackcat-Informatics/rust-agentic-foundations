VERDICT: BLOCKED

# T4 independent review (commit e9ea57e)

## Scope
`crates/raf-census/src/{group.rs,job.rs,census.rs,lib.rs}` and `tests/groups.rs` at e9ea57e, read against plan
Rev 2 > Rev 1 (Ordering, Collisions, Neighbours, Group and file context, Shim candidate, Job matching B1,
Entry-point fixture; rows R5, R8, R9), `docs/capabilities/census-ledger.md` (HOME/VARIANT/COPY) and purrdf
`helper-census/src/census.rs`. Worktree not edited; probes ran in a clone at `scratch/t4review` (a throwaway
`tests/probe.rs`, not committed anywhere).

## Commands
* `cargo test -p raf-census --locked`: exit 0 (21 unit, 19 forms, 4 golden, 14 groups, 20 units, 1 doctest).
* `make clippy`: exit 0 (two unused-dependency warnings for `serde_json` and one more crate, awaiting T5; not T4 code).
* Probe in the clone (`cargo test -p raf-census --locked --test probe -- --nocapture`): exit 0, output quoted in finding 1.
* No timing runs, no mutation runs.

## Adjudications
(a) Job matching by the print's shim fingerprint, not `shim_key`: WRONG as built (finding 1).
(b) Grouping in `Census::load` (`census.rs` calls `group::group(&units)?`): correct; a collision is `Error::Collision`
    naming both symbols and fails load. Digest is kept for singletons (the map holds every candidate before the
    `>= 2` filter), and the unit test with a third unrelated candidate proves it. Caveat only: units that are never
    candidates (shim-form units with `shim_key == None`) are not collision-checked; acceptable because they are
    never compared.
(c) Hex tables per unit and case: correct. One `HexTable` per (unit, case); derived `Ord` gives member then case;
    8 forms, wrong digit, 15 entries, mixed case, and function-held table all have fixtures with eligible neighbours.
    Shared constants: `> 0xffff`, >= 2 distinct packages, value ascending, members sorted; 0x10000/65536 one value;
    0xffff and one-package neighbours present.
(d) `Job::new` and builders: acceptable. Plain pub fields as planned; builders replace (not append) and the docs say
    "with `symbols` as ...", which is true. Acceptance fixture of Rev 2 matches exactly (Home, Home via entry point,
    Variant, Copy, 65536 constant as Copy, `copies == 2`, line 5 for BIG, empty list for no match, `rule:` matches nothing).
Determinism: group sort is total (count desc, tokens desc, fingerprint asc, kind asc; `(kind, fingerprint)` is unique),
members and hex tables and constants are sorted/BTree-ordered, matches sort by (symbol, file, line); the only
insertion-order dependence is the "first" symbol in `Error::Collision`, which follows the deterministic unit order.
No silent fallbacks found in T4 code.

## Findings

1. BLOCKING. `crates/raf-census/src/job.rs:78-96` (`Job::matches`), doc at `job.rs:148-151` and `job.rs:14`.
   Shim matching uses `print.shim` instead of the unit's `shim_key`. Grouping (`group.rs:179-189`) deliberately
   keeps trait-impl methods and `Self::` forwarders out of every shim group (R5, plan "Shim candidate": they are not
   clones). Matching then contradicts that. Probe (source: free fns `p`, `q` forwarding `helper(_, 1)`, plus
   `impl Make for S { fn make(x) { helper(x, 1) } }`, `S::via` and `r` forwarding via `Self::inner`):
   `GROUP Shim [demo::p, demo::q]` (make is not in it), then a job forbidding that group's fingerprint with home `p`
   yields `MATCH demo::S::make Copy`, `demo::p Home`, `demo::q Copy`. Consequences against the ledger contract:
   COPY is "any other match of the job's forbidden entries"; the forbidden shim id is a census id read off a shim
   group, so the group and the job disagree about what the id denotes; an enforced job would fail the gate on a
   method that the trait obliges and nobody can remove; `copies` is inflated; and the staleness rule "a `shim`
   fingerprint matches no unit" is masked when only an excluded trait-impl/`Self::` unit still carries the form.
   (purrdf matched by the print because it had no exclusion and grouped those units; the port added the exclusion but
   not its counterpart in matching.)
   Required fix: match the `shim` side by `unit.shim_key` (`unit.shim_key.as_ref().is_some_and(forbids)`), keep the
   body and table sides as they are; update the docs of `Census::match_jobs` and `JobForbidden::fingerprints` to say
   a shim id names the units that group under it (trait-impl methods and `Self::` forwarders are excluded);
   add to `tests/groups.rs` a fixture with the trait-impl method and a `Self::` forwarder beside eligible forwarders
   asserting they are absent from the job's matches (the existing `jobs_match_shim_and_table_fingerprints` has no such
   neighbour, which is why it passed).

2. Minor, fold into the fix of 1. `crates/raf-census/src/group.rs:182` binds `(Some(_), Some(hashed))` on
   `unit.shim_key` but then groups by `print.shim`'s fingerprint, so `shim_key` is used as a boolean and the two values
   are only equal by construction in `units/mod.rs:254-258`. Group by `shim_key` itself (or assert equality) so grouping
   and matching read one field and cannot drift. No behaviour change today.

No other findings. Everything else required by the plan rows R5 (T4 half), R8, R9 and the Rev 2 job acceptance is
present with fixture and neighbour, and the public docs are true apart from the shim wording in finding 1.
