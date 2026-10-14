# Prior-art assessment: Shared crates `raf-okf` + `raf-source`

Scope: issue 2 of Blackcat-Informatics/rust-agentic-foundations. Evidence came from
`issue.md`, `prior-art.md` and `brief.json` in this directory, plus direct reads of
the sibling repositories at the paths cited. `W` is
`/home/paudley/Active/rust-agentic-foundations/.worktrees/2-shared-crates-raf-okf-raf-source`.
Line numbers refer to the files as they were read on 2026-10-12. Three rustc rules
were checked with probes in a scratch directory that was deleted afterwards
(`rustc 1.100.0-nightly 2026-09-13`).

---

## 1. Precedent, per source

### 1.1 katamari `crates/katamari-helper-census/src/source.rs` (1438 lines). Port source for `raf-source`

Licence header: `AGPL-3.0-only` (lines 1-2). RAF is `MIT OR Apache-2.0 OR MulanPSL-2.0`,
and `deny.toml` allows only those three. Blackcat holds the copyright on all of these
files, so relicensing is the owner's decision. A port must not carry katamari text
under a permissive header unless the owner decides to relicense it. The purrdf copy
(1.2) is already permissively licensed and holds the same code, apart from the
katamari deltas listed in 1.2.

What it does, and how it maps to the RAF contract:

| Concern | Lines | Verdict |
|---|---|---|
| `Tree` trait (`read -> Option<String>`, `list -> Vec<(String, bool)>`) | 67-72 | **Adapt.** Keep the seam. Change the signatures to `Result` with typed errors, add an existence test, and return a file kind that separates regular, directory and other, not just a bool. |
| `Disk` | 75-101 | **Avoid as written.** `read_to_string(...).ok()` (83) follows symlinks, opens FIFOs (and blocks), has no size cap, and turns every I/O error, non-UTF-8 text included, into "not found". `list` (86-100) swallows `read_dir` errors and silently drops non-UTF-8 names (93). |
| `Memory` | 103-132 | **Port.** It infers directories from key prefixes. Add the `MemoryTree::from([...])` constructor the capability `Usage` shows. |
| Package discovery over parents | 237-255 | **Port** `walk_dirs(tree, parents)` (243) as configurable roots. Drop `walk()`'s hard-coded `["crates"]` (238). |
| Virtual manifest skip | 259-262 | **Port.** |
| Target roots: lib, bins, `src/bin/*`, `build.rs` | 1157-1208 | **Port the layout rules.** Note that `build.rs` becomes the crate `<pkg>::build_script` and is shipping (1201-1206). The issue names build.rs roots but `docs/capabilities/source-index.md` "Package discovery" does not, so the contract must be reconciled. |
| Test, bench and example targets, `<pkg>::{test,bench,example}_crate::<target>` | 57-63, 1214-1258 | **Port.** The naming matches the issue exactly. |
| `Manifest::read`, a hand-rolled line parser | 1099-1153 | **Avoid.** It reads only `key = "value"` lines. A trailing comment (`path = "x" # c`) or a single-quoted value (`path = 'x'`) fails `strip_suffix('"')` and is **silently ignored** (1119-1124). It ignores `autobins`/`autotests`/`autoexamples = false`, inline tables and dotted keys. The contract requires the `toml` parser, bounded by `max_file_bytes`. |
| Mod-tree following | 563-592 | **Adapt.** See latent defects L1-L3 in section 2. Flat-then-`mod.rs` lookup, the inline-module directory, and the children of a `#[path]`-loaded file at `parent_dir(file)` (581) are correct. The last was confirmed by probe: rustc looks for the children of `src/sub/explicit.rs`, loaded through `#[path]`, at `src/sub/child.rs`. |
| Raw identifiers `mod r#match;` | 565-568 | **Port the fix, decide the spelling.** See 1.2: the two repositories disagree. |
| `#[path]` attribute read | 1016-1032 | **Adapt.** It reads only a bare `#[path = "..."]`. It ignores `#[cfg_attr(pred, path = "...")]` and does not reject two `path` attributes. |
| Path folding `join_normalised` | 1040-1052 | **Avoid.** `..` above the root is silently clamped (`parts.pop()` on empty), and `RootDir` is skipped, so `#[path = "/etc/x.rs"]` becomes the repository-relative `etc/x.rs`. An escape is **retargeted inside the tree, not refused.** Purrdf's OKF reader documents and refuses exactly this class (`purrdf/crates/rdf/src/native_codecs/okf/reader.rs` 765-790). |
| `#[path]` escape detection (`PathEscapes`) | 39-42, 862-889 | **Avoid the semantics, keep the test idea.** It *records* a lexical escape from the package as a census finding. RAF must *refuse* an escape from the containment root, after canonicalisation, before reading. |
| `cfg(test)` / `#[test]` handling `is_test_only` | 979-1013, used at 390, 480, 493, 513, 540 | **Port the predicate logic, invert the action.** `all(test, …)` and `any(test, test)` count as test-only, while `any(test, unix)` and `not(test)` do not (test at 1305-1330). Katamari *drops* these items. The issue and the contract require them to be *tagged* and kept, including modules reached only through them. |
| Census exclusions `EXCLUDED_DIRS`, `is_excluded`, `is_outside_or_generated` | 54-55, 1054-1082, 344-352 | **Do not port into raf-source.** This is census policy: shipping walks skip `tests/`, `benches/`, `examples/` and `generated/`. raf-source must follow the mod tree as rustc does. A filtered view belongs to the census. |
| Symbol index `define`/`definition` | 650-679 | **Adapt.** `Definition` (176-187) has file, line, package, module and `documented`. The contract also needs **kind, visibility and the test tag**. `or_insert` (676-678) keeps the first definition and silently hides a second one, for example a lib and a bin both named `demo_a`, or cfg-variant twins. |
| Items covered | 388-473 | **Port.** fn, const, static, struct, enum, union, type, trait alias, trait plus its methods and consts, impl methods and consts, mod, `macro_rules!` with `#[macro_export]` hoisted to the crate root (451-468). Not covered: `extern crate … as …` (falls to `_ => {}` at 470) and foreign items. |
| Impl placement `settle` | 757-777 | **Adapt.** If the self type does not resolve, it falls back to `module::Last` (762-768), a silent fallback. A non-path self type (`impl Tr for &T`, `[T]`, tuples) is dropped (502-510). |
| `use` recording | 594-648 | **Port.** It handles renames, groups, `self` and globs. It ignores `ItemUse::leading_colon` (`use ::x`). `import` uses `or_insert` (642-647), so a duplicate alias is silently first-wins. |
| `absolutise` | 785-818 | **Port.** It handles `crate`, `self`, `super` and the local-vs-extern first segment. |
| `resolve` | 820-859 | **Adapt.** A BFS with a `seen` set is a sound cycle guard. But it returns `Option<String>`, so not-found, ambiguous and cyclic are indistinguishable. The silent `seen.len() > 4096` cap (832) turns a large search into "not found". It does not report that a result came through a glob. |
| Census payload: `Unit`, `BodyPrint`, `normalize`, `rules`, `rule_hits`, `scan_rules`, `relex`, `absolute_lines`, `ParamNames`, `trait_impl`, `fingerprinted` | 134-172, 223-233, 283-342, 681-755, 919-956 | **Do not port into raf-source.** Fingerprinting belongs to the census crates. Keep two ideas: the census needs **byte ranges** of item bodies so it can re-lex them (`relex` 934-945 already works from `span.byte_range()`), and it needs a **(file, byte offset) identity** to deduplicate a file reached by several module paths (285-292). Avoid `relex`'s `unwrap_or_default()` (944), which silently returns an empty stream on a lex error. |
| `scan_rules` parses **every** `.rs` under the package, reachable or not | 299-342 | **Avoid in raf-source.** This is census policy, and it widens the attack surface. Parse only reachable files. |

`Cargo.toml` (`katamari/crates/katamari-helper-census/Cargo.toml` 25-26):
`syn = { version = "2", features = ["full", "visit"] }` and
`proc-macro2 = { version = "1", features = ["span-locations"] }`. purrdf pins the
same two in its workspace (`purrdf/Cargo.toml` 130-131). `span-locations` is required
for `ident.span().start().line` and `byte_range()`. raf-source can drop syn's
`printing`/`proc-macro` defaults (`default-features = false`, features
`full, parsing, visit`) and avoid pulling in `quote`. **A blocker for the plan:**
`proc-macro2` depends on `unicode-ident`, which is licensed
`(MIT OR Apache-2.0) AND Unicode-3.0`. `W/deny.toml` line 12 allows only
`MIT, Apache-2.0, MulanPSL-2.0`, so `make check` (deny) fails as soon as syn enters
the graph. Adding `Unicode-3.0` (or a crate-scoped exception) is a supply-chain policy
decision that the comment at `deny.toml` 10-11 says needs an explicit look. It is not
a lint weakening, but it needs a stated decision. `libfuzzer-sys` is
`(MIT OR Apache-2.0) AND NCSA`, so keep `fuzz/` out of the deny graph in its own
cargo-fuzz workspace, or allow NCSA explicitly. `toml` (MIT OR Apache-2.0),
`winnow` (MIT), `libc` and `rustix` all pass.

Tests worth porting as fixtures (183-1437): re-export resolution and impl placement
(1278-1302), the test-only predicate matrix (1305-1330), the raw identifier
(1334-1341), the test, bench and example crates with a shared support file
(1346-1384), and path folding (1429-1437, with its expectation inverted from
"clamp" to "refuse").

### 1.2 purrdf `crates/helper-census/src/source.rs` (1367 lines). Named source of the `Tree` trait

- **The `Tree`, `Disk` and `Memory` code is byte-identical to katamari's** (purrdf
  57-122 against katamari 67-132). The diff has no hunk there. "Tree from purrdf" adds
  nothing beyond katamari, so one file is the port source. Prefer **purrdf's text**
  for licence reasons (purrdf header `MIT OR Apache-2.0 OR MulanPSL-2.0`, lines 1-2).
- Differences from katamari:
  - Roots are hard-coded to `["crates", "bindings"]` (223). **Purrdf-specific, do not port.**
  - Raw identifiers (543-550): the file name strips `r#` but the **symbol keeps `r#`**
    (`demo_a::r#match::flat`, test at 1255-1284). Katamari unraws both, giving
    `demo_a::match::found` (katamari 565-567, test 1334-1341). RAF must pick one,
    because bindings and ledger homes are written by people. The rustc-faithful
    spelling of a path is `r#match`. Whichever is chosen, `resolve` should accept both
    or refuse one with a typed error. It must not resolve one and silently miss the
    other.
  - Extra census rules: `lex_rules`, `structure_rules`, `layout_rules` (311-317).
    **Purrdf-specific, do not port.**
  - Purrdf has no `PathEscapes`, no `trait_impl` and no `include!` doc. Katamari added
    those.
  - `Cargo.toml` (purrdf `crates/helper-census/Cargo.toml` 34-48) depends on
    `purrdf-hash`, `purrdf-testkit`, `purrdf-lex` and `purrdf-jsonschema`. **None may
    enter RAF**, and the issue says "no purrdf deps".

### 1.3 A third walker found: gmeow-ontology `crates/build-inputs/src/modules.rs` (979 lines)

This walker was not named in the issue. It is **stricter than katamari on the rustc
rules** and is the best precedent for them. It is `AGPL-3.0-only`, so borrow the
patterns, not the text.

- `cfg_attr` expansion before `path` selection (243-276), and more than one `path`
  is an "ambiguous module path attributes" error (282-288).
- `name.rs` and `name/mod.rs` both present is an error (297-310). rustc gives E0761,
  confirmed by probe.
- `#[path]` on an **inline** module sets the directory for nested modules (626-635).
  rustc confirmed by probe: `#[path="pinl"] mod inl { mod deep; }` looks for
  `src/pinl/deep.rs`.
- `attribute_directory` is tracked separately from `directory` (181, 230-241,
  618-641), which is the rule that katamari lacks (L1).
- A cycle is detected with an active-file stack (201-202, "cyclic module/include").
- Containment `checked_path` (`crates/build-inputs/src/lib.rs` 207-238) is lexical:
  it refuses absolute paths and `..` above the root, and refuses **every** symlink
  component through `symlink_metadata`. That is stricter than the RAF contract, which
  allows symlinks that stay inside the root. It still checks before open (a TOCTOU),
  has no FIFO refusal, and has no size cap.
- Latent bug: a file loaded through `#[path]` gets the directory `parent/stem`
  (204-210 with `root=false`). rustc treats it as mod-rs and uses `parent/`
  (probe above). This is a defect in this precedent, so do not copy it.
- History: one commit (`a159730f8`), with no fix history.

### 1.4 OKF readers in the GMEOW and purrdf ecosystem

**Base spec.** `gmeow-gts/docs/GTS-OKF.md` 36-52 and 95-105 say `type` is the only
required key. `title`, `description`, `resource` and `tags` (repeated literals,
**re-emitted sorted**, 45), plus `timestamp` (`xsd:dateTime`, 46). A frontmatter-less
`index.md` is a navigation page (52, 104-105). Round-trip law: equal "modulo sorted
frontmatter keys, sorted tags, and YAML canonicalization", with body bytes identical
(154-155). Its rejections are listed at 167-185, and symlinks in a bundle are refused
(172).

**Unknown-key behaviour differs across the ecosystem:**

| Reader | Unknown key | Evidence |
|---|---|---|
| purrdf `lift_okf_bundle` | **hard error** `unrecognized frontmatter key` | `purrdf/crates/rdf/src/native_codecs/okf/reader.rs` 244-251; test 940-958 |
| gmeow-ontology import (calls purrdf) | hard error, closed vocabulary | `gmeow-ontology/docs/okf.md` 42-51, 88-92; `crates/pipeline/src/cli_ops/okf_import.rs` 52-70 |
| gmeow-gts `from_okf` | **preserved** as an `okf:<key>` extension, not an error | `gmeow-gts/rust/src/from_okf.rs` 291-304; GTS-OKF.md 47-48, 100-101 |

The issue's phrase "matching the GMEOW/purrdf reader behaviour" therefore means
purrdf and gmeow-ontology, not gmeow-gts. RAF's capability doc
(`W/docs/capabilities/okf-profile.md` 49-59, 123) is stricter still. Every key is
required, and `resource` is refused (`W/docs/architecture/okf-profile.md` 22-27).

**Purrdf reader (`reader.rs`, 1168 lines) and writer (`writer.rs`, 1336 lines).**
Licence `MIT OR Apache-2.0 OR MulanPSL-2.0`.

- Port the patterns: frontmatter fence detection with CRLF and an unclosed-fence error
  (284-345); a frontmatter byte cap before parse (305-313); duplicate keys refused at
  every level (885-938); the navigation-`index.md` branch (227-241); tags collected
  into a `BTreeSet` and emitted sorted (368-381); timestamp validated as `xsd:dateTime`
  (383-400); and a CommonMark-faithful inline-link scanner (567-760) with three
  hard-won fixes (section 2).
- Do not port: the YAML engine is `purrdf_lex::yaml` (`reader.rs` 12, 319-327), a purrdf
  dependency. It **accepts aliases** with node counting (`aliases: true`, 323). RAF
  refuses anchors and aliases (`okf-profile.md` 45-46). The limits differ too
  (`native_codecs/okf/mod.rs` 25-39: 16 MiB per document, YAML depth 64). RAF's are a
  1 MiB text and YAML depth 32. Do not copy the constants. RDF event emission, IRI
  minting and the loss ledger are RDF-specific.
- Latent gap: `extract_markdown_links` scans the whole body with **no fenced-code or
  code-span awareness** (no fence handling anywhere in `reader.rs`). A Rust snippet
  like `v[0](x)` inside a fence would be read as a link. RAF's body model must exclude
  fenced and inline code.

**gmeow-gts `okf.rs` writer** (`gmeow-gts/rust/src/okf.rs` 414-443) uses
`serde_yaml 0.9` (`rust/Cargo.toml` 78). That crate is deprecated, and its output
sorts keys alphabetically and drops comments. **Avoid**: RAF's writer must keep
comments (the SPDX lines) and use the order of the profile table
(`okf-profile.md` 76-78, 125).

**gmeow-ontology emitter** (`crates/pipeline/src/stages/okf.rs` 99-200, AGPL) is a
hand-written deterministic YAML emitter. Keys are written in insertion order, fixed
first and then sorted (16-17, 353, 359, 406). `yaml_scalar` is a total function: it
double-quotes with `\n`, `\r`, `\t`, `\xXX` and `\uXXXX` escapes when a control
character is present, single-quotes values that are unsafe as plain scalars, and
otherwise writes them plain. **Adapt the idea** for the RAF writer, but note the
escape mismatch in 1.5. The RAF reader would not round-trip this emitter's `\n`.

### 1.5 In-repo `W/tests/docs_contract.rs` (3851 lines) and `W/tests/support/mod.rs`

This is the existing restricted-YAML and Markdown reader. The header says "Only the
standard library is used" (10). The inventory comment says `raf_workplan::Gate`
replaces it (82-83).

| Part | Lines | Absorb into raf-okf? |
|---|---|---|
| `Value` model: Scalar, List, Map&lt;String,Vec&gt;, Items | 122-128 | **Adapt.** It is untyped strings. The profile needs typed values (state enum, `affects` = id plus section, `items` = claim plus outcome). |
| `split_key`, `strip_comment`, `quoted_scalar`, `is_block_indicator`, `plain_scalar`, `scalar`, `parse_inline_list`, `parse_sequence`, `parse_map`, `parse_fields`, `parse_document` | 136-405 | **Absorb as the seed of the frontmatter reader.** The subset is deliberately narrow, which is the right call: purrdf's general YAML reader needed a yaml-test-suite run that found 38 bugs (purrdf `88bafc1ee`). Gaps to close: (a) no positions or byte spans; (b) `parse_document` splits on `lines()` and re-joins the body with `\n` (389-405), losing CR and the final newline, so it cannot round-trip; (c) comments are stripped (144-175, 351), but the writer must keep them; (d) `quoted_scalar` decodes only `\"` and `\\` and keeps every other escape literally (182-190), so it must refuse unknown escapes or share one escape set with the writer; (e) no nesting, size or anchor bound beyond what the narrow grammar implies (anchors `&`/`*` are refused as leading characters at 220, which is good, and should be kept as an explicit typed error); (f) `parse_document` returns `None` for no frontmatter, but the capability says `Document::parse` *fails*, so the navigation case needs its own typed outcome. |
| `fenced_lines` | 411-426 | **Absorb, but fix.** It toggles on any line starting with three backticks or three tildes after any indentation. It does not match the fence character or length (a four-backtick fence containing three backticks mis-toggles), accepts four or more spaces of indentation, and has no awareness of blockquotes or lists. |
| `heading_text`, `github_anchor`, `anchors`, `sections` | 428-485 | **Absorb.** Add byte spans: `sections` returns copied text (470-485), but the contract needs spans for `replace_span`. ATX headings indented 1-3 spaces are not recognised, so state that or fix it. |
| `strip_html_comments`, `is_blank`, `outside_code` | 487-516 | **Absorb.** `outside_code` splits on single backticks only, so double-backtick code spans are misread. |
| `Marker`, `looks_like_proposed`, `scan_markers` (PROPOSED and generated blocks, nested, unclosed and orphan-end errors) | 993-1069 | **Absorb into the body model.** This is exactly the capability's body-model and marker-error list (`okf-profile.md` 61-72). It currently records line numbers only, and spans are needed. |
| `link_targets`, `definition_target`, the HTML attribute reader, `normalize` | 1123-1323 | **Absorb the extraction** (links, reference definitions, href/src) into the body model. **Keep resolution out.** Checking existence against tracked files and tracked symlinks (`check_links` 1325-1390) is cross-document and repository-aware. `okf-profile.md` 119-120 says raf-okf touches no file. |
| Per-document profile checks: `check_keys`, `check_scalars`, `check_id`, `check_section_layout`, `check_binding_contents`, `check_capability`, `check_change`, `check_items`, `check_architecture`, `check_document` | 593-990 | **Absorb into `Profile::validate`.** It is one document at a time and reports every violation. |
| `index_problem` (generated-only capability index) | 1572-1635 | **Absorb** as the "Index handling" rule (`okf-profile.md` 81-83). |
| Cross-document checks: `find_cycles`, `check_capability_state`, `check_cross_references`, `check_affects`, the inventory | 878-903, 1392-1570, 1703-1798 | **Do not absorb.** These belong to the workplan document model (`okf-profile.md` 56-59). |
| Repository gates: placeholder, banned-wording and SPDX scans | 1071-1121 | **Do not absorb.** These are repository policy, not the profile. |
| `support::read_regular` / `check_handle` | `tests/support/mod.rs` 264-331 | **Partial precedent for raf-source's handle check.** It runs `symlink_metadata`, then `open`, then `fstat` on the handle with a dev/ino comparison. It differs from the contract: the contract forbids a stat before open, uses a non-blocking open, follows in-root symlinks, and caps size. `decode_text` (348-353) is lossy, and raf-source should refuse non-UTF-8 instead. |
| Negative-fixture corpus | 1875-3851 (about 90 tests) | **Port as the raf-okf test corpus.** Every rule already has a "break exactly one thing" fixture. |

**Should `docs_contract.rs` become a consumer of raf-okf?** Yes, for parsing, the body
model and per-document validation, in the same change that delivers raf-okf.
`okf-profile.md` 20-23 makes raf-okf "the one reader and writer of RAF documents". If
both parsers survive, two interpreters of the profile coexist, which is the
duplication the census exists to catch. The cross-document checks and the inventory
stay in `docs_contract.rs` until raf-workplan replaces them. Consequences:

- the root package gains a path dev-dependency on `raf-okf`;
- the header claim on line 10 changes;
- `W/docs/architecture/okf-profile.md` 30-36 ("will enforce … once implemented")
  needs a present-tense update;
- behavioural parity must be shown by running the existing fixture corpus against the
  new reader before the duplicated functions are deleted.

---

## 2. Recurrence

**Trailer counts.** `GhpRsq-Defect-Class` in the last 200 RAF commits: `none` × 1
(`2c41082`), as reported by stagectl and re-counted. No precedent commit that touched
either `source.rs` or the purrdf OKF reader carries a `GhpRsq-Defect-Class` trailer
(checked commit by commit). **Trailer-based counts for every class below are
UNKNOWN.** The counts below are fix commits identified by reading commit subjects and
diffs.

### Defect classes revealed by fixes (counted)

| Class | Fix commits | Where |
|---|---|---|
| Raw-identifier module file names (`mod r#x;` → `x.rs`) | **2**, fixed independently with divergent symbol spelling | katamari `838a944`; purrdf `be69bace1` |
| Test, bench and example targets missing from the index | **1** | purrdf `7a5d95a89` |
| A file reached by several module paths counted twice | **1** | purrdf `7a5d95a89` (`fingerprinted`; `9ecc73c28` added the original) |
| Package roots hard-coded to one directory | **1** | katamari `7d2c303` (`walk_dirs`) |
| `#[path]` leaving its package went unnoticed | **1** (recorded as a finding, not refused) | katamari `c9870bf` |
| Trait-impl one-line slots reported as duplicates | **1** (census policy, not raf-source) | katamari `4f9c679` |
| OKF link scanner: escaped destination hid the `)` | **1** | purrdf `5ad4e1b15` |
| OKF link scanner: Unicode whitespace trimmed instead of the CommonMark class | **1** | purrdf `d64495d3f` |
| OKF link scanner: `[a] (b)` read as a link | **1** | purrdf `84abf779f` |
| General YAML reader correctness | **1 commit fixing 38 bugs** (20 invalid inputs accepted, 15 valid refused, 2 wrong values, 1 dropped key) | purrdf `88bafc1ee` |
| OKF numeric canonicalisation | **4** (`bc8b67748`, `d33d6a942`, `5a51bd665`, `3da25de51`) | Not applicable to RAF, which has no numeric profile values. Refuse numbers rather than canonicalise them. |
| RAF frontmatter subset hardening: unterminated quotes, unquoted `: `, indicator characters, block indicators, comments, marker spelling, FIFOs and symlinks, multi-line tag state | **1 commit, at least 6 classes** | RAF `dba307f` |
| Link to a tracked symlink | **1** | RAF `edf93d5` |

### Latent defects found by inspection or rustc probe (0 fix commits; incidence in real code UNKNOWN)

- **L1. `#[path]` inside an inline module** is resolved from the file's directory
  (`parent_dir(place.file)`, katamari 580, purrdf same). rustc probe: in `src/a.rs`,
  `mod inline { #[path = "other.rs"] mod inner; }` resolves to `src/a/inline/other.rs`.
  Katamari gives `src/other.rs`.
- **L2. `#[path]` on an inline module is ignored.** Katamari 569-578 walks `content`
  before it looks at the attribute. The rustc probe resolves to `src/pinl/deep.rs`.
- **L3. Both `x.rs` and `x/mod.rs` present** silently picks `x.rs` (katamari 584-589).
  rustc gives E0761, confirmed by probe.
- **L4. An escaping path is silently retargeted.** `join_normalised` clamps `..` and
  drops the root (1040-1052).
- **L5. `Disk` follows symlinks, opens FIFOs, reads without bound, and maps all errors
  to "not found"** (81-101).
- **L6. A manifest value with a trailing comment or single quotes is ignored**
  (1119-1124).
- **L7. `cfg_attr(…, path = …)` is not expanded** (1016-1032), and `cfg(test)` items
  are dropped instead of tagged.
- **L8. First-wins `or_insert` on definitions and imports hides ambiguity**
  (642-647, 676-678). `resolve` returns `Option`, with a silent 4096 cap (832).
- **L9. `use ::x`** (leading colon) and **`extern crate x as y`** are not modelled
  (594-636, 470).
- **L10. The unresolved-`mod` error has no line** (361-362), and the contract requires
  file and line.
- **L11. docs_contract fence toggling ignores fence character and length** (413-426).
- **L12. purrdf's link scanner ignores code fences** (reader 567-633).
- **L13. A mismatch between the YAML escape sets of reader and writer** would break
  round-trip (docs_contract 182-190 against gmeow-ontology `okf.rs` 172-200).
- **L14. gmeow build-inputs gives a `#[path]`-loaded file non-mod-rs directory
  ownership** (204-210), against the rustc probe.

Not found in any history (fix commits counted at **0**, latent status covered above):
glob-resolution bugs, `use`-cycle bugs (the BFS `seen` guard has existed since purrdf
`9ecc73c28`), and `mod.rs` against non-mod-rs parent-directory bugs in katamari, whose
handling is correct. Katamari's repository-wide trailer histogram shows
`unfalsifiable-check` × 15. Treat this as a warning: katamari's
`assert!(walked.resolve("demo_a::ops::tests::hidden").is_none())` (1300) would also
pass if resolution were broken outright. Fixtures should assert positive resolution
next to every negative.

---

## 3. Prior attempts and why they did not hold

- **In RAF.** None. `git log` over all 27 commits shows no `crates/`, `source.rs` or
  `fuzz/` path ever committed, and the workspace `members` list is empty
  (`W/Cargo.toml` 20). The closest earlier work is `tests/docs_contract.rs` (commits
  `b499578`, `dba307f`, `edf93d5`, `2faa697`, `80c14d7`): a std-only checker that
  holds the profile, which the capability now reassigns to raf-okf. It cannot serve as
  the crate because it has no spans, no writer, does not preserve bytes and has untyped
  values (1.5). It "holds" as a gate and is the right test oracle and fixture source.
  The source-index contract was hardened before any code existed (`6120029`: canonical
  containment, handle `fstat`, pre-scan budgets, 64 MiB stack, fuzz target). Every one
  of those parts was therefore specified with no implementation behind it.
- **katamari and purrdf census walkers.** These held as census-internal walkers, but
  they are not general source indexes. They drop test code instead of tagging it, they
  record `#[path]` escapes instead of refusing them, they have no containment, no size
  cap and no stack guard, and they return `Option` with no typed failure. Census policy
  (exclusions, fingerprints, rules) is mixed into the walk. Their fix history shows
  repeated patching of target coverage and raw identifiers, and the two copies have
  already drifted: purrdf `r#` against katamari's unraw.
- **gmeow-gts `from-okf` seam.** Retired in favour of purrdf's native codec
  (`gmeow-ontology/docs/okf.md` 36-40). It was permissive on unknown keys and used
  `serde_yaml`, which shows why it was not kept as the strict reader.

---

## 4. Port vs build fresh

| Contract item (source-index.md / okf-profile.md) | Precedent | Decision |
|---|---|---|
| `Tree` trait: list, read, exists | katamari 67-72 = purrdf 57-62 | **Adapt** (Result plus typed file kind, exists) |
| `MemoryTree` | katamari 103-132 | **Port** (add `From<[(&str,&str); N]>`) |
| `DiskTree::new` canonicalises the root once | none | **Build fresh** |
| Per-candidate canonicalise plus canonical-root-prefix check (`mod`, `#[path]`, `Cargo.toml` target `path`, every manifest) | none (katamari lexical clamp; gmeow lexical plus refuse all symlinks) | **Build fresh** |
| Typed escape error naming the module, attribute or manifest key, with the file not opened | katamari 862-889 records a finding only | **Build fresh** (reuse the message shape) |
| Do not descend into directory symlinks; a symlink loop ends at the first link | none | **Build fresh** |
| Open the canonical path **non-blocking**, then `fstat` the handle and refuse non-regular files, with no stat before the open | partial: `W/tests/support/mod.rs` 264-331 does stat, open, `fstat` and dev/ino | **Build fresh**, borrowing the `fstat`-on-handle step. Needs `OpenOptionsExt::custom_flags(libc::O_NONBLOCK)` (safe code, compatible with `unsafe_code = deny`) and a `cfg(not(unix))` story. |
| `max_file_bytes` (8 MiB default, `LoadOptions`, census `[census]`) on .rs and Cargo.toml | purrdf OKF `MAX_OKF_DOCUMENT_BYTES` pattern only | **Build fresh** (read through `take(cap+1)` on the handle) |
| Token pre-scan: delimiters 128, nested `<` 64, prefix operators 256, closure bars 128, `else if` 256, skipping strings and comments | none anywhere | **Build fresh** |
| Dedicated parse thread with a 64 MiB stack, **named after the file** | none | **Build fresh** (`std::thread::Builder::name().stack_size()`). See pitfall P3. |
| Fuzz target `fuzz/fuzz_targets/parse_source.rs` plus committed corpus `fuzz/corpus/parse_source/` | none | **Build fresh** |
| Corpus replay under `cargo test` in a child process with a three-outcome harness (typed refusal, exit 0 with an exact index, abort) | none | **Build fresh** |
| Overflow fixtures in debug and release on the caller's 2 MiB stack; composite shapes | none | **Build fresh** |
| `Cargo.toml` via the `toml` crate, bounded, and its recursion limit turned into an error | katamari 1099-1153 (line parser) | **Build fresh** (replace) |
| Package discovery over configurable roots; lib, bin, test, bench and example targets, default plus explicit paths, with the shipping flag | katamari 243-281, 1157-1258 | **Port** (resolve build.rs scope) |
| Mod-tree: `name.rs` / `name/mod.rs`, inline, nested, unresolved = error with file and line | katamari 563-592, 344-386 | **Adapt** (add L3 ambiguity and the line in the error) |
| `#[path]` relative-directory rules (top level, inside inline, on inline, `cfg_attr`) | gmeow build-inputs 243-310, 610-641 (pattern only) | **Adapt** from gmeow's model. katamari is wrong on L1 and L2. |
| Module cycle (`#[path]` back to an ancestor file) | gmeow 201-202 (stack) | **Adapt.** katamari has no guard and would recurse on a `#[path]` self-include. |
| `cfg(test)` **tagging**, including modules reached only through it | katamari 979-1013 (predicate) | **Adapt** (tag, don't drop) |
| Symbol index with kind, visibility, file, line and test tag; methods under the impl type | katamari 650-679, 757-777 | **Adapt** (add kind, visibility and the test tag; fail instead of the `settle` fallback) |
| Use-alias table: renames, groups, globs, `self` | katamari 594-648 | **Port** (add `leading_colon` and `extern crate` aliases) |
| `resolve` with typed `NotFound` / `Ambiguous` / `Cyclic`, and glob reported | katamari 820-859 (`Option`, silent cap) | **Build fresh on the BFS skeleton.** No precedent distinguishes ambiguous or cyclic, or reports a glob. |
| Determinism: sorted output, two loads equal | `BTreeMap` throughout katamari | **Port** |
| DiskTree ≡ MemoryTree equivalence fixture; DiskTree containment fixtures (symlinked file, `vendored/..`, manifest `../` and absolute, link-to-link, dir loop, FIFO) | none | **Build fresh** |
| OKF parse: frontmatter fence on line 1, positioned errors, non-mapping error | docs_contract 389-405; purrdf reader 284-345 | **Adapt** (add positions; navigation outcome) |
| Restricted YAML subset | docs_contract 136-386 | **Absorb and harden** |
| Bounds: 1 MiB text, YAML 32 levels checked before descent, anchors, aliases and merge keys refused, Markdown block nesting 64 | partial: the subset already rejects `&`/`*` leading characters (220); purrdf caps (different numbers) | **Build fresh** for the depth counters and the Markdown block-nesting counter |
| `Profile::validate`: closed key sets, typed values, all violations | docs_contract 593-990 | **Absorb** |
| Body model with sections, links, fences, generated and PROPOSED blocks **with spans** | docs_contract 411-485, 993-1069, 1291-1305 (line-based) | **Adapt** (spans, CommonMark fence rules, code-span-aware link scan with purrdf's three fixes) |
| `Document::render`: deterministic, **comment-preserving**, profile-table key order, body byte-faithful, `replace_span` | none preserves comments (gmeow-gts and purrdf drop them and sort alphabetically) | **Build fresh** (gmeow-ontology `yaml_scalar` 145-200 as the quoting reference) |
| Base OKF keys `title/description/resource/tags/timestamp`, tags re-emitted sorted | purrdf reader 368-400; GTS-OKF.md 36-52 | **Port the semantics only if a base layer is exposed.** See P10. |
| Capability index accepted only with a generated block | docs_contract 1572-1635 | **Absorb** |

**Contract parts with NO precedent in any examined repository** (build fresh):

1. Canonicalised containment, with a canonical-root prefix check on every candidate
   (mod files, `#[path]`, manifest target paths, manifests), and a typed error naming
   the module, attribute or key without opening the file.
2. Directory-symlink non-descent and symlink-loop termination in a `Tree` listing.
3. A non-blocking open followed by `fstat` on the handle to refuse non-regular files
   (the in-repo test support stats before opening, which the contract forbids).
4. A `max_file_bytes` cap for Rust source and manifests (`LoadOptions` and the census
   `[census]` key).
5. The token pre-scan and its five budgets.
6. The dedicated, file-named 64 MiB parse thread.
7. The `parse_source` fuzz target, its committed corpus, and the corpus replay under
   `cargo test` with the three-outcome child-process harness. Also the
   debug-and-release overflow and composite fixtures.
8. Typed `resolve` failures `Ambiguous` and `Cyclic` (and `NotFound` as a type, not
   `None`), and the "resolved through a glob" report.
9. Kind and visibility in the symbol index, and a test *tag* (as opposed to
   exclusion).
10. `Cargo.toml` read through `toml` with size and recursion bounds.
11. On the raf-okf side: a comment-preserving, profile-ordered, byte-faithful writer
    with `replace_span`; byte spans in the body model; and the YAML-depth and
    Markdown block-nesting counters.

---

## 5. Pitfalls to avoid

- **P1. Licence.** katamari and gmeow-ontology are AGPL-3.0-only. Port from purrdf's
  permissive copy, or record an explicit owner relicensing decision. Never paste AGPL
  text under RAF's SPDX header.
- **P2. `cargo deny` will fail on `unicode-ident`** (`Unicode-3.0`) once syn lands, and
  on `libfuzzer-sys` (`NCSA`) if the fuzz crate is in the deny graph. Decide the
  allow-list change explicitly. Do not work around the gate.
- **P3. proc-macro2 spans with `span-locations` are thread-local.** The fallback
  `SOURCE_MAP` is a `thread_local!` (proc-macro2 1.0.107 `src/fallback.rs` 326-327), and
  proc-macro2 types are `!Send` (`marker.rs` 12, `PhantomData<Rc<()>>`). So a
  `syn::File` **cannot leave the 64 MiB parse thread**. Every line number, byte range
  and index record must be extracted on that thread into owned, `Send` data. Spawn one
  thread per file, or call
  `proc_macro2::extra::invalidate_current_thread_spans()` between files, so the source
  map neither grows without bound nor resolves stale spans.
- **P4. Do not drop `cfg(test)` code.** Tag it. Katamari's drop behaviour (390 and
  others) would make workplan test bindings unresolvable.
- **P5. Do not port census policy into raf-source**: excluded directories, generated
  directories, parsing unreachable files, fingerprints, `trait_impl`, rule hits, path
  escapes as findings. The census applies its own view over the tagged index.
- **P6. No silent fallbacks.** The `Option` returns in `Tree`, `resolve` and
  `relex`, the 4096 cap, the `settle` fallback to `module::Last`, ignored manifest
  lines, clamped `..` and first-wins `or_insert` each turn an error into a plausible
  wrong answer. That is the "success with a wrong or partial index" outcome
  `SECURITY.md` 46-50 forbids.
- **P7. Lexical containment is not enough.** `link/..` escapes on disk
  (`source-index.md` "Path containment"). A canonicalise-then-open sequence still has
  a residual race between the two steps. Under the trust model (`SECURITY.md` 37-44)
  that race is out of scope, but state it rather than claim "no check separates from
  the open". `O_NOFOLLOW` on the canonical path, or a crate like `cap-std`
  (licence-compatible), narrows it.
- **P8. Containment root is per tree, not per workspace.** The census reference scan
  uses the reference package directory as its root
  (`census-reference-scan.md` 66-75, `SECURITY.md` 52-56). Do not hard-code the
  workspace root into `DiskTree`.
- **P9. Raw-identifier spelling.** Pick `r#x` or `x` for symbol paths once, document
  it in the capability, and test both spellings in `resolve`. The two precedents
  disagree.
- **P10. Base OKF keys against the RAF profile.** The issue lists base keys and sorted
  tags, but no RAF type admits `tags`, `timestamp` or `resource`
  (`W/docs/architecture/okf-profile.md` 22-27, 41-45). The capability writer orders
  keys by the profile table, not alphabetically. If raf-okf exposes no base-OKF layer,
  the "tags re-emitted sorted" checkbox can only be met by a base-layer API, so the
  planner must decide. Do not import gmeow-gts's permissive extension pass-through.
- **P11. YAML scope.** Keep the restricted subset. A general YAML reader needed 38 bug
  fixes in purrdf. Refuse numbers, booleans, anchors, aliases, merge keys, tags, flow
  mappings, block scalars and unknown escapes with typed, positioned errors. Make the
  reader's and writer's escape sets identical (L13).
- **P12. Markdown scanning.** Fences need a matching character and a length at least
  that of the opening fence, with at most three spaces of indentation. Link extraction
  must skip fenced and inline code (including double-backtick spans) and apply
  purrdf's three link fixes: adjacency, the CommonMark whitespace class, and escaped
  destinations.
- **P13. Index handling is two rules.** In base OKF, a frontmatter-less `index.md` is
  navigation (GTS-OKF.md 52). For RAF, `docs/capabilities/index.md` is valid only if it
  holds only an SPDX header and generated blocks (docs_contract 1582-1635). Model
  navigation as a distinct parse outcome, not as `None` and not as an error.
- **P14. A file reached by several module paths** (a shared `tests/common/mod.rs`, or
  `#[path]` reuse) must be indexed under every path, with a stable (file, byte offset)
  identity for consumers to deduplicate (katamari 23-26, 285-292). The contract does
  not say this yet.
- **P15. Unfalsifiable fixtures.** Pair every "does not resolve" assertion with a
  positive resolution in the same fixture. Make the DiskTree ≡ MemoryTree fixture
  compare full indexes, not counts.
- **P16. Keep `docs_contract.rs` and raf-okf from coexisting as two interpreters.**
  Switch the root test to raf-okf for parsing and per-document checks in the same
  change, and delete the duplicated functions once the fixture corpus passes on the new
  reader.
