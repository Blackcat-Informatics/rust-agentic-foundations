<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Implicit format captures and reachability

Read-only design preparation for the public API reproduction in
`scratch/adoption-probe/src/bin/format-capture-gap.rs`. No product edits, builds,
tests or process probes were performed. The supplied log demonstrates VALUE is
reached from direct argument syntax but omitted from the implicit capture;
escaped-brace and ordinary-literal neighbors are correctly absent. That log
belongs to the owner; this review did not rerun it.

## Finding and intended ownership

The missing dependency is real source use. Fixing its extraction in raf-source
lets the existing function/test code binding own the used constant through the
existing References edge. No new public API, schema key, constant binding-role
exception, fake use or weakened gate is needed. Directly binding a test constant
as production code is not an equivalent repair. Rewriting to redundant explicit
named arguments also fights the required Clippy policy and obscures the issue.

`reach.rs:485` scans tokens into paths; `TokenTree::Literal(_)` at 580 resets the
open run and records no dependency. This is why ordinary path scanning cannot
see an implicit format argument. Every file is lexed once per `(RelPath, edition)`
at 945, owned runs are placed in the innermost item/impl span at 713, and
`references` at 1009 resolves them in that unit's module/test context. Thus a
literal-derived candidate must enter this same ownership path. Do not scan all
string literals, or make a special case in the workplan coverage rule.

Rust's [format specification](https://doc.rust-lang.org/std/fmt/index.html)
defines implicit named arguments, escaped braces and named width/precision
parameters. The [macro scope reference](https://doc.rust-lang.org/reference/macros-by-example.html#scoping-exporting-and-importing)
distinguishes textual and path lookup. Those documented rules are the reason a
bare macro-name whitelist is insufficient. The implementation proposal below
is derived from the current local scanner, not a claim that macro expansion or
general Rust name resolution is already implemented.

## Smallest coherent private implementation

1. Extract lexical **format-call candidates**, not unconditional references,
   while scanning token groups. Retain the written macro path, invocation byte
   position, the literal argument position, parsed capture names and explicitly
   supplied named-argument names. Keep ordinary token traversal intact so actual
   argument expressions retain their current dependencies. A normal literal or
   a nonformat macro must add none.
2. Recognize only explicitly supported standard formatting invocations, using
   the unit's actual context during `references`. Once recognized, each implicit
   capture is a value use in that context, with the same References edge and
   from-node as an explicit argument. Never treat a format type specifier such
   as x or Debug syntax as an item. Numeric positions are not capture names.
3. Deduplicate the resulting capture by its meaning and module/test context;
   repeated placeholders and repeated calls in one unit should not repeatedly
   resolve the same value. Reuse the real value resolver. Its existing handling
   of locals is an approximation documented by source-index; do not silently
   claim new lexical local-scope precision solely for captures.
4. Store names as private owned decoded strings when necessary. Current Run
   segments are byte ranges of the original file (`Run`/`Lexed` at 222–269), and
   `references` indexes the original text at 1025. That representation cannot
   describe a name whose spelling comes from escapes inside a cooked literal.
   A small separate candidate/capture vector in Lexed/FileRuns avoids changing
   every ordinary segment to an owned string. Place candidates by invocation or
   literal start using the same innermost-unit rules, preserving text order.
   Do not return proc-macro spans from the lexer worker.

This is a genuine extension of the private reachability scanner. It must be
documented as such, with the accepted macro/literal forms and remaining macro
expansion limits. Do not turn arbitrary macro literals into dependencies to
make the adoption count disappear.

## Recognition boundary and shadowing

Prefer a small, exact recognizer for qualified standard macro paths and
unqualified prelude calls whose identity is established, rather than a broad
list matched only on final name. `std::format!` can itself be shadowed by a local
module named std; leading `::std::` uses edition-specific root rules. Existing
module resolution/prelude facts must establish which root is meant.

For an unqualified `format!`:

- A visible local/path-based macro named format is not the standard macro.
  Explicit imported macros and ambiguous or unresolved macro imports must not
  be treated as the standard macro just because no workspace item resolves.
  `resolve.rs:413` explicitly says an import of an outside item cannot reveal
  its namespace; NotFound therefore does not prove standard-macro identity.
- Textual `macro_rules! format` can be local to a function and takes precedence
  over path lookup. The ordinary source item index does not establish all such
  textual invocation scopes. Token nesting and declaration-before-call position
  must be tracked for supported cases; seeing any module item called format
  regardless of position is not exact textual shadowing either.
- A macro alias of the genuine standard macro should be supported only when its
  provenance is actually established. An external unresolved alias must not be
  guessed by spelling. no_std/prelude facts matter as well: `Index::no_std` and
  extern-name data exist, but absence of the macro candidate alone is not proof
  that a standard macro was imported through the prelude.

Choose a coherent supported macro set from the actual repository callers and
acceptance contract. Any required identity or argument form that remains
unsupported blocks that contract; it is not a completed adoption slice. A
targeted rewrite of a known standard call
to its genuine qualified standard path can be an honest caller change when
meaning is unchanged and a lexical-shadowing negative exists; it is not a
substitute for implementing the actual public `format!` reproducer if that is
the acceptance contract. Do not report exact general shadow resolution until
the scanner/index actually carries the necessary facts.

## Literal and argument correctness

Decode only a Rust string literal occupying the actual format argument. The
crate already has syn parsing support; using its LitStr reader for a literal
is smaller and less error-prone than inventing a second Rust escape decoder.
Accept raw literals with their hash delimiters and cooked escapes; reject or
classify invalid forms consistently with the existing syntactic-source
contract. A byte/C string or arbitrary expression is not that literal.

Formatting extraction must recognize argument identifiers and named count
references, skip doubled braces, respect format-option positions and subtract
explicit named arguments from implicit captures. For example:

```rust
format!("{VALUE}")                    // implicit VALUE
format!("{{VALUE}}")                  // no VALUE use
format!(r#"{VALUE}"#)                  // implicit VALUE
format!("{VALUE:WIDTH$.PREC$}")        // implicit VALUE, WIDTH, PREC
format!("{VALUE}", VALUE = other)     // other is explicit use; not captured VALUE
format!("{:WIDTH$}", item)            // explicit item, implicit WIDTH
format!("{0:1$}", item, width)         // only actual explicit arguments
```

These are proposed fixture shapes, not executed compiler results. Include
cooked literal escapes that decode to braces or identifier characters and a
raw literal containing the same backslashes, so the two are not conflated.
Test whitespace, Unicode identifiers, raw-identifier restrictions and keyword
captures against the actual toolchain before selecting identifier validation;
normal reachability's `blocked` keyword rule must not be assumed equivalent to
format capture grammar in every edition.

Do not collect every identifier followed by `$` anywhere in a literal: a fill
character or malformed spec is not a reference. Do not naïvely split the whole
token argument stream at commas: closures and angle generic arguments can
contain ungrouped comma tokens. A bounded format-macro argument parser should
consume actual expressions and top-level named arguments, rather than treating
every apparent `name =` in expression tokens as an explicit format binding.

Macro families have different format-argument positions. A minimal implemented
family should be explicit. `format!`/format_args/print-style calls take the
format literal first; write-style calls have a destination first; assert-style
messages follow condition or left/right operands; panic messages have edition
semantics. Adding those families requires their real argument grammar and
negative neighbors. A successful format! fix does not establish them all.

## Cache, work and memory

- Keep lexical candidates in the existing `(file, edition)` cache. One physical
  file can be reached by packages with different editions; never reuse one
  package's keyword decisions for another. Standard macro identity, explicit
  import provenance and captured-value resolution are unit/module/test-context
  decisions and cannot be cached solely by physical file or literal text.
- Existing `seen` keys contain context and text at 1038–1064. A capture has
  value semantics; do not let a previous unrelated multi-namespace ordinary
  path with the same text cause its necessary value resolution to be skipped.
  Either reuse a proven equivalent value-result cache or include semantic kind
  in the key. The same concern applies when multiple candidates name the same
  string under different standard/local macro identities.
- Charge every retained candidate, owned name, argument-name list, per-unit
  position vector and any macro-recognition cache with actual typed container
  layout and text capacity. Current `load` accounting at 982 charges only Run,
  raw segment and placement storage: adding a vector without extending it would
  undercharge. Preserve exact-limit/one-below ItemBytes refusal with useful
  location and cleanup on refusal.
- Existing lexer work tally is returned as `Lexed.tokens`, then added on the
  caller at 957 because lexer-thread test counters do not transfer. Formatting
  byte/character visits need their own returned work count or a precisely
  documented extension of an existing metric. Do not call worker-local
  work::add and assume the caller's ceiling sees it. Scan each selected decoded
  literal once and each invocation's argument list once; avoid rescanning a
  whole literal for each brace/capture or a whole file for each reached item.
- The current lexer temporarily owns TokenStream and Lexed before retained
  meter accounting. New decoded buffers and candidate names increase that
  temporary footprint. Keep the increase bounded by the input/literal and
  derived per-capture counts, release decoded scratch immediately, and audit
  actual allocation ordering. Do not claim a pre-allocation admission guarantee
  simply because the final cached vector is charged after construction.
- Existing dense ordinary-path tests pin 56-byte incremental storage and exact
  work formulas. A separate capture vector can preserve their behavior; if any
  common Run/segment layout changes, derive new accounting/pins from the actual
  representation rather than raise expectations until passing. Reuse existing
  mixed-edition cache exact/one-below tests and add capture-specific neighbors.

## Focused acceptance before adopting the repair

1. Run the supplied public API reproducer with captured/direct true and
   escaped/ordinary false; assert actual reached constant identity and
   References/from-node, not just a lower orphan count.
2. Add raw/cooked/escaped-brace, explicit named override, repeated capture and
   width/precision captured-constant cases with real rustc neighbors.
3. Add a local shadow macro ignoring its string, a function-local textual
   shadow, known qualified standard call beside it, external same-name import,
   normal literal, and different-format-family argument positions. Do not make
   local custom macro arguments acquire standard implicit dependencies.
4. Share one physical source file across editions and modules with different
   macro/value bindings; compare single-root unions with both combined root
   orders. This tests the actual cache bug risk, not only isolated lexing.
5. Count repeated-literal and many-capture work at growing sizes; verify
   retained-byte exact/one-below limits and no borrowed lexer span escapes.
6. Compile actual raf-workplan/cargo-workplan consumers and rerun the focused
   adoption/role gate after the coherent change. Only then can a fixed orphan
   prove the intended ownership path handles the constant.

No acceptance is waived by this proposal. Existing code-role restrictions,
normal-form identity, coverage policy and hook gates remain authoritative.

## Additional current source observations

The owner ran the public-API neighbor in
`scratch/adoption-probe/src/bin/format-named-gap.rs`; its exact output is retained
in `raw/queue-format-named-gap.log`. The named-argument call
`format!("{VALUE}", VALUE = OTHER)` currently reaches both VALUE and OTHER. The
direct argument reaches VALUE only, and an ordinary string reaches neither.
Thus subtracting explicit named arguments from newly extracted captures is
insufficient: the existing ordinary path scanner already treats the left-hand
VALUE binding as a value reference. A recognized standard invocation must also
suppress its actual named-binding identifier positions from ordinary resolution.
Do not apply that suppression to a custom macro or to a real assignment expression.

Any proposed syn expression parser for macro arguments needs an admission step
before recursion. The existing file pre-scan deliberately does not count ordinary
expression recursion inside a macro token group as if syn had parsed those
arguments. Re-parsing arbitrary macro arguments into syn expressions without
another bound would create a new stack/AST allocation path. The private
`prescan::prescan(file, text, max_tokens)` is available for a bounded argument
slice; offsets and errors must still name the original file and line. Keep
uncertain custom-macro identity distinct: a syntactically arbitrary custom macro
must not acquire a source refusal solely because its name resembles a standard
formatter. Decode only supported literal candidates on the owned lexer worker,
and preserve all ordinary paths and existing counted-work rules for noncandidates.

`SourceFile` already retains private `FileFacts` and its root attributes, so a
no-implicit-prelude check need not automatically introduce another public API or
duplicate root metadata. Module attributes and actual import/textual scopes still
need their own exact provenance. These are implementation directions from source
inspection, not a qualified completed fix.
