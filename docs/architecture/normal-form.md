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
algorithm identical, and the fingerprint is the short name of the result. This
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
a shim is the first 8 bytes of the BLAKE3 hash of the form's token sequence under
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
the recursion budget, and the lexer's span table is cleared when a load ends.

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

## Fingerprint

A fingerprint is `<kind>:` followed by an identifier.

- For the kinds `body` and `shim`, the identifier is the first 8 bytes of the
  BLAKE3 digest of the form, read as a big-endian number and written in 16
  lowercase hexadecimal digits.
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

The hashed encoding has the id `tokens-len-u32le-utf8/blake3/first8-be-hex16`
(`ENCODING`): each token of the form, in order, is written as its UTF-8 length as a
little-endian `u32` followed by its bytes, and the digest is the BLAKE3 hash of the
concatenation. A joined punctuation character is written with a trailing `~`, and a
delimiter as its character. A fingerprint string that has another kind, an
identifier of another width, uppercase hexadecimal digits, an unknown table name or
an empty rule id is refused.

The census keeps the full 32-byte digest of every unit it groups. Two units whose
digests differ but whose 8-byte fingerprints are equal are an error that names both
symbols.

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

The census renders two JSON reports. Both are closed schemas that read back what
they write. They are rendered with sorted keys, two-space indentation and one
trailing newline, and a fingerprint is a string. An integer constant is a
lowercase hexadecimal string with a `0x` prefix, because it can exceed what a JSON
number holds.

The baseline report lists what the workspace repeats. Its members are:

- `normal_form`, the version;
- `min_tokens`, the threshold the run used;
- `units`, the number of units;
- `body_groups` and `shim_groups`, each a list of groups with `fingerprint`,
  `tokens` and `members`;
- `hex_tables`, each with `symbol`, `file`, `line` and `table`;
- `shared_constants`, each with `value`, `packages` and `members`.

A member has `symbol`, `file` and `line`.

The index report lists every unit, group and job. Its members are:

- `normal_form`, the version;
- `units`, each with `symbol`, `package`, `file`, `line`, `shipping`, `tokens`,
  `body` and `shim`; `body` is the body fingerprint or `null`, `shim` is the
  `shim_key` or `null`, and `tokens` is 0 for a unit that is not a function;
- `groups`, each with `kind` (`body` or `shim`), `fingerprint` and `members`;
- `jobs`, each with `id`, `home`, `matches` (each with `symbol` and `class`) and
  `copies`.

Units sort by symbol, then file, then line.

## Version-bump rule

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
- Two different token sequences are found with the same 8-byte fingerprint in a
  real ledger.
- A release of the token source lexes unchanged code into other tokens, so
  fingerprints move with a dependency update and not with a version bump.
