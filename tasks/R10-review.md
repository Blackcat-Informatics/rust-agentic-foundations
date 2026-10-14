# R10 review: manifest and error-text hardening (S-9, S-10)

Reviewed: uncommitted change in /tmp/raf-par/p3-r10 (base e472918). The ResolveError part of S-10 is R5d's and is out of scope.

## Scope
error.rs (new `Esc`, applied to every user-derived field of TreeError, Origin, TargetProblem, LoadErrorKind, LoadError), glob.rs (Hash, `literal_key`, `matches_prefix` over pre-collected chars), manifest.rs (MAX_MEMBERS/MAX_EXCLUDES = 4096, typed Manifest error naming the key), members.rs (dedupe, `Excludes` literal set + wildcard list), tests error_text.rs and manifest_caps.rs, source-index.md (Contract, Verification, bindings).

## Evidence
- Gates: `cargo test -p raf-source --locked` debug and `--release` all pass (45 lib, plus error_text 2, manifest_caps 6, all other suites); workspace `clippy --all-targets -D warnings` clean; docs_contract (139) and repo_gates (55) pass; `make reuse` compliant (226/226).
- Exclude semantics: differential fuzz of old (e472918 src) vs new on 4000 random synthetic workspaces (literal, `*`, `?`, `[ab]`, nested, duplicate members/excludes; 108098 per-package loads, 6416 ok / 101682 typed errors): 0 differences. Real trees, loading every Cargo.toml directory: this repo (4), purrdf (49), lillith_brain (8), lillith_memetics (22), lillith_decodes (120), katamari (43): 0 differences. Literal excludes are matched per ancestor prefix, equal to the old prefix match; a segment with `[..]`, `?` or `*` is a Set/One/Run piece and stays on the wildcard path, so no pattern is mis-classified as literal.
- Caps: constants are exactly 4096; tests show 4096 loads and 4097 refused for both keys, with the message "has N entries; at most 4096 are allowed", file Cargo.toml, origin ManifestKey. Passing.
- Timing (release, probe in /tmp/raf-par/p3-r10-probe): 880 KB probe (20000 members, 200000 excludes) refused in 2 ms. 4096 real member dirs against 4095 distinct non-matching wildcard excludes (`zzN/*/x*`) loads in 51 ms. Adversarial star-heavy distinct wildcards (`*c*r*a*t*e*s*N`, first segment matched against `crates`) with 4096 dirs x 4095 patterns: 1.07 s (about 60 ns per match, linear in dirs x excludes, same cost as the old code, which was also unbounded in the number of exclude entries).
- Escaping: esc_term-style probe with ESC, BEL, CR, NEL (U+0085), ZWSP, U+202E, U+2066, U+FEFF in a `#[path]`: output is fully escaped (`\u{1b}`, `\u{7}`, `\r`, `\u{202e}`, `\u{200b}`, `\u{2066}`, `\u{feff}`, `\u{85}`), so `char::escape_debug` already covers bidi overrides and other format characters (they are non-printable per Rust's tables). No extra bidi handling needed. Raw fields are unchanged (Display only).
- Falsification: evidence file shows base failures and 3 mutations. I re-ran one: removing `Esc` from `UnresolvedTrait` in a copy makes `every_user_derived_field_of_every_display_is_escaped` fail while the other tests pass.
- Docs: Contract, Verification and the 10 new bindings are accurate (docs_contract passes; numbers match the tests).

## Findings
REQUIRED: none.

ADVISORY
1. `PathError`'s public `Display` (path.rs) still prints `input` raw. It is not reached through `LoadError` (PathError converts to `TreeError::OutsideRoot`, which is escaped), so S-10 as scoped is met, but a caller printing `RelPath::new(user).unwrap_err()` gets raw control characters. Consider escaping it in the same way (follow-up or a one-line change).
2. The doc/test claim "4096 distinct patterns of each kind load in well under a second" holds for the fixtures, but an adversarial star-heavy set over a 4096-directory tree takes about 1.07 s in release (linear, not a regression). Either soften the wording ("in about a second at worst") or cache the first-segment match per segment name; not needed for the gate.
3. `Esc` doubles backslashes (`\` becomes `\\`), so a path with a backslash displays differently from the raw value; this is the correct unambiguous choice but the Contract line could say "backslash is doubled".
4. `NonUtf8Name` now hand-writes quotes instead of `{name:?}`; behaviour is equivalent for the shown fields.

VERDICT: PASS

## Orchestrator follow-up on the advisories

1. `PathError`'s `Display` now escapes its input through `crate::error::Esc`. Test `path::display_tests::control_characters_in_a_refused_path_are_escaped` covers it. The mutation that removes the escaping fails the test, and it was reverted.
2. The Contract's pattern-cost sentence now gives the measured figures: about 50 ms for 4096 directories against 4095 wildcard excludes, and about 1 s for an adversarial star-heavy set. The old "well under a second" claim is gone.
3. The Constraints note that `escape_debug` also doubles a backslash.
4. Accepted as is: `NonUtf8Name` writes its quotes by hand, with equivalent output.

After the follow-up: workspace clippy with `-D warnings` is clean, and the full raf-source suite, docs_contract (139) and repo_gates (55) pass.
