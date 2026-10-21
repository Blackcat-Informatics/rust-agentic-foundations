# T2 report

Status: DONE. Commit 08071b8 (not pushed).

Files (crates/raf-census): src/lib.rs (exports, crate docs + doctest), src/normalize/{mod,form,passes,literal,shim,fingerprint,words}.rs,
tests/{forms,golden}.rs, tests/support/{mod,fixtures}.rs, examples/goldens.rs.

Public API: Normalizer::new(&CensusOptions) with tower/print/explain; Slice::new(file, first_line, text).params(names in signature order);
Tower (form(Level), structural_tokens, shim_candidate, print(min_tokens)); Print{tokens, body, shim: Option<Hashed{fingerprint, digest[32]}>};
Level (verbatim, shim, literal-tier, structural, value); Divergence{differs_through, equal_from, first: TokenDiff}; Fingerprint enum
Body|Shim|Table(TableKind)|Rule(RuleId) with FromStr/Display and ParseFingerprintError; manifest(), ENCODING, PASSES, thresholds, VALUE_ADAPTERS.

Decisions:
- Input is the body contents (no outer braces), as purrdf lexed. Lex failure, nesting over 128, or a slice over 2 GiB is Error::Source (no fallback).
- Levels are the forms after each pass: verbatim, rename_params (shim form), rename_locals (literal tier), kind_literals (structural), erase_held (value).
  Thresholds count structural tokens (before erase_held); body print hashes the value form; shim print hashes the shim form.
- Integers are respelled to decimal value + suffix at lex time (0xff = 255, 1_000 = 1000), so kind_literals is the true identity below 30 tokens.
  Integer kind by syntax: beyond u128 is $int.
- Params are `$0..` in signature order (self and repeats dropped); locals continue after the highest parameter number the body uses
  (not params.len()), so the tower is monotone for every pair regardless of unused trailing parameters. Rev 2 wording said one counter; this
  keeps one counter but makes locals independent of unused trailing params.
- Names inside tokens the value form erases (let annotations, turbofish, ref) are not numbered (`$_` at structural), so erasures never shift numbering.
- Format placeholders ({x}, {x:02x}) in string literals that are direct arguments of a macro call are rewritten through the param map and the local map;
  {}, {0}, unknown names, {{ and \u{..} are kept.
- Deviations from the port: `_` and weak keywords (gen union raw safe macro_rules) are kept; a range end (`..1`) is a literal, not a tuple index;
  field names after `.`/in struct braces are not renamed even if equal to a parameter.
- Shim candidate predicate ported exactly (top-level `let`/`;`, has_call, no self). shim_key exclusions are T3.
- Hash: u32 LE length + bytes per token, BLAKE3, first 8 bytes big-endian, `{:016x}`. Hand-computed digest pinned from b3sum.

Registry: tests/support/fixtures.rs (187 fixtures, 5 more pairs than at first commit (the table has 187 lines)). golden.rs pins MANIFEST and TABLE with the bump header and fails on
unpinned entries, orphan pins and moved values. forms.rs checks every pair, token counts of sized fixtures, monotonicity over all fixture pairs,
shim 5/6/29/30, tier 25/29/30/35, kept binary &/*/&&, parameter numbering, fail-hard with valid neighbours, depth 128/129, explain, digest, parse refusals.

Checks (exit): cargo test -p raf-census --locked 0 (12 unit, 19 forms, 4 golden, 1 doctest); example goldens 0 (no arguments);
docs_contract + self_workplan + repo_gates 0 (run with files staged); make fmt clippy doc 0; make reuse issue-refs 0;
cargo workplan check 0. docs_contract needed no bindings now (capability state implementing; bindings are T6).

## Fix commit (review findings)

Commit: see `git log` (fix(raf-census): number locals after every parameter a body uses, placeholders included). Not pushed.
- Finding 1: passes.rs next_number also reads `{$n` placeholders (placeholder_numbers); fixtures numbering_placeholder_parameter (+ _above_tier,
  _versus_argument) Distinct. No existing pin moved; 10 new lines added to the table (187 fixtures).
- Finding 2: golden.rs pin_problems(computed, pinned) used by every_fixture_fingerprint_is_pinned and by
  the_pin_check_names_an_unpinned_orphan_moved_and_duplicated_entry (neighbour passes; unpinned, orphan, moved, pinned twice, computed twice).
- Finding 3: numbering_unused_trailing_parameter (Equal(Verbatim)) and numbering_used_trailing_parameter (Distinct).
Checks (exit): cargo test -p raf-census --locked 0 (12/19/4/1); goldens example 0; docs_contract+self_workplan+repo_gates 0;
make fmt clippy doc 0; cargo workplan check 0.
