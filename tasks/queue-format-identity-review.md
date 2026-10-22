<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Unfinished format scanner: focused identity review

Read-only source inspection of current `reach/format.rs`, `format/identity.rs`,
`format/arguments.rs`, reach integration, public reach fixtures and existing
source-index contract. No builds or compiler tests run by this reviewer. The
source was not frozen, and these findings apply to the inspected implementation.

## Finding: opaque macro inputs gain standard identity

The scan loop descends every Group and records inner literal-leading macro
invocations without storing their enclosing macro-input context. `standard`
then judges that inner path as code of the surrounding function/module.
`format_references` immediately validates its stored arguments and adds its
decoded capture edges. The enclosing macro may discard or stringify those
tokens, so that positive identity is not established.

Concrete unexecuted source witness:

```rust
macro_rules! discard { ($($t:tt)*) => { () } }
const VALUE: u32 = 7;
pub fn f() { discard!(format!("{VALUE}")); }
pub fn grammar() { discard!(format!("{")); }
```

The implementation records the nested `format`, whose bare name is standard in
this module. It therefore invents a VALUE edge for f and raises a format-fields
ReachError for grammar, even though neither input is expanded as a format
invocation by discard. The existing malformed-custom-macro fixture only tests
the outer macro itself named `format`; it does not cover this nested input.

Smallest bounded correction: record enclosing macro-input ancestry on lexical
candidates. Infer and validate a nested format candidate only when every
enclosing input is an explicitly supported standard invocation whose parsed
expression position actually contains the nested call. Unknown/custom macros,
including stringify, remain opaque to the new capture pass. A plain brace or
parenthesis expression outside macro input should retain its normal behavior.
Keep ordinary existing token-path reachability separate; this correction does
not require expansion or changing its approximation.

Required neighbors: discarded nested capture and discarded malformed literal
both produce no invented edge/error; direct standard capture still works;
`format!("{}", format!("{VALUE}"))` still reaches VALUE if nested standard
expression inputs are supported. Do not merely suppress all nested groups.

## Finding: imported aliases bypass lexical std-head shadow proof

`standard` rejects lexical `std_bound` and imports named std only when the
invocation itself is qualified. Its unqualified-alias branch calls `imported`,
which proves `["std", "format"]` through `sysroot(index, module, ...)` alone.
That module index does not supply a function block's local use bindings.

Concrete unexecuted source witness:

```rust
#[macro_export]
macro_rules! custom_format { ($($t:tt)*) => { () } }
mod custom { pub use crate::custom_format as format; }
const VALUE: u32 = 7;
pub fn f() {
    use crate::custom as std;
    use std::format as render;
    render!("{VALUE}");
}
```

The lexical maps hold both std and render, but only render is consulted for the
unqualified invocation. `imported` sees literal std::format, module-level
sysroot proof succeeds, and the alias can invent VALUE reachability for a
custom macro. This is a missing lexical head proof, not an unresolved-external
whitelist problem.

Apply the qualified std-head checks to the import descriptor's declaration
scope as well. Block use bindings cover the whole block, so textual ordering
does not rescue the import. Reject uncertain/local std aliases conservatively;
keep the ordinary unshadowed `use std::format as render` neighbor accepted.

## Local value binding suppression is not required by the current contract

The existing source-index Constraints explicitly state that a local, parameter
or field shorthand sharing an item's name reaches that item. The references
edge is read as code of the item's module, not full local variable resolution.
Current `format_references` uses the value namespace via `index.reach_value`.
Thus a local VALUE beside a module VALUE is intentionally overapproximated in
the same way as an explicit ordinary VALUE token. Adding local-variable
suppression only for decoded captures would introduce a new semantic rule and
broader binding analysis; it is not necessary to preserve the current contract.

Explicit named format arguments remain different: `VALUE = OTHER` is macro
argument syntax, and the existing new fixture correctly suppresses the argument
label and implicit VALUE capture while preserving the OTHER expression edge.
Retain that narrow grammar-specific suppression.

No whole format implementation, budget proof or adoption acceptance is claimed.
