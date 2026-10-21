<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Census caller-span ownership review

Read-only review of the integrated strict checkout, locked proc-macro2 1.0.107,
and the owner's proposed native correction. No product edit, build, or reproduction
was run by this reviewer. The implementation owner independently reported an actual
public successful-load panic when reusing a caller span; the invalid-workspace
neighbor preserves the span. Parent retains final substantive review.

## Actual execution and affected APIs

`Census::load` calls `Workspace::load_with`, waits for it to return, then invokes
`units::collect` directly on its own caller thread. Source's per-file parsing
workers have already ended. There is **no worker boundary around collection** in
the inspected implementation. `collect` constructs `SpanMap`; dropping that guard
calls `proc_macro2::extra::invalidate_current_thread_spans`, on success, collection
failure, or panic. A source-load failure before collect never constructs the guard.

Within collection, `build -> Normalizer::tower -> Tower::new -> Form::lex` calls
`TokenStream::from_str`. `Lexer::place` immediately copies byte ranges and line
positions from spans into owned `Place`. `units::scan::decode` separately calls
`syn::parse_str::<syn::Lit>` for string/byte/C literals, also registering source
on the same thread. Both operations must be inside the owned execution boundary;
isolating just the initial token stream is insufficient.

The returned Form/Tower/Unit values hold owned text, byte ranges, lines, integers,
and fingerprint bytes, not proc-macro2 Span/TokenStream values. This makes an owned
collection worker plausible without changing normal-form data. The actual closure
must still compile to establish Workspace Sync and result Send; do not add unsafe
trait implementations to force that claim. Module-walk Rc values are temporary
load internals, not a reason to strengthen the public Tree trait.

Public `Tower::new` and `Normalizer::{tower,print,explain}` independently invoke
the same lexer on their caller's thread. They do not use `SpanMap`, so they do not
currently clear caller spans, but repeated standalone calls retain proc-macro2
source-map storage. A collect-only worker corrects Census loading; it does not
establish bounded standalone-normalizer storage or a thread-free core.

`examples/selection.rs` also explicitly invalidates spans after each measured
body. That is private example-program code with owned outputs, not a public
library session API. Preserve its ownership assumptions explicitly rather than
copying its global reset into reusable entry points.

## Dependency evidence and invalid old test claim

Installed `proc-macro2-1.0.107/src/extra.rs` documents that invalidation affects
all prior spans on the thread and can make later methods return wrong values or
panic; it can also panic in procedural-macro context. Its fallback implementation
truncates private `SOURCE_MAP.files` to the dummy entry. The inspected public extra
API exposes no scoped checkpoint/restore operation that preserves earlier files.

The current capability explicitly tells callers to finish using their spans
before Census loading. That documents the destructive behavior, but does not
repair ownership or grant an accepted exception. The normal-form ADR's cleanup
claim and the units module's caller-thread cleanup prose need correction with code.

The existing `loading_one_tree_twice_on_one_thread_leaves_no_span_map` test is
**not proof of its named property**. Its `first_offset()` reads a fresh token's
`Span::byte_range().start`. In locked 1.0.107 that resolves through
`FileInfo::byte_range`, subtracting the file's global source-map start; the first
token of every separately parsed string has local offset zero even as the TLS
map grows. Consequently `(0,0,0)` does not measure TLS-map size or retained bytes.
The owner and parent received this finding immediately. Use a genuine caller-span
preservation assertion and evidence of the actual owned lifetime; any dependency
source-map ordinal/allocation probe must state precisely what it measures.
Remove or replace the old storage assertion and its misleading test name; retaining
it as storage qualification would preserve a false claim. A separate source-range
test may still assert file-relative offsets, explicitly under that narrower name.

## Review of the selected small native correction

Parent selected a scoped **collection-only** worker before further RDF work.
This is coherent for the immediate native bug if implemented as follows:

- Load the Tree on the original public path, then borrow the completed Workspace
  and options in the collection worker. Do not require Tree: Send/Sync, move the
  entire public load into a worker, or copy the whole workspace. Preserve support
  for a caller-owned Tree using Cell or another non-Sync implementation.
- Remove `SpanMap` and the global-clear call entirely: fresh worker thread teardown
  already reclaims its private TLS store, including refusal and panic paths.
  Parent selected this simpler ownership path after review. If any temporary
  prototype retains a guard, construct it **inside the running worker body**;
  constructing it on the caller and capturing it lets failed spawn drop it on
  the caller and recreate the corruption on an error path.
- Return only owned units or an owned typed error. Convert literal/span errors
  into plain source data while on that worker. Propagate spawn failure as its own
  actionable typed error with an io source; do not mislabel it as malformed Rust
  or return a partial census. Existing Error is non-exhaustive, permitting a new
  variant without requiring external exhaustive matches to change.
- Join explicitly and resume an original panic payload. Do not swallow worker
  panics or turn them into successful/empty collection. Worker cleanup must run
  before the caller resumes. A test of invalid workspace loading does not exercise
  this worker's collection-error path because it exits before the worker exists.
- Keep token bytes, punctuation jointness, comment/literal handling, source-relative
  ranges and line locations identical. The move alone does not justify a normal
  form bump. Existing NF1 forms/fingerprint goldens and literal/report tests must
  verify byte identity. Include a normalizer-derived Form held across Census::load
  as an owned-result neighbor, and a live external proc-macro2 span on both success
  and applicable refusal paths.

If Workspace sharing fails to compile, inspect the exact non-Sync retained field;
do not silently broaden Tree's public requirements or introduce unsafe sharing.
The owner is compiling the real closure; this reviewer has not claimed that result.

## Portable completion options and limits

The collection worker preserves caller spans and bounds **repeated native Census
loads** by their owned worker lifetimes. It is **not** evidence of wasm-core
acceptance. Current source parsing already needs host workers; the new boundary
must remain visible in the portability preparation and open contracts for 43/46.

A complete portable solution needs an owned lexical/session boundary that does
not reset unrelated TLS. Two coherent design directions remain unimplemented:

1. A supported scoped lexer/source-map API that owns new files while preserving
   older caller files and nested-session lifetimes, returning only span-free data.
   That API is not present in the inspected dependency. Do not access private TLS
   through unsafe tricks or mutate the installed dependency as a local fix.
2. A span-free Rust token/literal pipeline with owned byte offsets and lines.
   It must cover both Form lexing and string decoding and reproduce NF1 token
   bytes, jointness, doc-comment treatment, Unicode/raw identifiers, nested groups,
   literals and diagnostics. Replacing only one proc-macro2 entry leaves the other
   registration path intact. A lexer change cannot silently alter already merged
   normal-form identities.

Simply removing cleanup is not bounded repeated-load storage. A host thread is
not a portable-core substitute. These are explicit design/acceptance limits,
not proposed waivers or reasons to conceal the current native regression.
