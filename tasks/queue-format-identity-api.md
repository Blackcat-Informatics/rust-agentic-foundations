<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Standard format identity: private API proposal

Read-only design inspection of the current issue 16 source. No builds, compiler
experiments, product edits or issue 28 behavior were imported. This proposes a
bounded recognition predicate for the explicitly supported `format!` family;
it is not macro expansion or full Rust macro resolution. Conservative refusal
here means decline to add an inferred capture edge, not silently weaken an
existing gate or classify the source program as invalid.

## Smallest usable identity boundary

Keep recognition private beside reach's scanner/resolution integration. Return
an explicit classification such as `StandardFormat`, `Other` or `Unknown`,
and extract implicit captures only for the first. A final path segment named
`format`, or `resolve_from` returning NotFound, is never positive identity.
Maintain separate entries for any additionally supported standard macros; their
argument grammar and string argument positions are not established by this
identity predicate.

Use a small positive provenance description:

1. An explicitly qualified path with exactly the supported standard crate and
   macro endpoint (`std::format` initially), after proving that its head means
   the compiler's standard crate in this target/context; or
2. A unique direct named import of that exact endpoint (`use std::format;` or
   `use std::format as render;`), with the same positive head proof and no
   competing macro binding at the invocation; or
3. The unqualified canonical name supplied by the enabled standard prelude,
   only after ruling out every relevant lexical/module/import competitor.

Do not turn arbitrary unresolved external paths into standard identities.
`use vendor::format as render`, `use unknown::*`, a local macro named `format`,
or a workspace crate named/renamed `std` are not standard provenance.

## Current exact interfaces

- `SourceFile` (`workspace.rs:44`) owns exact `text`, private `facts: FileFacts`
  and `reached: Vec<Reach>`. `Reach` (`modtree.rs:70`) distinguishes `target`,
  `module_path`, `test` and optional path predicate. One physical file can have
  multiple contexts; `SourceFile.package` is only its first package.
- `Index.modules[module]` gives `parent`, `root`, `target`, `defs` and `path`;
  `root_items[&root]` leads to the crate-root Item and its source file. Use that
  target root's file facts, not whichever physical file happens to be scanned.
- `Index.edition(module)` and `Index.no_std(module)` are per-target lookups
  (`index.rs:1285-1291`). `target_no_std` only records a direct empty-argument
  root `no_std` attribute (`index.rs:1089-1095`), intentionally not a potentially
  conditional `cfg_attr`. There is no existing no-implicit-prelude flag.
- `FileFacts.attrs` retains root/file inner `AttrFact { path, args }`
  (`facts.rs:138,335`). `ModFact.attrs` and `ItemFact.attrs` retain declaration
  attributes. `summarize` retains unhandled attributes, including
  `no_implicit_prelude` and `macro_use`; conditional attributes require checking
  their predicates/raw enclosing fact, not trusting only the derived no_std bit.
- `Index.imports_of(module, name)` supplies exact named-import positions;
  `named_of(module)` and `globs_of(module)` provide bounded slices.
  `UseDecl` carries module/file/line/leading_colon/test/visibility and flattened
  `UseKind::Named { name, segs }` or `Glob { segs }` (`index.rs:160-180`). It has
  no byte-level local block scope. The lexical scanner must own those intervals.
- `Index.lookup(module, Namespace::Macro, name)` reads settled local/import/glob
  bindings; its returned bool distinguishes glob route. `blocked(module, name,
  Namespace::Macro, None)` detects a named import that can still occupy that
  namespace even when lookup is absent (`settle.rs:472,576-647`). This matters
  for external imports whose definitions the index cannot inspect.
- `Index.outside_crate(position)` (`resolve.rs:368`) classifies the external
  starting crate of an actual named import using its path, edition, leading
  colon, own-import exclusion and local binding blockers. It is useful evidence
  for direct imports, but its returned string `std` alone does not certify
  sysroot identity: external dependencies named `std` must also be excluded.
- `Index.extern_name(module, name, test)` reads the target extern prelude.
  `ExternName.roots` names workspace roots; empty roots mean outside-workspace,
  **not** specifically the standard library (`index.rs:208-229,1276`). Reject
  a dependency-prelude `std` entry for this narrow sysroot proof, even if empty.
  `extern_names: BTreeMap<ItemId, SharedStr>` records original extern-crate
  names; `modules_for(id)` detects workspace-backed crate/module definitions.
- `Index.walk(from, segs, leading_colon, Namespace::Macro, Heads::Code { test })`
  is available privately (`resolve.rs:687`) for invocation-context competitors.
  Its `Walked.items` and failure types can establish indexed nonstandard or
  ambiguous endpoints. Public `resolve_from` uses **Heads::Use, test:false**
  (`resolve.rs:1449-1499`), so it is not the invocation identity predicate.
  `reach_run` combines all namespaces/prefix reachability and also cannot be
  used as a macro identity decision.

## Concrete narrow decisions

**Explicit std endpoint.** In edition 2018 or later, leading `::std::format`
can bypass local `std` module names, but still reject an active dependency-prelude
entry named `std` or uncertain root attributes. Unprefixed `std::format` also
requires no lexical type/module binding `std`, no indexed type binding and no
blocked named binding of `std` at the actual code lookup context. Check relevant
globs as competitors. A `mod std` defeats this proof even if the macro endpoint
inside it cannot resolve. Do not interpret the resulting missing endpoint as
permission to use the sysroot. For an initial smallest implementation, decline
edition 2015 qualification until its root-based head proof is implemented; the
existing `crate_relative(..., Heads::Code)` distinguishes its leading-colon rule.

**Named import and alias.** A unique `imports_of(module, invoked_name)` entry
whose flattened endpoint is exactly `["std", "format"]`, whose
`outside_crate(position)` is `Some("std")`, and whose std-head proof above
succeeds provides direct positive provenance. Preserve `leading_colon` and use
the declaration's edition/test context: edition 2015 use heads start at the
crate root; edition 2018+ use heads and code heads differ in glob/prelude
ambiguity (`Heads`, `crate_relative`, `outside_crate`). A lexical direct import
can produce the same descriptor from its tokens. Do not require that this
external standard item have indexed resolution sites: they are absent by
design. Conversely, do not accept every blocked external import as standard.
Multiple possible declarations, cfg alternatives, unresolved competing imports,
or glob-delivered aliases remain Unknown in this initial design.

**Bare prelude format.** Require canonical `format`, enabled standard prelude,
no local textual macro shadow, no active lexical use competitor, no indexed
macro binding, no macro blocker, and no uncertain macro-providing glob or
`#[macro_use]` import in the relevant lexical/module ancestry. Inspect ancestor
macro declarations as well as the invocation module: textual macro scope is
not established by one settled module table. An arbitrary resolved workspace
macro, whether direct or glob-imported, is Other regardless of its name.
Reject all potentially macro-providing unresolved globs unless their namespace
effect is positively proved; a missing lookup is insufficient.

**Prelude/attribute refusal.** Bare-prelude recognition must decline under
root or relevant module `no_implicit_prelude`, `no_std`, `no_core`, and conditional
forms that could select them. Do not infer that no_std has format in core.
Qualified/direct imported std can eventually be accepted with explicit positive
`extern crate std` provenance under such attributes, but simply refusing this
combination initially is smaller and honest. Likewise decline uncertain
`macro_use` sources rather than inventing their exports. This avoids adding
target-specific attribute evaluation or external macro inventory to this fix.

## Lexical ownership and cache requirements

Function-local `use` bindings are scoped over their containing block, including
text before the declaration; local `macro_rules!` textual scope begins after its
declaration and can flow into nested lexical/module contexts. Keep their exact
scope intervals and source ordering, with parent scope links, rather than a
file-wide name blacklist. Parent-module macro inheritance depends on the module
declaration position. Alias imports and local `std` type/module bindings require
their own namespace-sensitive entries. Macro bodies, generated local bindings
and conditional alternatives cannot acquire fabricated standard provenance.

Keep file/edition lexical descriptors in reach's existing `(RelPath, u16)` cache
(`reach.rs:855-858,948-949`). Do not cache an identity answer by file/edition
alone: it also depends on target/module, test flag, offset/lexical scope,
attributes and import provenance. Charge stored names, spans, scope entries,
descriptors and any new resolution cache/work to the existing reach budget.
Bound queries through existing import slices instead of rescanning all imports
for every capture. No public API/schema change is required for this private
predicate, and no source-index settling semantics need to change.

## Known initial limitations and required neighbors

Unresolved transitive reexports of `std::format`, crate aliases (`use std as s`),
glob-delivered external format macros, conditional prelude state and macro-use
external exports lack a direct positive certificate here. Classify Unknown until
an explicit bounded provenance rule and real tests are supplied. Do not advertise
full standard-macro resolution or silently treat these as proven standard.

Required actual source neighbors: canonical bare and qualified standard call;
direct import and renamed macro import; local macro before/after its declaration;
block-local use before/after textual declaration; workspace resolved macro and
glob macro; unresolved named external import and glob; root/module/conditional
no_implicit_prelude; no_std; local/dependency `std` shadow; leading-colon spelling;
same physical file in different module/target/edition contexts. Accepted standard
calls should reach the implicit captured constant; each rejected recognition
neighbor must show no invented edge. No execution of these cases is claimed by
this design note.
