# T3 report

Status: DONE. Commit: see `git log` (feat(raf-census): units and Census::load). Not pushed.

Files (crates/raf-census): src/census.rs (Census, FileCtx, load), src/units/{mod,scan,sig}.rs, tests/units.rs; src/lib.rs (exports, docs); src/normalize/mod.rs (crate-internal re-exports of Node/Group/Kind/Delim, parse_integer, literal kind).

API: Census::load(&impl Tree, roots, &CensusOptions) -> Result<Census, Error>; units(), unit(symbol), units_of(symbol), files(), file(path), options().
Unit { symbol, name, package, file, line, kind: UnitKind{Fn,Method,Const,Static}, trait_impl, via_self, shipping, params, print: Option<Print>, shim_key: Option<Fingerprint>, constants: BTreeSet<u128>, tables: BTreeSet<TableKind>, strings: Vec<StringLiteral{value,line}> } (non_exhaustive). FileCtx { package, path, shipping }.

Decisions:
- Units from Workspace::items_by_source; symbol = shipping reach first then smallest path; shipping = any reach is_shipping; package = chosen item's target package.
- Kinds: Fn; InherentMethod/TraitImplMethod/TraitDefaultMethod -> Method (trait_impl only for TraitImplMethod); Const/AssocConst -> Const; Static. Others, fn without body, required assoc const (no value) yield no unit.
- Function slice = contents of the braces, first line = body.lines.start; params parsed from span.start..body.start with syn as ItemFn / ImplItemFn / TraitItemFn by kind (so `default fn` in an impl parses). Const/static: item span parsed, initializer = bytes between `=` and `;`; first line computed from the newlines before it. Assoc consts: ImplItemConst, else TraitItemConst.
- Constants, strings (decoded via syn::Lit, absolute line from the lexer's place), hex tables (string/byte string equal to the 16 digits in one case; bracket array of char/byte literals) are read from the verbatim form. Tables are collected here (R7 lists them); the grouping/reporting of tables and shared constants is T4. Digits are generated with char::from_digit, not retyped.
- via_self: a call whose callee path starts with `Self::` (turbofish allowed); `helper(x, Self::ONE)` is not. shim_key = print.shim fingerprint unless trait_impl or via_self. Note the ported shim_candidate predicate does not see a turbofish call (`f::<T>(x)` ends in `>`), so such a body has no shim form; unchanged from T2.
- Unparsable signature/initializer are covered at the sig layer (units/sig.rs tests, each with a valid neighbour) because raf-source's own parse admits everything those parsers accept.
- max_file_bytes maps to LoadOptions.max_file_bytes; the error is Error::Load naming the file.

Checks (exit): cargo test -p raf-census --locked 0 (17 unit, 19 forms, 4 golden, 17 units, 1 doctest); docs_contract+self_workplan+repo_gates 0; make fmt clippy doc 0; cargo workplan check 0. No timing, no mutation runs.

## Fix commit (review findings)

Commit: see `git log` (fix(raf-census): ...). Not pushed.
1. Turbofish: shim.rs `has_call` treats `::<...>` (balanced, `->` aware) after a name as part of the name; goldens regenerated (no version bump; manifest unchanged; value_type_turbofish.a, value_const_turbofish_kept.a, value_bool_turbofish_kept.a/.b gained a shim); new golden fixtures shim_turbofish_forwarder pair (Equal Shim), shim_method_turbofish, shim_turbofish_type_differs (Equal Value, shim fingerprints differ), shim_turbofish_without_call (no shim); units test a_turbofish_call_is_a_forwarder_and_keeps_its_key_beside_a_twin asserts the `Self::` turbofish forwarder has a shim and no key.
2. units test a_self_call_at_any_depth_removes_the_shim_key: nested argument, nested callee argument, `Self::V(x, 1)` lose the key; `Self::ONE` argument and `Vec::<u8>::new` keep it.
3. units/mod.rs: body must be a braced block (strip `{`/`}`) else Error::Source; initializer range and newline count go through `outside` (Error::Source); unit test beside a valid neighbour.
4. scan.rs collects c"..." and byte strings; new StringKind {Str, Bytes, CStr} on StringLiteral; non-UTF-8 bytes are decoded lossily (U+FFFD) and such a literal never names a table (rule in the doc comment, pinned by byte_strings_and_c_strings_are_strings beside a valid byte-string table).
Checks (exit): cargo test -p raf-census --locked 0 (18 unit, 19 forms, 4 golden, 20 units, 1 doctest); docs_contract+self_workplan+repo_gates 0; make fmt clippy doc 0; cargo workplan check 0.
