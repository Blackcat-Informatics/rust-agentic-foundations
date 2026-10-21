# Enhancement audit: issue 3, raf-census core: normal form v1

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`

Inputs read: issue.md, issue-analysis.md, prior-art-assessment.md, AGENTS.md, CLAUDE.md,
`docs/architecture/normal-form.md`, all nine `docs/capabilities/census-*.md`,
`work/changes/census-normal-form.md`, `crates/raf-source` (`lib.rs`, `item.rs`, `workspace.rs`,
`facts.rs`, `budget.rs`, `Cargo.toml`), and the port source
`/home/paudley/Active/purrdf/crates/helper-census` (`normalize.rs`, `census.rs`, `source.rs`).
No `.goals` file exists; the standing constraints are AGENTS.md (fail hard, no fallbacks,
normal-form stability, no timing runs, no mutation passes). No item below conflicts with them.

Domain framing used for the TRANSFORMATION altitude: the clone-detection taxonomy (Type-1
textual, Type-2 renamed, Type-3 near-miss, Type-4 semantic), token normalisation with consistent
renaming (CCFinder, NiCad), and near-miss fingerprinting by winnowed k-grams (Schleimer,
Wilkerson, Aiken). The issue's four "forms" are a shadow of an **abstraction tower**: a chain of
erasures ordered by how much they identify, with a monotonicity law. The plan ports the tower as
four hand-wired functions and loses the law, the witness and the substrate that the next three
census changes (patterns, detectors, migration) need.

## Items

### TRANSFORMATION

1. **Build the normal form as an ordered tower of erasure passes over a span-carrying token
   tree, with the monotonicity law tested and a divergence witness exposed.**
   The forms are points on one lattice: verbatim ⊑ shim (own parameters renamed) ⊑ literal-tier
   structural (locals renamed, literals kept) ⊑ structural (literals kinded) ⊑ value (held-ness
   erased). Design `normalize` as a `Form` type (token tree, each token keeping its source byte
   range) and a fixed sequence of pure passes (`rename_locals`, `kind_literals`, `erase_held`,
   `rename_params`), with the fingerprint taken from the final flattened sequence. Three things
   fall out: (a) a generic property test over **every** fixture: equal at a finer level implies
   equal at every coarser level, and a token count that never grows, which catches a pass bug no
   hand-written pair happens to exercise; (b) `Normalizer::explain(a, b) -> Divergence` naming
   the coarsest level at which two bodies still differ and the first differing token with its
   source position, which is exactly the evidence the page's "Conditions to revisit" asks for
   and what a ledger author needs to answer "why are these not one group"; (c) the substrate the
   designed capabilities already assume: `cap:census-patterns` matches `tokens` "against the
   normalized tokens" with `$_` standing for "one token or one balanced group" (impossible on
   purrdf's flat `Vec<String>` of `"("`/`")"` strings), and `home-literal` / `vocabulary-literal`
   report a literal's line (needs the span). The hashed bytes are unchanged from what the plan
   would produce, so no fingerprint moves relative to the plan; only the shape of the code does.
   Also derive a **pass manifest** (the ordered pass ids and the thresholds) and pin it as one
   golden beside the fixture fingerprints, so a change to a pass that no fixture reaches still
   fails until `normal_form` is bumped.
   Cost: L. Payoff: decisive; it is the structure the next three issues in this area would wish
   existed, and it turns the version-bump rule and the revisit conditions from prose into checks.
   Where: Task 2 ("`normalize` module: token source ..., structural form, value-form erasures,
   literal tier, ..., shim form, `Fingerprint` type") and NEEDS-JUDGMENT 4 ("the value form is
   the structural form with the value-form erasures applied ... The structural form is an
   intermediate").

2. **Near-miss (Type-3) grouping by winnowed k-grams of the value form, as a report section,
   not a fingerprint kind.** The exact-hash groups find Type-1/2 clones only; the page's first
   "condition to revisit" (same algorithm, one statement reordered or added, reported as
   distinct "often enough that the census misses the duplicates it exists to find") is the
   Type-3 class, and the state of the art answers it with winnowing: hash every k-gram of the
   value-form sequence, keep the minimum of each window, and report pairs whose selected-hash
   overlap passes a threshold. Fingerprint kinds stay four (the issue fixes them); `near` is a
   baseline-report member with the pair, the overlap and the two `body:` ids, and it is
   advisory (never a ledger finding). `k`, the window and the threshold are fixed by the normal
   form version. Cost: L. Payoff: the census's largest miss class becomes visible from the
   first run, and the ratchet and ledger of later changes gain a candidate list for free. If
   the planner moves this to a tracked change, it needs its own record before this change
   completes (AGENTS "no deferrals"); "beyond issue scope" alone is not a disposition.
   Where: Task 4 (`group` module) and Task 5 (`Baseline`); `arch:normal-form` "Conditions to
   revisit".

### LEVERAGE

3. **`span-locations` is already on in this workspace; the plan says off.** Task 1 states
   "`proc-macro2` (span-locations off, default features)". `crates/raf-source/Cargo.toml:18` and
   `crates/raf-workplan/Cargo.toml:15` both enable `features = ["span-locations"]`, and cargo
   unifies features, so the census gets it whether or not it asks. Ask for it explicitly and use
   `Span::byte_range()` on the re-lexed body plus the body's `SpanFact::start_byte` to give
   every string literal its absolute line (R7 "strings"; `home-literal` and
   `vocabulary-literal` report lines), and to carry the spans item 1 needs. Spans never enter
   the hash. Cost: S. Payoff: removes a false statement from the plan and makes R7's strings
   usable by the detectors change. Where: Task 1, the `proc-macro2` dependency line; Task 3
   "string literals".

4. **Deliver the index report's reader now, closed and round-tripped.** `cap:census-migration`
   reads "the index report that the older version wrote" to map fingerprints through member
   symbols. A report that only serialises is a schema that is defined by accident. Give
   `Index` and `Baseline` `Deserialize` with `deny_unknown_fields`, a `from_json` entry point,
   and a test that `to_json` then `from_json` yields an equal value (plus one negative fixture
   with an unknown key). Cost: S. Payoff: migration consumes a contract instead of re-deriving
   one; the report shape becomes a tested part of `normal_form`. Where: Task 5 (`report`
   module).

5. **One `Group` type shaped for every downstream consumer, plus the per-file context the Rule
   API names.** `Group { kind: Body|Shim, fingerprint, tokens, members }` with a
   `member_symbols()` that returns the sorted set, used by the baseline and index reports here
   and designed for the ledger's `[[distinct]]` exact-set match ("the census group of its kind
   whose member set equals `members` exactly") and the debt record's `{ "kind", "members" }`.
   Add a `Census::unit(symbol)` lookup (the `fingerprint <SYM>` command and migration's member
   lookup both need it). Define the per-file descriptor (`package`, repository-relative path,
   `shipping`) as the type `cap:census-rule-api` calls `FileCtx`, since `units::collect`
   computes those three fields for every unit anyway. Cost: S. Payoff: the ledger, ratchet and
   rule-api changes build on types instead of re-shaping JSON. Where: Task 3 (`units`) and
   Task 4 (`group`).

### UTILITY

6. **Rename inline format captures inside string literals at the literal tier.** The headline
   promise ("renamed locals collapse") fails for the most common modern idiom in small bodies:
   `format!("{name}: {count}")` and `format!("{n}: {c}")` differ under 30 tokens because the
   literal is kept verbatim and the captured locals live inside it. Rewrite `{ident}` /
   `{ident:spec}` placeholders in string literals (locals only; `{}` and `{0}` untouched) through
   the same rename map before the literal is emitted. The current R1 acceptance check is also
   under-specified: "`format!("{x:02x}", x = b)` vs renamed local gives one value form" holds
   only if the renamed local is `b`, not `x`, and only at or above the literal tier; state the
   two fixtures (renamed named argument, renamed inline capture) and the tier each sits in.
   Cost: M. Payoff: closes the largest Type-2 gap in v1 before the version is pinned (fixing it
   later costs a `normal_form` bump and a migration). Where: Task 2; completeness-contract row
   R1.

7. **A golden regeneration route.** The version-bump rule says the goldens are "regenerated in
   the same change", and tests must not write tracked files. Add `examples/goldens.rs` that
   prints the fixture-to-fingerprint table in the exact form `tests/golden.rs` pins, so a bump is
   a paste, and the pass manifest of item 1 regenerates the same way. Cost: S. Payoff: the bump
   path is mechanical and reviewable. Where: Task 2 (`tests/golden.rs`).

### ROBUSTNESS

8. **Deduplicate units by source identity; the plan as written reports a file as a clone of
   itself.** `raf_source::Item` docs: "A file that several module trees reach has one `Item`
   for each path it is reached under"; `Workspace::items()` yields them all. A module shared by
   `src/lib.rs` and `src/main.rs` with `mod shared;`, or a `#[path]` include, produces two
   items with one body and two symbols, so Task 3 "units module over `Workspace::items`" makes
   every such function a two-member `body` group. Collapse by `Item::source_key()`
   (`Workspace::items_by_source` gives the groups), choose the canonical path deterministically
   (a shipping path over a test path, then the smallest path), and record `shipping` as the OR
   over the reaches. Fixture: a workspace whose lib and bin both declare `mod shared;` yields
   one unit and no group. Cost: S. Payoff: removes a false positive that would hit most
   real workspaces on the first run. Where: Task 3.

9. **Const and static units have no token source in the plan.** `Item::body` is `None` for
   `Static` and for every `Const` (`crates/raf-source/src/facts.rs` passes `None` at the
   `ItemKind::Static` and both `ItemKind::Const` call sites); only functions carry a body span.
   Task 3 lists "const, static ... integer constants by value, string literals, table
   candidates" without saying where the tokens come from. Either slice `Item::span` and take
   the tokens after the first depth-0 `=` (the type must not contribute: `const T: [u8; 0x10000]`
   is not a shared large constant), or add an initializer span to `raf-source` (its structs are
   `#[non_exhaustive]`, so the field is additive). Fixture: a const whose type names `0x10000`
   and whose initializer does not, in two packages, is not reported. Cost: S for the slice,
   M for the raf-source field. Payoff: R7 and R9 are unimplementable as planned without this.
   Where: Task 3.

10. **State how parameter names are obtained, and cover pattern parameters.** The shim form
    renames "only the function's own parameters" (R5) and purrdf reads them from
    `syn::Signature`; `raf_source::Item` carries no signature. Name the mechanism: parse the
    text from `span.start_byte` to `body.start_byte` with the re-exported `raf_source::syn`
    (same version as the index, as `lib.rs` promises), and collect every bound identifier,
    including `mut x`, `ref x`, `(a, b): (u8, u8)`, `Point { x, y }: Point`, and `self`.
    Fixtures for a tuple-pattern parameter and a struct-pattern parameter. Cost: S. Payoff: a
    shim whose parameter is a pattern is otherwise never renamed and never groups. Where: Task 3
    "method (with `trait_impl`, `via_self` for shim_key)".

11. **A re-lex failure is a hard error, not an empty body.** purrdf's `relex`
    (`source.rs:916-927`) does `TokenStream::from_str(slice).ok()).unwrap_or_default()`: a slice
    that fails to lex becomes an empty body with a valid fingerprint. AGENTS forbids the
    fallback. Make it a typed error that names the file and line, and test it by calling the
    normaliser on an unlexable slice directly (a loaded workspace cannot produce one, since
    `raf-source` parsed it). Cost: S. Payoff: fail-hard contract honoured in the one place the
    port violates it. Where: Task 2 "token source (re-lex body text with `proc_macro2`)".

12. **Specify the hashed byte encoding normatively and make it injective.** purrdf hashes
    `form.join(" ")`. `arch:normal-form` says only "the BLAKE3 hash of the form's token
    sequence"; the sequence-to-bytes encoding is part of the version and is unstated, so R11
    ("the normative description of v1") is incomplete. Since v1 is fresh, use a length-prefixed
    encoding (or a separator byte no token can contain) so distinct sequences hash distinct
    bytes by construction, and write the encoding into the page's Fingerprint section with one
    fixture that pins the digest of a known sequence computed by hand. Cost: S. Payoff: the
    normative text becomes complete and the injectivity argument becomes a construction.
    Where: Task 2 (`Fingerprint`), Task 6 (`arch:normal-form`, Fingerprint section).

13. **Detect 64-bit collisions instead of silently grouping them.** The page's third revisit
    condition is "two different token sequences are found with the same 8-byte fingerprint". The
    grouping already has the full form per unit; when two units share a fingerprint and their
    full 32-byte digests differ, fail hard naming both symbols. Fixture: two units given forged
    prints through the group builder's internal entry point. Cost: S. Payoff: the condition is
    observed on the run it happens instead of being a false group in a ledger. Where: Task 4.

14. **Pin the lexical edge cases before v1 is frozen; each later fix is a version bump.**
    (a) `gen` is a reserved keyword in edition 2024 and is absent from purrdf's `KEYWORDS`, so
    it is renamed as a local; (b) the weak keywords `union`, `raw`, `safe` and `macro_rules`
    are renamed as locals (`&raw const x` in one body and `&raw const y` in another agree by
    luck, `union U {}` declared in a body half-agrees); (c) an integer literal above `u128::MAX`
    falls to `$float` because the kind is decided by value, not by spelling; decide `$int` by
    syntax. Decide each, and pin the decision with a fixture. (d) Nesting: `structural_level`
    recurses once per group; `raf_source::Budget::Delimiters` (128) bounds the depth before the
    census sees the file, so state that reliance and add one fixture at 128 levels so a change
    to either side is caught. Cost: S. Payoff: no surprise bump in the first month of use.
    Where: Task 2.

15. **Thread `min_tokens` through the entry point and refuse 0.** The completeness contract's
    real entry point is `raf_census::Census::load(&tree, roots)`, but `min_tokens` is a ledger
    setting ("20 by default, set by `[census] min_tokens`") and the baseline report prints it;
    the API has nowhere to take it. Add `CensusOptions { min_tokens }`, fail hard on 0 (every
    empty body would otherwise be a group), and add the fixture pair at 10 and 20 over a
    15-token body. Cost: S. Payoff: the ledger change can set it without an API change.
    Where: Completeness contract "Real entry point"; Task 5 (`Baseline` `min_tokens`).

## Summary

Items 8 and 9 are verified defects in the plan as written (self-clone groups on multi-reach
files; no token source for const and static units), item 3 is a false statement about the
dependency set, and items 1 and 2 are the structural choices that decide whether the next three
census changes build on this crate or around it. Nothing here conflicts with AGENTS.md; no item
asks for a timing run or a mutation pass.

VERDICT: NEEDS-REVISION
