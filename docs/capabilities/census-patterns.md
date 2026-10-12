---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:census-patterns
title: "Declarative census patterns"
description: "Ledger-declared [[pattern]] matchers that flag code by call, path, macro, type, literal or token sequence."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:census-detectors, cap:census-ledger]
---

# Declarative census patterns

## Purpose

A repository often needs to forbid one specific thing, such as a call to a banned
function or a macro, without writing Rust. A `[[pattern]]` is a detector declared in
the ledger. It covers the common shapes by data, and leaves structural checks to
the `Rule` trait.

## Contract

The ledger holds zero or more `[[pattern]]` tables. Each has:

- `id`: required; the pattern identifier, unique across patterns and built-in
  detector ids.
- `summary`: required; one line saying what the pattern flags.
- exactly one matcher key, from this closed set:
  - `call`: a function path; matches a call of that path.
  - `path`: a path; matches any use of that path.
  - `macro`: a macro name; matches an invocation of it.
  - `type`: a type path; matches the type wherever it is named.
  - `literal_regex`: a regular expression; matches a string literal that it
    matches.
  - `tokens`: a normalized token sequence in which `$_` matches any one token or
    one balanced group.

A pattern with no matcher, or with more than one, is an error.

A hit is reported as `rule:<id>`. A job references a pattern by listing its `id` in
`forbidden.patterns`, and a pattern that no job references is reported as unused
and fails the staleness check. A pattern that matches nothing in the workspace is
reported as stale when a job references it and the job is enforced.

Matching uses the same token source as the normal form (see
[census-normal-form](./census-normal-form.md)): the file is lexed with
`proc_macro2`, macro arguments included, and `tokens` is compared against the
normalized tokens, so locals are renamed before the comparison.

## Usage

A pattern is declared once and attached to the job it protects:

```toml
[[pattern]]
id = "no-unwrap-env"
summary = "Reading an environment variable with unwrap."
tokens = "std :: env :: var ( $_ ) . unwrap ( )"

[[job]]
id = "config-env"
summary = "Environment access goes through the config module."
home = "demo::config::env_var"
entry_points = []
enforced = true

[job.forbidden]
names = []
constants = []
fingerprints = []
patterns = ["no-unwrap-env"]
```

## Examples

One example per matcher form, as designed ledger entries:

```toml
[[pattern]]
id = "no-sleep-call"
summary = "Thread sleep in library code."
call = "std::thread::sleep"

[[pattern]]
id = "no-instant-path"
summary = "Direct use of the std clock."
path = "std::time::Instant"

[[pattern]]
id = "no-println-macro"
summary = "Printing from library code."
macro = "println"

[[pattern]]
id = "no-std-mutex-type"
summary = "The std mutex type."
type = "std::sync::Mutex"

[[pattern]]
id = "no-http-literal"
summary = "A hard-coded plain-HTTP URL."
literal_regex = "^http://"
```

## Constraints

- A pattern matches syntax. It does not resolve imports across crates, so a path
  matcher sees the path as written after the file's own `use` aliases are applied.
- `literal_regex` uses the `regex` crate dialect and matches string literals only.
- `$_` stands for one token or one balanced group, never a variable-length run.
- Structural conditions, such as "inside a loop", are not expressible. Those belong
  in a `Rule`.

## Explanation

Most repository bans are a name: a function, a macro, a type. Writing a Rust rule
for each one makes a ban cost a test file and a build. Declaring it in the ledger
costs a table and keeps the ban beside the job it protects. The matcher set is small
and closed so that patterns stay reviewable, and the `Rule` trait remains the
extension point for anything that needs structure.

## Verification

- A fixture per matcher form (`call`, `path`, `macro`, `type`, `literal_regex`,
  `tokens`) has a positive source that yields one `rule:<id>` hit and a negative
  source that yields none.
- A `tokens` fixture shows `$_` matching one token and one balanced group, and not
  a run of two tokens.
- A pattern with zero matchers and a pattern with two matchers each fail ledger
  parsing.
- A job that lists an unknown pattern id fails, and an unreferenced pattern is
  reported.
- A pattern with the same id as a built-in detector fails ledger parsing.
