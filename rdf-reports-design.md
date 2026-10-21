# RDF 1.2 census reports: design

Scope: replace the JSON baseline and index reports of `raf-census` with RDF 1.2
reports in Turtle 1.2, deterministic and read back by a strict reader. Drop
`serde_json` (and `serde`) from `raf-census`. Update the designed downstream
capabilities (cli, migration, debt ratchet, ledger) to match. The API may break.

Everything below was measured in `$S/scratch/rdfprobe` (copies of this
workspace's manifests and lock, never the worktree). No timing runs were made.

---

## 0. Recommendation

Take **`purrdf-lex =3.0.1`** (one crate plus its zero-dependency root
`purrdf-hash`) as the lexical foundation, and build the report model, the
canonical Turtle 1.2 writer and the strict reader **in `raf-census`**, on top of
purrdf-lex's W3C terminals, escape decoders, percent-encoding sets and canonical
RDF 1.2 term spelling. The reports become **ground** RDF 1.2 graphs (no blank
nodes) over a `https://blackcatinformatics.ca/raf/census#` vocabulary. Fingerprints
are absolute IRIs whose prefixed names spell exactly their `Display` text
(`body:6bb022e241a79a28`). Local nodes are fragment IRIs minted from stable keys
under a **content-addressed `BASE`**: the BLAKE3 hash of the document. Each job
match is an asserted triple with a **named RDF 1.2 reifier** carrying its match
class (`~ <#match/…> {| census:matchClass census:Copy |}`). There is one
canonical spelling per report. The reader
parses a closed profile of Turtle 1.2, checks the closed shape, re-renders, and
refuses any byte difference and any `BASE` digest that does not name the body.

Measured costs and benefits of this route:

- The `raf-census` dependency closure shrinks from 29 to 26 crates.
- `cargo deny --workspace` passes with one pinned, crate-scoped licence exception
  for `Unicode-3.0`.
- No duplicate versions.
- `wasm32-unknown-unknown` builds clean.
- MSRV is unchanged.

`purrdf-rdf`, the only published purrdf crate with a Turtle parser, fails the
bans gate (`syn` 2.0.119 beside 3.0.8) and adds 60 packages. It is the right
long-term reader once purrdf splits its text codec out (section 8).

---

## 1. Dependency route

### 1.1 What each purrdf crate offers

| Crate (3.0.1) | RDF 1.2 model | Turtle/N-Triples 1.2 write | Turtle/N-Triples 1.2 strict parse | Notes |
| --- | --- | --- | --- | --- |
| `purrdf-hash` | no | no | no | zero-dependency root; BLAKE3, hex |
| `purrdf-lex` | no (terms only) | canonical term spelling: `term_syntax::{write_iri, write_literal, write_blank, TRIPLE_TERM_OPEN/CLOSE}`, `literal_escape`, `iri_escape` | lexical layer only: `terminals` (exact `PN_CHARS_BASE`/`PN_CHARS_U`/`PN_CHARS`/`WS`), `decode_uchar`, `echar_value`, `skip_ws` | also `percent` (RFC 3986 sets; custom `EncodeSet = fn(&[u8]) -> Option<usize>`) and a JSON reader (relevant to legacy migration, section 7.2) |
| `purrdf-iri` | no | no | IRIs (RFC 3987), CURIEs, prefix maps | IDNA and BCP 47 tables a report never needs |
| `purrdf-core` | yes: interned RDF 1.2 *dataset* IR, triple terms, reifiers | `turtle` (full-IRI emitter), `turtle_render::render` (canonical review Turtle), `fno::to_ntriples` | **none** (its docs: the text parser lives in `purrdf`) | dataset/quad model; `build.rs` shells out to `rustc` |
| `purrdf-rdf` | yes | yes (`serialize_dataset`) | **yes**: `parse_dataset(bytes, "text/turtle", base)`, Turtle/TriG/N-Triples/N-Quads 1.2, `VERSION`, triple terms, `~` reifiers, annotations | drags GTS, SPARQL algebra/eval, ed25519, rayon, AES-GCM, zstd |
| `purrdf` | umbrella | | | everything above and more |

No purrdf crate has optional features, which is consistent with the no-optionality
rule. A route cannot slim a crate down. It can only choose which crates to take.

### 1.2 Measurements (scratch copies of this workspace, `cargo` offline)

Baseline: `raf-census` normal closure is 29 crates (all targets); the lock has 49
packages; `cargo deny --offline --workspace check bans licenses sources` passes.

| Route added to `raf-census` | Lock packages added | `raf-census` closure | `deny` bans | `deny` licences | `wasm32-unknown-unknown --lib` |
| --- | --- | --- | --- | --- | --- |
| **`purrdf-lex` only, `serde`+`serde_json` dropped** | **2** (`purrdf-lex`, `purrdf-hash`) | **26** | ok | ok with 1 exception (`purrdf-lex@=3.0.1`: `Unicode-3.0`) | `purrdf-lex` ok |
| `purrdf-lex` + `purrdf-iri` | 3 | 33 (serde kept) | ok | 2 rejections (`purrdf-lex`, `purrdf-iri`) | ok |
| `purrdf-core` | 19 (adds `regex`, `regex-automata`, `regex-syntax`, `aho-corasick`, `sha2`, `digest`, `block-buffer`, `crypto-common`, `hybrid-array`, `typenum`, `const-oid`, `purrdf-{cdt,deflate,events,xsd,iri,lex,hash}`) | 49 | ok | 3 rejections (`purrdf-core`, `-iri`, `-lex`) | `purrdf-core` ok |
| `purrdf-rdf` (+ core) | 60 (adds `rayon`, `crossbeam-*`, `aes`, `aes-gcm`, `ghash`, `polyval`, `ctr`, `cipher`, `structured-zstd`, `twox-hash`, `getrandom`, `wasm-bindgen*`, **`syn 2.0.119`**, …) | 89 | **FAIL**: `found 2 duplicate entries for crate 'syn'` | 3 rejections | not pursued |
| `purrdf` umbrella | 74 | (tree 192 lines) | **FAIL** (same `syn`) | 4 rejections | not pursued |

Where `syn` 2 comes from: `purrdf-sparql-eval` has
`[target.'cfg(target_arch = "wasm32")'.dependencies] wasm-bindgen = "=0.2.125"`,
whose macro support uses `syn` 2. `deny.toml` gates all targets (no `[graph]
targets`), so this is a real duplicate. On a wasm32 build the duplicate is in the
actual graph. Narrowing `[graph] targets` to get green would weaken the gate and
give up the wasm32 target, so it is not an option.

Licences: `purrdf-hash` is `MIT OR Apache-2.0 OR MulanPSL-2.0`. `purrdf-lex`
declares `(MIT OR Apache-2.0 OR MulanPSL-2.0) AND MIT AND Unicode-3.0`, because it
ships the Unicode character tables. `purrdf-iri` and `purrdf-core` declare
`(…) AND Unicode-3.0`. `MIT` is already allowed. The only new term is
`Unicode-3.0`, handled exactly like the existing `unicode-ident@=1.0.27`
exception.

The other checks:

- Sources: everything resolves from crates.io.
- MSRV: `purrdf-*` declare `rust-version = "1.98"`, at or below 1.98.1.
- Unsafe code: `purrdf-lex` is `#![forbid(unsafe_code)]`. `purrdf-hash` denies
  unsafe code at its root and allows it only in its SIMD backends.

### 1.3 Decision: `purrdf-lex =3.0.1` plus an in-repo writer and reader

Why purrdf-lex rather than a wholly in-repo lexer:

- The census exists to stop Blackcat repositories from re-implementing one
  helper seven times. Hand-transcribing the Turtle `PN_CHARS*` tables, `UCHAR`,
  `ECHAR` and canonical literal escaping into RAF would be exactly the duplicate
  the census is built to find. purrdf-lex is the one Blackcat copy of those laws,
  and its own conformance suites test it.
- Two packages, both pure Rust and wasm-clean, with no third-party dependency.

Why not `purrdf-core`: it has a writer and no parser. It costs 19 packages
(regex, sha2) and three licence exceptions. Its model is a quad dataset, and its
canonical renderer orders blocks by content, not by the report layout. RAF would
still need its own reader, so the writer alone does not pay for the weight.

Why not `purrdf-rdf`: it fails `multiple-versions = "deny"` and brings
threading, crypto and compression into a report path.

Why not fully in-repo: it would re-implement lexical laws that already have one
Blackcat home, and add no capability over purrdf-lex.

Pin exactly: `purrdf-lex = "=3.0.1"`. The canonical report bytes depend on
purrdf-lex's escaping decisions (`literal_escape::Carrier::Canonical`,
`iri_escape`), just as fingerprints depend on `blake3 =1.8.7`. A new release is
reviewed and moves the pin, with the report goldens, in one change. The
manifest comment should say so.

`deny.toml` and `fuzz/deny.toml` gain the same block. The shared sections must
stay identical; `tests/repo_gates.rs` `deny_problem` enforces this. The fuzz
harness does not depend on `raf-census`, so its exception is unused there.
`unused-allowed-license = "allow"` covers allowances; check that cargo-deny 0.20.2
does not warn on an unused exception, and if it does, note it in the fuzz copy.

```toml
# `purrdf-lex` ships Unicode's own character tables (the normalisation and
# grammar-class data), licensed under the Unicode terms (`Unicode-3.0`) in
# addition to MIT or Apache-2.0 or MulanPSL-2.0. The terms permit use and
# redistribution of the data as compiled in. The exception is scoped to this
# crate and pinned to the audited version, so a new release is re-reviewed.
[[licenses.exceptions]]
crate = "purrdf-lex@=3.0.1"
allow = ["Unicode-3.0"]
```

Wasm: the report code is pure (`&str` in, `String` out, no `std::fs`, no clock).
`purrdf-lex` checks clean for `wasm32-unknown-unknown`. `raf-census` itself does
not build for wasm32 today because of `raf-source` (section 8.2), which is a
pre-existing problem and not caused by this change.

---

## 2. Vocabulary and identity

### 2.1 Namespaces (no tracker numbers, no process wording)

| Prefix | IRI | Holds |
| --- | --- | --- |
| `census:` | `https://blackcatinformatics.ca/raf/census#` | classes, properties, the match-class individuals (hash namespace: small, closed) |
| `body:` | `https://blackcatinformatics.ca/raf/census/nf/{N}/body/` | `body` fingerprints of normal form N |
| `shim:` | `https://blackcatinformatics.ca/raf/census/nf/{N}/shim/` | `shim` fingerprints of normal form N |
| `table:` | `https://blackcatinformatics.ca/raf/census/table/` | table ids (names, not hashes: version-free) |
| `rule:` (reserved) | `https://blackcatinformatics.ca/raf/census/rule/` | rule ids; no report writes one yet |
| (base) | `https://blackcatinformatics.ca/.well-known/genid/raf-census/{blake3-256}` | the document and its local nodes |

Fingerprint IRIs. The prefix label equals the fingerprint kind, so the
prefixed name is byte-for-byte the `Display` form:

- `body:6bb022e241a79a28` is `<https://blackcatinformatics.ca/raf/census/nf/1/body/6bb022e241a79a28>`.
- `table:hex-lower` is `<…/census/table/hex-lower>`.

PN_LOCAL allows a leading digit, and 16 hex digits and table names are always
valid locals. Under the version-bump rule, a `body:`/`shim:` fingerprint is
comparable only inside one normal form, so its IRI carries the version. Under
normal form 2 the same hex digits name a different resource, so a store that
merges reports of two versions can never confuse them. A store that merges
reports of two *repositories* under one normal form joins equal fingerprints:
the cross-repository duplicate detection the census was created for. `table:`
and `rule:` are names that migration carries unchanged, so they carry no
version.

A rule id whose local part is not a valid PN_LOCAL (one ending in `.`, or
starting with `-` or `.`) is written as a full `IRIREF`, never with PLX escapes.
The canonical rule is: prefixed name if and only if the local is a valid
PN_LOCAL with no escape.

### 2.2 Local nodes: fragment IRIs under a content-addressed base

Units, groups, constants, jobs and match reifiers are facts about one workspace
snapshot. Their identity is local to that snapshot. Three options were weighed:

- **Absolute IRIs in a fixed namespace.** Merging two runs, or two repositories,
  in one store would fuse `src/lib.rs:1 demo::alpha` from both into one node with
  conflicting `census:tokens`. Wrong.
- **Fragment IRIs with no `BASE`** (`<#unit/…>`). Merge-safe, but the graph then
  depends on where the file is. Measured: `purrdf-rdf` with no caller base
  refuses the document (`iri-relative-no-base`), so `cargo census index |
  rdf-tool` breaks. That fails the portability goal.
- **Fragment IRIs under `BASE <…/genid/raf-census/{digest}>`**, where `digest` is
  the BLAKE3-256 of the document without its `BASE` line. Chosen. Every strict
  RDF 1.2 parser reads the document standalone. Equal reports mint equal IRIs, so
  merging them is harmless, and different reports mint disjoint IRIs. The text
  stays readable because only fragments appear in it. The base also makes the
  report tamper-evident: an edit that does not re-digest is refused. The
  `/.well-known/genid/` path is the form RDF Concepts recommends for IRIs minted
  in place of blank nodes, under a domain the minter controls.

Digest rule: BLAKE3-256 of the UTF-8 bytes of the document with its second line
(the `BASE` line and its LF) removed, as 64 lowercase hex digits. The input
therefore starts `VERSION "1.2"\nPREFIX …`. Use the `blake3 =1.8.7` crate that the
fingerprints already use, so RAF's own code has one BLAKE3 (see 8.3).

Policy: **every report graph is ground.** There are no blank nodes, so two
reports are graph-equal exactly when their sorted N-Triples are equal. No
RDFC-1.0 is needed anywhere.

Fragment minting (all ASCII; non-ASCII bytes are percent-encoded, which is
maximally portable). Two encode sets, both `purrdf_lex::percent::push_encoded`
with RAF-defined `EncodeSet` functions:

- `COMPONENT`: keeps RFC 3986 `unreserved`, `sub-delims` and `:`, and encodes
  everything else, including `/ @ # ? % [ ]`, space and every byte at or above
  0x80.
- `FILE`: purrdf-lex `percent::PATH` (keeps `pchar` and `/`), which encodes
  `% # ?`, space and non-ASCII.

| Node | Fragment | Example |
| --- | --- | --- |
| the report | (empty: `<>`) | `<>` |
| unit | `unit/{COMPONENT(symbol)}@{FILE(file)}:{line}` | `<#unit/demo::alpha@src/lib.rs:1>` |
| group | `group/{fingerprint Display}` | `<#group/body:6bb022e241a79a28>` |
| shared constant | `constant/{decimal value}` | `<#constant/65536>` |
| job | `job/{COMPONENT(id)}` | `<#job/sum>` |
| match reifier | `match/{COMPONENT(id)}/{unit key as above}` | `<#match/sum/demo::BIG@src/lib.rs:7>` |
| debt group (debt record) | `debt/{kind}/{COMPONENT(sym)},{COMPONENT(sym)}…` (members sorted) | `<#debt/body/demo_a::len,demo_b::len>` |

Injectivity:

- `COMPONENT` encodes `@` and `/`, so the symbol and the job id end at the first
  separator.
- The line is the digits after the last `:`.
- `,` never occurs in a symbol.

The reader never parses a fragment. It re-mints the fragment from the node's
properties and requires equality, so a hand edit that changes `census:line`
without changing the IRI is refused. Validated in the probe:
`<#unit/demo::r%23type%40x@my%20dir/a%23b:c.rs:3>` with literal
`"demo::r#type@x\"q\\é"` parses in purrdf-rdf.

### 2.3 Classes, properties and the closed shape

This table is the single source of truth. Implement it as a `const` table in
`raf_census::report::vocab` (`SHAPES`). The reader enforces it, the writer
validates against it, and `tests/page.rs` checks that the architecture page
states it row for row. Counts are exact. `n..` means at least n.

| Report | Class (subject) | Property | Count | Object |
| --- | --- | --- | --- | --- |
| baseline | `census:Baseline` (`<>`) | `census:normalForm` | 1 | integer, fits `u32` |
| baseline | `census:Baseline` | `census:reportForm` | 1 | integer, equals `REPORT_FORM` (1) |
| baseline | `census:Baseline` | `census:minTokens` | 1 | integer at least 1 |
| baseline | `census:Baseline` | `census:unitCount` | 1 | integer, at least the unit nodes |
| index | `census:Index` (`<>`) | `census:normalForm` | 1 | integer, fits `u32` |
| index | `census:Index` | `census:reportForm` | 1 | integer, equals 1 |
| both | `census:BodyGroup` / `census:ShimGroup` | `census:fingerprint` | 1 | `body:`/`shim:` IRI of the report's normal form, kind agreeing with the class |
| both | group | `census:tokens` | 1 | integer |
| both | group | `census:member` | 2.. | a `census:Unit` node of the document |
| baseline | `census:SharedConstant` | `census:value` | 1 | integer above 65535, fits `u128` |
| baseline | `census:SharedConstant` | `census:packageCount` | 1 | integer at least 2 |
| baseline | `census:SharedConstant` | `census:member` | 2.. | unit node |
| baseline | `census:Unit` | `census:symbol` | 1 | string |
| baseline | `census:Unit` | `census:file` | 1 | string |
| baseline | `census:Unit` | `census:line` | 1 | integer at least 1 |
| baseline | `census:Unit` | `census:holds` | 0..2 | `table:hex-lower`, `table:hex-upper` |
| index | `census:Unit` | `census:symbol`, `census:package`, `census:file` | 1 each | string |
| index | `census:Unit` | `census:line` | 1 | integer at least 1 |
| index | `census:Unit` | `census:shipping` | 1 | boolean |
| index | `census:Unit` | `census:tokens` | 1 | integer (0 for a non-function) |
| index | `census:Unit` | `census:body` | 0..1 | `body:` IRI |
| index | `census:Unit` | `census:shim` | 0..1 | `shim:` IRI (the `shim_key`) |
| index | `census:Job` | `census:jobId`, `census:homeSymbol` | 1 each | string |
| index | `census:Job` | `census:copies` | 1 | integer, equal to its `census:Copy` matches |
| index | `census:Job` | `census:match` | 0.. | unit node; each occurrence has exactly one reifier |
| index | match reifier (untyped) | `rdf:reifies` | 1 | `<<( job census:match unit )>>`, asserted in the document |
| index | match reifier | `census:matchClass` | 1 | `census:Home`, `census:Variant` or `census:Copy` |
| debt | `census:DebtRecord` (`<>`) | `census:normalForm`, `census:reportForm` | 1 each | integer |
| debt | `census:BodyDebt` / `census:ShimDebt` | `census:memberSymbol` | 2.. | string |
| debt | `census:JobDebt` | `census:jobId` | 1 | string |
| debt | `census:JobDebt` | `census:copies` | 1 | integer at least 1 |

Further closure rules:

- Every subject has exactly one `rdf:type`, from the classes of its report kind.
- Every unit node is referenced. In the baseline, a unit is referenced by a
  group, a constant or a `census:holds`. In the index, every unit is listed.
- No other subject, predicate, class, individual, datatype or namespace occurs.

`census:tokens` keeps the meaning of the code's `tokens`. The concurrent
token-count work in this worktree makes it the value-form token count. The
vocabulary documentation states whatever the architecture page states.

Parity with the JSON, and the deliberate changes (API breaks are allowed):

- `normal_form` becomes `census:normalForm`, an integer.
- A constant's `value` becomes a decimal `xsd:integer`. JSON needed hex text
  because of its number limits; `xsd:integer` is arbitrary precision.
- JSON's `"body": null` becomes absence (0..1). The report is complete by its
  class contract, so absence means "none". The JSON fixture "a missing `body`
  key is refused" becomes "two `census:body` are refused".
- Hex tables become `census:holds` on the unit node, so a unit that is a group
  member and holds a table is one node.
- Index groups gain `census:tokens`, so groups have one shape in both reports and
  their order is derivable.
- Matches point at the unit node, so they gain file and line, which `JobMatch`
  already has.
- Jobs are ordered by id. A report has no other way to carry the ledger's order,
  and the ledger's order is not information. Duplicate job ids are refused
  (`Census::index` returns `Result`), because two jobs would mint one IRI.

### 2.4 Where the version lives

- `census:normalForm` on the report node is what the reader returns as
  `normal_form`. The caller applies `check_normal_form`. The reader accepts any
  declared version, because migration reads an older index.
- The `body:`/`shim:` prefix IRIs carry the same N. The reader refuses
  disagreement (`nf/2` prefixes under `census:normalForm 1`).
- `census:reportForm` (new constant `REPORT_FORM = 1`) is the layout version: the
  vocabulary, the shape and the canonical layout. The reader refuses another
  value with an error that names both, mirroring `check_normal_form`. Closed
  shapes make every vocabulary addition a breaking change. Migration therefore
  needs an explicit layout gate to read an older index safely, which the JSON
  never had. A change to `SHAPES` or to the layout bumps `REPORT_FORM`.
- `VERSION "1.2"` is the first line of every report: the RDF 1.2 announcement.
  It is required. `"1.2-basic"` is refused, because the reports use triple
  terms.

### 2.5 Where triple terms are the right model, and where they are not

RDF 1.2 triple terms are for statements about statements. Each candidate was
tested against that:

- **Job match class: yes.** `job census:match unit` is the job's finding (the
  unit carries something the job forbids). Home, variant or copy is a second
  judgement *about that finding*, grounded in different facts: the job's home,
  entry points and variant list. A named reifier
  (`<#match/sum/…> rdf:reifies <<( <#job/sum> census:match <#unit/…> )>>`) gives
  the occurrence a stable name. The designed capabilities will hang further
  statements on that name: the CLI prints the evidence (`[name ^hex_encode.*]`),
  the ledger has variant reasons, and staleness findings refer to it.
  Sub-properties (`census:copyMatch`) were the alternative. They are lossless
  today but give the match no node to annotate.
- **Group membership: no.** A membership carries no datum of its own. Tokens
  belong to the group, and member order is derived from the members' keys. A
  triple term there would be decoration.
- **Unit to fingerprint, unit to table: no.** These are plain attributes.

The written form is the Turtle 1.2 annotation with a named reifier:
`s p o ~ <#match/…> {| census:matchClass census:Copy |}`. It asserts the triple
and states the reifier in one place. Validated: purrdf-rdf expands each to the
asserted triple, `rdf:reifies <<( … )>>` and the annotation. Only one level of
annotation exists, so the reader never recurses.

---

## 3. Canonical serialisation

Equal reports give equal bytes, and every report has exactly one spelling.

1. UTF-8, no BOM, LF only, no tab, no trailing space, no comment, exactly one
   final LF.
2. Line 1 is `VERSION "1.2"`. Line 2 is `BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/{digest}>`.
3. Then the fixed prefix block of the report kind, one `PREFIX label: <iri>` line
   per prefix, sorted by label: baseline `body census shim table`, index
   `body census shim`, debt `census`. The block is fixed, not "only used
   prefixes", so it never moves with content. Then one blank line.
4. Blocks are separated by exactly one blank line, in this order:
   - baseline: report node, groups, shared constants, units;
   - index: report node, groups, jobs, units;
   - debt: report node, debt groups, job debts.

   Groups follow census order (member count descending, tokens descending,
   fingerprint ascending), which is derivable, so there is no rank property.
   Constants are ascending by value, jobs ascending by id, and units by symbol,
   then file, then line. Debt groups are by kind, then member list; job debts by
   id.
5. A block is `subject a census:Class ;`, then one line per predicate,
   `    predicate object ;`, with the last ending ` .`. Predicates follow
   code-point order of their full IRI (all are `census:`, so local-name order).
   Further objects of one predicate continue as ` ,` then a new line indented 8
   spaces. Multi-objects follow model order: members and matches by symbol, file
   and line; `holds` by table name.
6. Terms:
   - An IRI in a fixed namespace with a valid local is a prefixed name; any
     other IRI is a relative fragment through `term_syntax::write_iri`.
   - `rdf:type` is `a`.
   - Strings use `term_syntax::write_literal` (canonical N-Triples escaping,
     `xsd:string` elided).
   - Integers are bare canonical decimals (no sign, no leading zero).
   - Booleans are `true` or `false`.
   - The match annotation is `object ~ <#match/…> {| census:matchClass census:X |}`.
7. The digest is computed last, over the text without line 2, and line 2 is
   inserted.

The canonical text of the built-in workspace follows. Its numbers are those of
the committed JSON golden at HEAD. The value-form token work in progress here
may change `tokens` and the fingerprints, so the implementation pastes the
example's real output. These exact bytes parse in purrdf-rdf 3.0.1 with
**no caller base**: 36 and 79 triples, 3 triple terms. The digests below are the
real BLAKE3 of these sample bodies (`b3sum 1.8.7`).

Baseline (1668 bytes):

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/301ffdbbcfad67b584bc8139bec54c513872136a1dde48d4e41c9c1052b781b1>
PREFIX body: <https://blackcatinformatics.ca/raf/census/nf/1/body/>
PREFIX census: <https://blackcatinformatics.ca/raf/census#>
PREFIX shim: <https://blackcatinformatics.ca/raf/census/nf/1/shim/>
PREFIX table: <https://blackcatinformatics.ca/raf/census/table/>

<> a census:Baseline ;
    census:minTokens 20 ;
    census:normalForm 1 ;
    census:reportForm 1 ;
    census:unitCount 7 .

<#group/body:6bb022e241a79a28> a census:BodyGroup ;
    census:fingerprint body:6bb022e241a79a28 ;
    census:member <#unit/demo::alpha@src/lib.rs:1> ,
        <#unit/demo::beta@src/lib.rs:2> ;
    census:tokens 21 .

<#group/shim:203616cb54a3e53d> a census:ShimGroup ;
    census:fingerprint shim:203616cb54a3e53d ;
    census:member <#unit/demo::one@src/lib.rs:4> ,
        <#unit/demo::two@src/lib.rs:5> ;
    census:tokens 6 .

<#unit/demo::alpha@src/lib.rs:1> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 1 ;
    census:symbol "demo::alpha" .

<#unit/demo::beta@src/lib.rs:2> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 2 ;
    census:symbol "demo::beta" .

<#unit/demo::digits@src/lib.rs:6> a census:Unit ;
    census:file "src/lib.rs" ;
    census:holds table:hex-lower ;
    census:line 6 ;
    census:symbol "demo::digits" .

<#unit/demo::one@src/lib.rs:4> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 4 ;
    census:symbol "demo::one" .

<#unit/demo::two@src/lib.rs:5> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 5 ;
    census:symbol "demo::two" .
```

Index (2955 bytes):

```turtle
VERSION "1.2"
BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/b1cc0333471f5c6b13fe484183d18cb8236eabe50acfefc3b67a574ef7d4cd6d>
PREFIX body: <https://blackcatinformatics.ca/raf/census/nf/1/body/>
PREFIX census: <https://blackcatinformatics.ca/raf/census#>
PREFIX shim: <https://blackcatinformatics.ca/raf/census/nf/1/shim/>

<> a census:Index ;
    census:normalForm 1 ;
    census:reportForm 1 .

<#group/body:6bb022e241a79a28> a census:BodyGroup ;
    census:fingerprint body:6bb022e241a79a28 ;
    census:member <#unit/demo::alpha@src/lib.rs:1> ,
        <#unit/demo::beta@src/lib.rs:2> ;
    census:tokens 21 .

<#group/shim:203616cb54a3e53d> a census:ShimGroup ;
    census:fingerprint shim:203616cb54a3e53d ;
    census:member <#unit/demo::one@src/lib.rs:4> ,
        <#unit/demo::two@src/lib.rs:5> ;
    census:tokens 6 .

<#job/sum> a census:Job ;
    census:copies 2 ;
    census:homeSymbol "demo::alpha" ;
    census:jobId "sum" ;
    census:match <#unit/demo::BIG@src/lib.rs:7> ~ <#match/sum/demo::BIG@src/lib.rs:7> {| census:matchClass census:Copy |} ,
        <#unit/demo::alpha@src/lib.rs:1> ~ <#match/sum/demo::alpha@src/lib.rs:1> {| census:matchClass census:Home |} ,
        <#unit/demo::beta@src/lib.rs:2> ~ <#match/sum/demo::beta@src/lib.rs:2> {| census:matchClass census:Copy |} .

<#unit/demo::BIG@src/lib.rs:7> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 7 ;
    census:package "demo" ;
    census:shipping true ;
    census:symbol "demo::BIG" ;
    census:tokens 0 .

<#unit/demo::alpha@src/lib.rs:1> a census:Unit ;
    census:body body:6bb022e241a79a28 ;
    census:file "src/lib.rs" ;
    census:line 1 ;
    census:package "demo" ;
    census:shipping true ;
    census:symbol "demo::alpha" ;
    census:tokens 21 .

<#unit/demo::beta@src/lib.rs:2> a census:Unit ;
    census:body body:6bb022e241a79a28 ;
    census:file "src/lib.rs" ;
    census:line 2 ;
    census:package "demo" ;
    census:shipping true ;
    census:symbol "demo::beta" ;
    census:tokens 21 .

<#unit/demo::digits@src/lib.rs:6> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 6 ;
    census:package "demo" ;
    census:shipping true ;
    census:symbol "demo::digits" ;
    census:tokens 1 .

<#unit/demo::helper@src/lib.rs:3> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 3 ;
    census:package "demo" ;
    census:shipping true ;
    census:symbol "demo::helper" ;
    census:tokens 3 .

<#unit/demo::one@src/lib.rs:4> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 4 ;
    census:package "demo" ;
    census:shim shim:203616cb54a3e53d ;
    census:shipping true ;
    census:symbol "demo::one" ;
    census:tokens 6 .

<#unit/demo::two@src/lib.rs:5> a census:Unit ;
    census:file "src/lib.rs" ;
    census:line 5 ;
    census:package "demo" ;
    census:shim shim:203616cb54a3e53d ;
    census:shipping true ;
    census:symbol "demo::two" ;
    census:tokens 6 .
```

---

## 4. Strict reader

### 4.1 Accepted syntax: a closed profile of Turtle 1.2

- Directives: SPARQL-style only, in the order `VERSION`, `BASE`, `PREFIX`…
  (`@prefix`, `@base` and `@version` are refused; the canonical form never
  writes them).
- Terms:
  - `IRIREF`, with `UCHAR` decoded by `purrdf_lex::terminals::decode_uchar`;
  - `PNAME_NS`/`PNAME_LN` over the purrdf-lex `PN_CHARS*` predicates, with no
    PLX escapes;
  - the keyword `a`;
  - `STRING_LITERAL_QUOTE` (`ECHAR` through `echar_value`, plus `UCHAR`);
  - unsigned `INTEGER`;
  - `true` and `false`.
- Punctuation: `.`, `;`, `,`, `~`, `{|`, `|}`. The grammar is
  `subject predicateObjectList '.'`, where an object may be followed by one
  `~ iri {| predicateObjectList |}`. The annotation block holds no annotation.
- Refused with `Syntax { line, column, reason }` naming the construct:
  - blank nodes (`_:x`, `[]`);
  - collections;
  - the `<<( )>>` and `<< >>` forms (a report writes only the annotation form);
  - long, single-quoted or language-tagged strings and `^^` literals;
  - decimals and doubles;
  - comments;
  - an anonymous reifier (`{|` without `~`);
  - a nested annotation.

Nesting depth is one by construction, so the parser is iterative, cannot
overflow the stack and runs in linear time. It needs no depth bound of its own.
Byte caps belong to the file-reading callers (section 7).

### 4.2 Pipeline: check order fixes which error a fixture sees

1. Lex and parse into ground triples plus reifier triples (`Term = Iri | Literal
   | TripleTerm`).
2. Header checks: `VERSION "1.2"` comes first, `BASE` has the genid form, and the
   prefix block is the report kind's fixed block.
3. Shape: the report node's class must be the kind asked for. Reading an index
   as a baseline is `Kind { expected, found }`. Then classes, predicates,
   cardinalities, datatypes and ranges, individuals, reference closure (dangling
   objects, orphan units), and consistency:
   - a minted fragment must equal the fragment re-minted from the node's
     properties;
   - a group's class must agree with its fingerprint kind;
   - `body:`/`shim:` must agree with `census:normalForm`;
   - `census:copies` must equal the count of `census:Copy` matches;
   - every match must have exactly one reifier, reifying an asserted match;
   - `census:reportForm` must be 1;
   - job ids must be unique.

   The result is the typed `Baseline` or `Index`.
4. Canonical: re-render the typed value without line 2 and compare it with the
   input without line 2. The first differing line is `NotCanonical { line }`.
5. Digest: BLAKE3 of the input without line 2 must equal the `BASE` digest,
   else `Digest { declared, computed }`.

Semantic errors come first, so a negative fixture made by editing the canonical
text reports its own fault, not "not canonical" or "digest". A whitespace-only
edit reports `NotCanonical`. A consistent re-edit of values without re-digesting
reports `Digest`.

### 4.3 Error type

`Error::Report(String)` becomes `Error::Report(ReportError)`, with a
`#[non_exhaustive]` `ReportError` whose variants are: `Syntax`, `Version`,
`Form { declared, implemented }`, `Kind`, `Class`, `UnknownPredicate`,
`Missing`, `Repeated`, `Value`, `Dangling`, `Orphan`, `Name`, `Inconsistent`,
`NotCanonical`, `Digest` and `DuplicateJob`. Each variant's `Display` names the
subject (as its fragment) and the predicate or term, so the substring
assertions of today's fixtures map directly. Separately, `tests/support/builtin.rs`
currently wraps a `DiskTree::new` failure in `Error::Report`. That is a load
failure, not a report failure, and it should use the load variant.

### 4.4 Negative fixtures, mirroring `deny_unknown_fields`

Each fixture edits the canonical built-in text. Every refusal sits beside the
unedited text, which reads (AGENTS.md: a refusal is a claim too).

| JSON fixture today | RDF fixture |
| --- | --- |
| unknown top-level key `extra` | `census:extra 1 ;` on `<>`: `UnknownPredicate` naming `census:extra` |
| unknown nested key `column` in a member | `census:column 1 ;` on a unit: names `census:column` |
| unknown key `owner` on a job | `census:owner "x" ;` on `<#job/sum>` |
| missing `min_tokens` | line `census:minTokens 20 ;` removed: `Missing census:minTokens` on `census:Baseline` |
| `normal_form: "one"` | `census:normalForm "one"`: `Value`, integer expected |
| class `"duplicate"` | `census:matchClass census:Duplicate`: `Value` naming the individual |
| missing `body`/`shim` key | (no analogue: absence is "none") replaced by two `census:body` on a unit: `Repeated` |
| index read as baseline and back | `Kind` both ways |
| new | `a census:Widget`: `Class`; a predicate in a foreign namespace; a member with no unit block: `Dangling`; a baseline unit referenced by nothing: `Orphan`; `census:line 9` under fragment `:1`: `Name`; a `census:BodyGroup` with a `shim:` fingerprint, or `census:copies 3`: `Inconsistent`; `nf/2/` prefixes under `normalForm 1`; `census:reportForm 2`: `Form` naming 2 and 1; no `VERSION`, `VERSION "1.1"`, `"1.2-basic"`; `_:b`, `[]`, `( )`, `# c`, `"""x"""`, `@prefix`, `{|` without `~`; an extra space: `NotCanonical`; one hex digit of `BASE` changed: `Digest`; duplicate job ids through `Census::index`: `DuplicateJob` |

---

## 5. Rust API and module layout (breaking)

- `raf-census/Cargo.toml`:
  - remove `serde` and `serde_json`;
  - add `purrdf-lex = "=3.0.1"`, with a pin comment;
  - description becomes "… grouping and the RDF 1.2 baseline and index reports."
- `GroupKind` and `MatchClass` lose their serde derives. Their IRIs live in the
  report module, not on the enums.
- `src/report.rs` becomes `src/report/`:
  - `mod.rs`: typed rows, and `Census::baseline` / `Census::index`;
  - `vocab.rs`: namespaces, term constants, `SHAPES`, `REPORT_FORM`;
  - `mint.rs`: encode sets and fragment minting;
  - `write.rs`: canonical writer and digest;
  - `read.rs`: lexer and profile parser;
  - `shape.rs`: closed shape from graph to typed value, also used by the writer
    to validate.

  The Turtle layer (`write`, `read`, term model) holds no census type, so it can
  later become a shared crate (section 8.5).
- Typed changes:
  - `BaselineGroup.fingerprint`, `IndexGroup.fingerprint` and
    `IndexUnit.{body, shim}` become `Fingerprint` / `Option<Fingerprint>`;
  - `TableRow.table` becomes `TableKind`;
  - `ConstantRow.value` becomes `u128`;
  - `IndexGroup` gains `tokens`;
  - `MatchRow` gains `file` and `line`;
  - `Index.jobs` is sorted by id.
- `Baseline::to_turtle(&self) -> Result<String, Error>` and
  `Index::to_turtle(...)`. The writer validates the typed value against `SHAPES`,
  so every text it writes is a text the reader reads. A hand-built group of one
  member is refused.
- `Baseline::from_turtle(&str) -> Result<Baseline, Error>` and
  `Index::from_turtle(...)`.
- `Census::index(&self, jobs: &[JobReport]) -> Result<Index, Error>`
  (`DuplicateJob`).
- New exports: `REPORT_FORM`, `ReportError`, and `report::vocab` (namespace
  constants, `fingerprint_namespace(normal_form, kind)`, `SHAPES`) for the cli,
  migration and debt-ratchet code.
- `lib.rs` crate docs: the sentence "both are closed schemas, `to_json` writes
  sorted keys …" becomes "both are RDF 1.2 Turtle documents with a closed shape:
  `to_turtle` writes the one canonical spelling and `from_turtle` reads back
  exactly that."

---

## 6. Exact replacement list in this change

### 6.1 Code and configuration

| File | Change |
| --- | --- |
| `crates/raf-census/Cargo.toml` | as section 5 |
| `crates/raf-census/src/report.rs` | replaced by `src/report/{mod,vocab,mint,write,read,shape}.rs` |
| `crates/raf-census/src/lib.rs` | docs paragraph; `pub use report::{…, REPORT_FORM, ReportError}`; `pub mod` path for `vocab` |
| `crates/raf-census/src/error.rs` | `Report(ReportError)`; doc "a report could not be rendered, or a text is not a report"; the `each_variant_renders_its_facts` test gains a `Report` case |
| `crates/raf-census/src/group.rs`, `src/job.rs` | drop `use serde`, the derives and `#[serde(rename_all)]` |
| `crates/raf-census/tests/support/builtin.rs` | `render` uses `to_turtle()?` and `census.index(&reports)?`; `of_dir` stops wrapping a `DiskTree` failure as `Error::Report` |
| `crates/raf-census/examples/baseline.rs` | docs only: "prints the baseline and the index report as RDF 1.2 Turtle"; output shape `baseline\n<ttl>index\n<ttl>` kept |
| `deny.toml`, `fuzz/deny.toml` | the `purrdf-lex@=3.0.1` `Unicode-3.0` exception, identical in both |
| `Cargo.lock` | `+purrdf-lex 3.0.1`, `+purrdf-hash 3.0.1`; `serde_json` stays (cargo-workplan) |

### 6.2 `crates/raf-census/tests/reports.rs` (rewritten, names kept where the claim survives)

- `BUILTIN`: the two Turtle documents above, under the `baseline`/`index`
  headings.
- `the_example_output_is_the_pinned_golden`: unchanged claim.
- `the_example_output_names_the_expected_group_fingerprints`: asserts
  `body:…`/`shim:…` and `census:jobId "sum"`.
- `the_capability_examples_hold_the_example_output`: blocks are
  ```` ```turtle\n{report}``` ````.
- `both_reports_read_back_what_they_write`: `from_turtle`/`to_turtle`, and equality
  with `census.baseline()`.
- `output_has_sorted_keys_and_one_trailing_newline` becomes
  `output_is_the_canonical_turtle`. It checks line 1, the `BASE` form and that its
  digest is BLAKE3 of the text without line 2, the fixed prefix block, LF only,
  no trailing space, one final LF, `a` first, predicates in IRI order, and no
  `_:` anywhere (ground).
- `an_unknown_key_missing_key_or_wrong_type_is_not_a_report` splits into:
  - `unknown_terms_are_not_a_report`;
  - `missing_repeated_and_mistyped_properties_are_not_a_report`;
  - `names_that_disagree_with_their_node_are_not_a_report`;
  - `constructs_outside_the_profile_are_not_a_report`;
  - `an_edited_or_respaced_text_is_not_a_report`;
  - `an_index_is_not_a_baseline`.

  The fixtures are those of 4.4, each beside the unedited neighbour.
- `the_index_renders_the_classified_matches_of_every_job`: `census.index(…)?`;
  `jobs[0]` is `idle` (sorted by id); asserts
  `census:matchClass census:Variant`.
- `a_shared_constant_is_hexadecimal_text` becomes `a_shared_constant_is_an_integer`:
  `row.value == 0x1_0000_0000_0000_0000_0000`, text holds
  `census:value 1208925819614629174706176`, and the round trip.
- `two_fresh_loads_of_this_repository_give_identical_bytes`: `contains("census:normalForm 1")`.
- `two_fresh_loads_of_the_fixtures_give_identical_bytes` and
  `an_absolute_directory_reports_the_same_as_the_relative_spelling`: unchanged.
- New tests:
  - `duplicate_job_ids_are_refused`;
  - `awkward_names_round_trip`: symbols and files with space, `#`, `%`, `@`,
    `"`, `\` and non-ASCII;
  - `a_report_of_another_report_form_is_refused_naming_both`;
  - `an_index_of_another_normal_form_reads_and_keeps_its_version`: a typed
    `Index` with `normal_form: 2` round-trips with `nf/2/` prefixes; this is the
    migration input;
  - `the_graph_of_each_example_is_its_pinned_triple_set`: the reader's triples as
    sorted N-Triples equal a pinned text. The pinned text is the expansion an
    independent RDF 1.2 parser (purrdf-rdf) produced from the same bytes, which
    gives conformance evidence without a dependency that fails the gate. Its
    expansion was 36 and 79 triples for the samples above.
- Unit tests replacing `report::tests::render_ends_with_one_newline_and_sorts_keys`:
  - `report::mint::tests::fragments_are_injective_over_their_separators`;
  - `report::mint::tests::fragments_are_ascii_and_percent_encode_reserved_bytes`;
  - `report::write::tests::the_digest_covers_everything_but_the_base_line`;
  - `report::read::tests::each_token_outside_the_profile_names_its_line_and_column`;
  - `report::shape::tests::the_writer_refuses_a_value_outside_the_shape`.

### 6.3 `crates/raf-census/tests/page.rs`

- Drop the `BASELINE` and `INDEX` member-name lists and the JSON literal
  documents.
- `the_page_names_every_report_member_of_both_reports` becomes
  `the_page_states_the_closed_shape_of_every_report`. Every `SHAPES` row appears
  in the Reports section as `| report | `census:Class` | `census:property` | count | object |`,
  and the section's shape table has no other row. This is the same pattern as
  `PASSES` and `VALUE_ADAPTERS`.
- `the_member_lists_are_the_closed_schemas` becomes
  `the_shape_table_is_the_reader_shape`. For each class, the built-in text with
  one required property removed is refused as `Missing`, and with one property
  outside the table added is refused as `UnknownPredicate`. The unedited text
  reads.
- Keep the concurrent additions (value-form threshold and numbering tests)
  untouched.

### 6.4 Documents in this change

- `docs/architecture/normal-form.md`, section **Reports**, rewritten:
  - The census renders two RDF 1.2 reports as Turtle 1.2: ground graphs over the
    `census:` vocabulary, fingerprint IRIs per normal form, fragment IRIs under
    the content-addressed `BASE`, the named reifier of each job match, the
    canonical layout rules of section 3, and `census:reportForm`.
  - Then the closed-shape table of 2.3 (the page test reads it).
  - The sentence "Units sort by symbol, then file, then line" stays as a layout
    rule.
  - "Compatibility": add that a report names its `census:normalForm`, that the
    reader returns it for the caller's `check_normal_form`, and that
    `census:reportForm` gates the layout.
  - "Version-bump rule": add one sentence. A change to the report vocabulary,
    shape or layout bumps `census:reportForm`, not `normal_form`.
- `docs/capabilities/census-normal-form.md`:
  - frontmatter `bindings.code`: remove `Baseline::from_json`,
    `Baseline::to_json`, `Index::from_json` and `Index::to_json`; add
    `Baseline::from_turtle`, `Baseline::to_turtle`, `Index::from_turtle`,
    `Index::to_turtle`, `REPORT_FORM` and `ReportError`;
  - `bindings.tests`: the renamed and new tests of 6.2 and 6.3;
  - Contract: the report bullet rewritten (members become vocabulary terms,
    "closed shape", "one canonical spelling", "content-addressed base",
    "constants are `xsd:integer`"); the `Error` bullet names `Report` as a typed
    report error;
  - Usage: "`Census::index` refuses duplicate job ids";
  - Examples: the two ```` ```json ```` blocks become ```` ```turtle ````
    blocks with the example's exact output, and the lead-in mentions the
    annotated matches;
  - Verification: "a round trip through their Turtle, refusal of an unknown
    predicate or class, a missing, repeated or mistyped property, a name that
    disagrees with its node, a construct outside the profile, a respaced text and
    an edited text whose base no longer names it", "the graph of each Example is
    pinned as N-Triples", and "the page states the closed shape";
  - regenerate the verification matrix with `make docs-sync`.
- `work/assessments/census-normal-form-{purpose,contract,examples,constraints,verification}.md`:
  these bind `Baseline::from_json`, `Baseline::to_json`, `Index::from_json` and
  `Index::to_json`, and their section digests move. Re-run the assessment
  workflow so they are current. They are also being edited concurrently, so
  regenerate them last.
- `CHANGELOG.md` `[Unreleased]` `raf-census` entry: "renders the
  byte-deterministic baseline and index JSON reports" becomes "renders the
  baseline and index reports as RDF 1.2 Turtle (ground graphs over the census
  vocabulary, one canonical spelling, a content-addressed base, a named reifier
  for each job match) and reads them back through a strict reader of their
  closed shape". Add the new dependency sentence, in the style of the
  `cargo-workplan` entry: `purrdf-lex` 3.0.1 (pinned), with the one scoped
  `Unicode-3.0` exception; `serde` and `serde_json` are no longer dependencies of
  `raf-census`.
- `README.md` line 34 and `AGENTS.md` lines 67 and 76: "the baseline and index
  reports" becomes "the RDF 1.2 baseline and index reports". AGENTS.md is also
  modified concurrently; apply this as a one-phrase edit.
- `fuzz/corpus/parse-document/seed_*census*` are fuzz inputs, not contracts.
  Leave them.

---

## 7. Downstream designed capabilities: doc edits

### 7.1 `docs/capabilities/census-cli.md`

- Table: `baseline [--json PATH]` becomes `baseline [--out PATH]`: "Prints every
  census group. With `--out PATH`, writes the baseline report (RDF 1.2 Turtle) to
  `PATH`." There is one format, so the flag names no format.
- `index`: "Prints the index report, every unit with its fingerprints, as RDF 1.2
  Turtle."
- `debt --write`: `census-debt.json` becomes `census-debt.ttl` (also in the
  Gate paragraph).
- `migrate --old-baseline <FILE> …` becomes
  `migrate (--old-baseline <FILE> | --old-index <FILE>) --ledger <FILE> --out <FILE>`
  (see 7.2).
- Usage block: `cargo census baseline --out /tmp/baseline.ttl`, and
  `cargo census migrate --old-baseline clone-baseline.json --ledger old.toml --out helpers-ledger.toml`
  stays for the legacy form.
- Exit codes: add to code 2 "a report file that is not a census report (the
  reader's refusal)".
- Constraints: "besides `baseline --out PATH` and `migrate --out`". Add "Report
  output is the canonical Turtle of census-normal-form, so a run's output is
  byte-identical to `Baseline::to_turtle`/`Index::to_turtle`."
- Verification: "`baseline --out PATH` writes a file byte-equal to the baseline
  report of census-normal-form, which `Baseline::from_turtle` reads" and "`index`
  stdout is byte-equal to `Index::to_turtle`".

### 7.2 `docs/capabilities/census-migration.md`

- Contract command line: split the inputs. `--old-baseline <legacy.json>` (the
  legacy baseline of an earlier census copy, with a legacy ledger) and
  `--old-index <index.ttl>` (the index report the older normal form wrote, with a
  current-format ledger). Exactly one is required and must match the ledger kind,
  otherwise exit 2. This replaces the one overloaded flag; RAF never sniffs a
  format.
- "In an index report, the members of a fingerprint are the units that list it"
  becomes "… the units whose `census:body` or `census:shim` names it".
- New Contract bullets:
  - The index is read with `Index::from_turtle`, which refuses any text outside
    its closed shape.
  - Its `census:normalForm` must be older than the implemented one, otherwise
    exit 2 ("nothing to migrate").
  - A `census:reportForm` the command does not know is a usage error naming both
    forms.
  - The legacy JSON baseline is read with `purrdf_lex::json` (already a
    dependency, no `serde_json`), and only in this import path.
- Usage: `cargo census index > index-before-upgrade.ttl` and
  `cargo census migrate --old-index index-before-upgrade.ttl --ledger helpers-ledger.toml --out helpers-ledger.new.toml`.
- Constraints bounds:
  - The ledger and the legacy baseline keep the 1 MiB and 32-level bounds.
  - The index report is read under `max_file_bytes`. An index of a large
    workspace exceeds 1 MiB, and the reader nests one level by construction.
  - A larger file is a typed error naming the file.
- Verification:
  - "the index report that version wrote" becomes a fixture Turtle index at
    `nf/0` or `nf/2`, as the test needs;
  - add "an index that is not canonical, has an unknown predicate or a base that
    does not name it fails naming the file and writes no output";
  - add "`--old-baseline` given with a current-format ledger, or `--old-index`
    with a legacy one, exits 2";
  - the 33-level fixture applies to the legacy JSON only.

### 7.3 `docs/capabilities/census-debt-ratchet.md`

- Frontmatter `description`: "… stored in census-debt.ttl". Run `make docs-sync`
  so `docs/capabilities/index.md` follows.
- Every `census-debt.json` becomes `census-debt.ttl`, in the Contract, Usage,
  Examples (`census-debt.ttl: 1 group, 0 jobs with copies`) and Constraints.
- The ```` ```json ```` record becomes this canonical Turtle record (shape rows
  `debt` in 2.3):

  ```turtle
  VERSION "1.2"
  BASE <https://blackcatinformatics.ca/.well-known/genid/raf-census/{digest}>
  PREFIX census: <https://blackcatinformatics.ca/raf/census#>

  <> a census:DebtRecord ;
      census:normalForm 1 ;
      census:reportForm 1 .

  <#debt/body/demo_a::len,demo_b::len> a census:BodyDebt ;
      census:memberSymbol "demo_a::len" ,
          "demo_b::len" .

  <#job/hex-encode> a census:JobDebt ;
      census:copies 3 ;
      census:jobId "hex-encode" .
  ```
- Constraint "The file is JSON with a stable key order and a trailing newline"
  becomes "The file is RDF 1.2 Turtle in the census's canonical form, read back
  through the same strict reader, so equal debt gives byte-equal files and a
  hand edit is refused."
- Explanation addition: the record's base line changes with every content
  change. A review diff shows one changed base line beside the changed debt.
- Verification: add "a hand-edited record (a member added without re-digesting)
  fails the check as not a debt record, naming the file".

Also `docs/capabilities/census-rule-api.md` (lines 62 and 193),
`work/changes/census-debt-ratchet.md` (description) and
`work/changes/census-rule-api.md` (claim): `census-debt.json` becomes
`census-debt.ttl`.

### 7.4 `docs/capabilities/census-ledger.md`

The ledger stays TOML. It is a hand-edited input, and the decision scoped the
change to reports.

- Constraints: "Only TOML is read. The census has no hand-written parser." is
  false after this change. It becomes: "The ledger is read only as TOML, through
  the `toml` crate. The census's one hand-written reader is the report reader of
  census-normal-form, which reads only the census's own canonical RDF 1.2
  reports."
- Contract (`normal_form` bullet): add that the ledger's `normal_form` and a
  report's `census:normalForm` pass through the same `check_normal_form` gate.
- The note "read with the `toml` crate into `serde` structures" stays. When the
  ledger is built it brings `serde` back into `raf-census` through `toml`, which
  this change removes for now because the reports no longer use it.

---

## 8. Findings outside this change (each a tracked change of its own)

1. **`cargo-workplan` still uses `serde_json`** (`src/{assess,change,command,evidence,render}.rs`
   and its tests). Under "if you find yourself using JSON you have failed", its
   `--json` outputs are the next RDF 1.2 migration, and they would reuse the
   Turtle layer of 8.5.
2. **`raf-source` does not compile for any 32-bit target, wasm32 included.**
   `crates/raf-source/src/glob_scope.rs:113` has
   `const COMPANION: usize = 1 << 40;`, which gives E0080 / `arithmetic_overflow`
   on `wasm32-unknown-unknown`. So `raf-census` cannot be wasm-built today, and
   neither can anything above it. The fix is a `u64` level, plus a wasm32
   `cargo check` in `make cross-check`.
3. **Two BLAKE3 implementations.** `purrdf-hash` (now in the graph) has a native,
   wasm-clean BLAKE3. Moving fingerprints from `blake3 =1.8.7` to
   `purrdf_hash::blake3` removes `blake3`, `arrayvec`, `constant_time_eq`,
   `cpufeatures` and the `cc` build chain. What is hashed does not change, so
   `normal_form` does not move. The goldens prove equality, and the report digest
   moves with it.
4. **Upstream purrdf:** split the line/Turtle text codec out of `purrdf-rdf`
   into a lean crate with no SPARQL, GTS, crypto or rayon, and no wasm-bindgen in
   the default graph. Once that passes RAF's bans gate, RAF's profile parser
   becomes `parse_dataset` plus `shape.rs`, and the pinned N-Triples golden
   becomes a live oracle test. This is the SUBSUME/EXTEND path.
5. **Publish the vocabulary:** render `SHAPES` as an RDF 1.2 vocabulary plus
   SHACL 1.2 shapes (`sh:closed true`). Any SHACL engine could then validate a
   RAF report, and a test would pin the rendered document.
6. The ledger as TOML against "RDF 1.2 only" is a strategic question for the
   ledger capability, not for this change.

---

## 9. Probe artefacts (scratch only, reproducible)

- `$S/scratch/rdfprobe/ws`: copy of the workspace manifests and lock. The
  variants `ws-lexonly` (chosen route), `ws-lex`, `ws-core`, `ws-rdf` and
  `ws-umbrella` each add one route to `crates/raf-census/Cargo.toml`.
  `cargo deny --offline --workspace check bans licenses sources` and
  `cargo tree --workspace` were run on each.
- `$S/scratch/rdfprobe/{base,lexonly,lex,core,rdf,umbrella}.txt`: lock package
  sets per route.
- `$S/scratch/rdfprobe/oracle`: a standalone crate on `purrdf-rdf =3.0.1` that
  parses Turtle (`NOBASE=1` passes no caller base) and prints N-Triples.
- `$S/scratch/rdfprobe/samples/{baseline,index}.final.ttl`: the section 3
  documents. `final.out` is their independent N-Triples expansion (36 and 79
  triples). `awkward.ttl` is the escaping check. `oracle.out` and `based.out` are
  the no-`BASE` and wrong-digest-rule iterations.
