VERDICT: PASS

# T3 recheck of fix commit e1073f9

Commands: `cargo test -p raf-census --locked` exit 0 (18 unit, 19 forms, 4 golden, 20 units, 1 doctest);
`make clippy` exit 0 (only the T1 unused serde/serde_json manifest warnings, as before); probe clone at e1073f9
(`cargo test --test probe`, 4 probes) exit 0. No timing or mutation runs.

1. Turbofish: CLOSED. `has_call` now treats a balanced `::<...>` (arrow-aware, depth cannot underflow because it
   starts on `<`) as part of the name. Probe: `helper::<u8>(x, 1)` now has shim and key; `Self::inner::<u8>` has
   shim, no key (units test asserts it). New fixtures: forwarder pair (Equal Shim), method turbofish, type differs
   (Equal Value, shim differs), turbofish without a call (no shim).
   Golden diff check: the only changed pre-existing lines are value_type_turbofish.a, value_const_turbofish_kept.a,
   value_bool_turbofish_kept.a and .b, each gaining a shim because its body is a turbofish call; every body
   fingerprint is unchanged, the other fixtures are untouched, the manifest is unchanged, the rest are added lines.
2. `Self::` at any depth: CLOSED. Probe: nested argument call, nested callee argument and `Self::V(x,1)` are
   via_self with no key; `Vec::<u8>::new` keeps its key; test a_self_call_at_any_depth_removes_the_shim_key pins them.
   (T6 still has to state the rule on the page, as recorded in the review.)
3. Silent fallbacks: CLOSED. Body must be a braced block, the initializer range and newline count go through
   `outside`, all `Error::Source`; no `unwrap_or_default`/`saturating_sub` remain; unit test beside a valid neighbour.
4. String literals: CLOSED. Probe: `b"\xff\xfe"` is kept (Bytes, lossy, never names a table), `c"hi"` kept (CStr),
   lines 13/14/15/17 correct; `StringKind` exported; rule documented and pinned by a test beside a valid table.
   Repository census: 6329 units, deterministic across two loads, no string before its unit's line.

No new findings.
