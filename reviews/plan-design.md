# Plan design review (SOLID / ETHOS): raf-census core, normal form v1

Reviewed plan: `/home/paudley/Active/rust-agentic-foundations/.worktrees/3-raf-census-core-normal-form-v1/.stage/raf-census-core-normal-form-v1/plan.md`

## Code read

- `crates/raf-source/src/{lib,workspace,item,facts,target,error,options}.rs`, `crates/raf-source/Cargo.toml`
- `docs/architecture/normal-form.md`, `docs/capabilities/census-{normal-form,ledger,rule-api,cli,detectors,migration,debt-ratchet}.md`
- `work/changes/census-{normal-form,ledger}.md`, `AGENTS.md`, root `Cargo.toml`, `tests/{repo_gates,docs_contract,self_workplan}.rs` (the `mod support;` declarations)
- Port source: `/home/paudley/Active/purrdf/crates/helper-census/src/{normalize,census,source,layout}.rs`

## Findings

### BLOCKING 1: units are collected per module path, so one source item becomes a false group of itself

Plan: "`units` module over `raf_source::Workspace`: fn, method ..., symbol (item path), package, file, line, shipping (target)" (Task 3) and "`units::collect` over a MemoryTree workspace with lib + test target: one unit per kind with every field asserted" (R7).

`raf_source::Item` is one record per path a file is reached under: "A file that several module trees reach has one `Item` for each path it is reached under; the items share their file and offset and differ in `path` and `target`" (`crates/raf-source/src/item.rs:269-272`). `Workspace::items()` yields all of them (`workspace.rs:286-291`). The port deduplicated on `(file, byte offset)` before fingerprinting (`source.rs:220-222`, `source.rs:267-276`: "a file reached twice yields its units once"); the plan drops that step and names no replacement.

Consequence: a `tests/common/mod.rs` declared by two test targets produces two units with equal `body`/`shim` fingerprints and different symbols, which is a group of two members that are one function. This repository has exactly that shape: `tests/support/mod.rs` is declared by `tests/repo_gates.rs:37`, `tests/docs_contract.rs:48` and `tests/self_workplan.rs:23`, so a baseline over `.` reports every helper in it as a three-way duplicate. The R14 check over `crates/` alone does not exercise this. Deduplicating by symbol instead would be wrong the other way: a trait impl method on a non-local type has the path `<trait>::<method>` (`item.rs:282-286`), so two `impl Display for <non-local>` bodies share one symbol and would collapse.

REWRITE TO: `units::collect` iterates `Workspace::items_by_source()` (`workspace.rs:471-476`), which already groups the ids that share `(file, start_byte)` and sorts the group by path; one unit per group, `symbol` = the first id's `path`, `target`/`package` from that id, `shipping = ids.iter().any(|id| ws.is_shipping(ws.item(*id)))` (`workspace.rs:348-354`, which also honours `#[cfg(test)]` on the item). The unit's identity in groups and reports is `(file, line)` beside the symbol, never the symbol alone. Add to the R7 fixture a `tests/common/mod.rs` declared by two test targets and a `#[path]`-shared file reached by the lib and a test, and assert one unit each with `shipping: true` for the second.

### BLOCKING 2: the token source for constants and for parameter names does not exist in `raf_source::Item`; the plan's "`syn` only if needed" defers a decision the design needs now

Plan: "deps `raf-source` (path), `proc-macro2` ..., `syn` only if needed for item kinds (else not)" (Task 1); "const, static; ... integer constants by value, string literals, table candidates" and "method (with `trait_impl`, `via_self` for shim_key)" (Task 3).

`Item.body` is `Option<SpanFact>`: "The braced body, or the function block, if there is one" (`item.rs:301-302`). The extractor passes `None` as the body of every `Const` and `Static` (`facts.rs:1019-1042`), so a constant unit has only `Item.span`, "The whole item, attributes included" (`item.rs:299-300`). There is no span of the initializer expression and no signature: the port read `constant.expr.span()` (`source.rs:734`) and walked `signature.inputs` with a `syn` visitor for the parameter names that the shim form renames (`source.rs:703-706`, `normalize.rs:548-576`).

Consequence: lexing `Item.span` text for a constant feeds the attributes and the type into the unit. `proc_macro2` lexes `/// text` as `# [doc = "text"]`, so every documented constant reports its doc comment as a string literal; `const TABLE: [u8; 256] = ...` reports `256` as an integer constant, and `const BIG: u64 = ...` under `#[cfg(target_pointer_width = "64")]` reports `"64"` as a string. Shared-constant and hex-table reports are then wrong on exactly the units they exist for. For functions, without parameter names the shim form cannot rename "only the function's own parameters" (`arch:normal-form` § Shim form), so `fn a(text: &str) { write(text, 16) }` and `fn b(value: &str) { write(value, 16) }` are two shims.

REWRITE TO: `syn` is a required dependency of `raf-census` from Task 1 (use the `raf_source::syn` re-export, `lib.rs:198-200`, so the parse agrees with the index). For each unit, parse `source[item.span]` with `syn::parse_str::<syn::Item>` (or `ImplItem`/`TraitItem` by `SymbolKind`): a constant's token source is `expr.to_token_stream()` of the `ItemConst`/`ItemStatic`/`ImplItemConst`; a function's parameter set is the `PatIdent`s of `sig.inputs`, and its body is `source[body]` without the braces, re-lexed with `TokenStream::from_str` exactly as the port does (`source.rs:916-927`) so macro arguments are token trees. State this in Task 3, and add the acceptance check "a documented constant with a typed array initializer reports no string from its doc comment and no integer from its type".

### BLOCKING 3: the port's `relex` turns an unlexable body into an empty form; porting it as written is a silent fallback

Plan: "Port from purrdf `normalize.rs` with its thresholds" (Task 2) and "the unit extraction and relex parts of `source.rs` (`relex`, ...)" (Prior art).

`relex` is `source.get(start..end).and_then(|slice| TokenStream::from_str(slice).ok()).unwrap_or_default()` (`source.rs:923-926`): a slice that does not lex, or a span outside the text, becomes an empty stream, hence `tokens: 0`, no fingerprint, no constants, no strings, and no error. `Workspace::source(path)` is `Option<&str>` (`workspace.rs:281-283`) and the same `unwrap_or_default` pattern would read a missing file as an empty body.

Consequence: a unit the census cannot read disappears from every report with no finding, which is the "no silent fallbacks" case of `AGENTS.md` § 5 and the "fails loudly" requirement of this crate. Both inputs are invariants `raf-source` already established (it parsed the file and recorded the spans), so a miss here is a bug, not an input condition.

REWRITE TO: `units::collect` returns `Result<Vec<Unit>, Error>`; a missing `Workspace::source` for an item's file, a body span outside the text, or a `TokenStream::from_str` failure is `Error::Internal { file, line, what }`, in the style of `LoadErrorKind::Internal` (`crates/raf-source/src/error.rs:320-325`). A `Fn`-kind item with `body: None` (a required trait method or a verbatim bodiless function, `facts.rs:1165-1193`) is the one legitimate absence and yields no unit. Name the `SymbolKind` set that yields units: `Fn`, `InherentMethod`, `TraitImplMethod`, `TraitDefaultMethod`, `Const`, `AssocConst`, `Static` (`item.rs:142-184`).

### BLOCKING 4: the index report takes a `JobMatch` list that nothing in this change produces

Plan: "This change delivers the index report with a `jobs` member whose rows are produced from a `JobMatch` list the caller supplies, with its shape, sorting and determinism tested on supplied matches" (Scope boundaries); `Index` "(..., jobs from supplied `JobMatch`es)" (Task 5).

The classification a match carries is defined by the ledger capability (`docs/capabilities/census-ledger.md:92-98`: HOME, VARIANT, COPY, `copies` per job) and in the port by `Match { symbol, package, file, line, reasons, in_home, variant }` (`census.rs:182-201`), produced by `matches(job, workspace, home_package)` (`census.rs:225`), which needs the `Job` type the ledger change delivers. The only production caller in this change passes an empty list ("The census-cli `index` command shows no jobs until the ledger exists"). The migration consumer of the index report needs only units and their fingerprints (`census-migration.md:35-41`), and the designed index shape has `normal_form` and `units` only (`census-normal-form.md:118-126`).

Consequence: `JobMatch` is a type designed before its requirements, reachable only through fabricated input, and its row shape is pinned by the R10 JSON goldens. When `change:census-ledger` defines `Match`, either it bends to a guessed struct or it changes a golden-pinned report shape in a change that is not about the normal form. That is speculative generality with a named cost.

REWRITE TO: the index report of this change is `{ normal_form, units: [{ symbol, package, file, line, shipping, tokens, body, shim }], groups: [{ kind, fingerprint, members }] }`. Drop `JobMatch` and the `jobs` member; the item T1 adds to `change:census-ledger` becomes "The index report gains a `jobs` member listing the classified matches of every job", and R10's "job matches" is `moved` there in the `change:census-normal-form` item split, beside the ledger-file fixture already moved.

### BLOCKING 5: the shim candidate predicate is not stated, and the rule "the page wins" makes every 6 to 29 token body a shim

Plan: "A body of 6 to 29 tokens additionally has a shim form" (NEEDS-JUDGMENT 3) and "any divergence found while building is resolved in favour of the page unless the page is unimplementable" (prior-art 5).

`arch:normal-form` says "A body with at least `MIN_SHIM_TOKENS` (6) and fewer than `SHIM_MAX_TOKENS` (30) structural tokens also has a shim form" (`docs/architecture/normal-form.md:83-85`), unconditionally. The port only gives a shim form to a candidate: no `let`, no statement boundary before the tail, at least one call, and no `self` (`normalize.rs:578-594`), with the reason "a delegation on `self` is bound to its own type and cannot collapse with another type's" (`normalize.rs:579-581`). The page's `shim_key` exclusions (trait-impl methods, `Self::` forwarders) are a different, narrower set.

Consequence: implemented as the page reads, `fn len(&self) -> usize { self.items.len() }` (six structural tokens) has the shim form `self . items . len ( )`, identical for every type with an `items` field, so unrelated types form shim groups across the workspace; `{ let x = a; x + 1 }` and every short accessor get a shim fingerprint too. A fingerprint is frozen at version 1 once delivered (`normal-form.md:128-135`), so a predicate left to the implementer cannot be corrected without a version bump and a `migrate`.

REWRITE TO: the plan states the shim candidate predicate as part of the normal form, not as an implementation detail: a body of 6 to 29 structural tokens is a shim candidate when it is a single expression (no `let`, no `;` before the tail), holds a call, and does not mention `self`; `shim_key` then excludes a `TraitImplMethod` and a body whose call path starts with `Self::`. T2 ports `shim_candidate`, `has_call` and `mentions_self`; T6 writes the predicate into § Shim form before the goldens are regenerated. State in the same place whether a unit that has a shim fingerprint also joins a `body` group: the port excludes it (`census.rs:29-32`, "grouped by its shim form, never by its structural form"); the plan's "both" means a 20 to 29 token forwarder pair appears in a body group and a shim group and needs two `[[distinct]]` rows to sanction.

### ADVISORY 6: `min_tokens`, `max_file_bytes` and the error type have no place in `Census::load(&tree, roots)`

Plan: "Each check runs through the public API `raf_census::Census::load(&tree, roots)`" and `Baseline (normal_form, min_tokens, ...)` (Task 5).

`min_tokens` decides which units have a `body` fingerprint ("A body with fewer structural tokens than this has no body fingerprint", `normal-form.md:101`; the plan's own "fingerprinted as `body` only if it reaches `min_tokens`") and is a ledger setting with `max_file_bytes` (`census-ledger.md:41-48`), which maps to `LoadOptions.max_file_bytes` (`crates/raf-source/src/options.rs:24-28`). A `load` that bakes in 20 and `LoadOptions::default()` means the ledger change adds a second constructor and threads the value through `units`, `group` and `report` after the goldens were built against the implicit default. `check_normal_form` and the internal error of finding 3 also need an error type the plan does not name, which `census-rule-api.md:117` already calls `raf_census::Error`.

REWRITE TO: `Census::load(tree, roots, &CensusOptions)` with `CensusOptions { min_tokens: usize, load: LoadOptions }` and a `Default` of 20 and `LoadOptions::default()`, mirroring `Workspace::load_with` (`workspace.rs:135-139`); `min_tokens` is a parameter of `units::collect`. Define `pub enum Error { Load(LoadError), NormalForm { declared: u32, implemented: u32 }, Internal { file: RelPath, line: usize, what: String } }` in Task 2 and return it from `load`, `collect` and `check_normal_form`.

### ADVISORY 7: `trait_impl` and `via_self` on the unit are two booleans whose only consumer is one grouping rule

Plan: "method (with `trait_impl`, `via_self` for shim_key)" (Task 3); "`group` module: body groups and shim groups (shim_key)" (Task 4).

Carried as public fields, the two flags make `group` reconstruct `shim_key` from them, and any other reader of `Unit` (the ledger's `distinct` matching, the index report's `shim` column) must know to combine them the same way or report a shim the census will not group.

REWRITE TO: `units::collect` computes `shim_key` once and stores `Unit { print: Print { tokens, body: Option<Fingerprint>, shim: Option<Fingerprint> }, shim_key: Option<Fingerprint>, .. }`, with `shim_key` `None` for the excluded cases; `group` keys shim groups on `shim_key` and nothing else. Keep `SymbolKind` on the unit if a report needs it, not the derived booleans.

### ADVISORY 8: a `Fingerprint { kind: Kind, id: String }` admits `table:` names and `rule:` ids the parser does not check

Plan: "`Fingerprint` type and `Kind`" (Task 2); "`Fingerprint` parse refuses other kinds/widths (negative fixture)" (R6).

The port validated `table:` against its two names and `rule:` against the registries in one function (`normalize.rs:84-102`). A kind-plus-string record validated only for hex width lets `table:hex-lowerx` through, and the ledger, migration and detector changes each re-validate.

REWRITE TO: `pub enum Fingerprint { Body(Digest), Shim(Digest), Table(Table), Rule(RuleId) }` with `Digest([u8; 8])`, `enum Table { HexLower, HexUpper }` and `RuleId` a newtype that the constructor restricts to lowercase kebab case (`census-rule-api.md:165`); `Display` renders `<kind>:<id>` and `FromStr` is the one parser. Registry membership of a `RuleId` stays the ledger's check.

### ADVISORY 9: the crate would carry a hex-format loop and a hex-digit table of its own

Plan: "hex-table detection (`table:hex-lower` / `table:hex-upper` ...)" (Task 4) and the fingerprint rendering of R6.

The port rendered the sixteen digits through the workspace's own encoder "so the census does not carry the table it looks for" (`normalize.rs:761-777`); this repository has no such encoder. A `format!("{:02x}")` loop over the eight digest bytes is the positive fixture of the `hex-format-loop` detector (`census-detectors.md:35`), and a literal `"0123456789abcdef"` is a `table:hex-lower` unit. Both are found when the census runs over `crates/` of this repository (R14) and, once the detectors change lands, must be sanctioned in this repository's own ledger.

REWRITE TO: render the digest as `format!("{:016x}", u64::from_be_bytes(digest[..8]))` and build the comparison digits as `(0..16).map(|d| char::from_digit(d, 16))` with `to_ascii_uppercase` for the upper table; state in the goldens that the reference value of a known form was computed with `blake3::hash` directly.

### ADVISORY 10: `proc-macro2` with `span-locations` off cannot give string-literal lines

Plan: "`proc-macro2` (span-locations off, default features)" (Task 1); units carry "string literals" (Task 3), which the port records with their line (`normalize.rs:716-735`, `literal.span().start().line`), offset to the file by the body's first line (`source.rs:929-938`).

`Span::start` and `byte_range` need the feature. It is on in the workspace build only through `raf-source` (`crates/raf-source/Cargo.toml:18`), so the plan's dependency line disagrees with the design it ships.

REWRITE TO: declare `proc-macro2 = { version = "1", default-features = false, features = ["span-locations"] }` in `raf-census`, and state in Task 3 that a literal's line is `SpanFact.lines.start + relexed_line - 1` (`facts.rs:41-49`).

## Verdict

The forms, fingerprint, grouping and report design follow the port and the normative page closely, and `raf-source` already provides the seams (`items_by_source`, `is_shipping`, `source`, `SpanFact`, the `syn` re-export) the crate needs. The blocking findings are the places where the plan relies on a port behaviour `raf-source` does not reproduce (findings 1 to 3), a report member with no producer (4), and a normal-form predicate left unstated in a version that freezes it (5).

VERDICT: NEEDS-REVISION
