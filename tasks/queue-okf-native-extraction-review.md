<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Selective OKF native-primitives extraction review

Read-only source review of the strict checkout against origin/main, and native
APIs at RAF's pin `5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43` and local PurRDF
main. The inspected ECHAR, hash/hex/MMIX, allocation probe, RFC3339, numeric
and exact decimal files have no diff between that pin and local main. No
builds, product edits or scripts were run. This is an extraction boundary
review, not full delivery or gate acceptance.

## Concrete compatibility finding

`validate_timestamp("2026-13-01")` changes typed diagnostics: origin/main
reads the complete timestamp's lexical fields before range checks and returns
Malformed when `T`/time is missing. The donor's native parser checks month
range immediately and returns OutOfRange { field: "month" }. Similar mixed
lexical/range defects change precedence. The existing valid/range/syntax
fixtures do not cover this two-defect input. Preserve the original boundary
with a real regression fixture and neighboring complete out-of-range month
and valid timestamp before claiming error compatibility.

Native API gap: public rfc3339::{parse,parse_date,parse_time} all validate field
ranges while consuming grammar; Cursor::field/full_date/full_time are private.
No public pure date/time shape validator was found in lex terminals/scan or
xsd temporal. temporal::read_temporal/parse_datetime are value readers for
a different grammar; seconds_field_text is not a complete RFC shape check.
A native lexical-only RFC seam would address the precedence requirement.
Do not restore the removed generic RAF date parser or parse fabricated dates
to manufacture a shape certificate. This finding concerns invalid-input
diagnostic ordering, not a demonstrated accepted-grammar regression.

## Coherent selected packet

- `frontmatter/scalar.rs`: take only the closed ESCAPES table's native five
  ECHAR values plus explicit RAF 0/NUL extension, and native parse_double
  numeric quoting. Keep actual reader/writer/representability consumers of
  that same table. Native b/f/apostrophe escapes stay refused. Float quoting
  explicitly normalizes Rust's former inf/infinity/NaN spellings before native
  XSD parsing; RAF reserved words, YAML .inf/.nan and base-prefix quoting stay
  intact. No whole-YAML parser migration or scalar normalization is implied.
- `profile/timestamp.rs`: native RFC parse plus RAF uppercase-T/Z policy,
  second-60 refusal and day-field naming adapter. Native accepts arbitrary
  fraction length while retaining only nanos internally, but RAF validates
  and discards the instant, so authored timestamp text is not truncated.
  Native year 0000 and numeric offsets match the old validator's ranges.
  The diagnostic precedence finding above must be resolved separately.
- `profile/typed.rs`: only Number's retained native Decimal, parse_decimal,
  borrowed exact(), to_f64 and Ord comparison, plus removal of the local
  private Decimal comparator. Retain the original narrower grammar: no plus,
  exponent, leading/trailing whitespace, leading dot or trailing dot. Text
  equality remains authored spelling equality. Native numeric zero erases
  sign, so copysign from the retained text is necessary for value()'s negative
  zero compatibility. Large exact integers/fractions and overflow-to-infinity
  display remain supported. exact() exposes a native public type intentionally;
  it does not require the strict line-percentage consumer to ship with it.
- `profile/records.rs`: only remove is_lower_hex and replace digest/commit
  predicates. decode_32_canonical accepts exactly 64 lowercase hex bytes and
  returns an inline array; commit keeps the exact 40-or-64 byte length gate
  and native nibble_canonical. No generic hash kernel, allocation or RDF model
  is introduced. Exclude every new debt/ceiling/line-row function and import.
- Tests: native CountingAllocator/CurrentThreadWindow in bounds/alloc_bound,
  and native lcg64_next with MMIX increment in body_robustness. The multiplier,
  increment, seed, >>33 projection, 20,000 cases and token alphabet are unchanged;
  this preserves the deterministic corpus rather than inventing a new one.

## Assertion ownership and caps

Inspected refusal fixture independently pins all six escape pairs and exact
UnknownEscape (2,5) for native b/f/apostrophe, with a literal-apostrophe valid
neighbor. Preserve existing six-output writer roundtrip. The constructor
style fixture includes +1e2, .5, 1., INF, -infinity, +NaN, -.Inf plus 1.5x and
infinite plain neighbors, and actual render/parse identity.

Keep schema's standalone valid_timestamps_are_accepted and
malformed_and_out_of_range_timestamps_are_refused assertion bodies, including
uppercase/ASCII shape restrictions, day/month/leap-year and offset fields,
real month-end/offset leap-second refusal with :59 neighbors. Do not copy the
whole modified schema test inventory: its Debt/removed-strictness assertions
belong to the other delivery.

Keep records' a_digest_is_sixty_four_lowercase_hex_digits,
a_revision_is_a_full_commit_id, a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float,
numbers_compare_exactly_by_sign_whole_part_and_fraction and Number doctests.
They assert exact typed errors, uppercase/length/nonhex refusal, 40/64 valid
commit neighbors, numeric ties/long fractions/400-digit values and signed zero.
Missing useful narrow neighbor: direct Number::value comparison for a
subnormal/underflow case and a halfway rounding case, since storage changed
from Rust f64 parse to native exact conversion. Native documentation promises
correct rounding; this review did not independently execute that claim.

`bounds.rs` retains Cap<CountingAllocator> at the original process-wide 1 GiB,
the same 1 MiB body/60 KiB heading, 4-times-heading coefficient and 64 KiB
allowance. alloc_bound retains 8-times-input plus 16 KiB for actual anchor
and late-alias refusal. Inputs are constructed before the window, results
remain alive at closing, and the measured parse paths are synchronous on the
calling thread. Native per-thread windows intentionally exclude sibling harness
allocations; no parallel-parser or whole-process ownership claim is made.
The old alloc_bound Cap<System> had usize::MAX, not a meaningful hard cap;
removing that wrapper does not remove the separate bounds.rs 1 GiB protection.

## Dependencies and excluded coupling

Production needs pinned purrdf-hash, purrdf-lex and purrdf-xsd; dev needs
purrdf-alloc-probe and the existing cap. Native hash has no runtime dependencies,
lex uses hash, xsd uses hash+lex, and alloc-probe has no dependencies. This
closure does not require purrdf-core/RDF, Census, Workplan, source settling or
new strict policy. The manifests describe the foundations as wasm-clean;
no wasm build was executed here. Keep root/fuzz lock consistency as required
by actual manifests, without importing unrelated native RDF dependency entries.

Do not copy whole typed.rs/records.rs, whole strict records/schema fixtures,
DEBT_SLUG, ChangeItem.reason, Debt types, removal of strictness schema, new
record kinds or strict policy docs. The native primitives and selected actual
neighbors can be applied independently to origin/main. Tests using the old
record profile remain the relevant extraction context. Existing owner-reported
strict passes qualify that donor snapshot only; the extracted snapshot needs
its actual caller checks and required gates.

## Required timestamp precedence adapter

The proposed closed authored-shape byte guard is justified profile policy in
the absence of a native shape seam, provided its phase matches the old API.
A whole timestamp template checked before native range validation is too
strong: `2026-13-01T00:00:00` was OutOfRange(month), since the old validator
checks ranges before timezone grammar. A whole-template guard would instead
make it Malformed. Similarly `2026-01-01T00:00:60oops` was OutOfRange(second)
before reading the invalid timezone; native RFC parsing reads the timezone
before its leap-second judgement.

Smallest source-derived adapter preserving these phases:

1. Check only the fixed ASCII authored date/clock shape through seconds, and
   require at least one digit if the immediate suffix starts a fraction.
   Do not validate the remaining timezone here. This retains the old
   lexical-before-range prefix ordering, including `2026-13-01` -> Malformed.
2. Invoke native RFC parsing once. Preserve native OutOfRange for month/day/
   hour/minute/second, which represent the old pre-timezone field order.
3. Once those earlier field failures are excluded, authored seconds `60`
   produce RAF OutOfRange(second), overriding a later native timezone syntax
   or offset-range failure. This is the RAF no-leap policy, not calendar math.
4. Otherwise map the native timezone/remaining failures normally, retaining
   day-of-month -> day naming and the uppercase profile boundary.

No numeric conversion, leap-year/calendar table, fabricated date, duplicated
native value parser or second native parse is needed. Required literal
neighbors include the three precedence examples above, complete bad month,
bad month plus seconds 60, valid date plus seconds 60/bad timezone, and a
fully valid timestamp. This is a proposed bounded repair, not an inspection
or execution claim about code that has not yet landed.

## Implemented phased adapter recheck

Read the actual issue 53 clock_suffix/offset_shape/validate_timestamp delta.
The concrete diagnostic finding is closed in inspected source. Fixed ASCII
date/clock and nonempty fraction checks precede native validation, but leave
timezone suffix validation to its later phase. Native earlier field ranges
retain priority; the RAF seconds-60 override precedes timezone shape/range
mapping. offset_shape also preserves the old complete-offset lexical-before-
offset-range ordering, such as +99:xx -> Malformed and +99:00 -> offset hour.
Only policy spelling/precedence is local; ranges/calendar/leap computation
remain native and the parser is invoked once. Fraction slicing follows a run
of ASCII digits and therefore preserves UTF-8 boundaries.

Inspected all 17 exact refused identities and five valid neighbors in
schema::timestamp_diagnostics_preserve_lexical_calendar_and_offset_order.
They include prefix/fraction syntax versus bad month, leap versus bad suffix,
month versus leap and offset lexical/range precedence. No remaining concrete
API/output/error regression was found in this narrowed repaired delta.
Execution is pending collection by the primary; this reviewer ran no build.
