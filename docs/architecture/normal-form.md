---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:normal-form
title: "The census normal form, version 1"
description: "The normative definition of how the RAF census turns a function body into a fingerprint."
---

# The census normal form, version 1

The census finds duplicated helpers by comparing what function bodies say, not how
they are spelled. The normal form is the rewrite that makes two spellings of one
algorithm identical, and the fingerprint is the stable name of the result. This
page is the normative definition of version 1. The ledger declares it as
`normal_form = 1`, and every fingerprint a ledger names is a version 1
fingerprint.

The capability that applies this definition is
[census-normal-form](../capabilities/census-normal-form.md). The OKF document
shape of this page is stated in [the profile](./okf-profile.md).

## Decision

The census compares function bodies by their tokens under one versioned normal
form and names each result by a fingerprint. The form is a fixed sequence of four
pure passes over the lexed tokens. The names a body binds and its parameters are
renamed by order of first appearance, the reference and adapter noise that does not
change a value is removed, a body whose value form has fewer than 30 tokens keeps
its literals verbatim, and a body whose value form has 6 to 29 tokens that is a thin
forwarder also has a shim form that groups forwarders. The fingerprint of a body or
a shim is the complete 32-byte BLAKE3 hash of the form's token sequence under
one fixed byte encoding. A version is fixed by the content of its first merge to the
default branch: after that, any change that can alter a fingerprint is a new
version, and a ledger written under another version is refused until
`cargo census migrate` rewrites it.

## Token source

A body is lexed with `proc_macro2`, so the census sees tokens, not text. Whitespace
and ordinary comments (`//` and `/* */`) are not tokens. A doc comment inside a body
is not a comment to the lexer: it is the attribute tokens `# [ doc = "text" ]`, so
its text counts like a string literal and its tokens count toward every threshold.
The names inside an attribute (`#[..]` and `#![..]`) are the attribute's own and are
never renamed as locals. The input is the text between the braces of a
function body, or the initialiser expression of a constant or a static, never its
type or attributes. The arguments of macro invocations are token trees too, so
`format!("{x:02x}", x = b)` and a call with the same shape compare by their tokens
like any other code. The names that a function's parameters bind and the extent of a
constant's initialiser are not found by parsing the item again: they are facts of the
parse of `raf-source`, which runs on its own budgeted stack, so the census parses no
signature and no item, a signature too deep for that parse is a load error that names
the recursion budget. Unit collection runs on a fresh owned thread, whose
thread-local lexer storage ends with it; it never invalidates caller-owned spans.
The supplied Tree remains on the caller's thread and need not be Send or Sync.

- A token is an identifier, a literal, a punctuation character or a delimiter.
  Both delimiters of a group are tokens, and every threshold counts them. A
  punctuation character that is joined to the next one (`&&`, `..`) is a different
  token from the same character alone.
- Every integer literal is respelled at lex time to its decimal value followed by
  its type suffix, so `0xff`, `0b1111_1111` and `255` are the token `255`, and
  `1_000` is `1000`. An integer beyond `u128` keeps its digits without
  underscores. A float is as written. In this page "verbatim" means the tokens as
  lexed with integers in this decimal spelling.
- A source slice that does not lex, or that nests delimiters deeper than
  `MAX_NESTING` (128), is an error that names the file and line. There is no
  fallback and no partial result.

## The passes

The normal form is the sequence of four passes below. Each is a pure function from
one token tree to the next, none adds a token, and every token keeps its source
position. The forms after each pass are the levels of the form, and the levels are
ordered from the finest to the coarsest:

| Level | Produced by | Contents |
| --- | --- | --- |
| `verbatim` | the lexer | The tokens as lexed. |
| `shim` | `rename_params` | The function's own parameters renamed, in signature order. |
| `literal-tier` | `rename_locals` | The parameters and the names the body binds renamed together, by first appearance, literals kept. |
| `structural` | `kind_literals` | Literals abstracted to their kind unless the value form is under the literal tier. |
| `value` | `erase_held` | The spellings that only change how a value is held dropped. |

Two bodies with the same parameter list that are equal at one level are equal at
every coarser level, and a level never has more tokens than the finer one before
it. The `verbatim` level is the text alone, so two bodies with the same text and
different parameter lists can differ at `shim` and are equal again from
`literal-tier`. The pass ids, in order,
are `rename_params`, `rename_locals`, `kind_literals` and `erase_held`.

## Structural form

The structural form renames what a body names for itself and keeps what it names
for others. It is the form after `kind_literals`; the renaming is done by the two
passes before it.

- `rename_params` numbers the function's own parameters `$0`, `$1`, … in
  signature order. Every identifier bound by a parameter pattern counts (`mut x`,
  `ref x`, tuple and struct patterns); `self` is not a parameter, and a repeated
  name counts once. A name after a `.`, a field name, a lifetime, the tail of a path
  and a macro name are not parameters. The `shim` level keeps this numbering,
  because the order in which a forwarder passes its arguments is its meaning:
  `f(y, x)` and `f(x, y)` are different shims.
- `rename_locals` renames only the names that the body or its signature binds, and
  it numbers them together with the parameters on one counter, by order of first
  appearance, from the first token of the body, a placeholder in a format string
  included. A parameter the body does not use therefore does not shift the
  numbering, and two bodies that use the same parameters in another order have the
  same form. The names a body binds are:
  - the names of a `let` pattern, an `if let` or `while let` pattern, a `for`
    pattern and a `match` arm pattern, a tuple or struct pattern included, and a
    name bound by `name @ pattern`;
  - the parameters of a closure and of a function declared inside the body, and
    that function's name;
  - the types a body declares for itself (`struct`, `enum`, `union`, `trait`,
    `type`);
  - labels and lifetimes, a macro metavariable (`$x`) and a name given to a macro
    argument (`x = value`).

  Every use of a bound name is renamed with it, in call position (`let f = ..;
  f(v)`) and when it shadows an item. A name inside a token that the value form
  erases (a `let` type annotation, a turbofish, a `ref`) is not numbered and is
  written `$_`.
- Every other identifier is the name of an item the body uses, and is kept: a
  function passed by name (`map(double)` and `map(triple)` are different), a
  constant or a static in any case, a type, a variant, a path segment, a macro and
  a primitive type name. A name in field position is kept even when a local of the
  same name exists: in `S { x }` the field `x` is kept, so `S { x }` and `S { y }`
  are different.
- The block after the head of an `if`, `else if`, `while`, `match` or `for` is a
  block, never a struct literal, so a name before it (`if next == MAX { next }`)
  does not keep the locals inside.
- Inline format captures follow the renaming: in a string literal that is a
  direct argument of a macro invocation, `{name}` and `{name:spec}` placeholders
  that name a renamed parameter or bound name are rewritten to its new name. `{}`,
  `{0}`, a name that is not bound, `{{` and `\u{..}` are kept.
- Field names, method names, keywords, operators and delimiters are kept. The
  reserved word `gen` and the weak keywords `union`, `raw`, `safe` and
  `macro_rules` are keywords for this purpose, and `_` is kept. The closed list is
  `WEAK_KEYWORDS`: `gen`, `union`, `raw`, `safe`, `macro_rules`.
- A tuple index (`pair.0`) is a field and is kept in every form; the end of a
  range (`..1`) is a literal.

Two bodies that differ only in the names they bind, and in the order of their
parameters, have the same structural form.

## Value form

The value form is the structural form after `erase_held`. It drops the reference
and adapter noise that does not change the value an expression computes. These
tokens are removed:

- borrows (`&x`, `&mut x`, `&&x`), derefs (`*x`) and `ref` and `ref mut` bindings;
- a value adapter called with no arguments, from this closed list
  (`VALUE_ADAPTERS`): `as_str`, `as_ref`, `as_slice`, `as_mut`, `as_deref`,
  `borrow`, `borrow_mut`, `clone`, `cloned`, `copied`, `to_owned`;
- a full-range reindex (`x[..]`); a partial range (`x[1..]`) is kept;
- the type annotation of a `let` binding;
- a turbofish that names types only (`::<T>`); a turbofish with a literal, `true`,
  `false` or a braced const argument is kept;
- the reference marker inside an index, so `m[&k]` is `m[k]`.

The binary `&`, `&&` and `*` operators and their compound assignments are kept,
because they change the value.

## Literal tier

A body whose value form has fewer than `LITERAL_TIER` (30) tokens keeps its literals
verbatim: `kind_literals` is the identity pass there. At that size the constants are
most of what the body says: two `match` arms that map the same variants to
different strings are two tables, and two that map them to the same strings are
one table written twice. A body whose value form has 30 tokens or more has its literals abstracted
to their kind: `$int`, `$float`, `$str`, `$char`, `$byte`, `$bytes` and `$cstr`.
An integer literal is an integer by syntax, whatever its size. Integers are
compared by value in any spelling, so `0xcbf2_9ce4_8422_2325` and its decimal
spelling are one constant.

## Shim form

A shim is a short body that forwards to another function. A body is a shim
candidate when it is one expression, which is judged at the top level of the body
only: the body has no `let` and no `;` before its tail, though a closure inside an
argument may hold both, so `g(a, |x| { let y = x; y })` is a candidate. It contains
at least one call, and does not mention `self`. A call is a name or a macro bang
followed directly by a parenthesised group, and a turbofish belongs to its name, so
`f::<T>(x)` is a call. A shim candidate with at least `MIN_SHIM_TOKENS` (6) and
fewer than `SHIM_MAX_TOKENS` (30) value-form tokens has a shim form, which is the
`shim` level: the function's own parameters are renamed, and the callee, the fixed
arguments, the locals and the literals are kept as written. Two functions that
forward to the same place with the same fixed arguments are the same shim.

A shim candidate that reaches `min_tokens` still has a `body` print, which the
index lists, but it never joins a body group: a forwarder is never compared by
structure, only by its shim form.

The `shim_key` is the shim fingerprint a forwarder groups and is matched under. It
is absent for two cases:

- methods in a trait implementation, whose signature a trait fixes, so their
  forwarding is not a free choice;
- forwarders that call through `Self::`, which forward within one type. A body is
  such a forwarder when any call at any depth, in a callee or in an argument, has a
  callee path that starts with `Self::`; a `Self::` constant used as an argument
  (`helper(x, Self::ONE)`) is not a call.

A unit without a `shim_key` is in no shim group and no shim match, though it may
have a shim form. Bodies shorter than 6 tokens have no shim form, so `{ f(x) }`
never forms a group.

## Thresholds

Every threshold counts the tokens of the value form, delimiters included: the
`value` level, which is `erase_held` of the `literal-tier` form. The count is taken
before `kind_literals`, so the literal tier is decided on it too, and two bodies
with the same value form always make the same threshold decisions. Noise the value
form erases never moves a body across a threshold.

| Name | Value | Meaning |
| --- | --- | --- |
| `DEFAULT_MIN_TOKENS` | 20 | The default of `min_tokens`, which a run takes from `CensusOptions::min_tokens`; the ledger key that carries it is the ledger capability's. A body with fewer value tokens has no body fingerprint. |
| `LITERAL_TIER` | 30 | A body with fewer value tokens keeps its literals verbatim. |
| `MIN_SHIM_TOKENS` | 6 | A body with fewer tokens has no shim form. |
| `SHIM_MAX_TOKENS` | 30 | A body with this many tokens or more has no shim form. |
| `MAX_NESTING` | 128 | Delimiters nested deeper than this are refused. |

`min_tokens` is a ledger setting and must be at least 1. The other values are fixed
by the normal form version.

The [counted selection rationale](../census-selection.md) records candidate
floors, literal tiers and shim bounds, labelled outcomes and inspected source
groups. The selected values trade sensitivity for review volume; they are not
a claim of real-source precision or a universal optimum.

## Fingerprint

A fingerprint is `<kind>:` followed by an identifier.

- For `body` and `shim`, the identifier is the complete 32-byte BLAKE3 digest
  written in 64 lowercase hexadecimal digits. A human display can abbreviate it
  to sixteen digits and an ellipsis with `Fingerprint::abbreviated`; canonical
  display, parsing and machine identities always use all 64 digits.
- For the kinds `table` and `rule`, the identifier is a fixed name: `table:hex-lower`
  and `table:hex-upper` are the tables, and a rule id is a non-empty name of ASCII
  letters, digits, `-`, `_` and `.`, such as `rule:std-default-hasher`.

The four kinds are:

| Kind | What it names |
| --- | --- |
| `body` | A function body in its value form: the digest of the `value` level. |
| `shim` | A forwarder in its shim form: the digest of the `shim` level. |
| `table` | A recognised table, such as a hex-digit table. |
| `rule` | A hit of a detector, a declarative pattern or a `Rule`. |

The hashed encoding has the id `tokens-len-u32le-utf8/blake3/hex64`
(`ENCODING`): each token of the form, in order, is written as its UTF-8 length as a
little-endian `u32` followed by its bytes, and the digest is the BLAKE3 hash of the
concatenation. A joined punctuation character is written with a trailing `~`, and a
delimiter as its character. A fingerprint string that has another kind, an
identifier of another width, uppercase hexadecimal digits, an unknown table name or
an empty rule id is refused.

The native Rust BLAKE3 implementation comes from `purrdf-hash`, which has no
runtime dependencies and needs no external C or assembly build. Its portable
hash implementation supports `wasm32`;
complete census loads also require the source dependency to support the target.
The core accesses source only through its supplied `Tree`; normalization, hashing,
grouping and report conversion use owned values and invoke no filesystem or
process APIs. Host reference discovery owns `cargo metadata`.

The census stores one complete digest per body or shim print and uses every byte
as its grouping key. Correctness relies on BLAKE3 collision resistance rather than
detecting a full-digest collision. No separate truncated digest or collision guard
is needed.

## Units

A unit is one source item that has a body or an initialiser. The census collects
units through the source index, once for each source item, whatever number of
targets reach it, so a module declared by a library and a binary yields one unit.

- The kinds are: a function, a method (inherent, in a trait implementation or a
  trait default body), a constant (an associated constant included) and a static.
  Every other kind of item yields no unit, and so does a function or a constant
  that has no body or value.
- A unit has a symbol, a package, a file, a line, a shipping flag, a print, the
  integer constants it holds by value, the tables it holds and the string literals
  it holds with their lines. A byte string, a C string and a string are all string
  literals; a literal that is not valid UTF-8 is never a table.
- Its identity is its file, line and symbol. The symbol is chosen deterministically:
  a path that a shipping target reaches over one only a test target reaches, then
  the smallest path. The unit is shipping when some target that reaches it ships and the item is not test code in that reach.
- The print holds the value-form token count, the `body` fingerprint when the count
  reaches `min_tokens`, and the `shim` fingerprint when the body has a shim form.
- A constant or a static has no print and no fingerprint: its token count in the
  index is 0 and its `body` and `shim` are `null`, and it never joins a body or shim
  group. What it contributes is the integer constants, the string literals and the
  tables read from its initialiser expression, never from its type or attributes.
  Those feed the shared constants, the hex tables and the constants of a job.

## Grouping

The census groups units by fingerprint into a map from fingerprint to members. A
group has two or more members, and a group of one is dropped. A unit with a shim
form is never body-grouped. A unit joins the shim group of its `shim_key` when it
has one.

Groups sort by member count descending, then value tokens descending (the
largest count among the members), then fingerprint ascending. Members sort by
symbol, then file, then line, so the same source always produces the same output.

## Tables and shared constants

A hex-digit table is a string, a byte string, or an array of 16 `char` or `u8`
literals that is exactly the sixteen digits `0123456789abcdef` in lowercase or
`0123456789ABCDEF` in uppercase, in order. Another count, a wrong digit or mixed case
is not a table. The table is reported as `table:hex-lower` or `table:hex-upper`
with the unit that holds it.

A shared large constant is an integer value above `0xffff` that units of two or
more packages hold. Every spelling of one value is one constant.

## Jobs

A job names a home symbol, entry-point symbols, variant symbols and what is
forbidden elsewhere: fingerprints and integer constants. A unit matches a job when
its `body` fingerprint, its `shim_key` or a `table:` id of a table it holds is a
forbidden fingerprint, or when one of its integer constants equals a forbidden
constant by value. A `rule:` fingerprint matches no unit of the census. A match is
`home` when the unit's symbol is the job's home or one of its entry points,
`variant` when it is a listed variant, and `copy` otherwise. A job counts its
copies.

## Reports

The census writes two ground RDF 1.2 graphs as canonical Turtle and N-Triples: the baseline
summarizes repeated bodies, shims, tables and large constants; the index lists
every unit, group and classified job match. Every subject has exactly one type.
Selected JobQuery, CountQuery and FingerprintQuery reports use their own closed
root classes rather than relabeling a partial Index. A job query retains its
Job, Match and full index-unit nodes; a count query contains jobId and exact
nonnegative copies; a fingerprint query links every matching full index unit.
Query roots also declare normalForm and reportForm. JobQuery has exactly one
job link and matching jobId; FingerprintQuery has a symbol and one or more
member links whose units have that symbol. Unrelated nodes refuse.
There are no blank nodes or named graphs. The census vocabulary is
`https://blackcatinformatics.ca/raf/report/1#` (called `census:` in the shape table).
Body and shim fingerprint IRIs
include their normal-form version; table names are version-free.

The [shared report profile](../capabilities/rdf-report-profile.md) owns the closed
ground roles, canonical bytes, snapshot BASE and single normalized-body identity
law used by both formats. Census report profile 5 uses that law without changing
normal-form fingerprint bytes. Equal content mints equal snapshot IRIs.
PurRDF supplies the native RDF terms, parser, traversal, term ordering, lexical
output, standard vocabularies and BLAKE3 implementation. Domain validation reuses
the profile's native-term storage without a second converted graph.
Units are named from symbol, file, line and start byte, percent-encoded with the
shared RFC 3986 encoder. A same-line cfg alternative therefore has a distinct node.

Presentation uses the shared graph ordering and complete predicates. Typed domain
rows returned by the reader retain census ordering: groups by descending members,
descending tokens, then fingerprint; constants by value; jobs by id; units and
members by symbol, file, line and start byte. Integer constants are decimal
xsd:integer literals. Changing presentation does not alter those typed values.

A job match asserts `job census:match unit`; its named RDF 1.2 reifier carries
`a census:Match` and its Home, Variant or Copy classification. An explicit
`rdf:reifies` triple supplies the corresponding depth-one triple term. A match
requires exactly one reifier, and copies equals
the number classified Copy.

The closed shape is stated below. Every class also requires exactly one rdf:type;
no other subject, predicate, class, individual or datatype is admitted. Baseline
units must be referenced by a group, shared constant or holds property. Every
reference must resolve, and minted node names must agree with their properties.

| Report | Class | Property | Count | Object |
|---|---|---|---|---|
| baseline | `census:Baseline` | `census:minTokens` | 1..1 | positive integer |
| baseline | `census:Baseline` | `census:normalForm` | 1..1 | u32 |
| baseline | `census:Baseline` | `census:reportForm` | 1..1 | report version |
| baseline | `census:Baseline` | `census:unitCount` | 1..1 | integer |
| index | `census:Index` | `census:normalForm` | 1..1 | u32 |
| index | `census:Index` | `census:reportForm` | 1..1 | report version |
| index | `census:Index` | `census:referenceSource` | 0..unbounded | reference source |
| job-query | `census:JobQuery` | `census:normalForm` | 1..1 | u32 |
| job-query | `census:JobQuery` | `census:reportForm` | 1..1 | report version |
| job-query | `census:JobQuery` | `census:jobId` | 1..1 | string |
| job-query | `census:JobQuery` | `census:job` | 1..1 | job |
| count-query | `census:CountQuery` | `census:normalForm` | 1..1 | u32 |
| count-query | `census:CountQuery` | `census:reportForm` | 1..1 | report version |
| count-query | `census:CountQuery` | `census:jobId` | 1..1 | string |
| count-query | `census:CountQuery` | `census:copies` | 1..1 | integer |
| fingerprint-query | `census:FingerprintQuery` | `census:normalForm` | 1..1 | u32 |
| fingerprint-query | `census:FingerprintQuery` | `census:reportForm` | 1..1 | report version |
| fingerprint-query | `census:FingerprintQuery` | `census:symbol` | 1..1 | string |
| fingerprint-query | `census:FingerprintQuery` | `census:member` | 1..unbounded | unit |
| both | `census:BodyGroup` | `census:fingerprint` | 1..1 | body fingerprint |
| both | `census:BodyGroup` | `census:member` | 2..unbounded | unit |
| both | `census:BodyGroup` | `census:tokens` | 1..1 | integer |
| both | `census:ShimGroup` | `census:fingerprint` | 1..1 | shim fingerprint |
| both | `census:ShimGroup` | `census:member` | 2..unbounded | unit |
| both | `census:ShimGroup` | `census:tokens` | 1..1 | integer |
| baseline | `census:SharedConstant` | `census:value` | 1..1 | large integer |
| baseline | `census:SharedConstant` | `census:packageCount` | 1..1 | package count |
| baseline | `census:SharedConstant` | `census:member` | 2..unbounded | unit |
| both | `census:Unit` | `census:symbol` | 1..1 | string |
| both | `census:Unit` | `census:file` | 1..1 | string |
| both | `census:Unit` | `census:line` | 1..1 | positive integer |
| both | `census:Unit` | `census:startByte` | 1..1 | integer |
| baseline | `census:Unit` | `census:holds` | 0..2 | table |
| index | `census:Unit` | `census:package` | 1..1 | string |
| index | `census:Unit` | `census:shipping` | 1..1 | boolean |
| index | `census:Unit` | `census:tokens` | 1..1 | integer |
| index | `census:Unit` | `census:body` | 0..1 | body fingerprint |
| index | `census:Unit` | `census:shim` | 0..1 | shim fingerprint |
| index | `census:Unit` | `census:referenceSource` | 0..1 | reference source |
| index | `census:ReferenceSource` | `census:package` | 1..1 | string |
| index | `census:ReferenceSource` | `census:version` | 1..1 | string |
| index | `census:ReferenceSource` | `census:sourceKind` | 1..1 | string |
| index | `census:ReferenceSource` | `census:repository` | 1..1 | string |
| index | `census:ReferenceSource` | `census:selector` | 1..1 | string |
| index | `census:ReferenceSource` | `census:revision` | 1..1 | string |
| index | `census:ReferenceSource` | `census:manifest` | 1..1 | string |
| index | `census:ReferenceSource` | `census:checksum` | 1..1 | string |
| index | `census:ReferenceSource` | `census:observation` | 1..1 | string |
| index | `census:ReferenceWitness` | `census:consumer` | 1..1 | unit |
| index | `census:ReferenceWitness` | `census:reference` | 1..1 | unit |
| index | `census:ReferenceWitness` | `census:fingerprint` | 1..1 | reference fingerprint |
| index | `census:Job` | `census:jobId` | 1..1 | string |
| index | `census:Job` | `census:homeSymbol` | 1..1 | string |
| index | `census:Job` | `census:copies` | 1..1 | integer |
| index | `census:Job` | `census:match` | 0..unbounded | unit |
| index | `census:Match` | `rdf:reifies` | 1..1 | triple term |
| index | `census:Match` | `census:matchClass` | 1..1 | match class |
| debt | `census:DebtRecord` | `census:normalForm` | 1..1 | u32 |
| debt | `census:DebtRecord` | `census:reportForm` | 1..1 | report version |
| debt | `census:BodyDebt` | `census:memberSymbol` | 1..unbounded | string |
| debt | `census:ShimDebt` | `census:memberSymbol` | 1..unbounded | string |
| debt | `census:BodyDebt` | `census:multiplicity` | 0..unbounded | multiplicity |
| debt | `census:ShimDebt` | `census:multiplicity` | 0..unbounded | multiplicity |
| debt | `census:MemberMultiplicity` | `census:symbol` | 1..1 | string |
| debt | `census:MemberMultiplicity` | `census:copies` | 1..1 | package count |
| debt | `census:JobDebt` | `census:jobId` | 1..1 | string |
| debt | `census:JobDebt` | `census:copies` | 1..1 | positive integer |

Job-query units use every index Unit property and the common Unit properties.
Its selected Job and Match nodes use the index rows above. Fingerprint-query
units use those same Unit rows. A count query has only its root. Each query
root is the only root in its graph; every node must belong to its selected
result, with no unrelated groups, jobs, matches or units.

Reference units name their source receipt. A source without indexed units has no
receipt node. Witnesses link a first-party consumer and reference unit with their
equal complete body or shim fingerprint. Selected queries retain each incident
witness and its endpoint/source closure.

A debt group contains at least two physical members. Distinct sites sharing one
canonical symbol use an owned `MemberMultiplicity` with at least two copies;
ordinary symbols imply one. Every multiplicity belongs to exactly one group,
and redundant count-one nodes are refused. Report form 5 includes this counted
shape. Earlier report layouts are refused by the ordinary readers; baseline and
index reports can be regenerated from source without changing fingerprints.

## Version-bump rule

The current Census report form is 5. The explicit native layout conversion
accepts forms 3 and 4 for Baseline, Index, Ledger and DebtRecord. Conversion edits
and returns the original graph, validates the selected current shape and
preserves every normalized assertion except reportForm. Owned snapshot IRIs
rebase; external IRIs, literal pins and historical normalForm values do not
change. Opaque Ledger extension assertions remain intact. Historical Index
carriers cannot contain reference observations or witnesses; conversion never
invents them. Ordinary readers require form 5. Layout conversion never authorizes
policy execution or debt growth.

A change to report vocabulary, shape or canonical layout bumps `census:reportForm`,
not `normal_form`.

A normal form version is fixed by the content of its first merge to the default
branch. After that, any change that can alter a fingerprint of any input is a new
version with a migration path. That covers the token source, every form above, every
threshold, the kind prefixes, the byte encoding, the hash function and the width of
the digest. The change bumps the `normal_form` number, and the census keeps the old
behaviour out of the new version. A change that cannot alter any fingerprint, such
as a faster implementation, does not bump the version.

## Golden-test rule

The census keeps a fixture set of source snippets together with the fingerprint
each one has under the current version. A golden test recomputes every fingerprint
and compares it to the pinned value. A change to the normal form that moves a
pinned fingerprint fails the test until the pinned values are regenerated in the
same change, and once the version is fixed, until the version is bumped as well. The
fixtures cover each form: structural, value, literal tier, shim, and macro
arguments, and each form of binder beside the names no one binds. A pinned manifest
of the encoding, the ordered passes, the levels, the thresholds and the closed word
lists sits beside the fingerprints, so a change to any of them moves a pinned value
too. A fixture without a pinned value fails the test, and so does a pinned value
without a fixture.

## Compatibility

The report reader returns its declared `census:normalForm` without gating it; callers
apply `check_normal_form`, so migration can read another normal form. Its body and
shim namespaces must agree with that version. `census:reportForm` gates the layout
and must equal the implemented `REPORT_FORM` (5).

A ledger names the version it was written for with `normal_form`. If the value differs
from the version the census implements, the census fails hard before reading any job.
The message states the ledger's version and the census's version and tells the user to
run `cargo census migrate`, which maps the old fingerprints to the new ones. The
census never reads a ledger under a different version, and never guesses at a mapping;
`migrate` is the one reader of an older-version ledger, and it rewrites the ledger at
the implemented version.

## Conditions to revisit

- Bodies that compute the same value are reported as distinct because they differ
  in something the forms keep, such as a method call written as a function call or
  independent statements in another order, often enough that the census misses
  the duplicates it exists to find.
- Bodies that compute different values share a fingerprint through the value form
  or the literal tier often enough that ledger entries group code that is not
  duplicated.
- Two different token sequences are found with the same complete BLAKE3 fingerprint in a
  real ledger.
- A release of the token source lexes unchanged code into other tokens, so
  fingerprints move with a dependency update and not with a version bump.
