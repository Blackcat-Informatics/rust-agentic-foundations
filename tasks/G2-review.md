VERDICT: PASS

# G2 review of remediation C2 (GAP-05), commits 0f7aaa3 and 5a3146f

## Scope
Abort on the real entry point; remaining `syn` and lexer use on the caller's thread; the new
`Item::parameter_bindings` and `Item::initializer`; `Item` size and sharing; the span-table
drop guard and its test; source-index, census and page docs; assessments and SPDX headers.
The worktree was not edited. Probes ran in a scratch clone at `$S/scratch/c2review` (checked
out at 5a3146f, own target dir), with two throwaway test files that were not committed.

## Commands and exit status
* `cargo test -p raf-source -p raf-census --locked`: 0, all suites ok.
* `cargo test --locked --test docs_contract --test self_workplan --test repo_gates`: 0
  (28, 68 and 1 tests).
* `make clippy reuse`: 0; REUSE reports 571/571 files with copyright and license information.
* Scratch probe, `Census::load` on a 2 MiB thread: 0 (all outcomes listed in finding 1).
* Scratch probe, public `Normalizer::print` on a 2 MiB thread: 0.
* `make cross-check`: not run. The targets are not installed locally and CI runs it. The change
  adds no platform code.
* No timing runs and no mutation passes.

## Findings
1. **The abort is gone.** The gap's shapes on a 2 MiB thread through `Census::load`:
   * 8x60 as a parameter type, a return type, a where-clause bound (`T: Into<..>`) and a
     const type all return Ok with one unit.
   * 24x60 returns `Error::Load` naming "parser recursion (1927)" for all four shapes.
   * 1x200 returns `Error::Load` naming "nested `<` (64)".
   * Delimiter nesting of 50, 120 and 127 loads for a const initializer in parens, a static
     initializer in braces, an array initializer, a function body, and an associated const
     beside a method with tuple parameters.
   * Nesting of 200, 1000 and 5000 is `Error::Load` naming the delimiter budget (128).
   * No abort and no panic in any case.
2. **No `syn` re-parse is left in raf-census.** `units/sig.rs` is deleted. The only `syn` call is
   `syn::parse_str::<syn::Lit>` in `units/scan.rs:85`, which decodes one string literal token.
   That is not a signature or an item parse and it does not recurse over nesting. The
   remaining `proc_macro2` use is the lexer of bodies and initializers in
   `normalize/form.rs`, which is bounded by `MAX_NESTING`. The public `Normalizer::print`
   probe at nesting 100, 1000 and 100000 on 2 MiB gave Ok, then a delimiter-budget `Error`,
   with no overflow. The Constraint text is true as written.
3. **New raf-source API.** `with_bindings` is called for free functions, impl methods, trait
   default and required methods, and bodiless (extern/attribute-macro) functions. `with_initializer`
   is called for const, static, impl assoc const, and a trait const only when it has a default.
   * The visitor skips types, so const generics, `fn(inner: u8)` types and generic defaults
     add no binding.
   * `self`, including a typed `self: Box<Self>`, is a `Receiver` and is listed first.
   * Closures in parameter position are not params, as stated.
   * The initializer range runs from after `=` to before `;`, so a `;` inside a string is
     safe and the type and attributes are excluded. The tests prove this.
   * Bound tests cover self, mut, ref, tuple, struct, slice, raw ident, type names, trait
     default and required, and a required const.
   * The bindings are read on the parse thread and charged to the fact meter. The `Arc`
     header (16 bytes) is not charged. This is negligible and not a defect.
4. **Memory and sharing.** `Item` is one `Option<Arc<ItemDetail>>` pointer (8 bytes); the
   report and the passing `an_item_record_is_at_most_two_hundred_fifty_six_bytes` test put it
   at 232 bytes, under 256. The `detail` is created once per fact. Each reach clones the `Arc`
   (`index.rs:632`, `:1239`), so there is no per-reach copy of the names. The pseudo-module
   and `mod` items carry `None`. Observation: the test
   `bindings_and_an_initializer_are_shared_by_every_path_that_reaches_the_file` compares values,
   not `Arc` pointer identity, so it proves equality, not sharing. The code does share.
5. **Span table clearing.** `SpanMap` is a drop guard bound at the top of `collect`, so it runs
   on the `?` error returns as well as on success. The twice-on-one-thread test reads the first
   token offset (0 only when the table is empty) before, between and after two loads. Without
   the guard the offset would be non-zero between loads, so the test does observe it.
   * Low: no test forces a failure inside `collect` to observe the guard on the error path.
     The code path is correct by construction. A failing `Census::load` before `collect` (a
     load error) never lexes, so there is nothing to clear.
   * Low, design note: `invalidate_current_thread_spans` clears the whole thread-local table.
     A caller that holds its own `proc_macro2` spans on the same thread would have them
     invalidated by `Census::load`. The Constraint says the table is cleared but does not warn
     about this side effect. Not blocking.
6. **Docs.** The source-index Contract (bindings and initializer, bound tests, the facts
   paragraph), the census Constraint (bound that holds: 64 MiB parse stack, recursion budget,
   `MAX_NESTING` 128), the Error text, the removal of the old `sig` tests from the bindings,
   and the page's Token source paragraph all match the behaviour observed in finding 1.
   `docs_contract` passes.
7. **Assessments and headers.** All six touched assessment files
   (`source-index-{constraints,contract,examples,usage}.md`, `census-normal-form-{constraints,contract}.md`)
   begin with `---` and the two SPDX lines (copyright, `CC-BY-4.0`). A scan of every
   `work/assessments/*.md` found no file without an SPDX license line, and `make reuse` passes
   571/571. The worktree is clean.
8. **Tool defect to report (not a code defect).** `assess --verdict` strips the SPDX header from
   an assessment file when it accepts it. The implementer restored the headers by hand. The
   defect is in the assessment tool and should be filed against it so that accepting an
   assessment never removes the header.
9. **Environment.** `make cross-check` was not run here (finding in Commands). CI covers it.

No blocking finding.
