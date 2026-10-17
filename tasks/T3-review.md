# Task 3 review: raf-workplan records, digest and loading

## Scope

The staged diff on top of 8844239 (33 files, +4049/-186): `raf-okf` constructors,
`to_document` writers, `name()`/`text()`, the linear limits check; `raf-workplan`
`digest`, `files`, `spdx`, `record`, the loader for the record directories, the
graph nodes/edges/queries and six finding kinds; tests; the okf-profile and
workplan-document-model documents and records; CHANGELOG. Read against plan.md
(D1-D3, D5, D9, D13, D15, Task 3, R2-R4, carried items), T1/T2 reviews, AGENTS.md,
SECURITY.md. All experiments ran in a scratch worktree (HEAD plus the staged diff,
byte-identical to `git diff --cached`), removed afterwards; the reviewed worktree
was not modified and its staged diff is unchanged.

## Evidence

1. **SHA-256 and encoding.** A scratch test compared `Digest::record(t)` with
   `sha256sum` over the documented encoding `record NUL NUL <len> NUL <t>`: text
   lengths 0..300 (all 64 residues mod 64 of the encoded length, so every padding
   boundary 55/56/63/64/65/119/120 is crossed), 200 random texts of mixed
   ASCII/NUL/tab/multi-byte UTF-8 up to 400 chars, a 1 MiB text, and six CR/LF
   mixes. All equal. `Digest::section` equals `sha256sum` of
   `section NUL <slug> NUL <len> NUL <section_digest text>` for LF, CRLF and
   lone-CR bodies. Domain separation (kind first), name/text boundary and field
   concatenation tests in the crate pass; with NUL-free kinds and names (slugs,
   resolved paths, empty) the encoding is injective. CRLF equals LF; a lone CR is
   kept (`a\rb` differs from `a\nb`); determinism across runs is tested.
   `Digest` parse refuses uppercase, leading/trailing space, trailing newline, 63
   and 65 digits, empty, `+`/`0x` prefixes, `g`, and 32 two-byte characters (64
   bytes); display round-trips.
2. **raf-okf writer.** Additive API (`new` constructors, `to_document`,
   `EvidenceVerdict/AssessmentVerdict::ALL/name`, `EvidenceSubject::text`);
   `#[non_exhaustive]` attributes unchanged and the constructors are the coherent
   way out of crate. Hostile round trip: 53 hostile strings (newlines, `a\nverdict:
   fail`, CRLF, `: `, ` #`, quotes, backslash, flow brackets, `- `, YAML keywords,
   `&`/`*`/`!`/`|`/`>`/`?`, `---`, U+2028, U+0085, BOM, NUL, DEL, a claim marker,
   issue-like tokens, unicode, 5000 chars) in command, revision, `for`, tags,
   timestamp, input name/digest, subject, measure name, coverage scope/reason,
   assessment input name. Every case either round-trips to an EQUAL record (command
   47, tags 47, value names 10, coverage 5), is refused by the writer
   (`ConstructError`), or is refused by the profile on read-back. No rendering
   parsed as a different record or injected a key. Slugs: only kebab slugs are
   accepted by `Record::path`. 5000 values + 5000 limits round-trip in 70 ms
   (debug).
3. **Loader.** Record directories read; `singleton-other-file` fixtures cover
   `other.md`, `index.md`, `notes.txt`, `default.md.bak`, with and without the
   record; nested record is `nested-directory`. Absent directories: the whole
   pre-existing suite passes, and `cargo run -q -p cargo-workplan -- check` on this
   repository gives byte-identical output and exit 0 at HEAD and with the diff.
   The six new kinds are in `OWN_KINDS` and the closed-list test passes; each has
   a fixture with fix text and a clean neighbour; a broken record/document named
   by a reference is not reported as missing.
4. **Graph.** Covers/Claim/Assesses/Scope edges, record and crate nodes, queries
   sorted as documented; claims in PROPOSED blocks are not edges (fixture);
   unknown evidence names file and line; `claim-evidence-not-for` (claim cites
   evidence whose `for` lacks the capability) matches D4's "both are required and
   a mismatch is a finding". 100k-claims fixture passes; `add_claims` uses hash
   sets (linear).
5. **Writer.** Experiments: `work/` symlinked outside, `work/evidence` symlinked
   outside, a file at `work/evidence` -> refused, nothing written outside; a
   dangling symlink at the target -> `Exists`, link target not created; `replace`
   through a link -> `Missing`, outside file untouched; directory at target ->
   `Exists`/`Missing`; 20 rounds of 16 concurrent `create` of one id -> exactly one
   Ok and 15 `Exists`, 20 records load. `replace` in a read-only directory fails
   with `Write`, target byte-identical, no temporary left; a successful replace
   keeps mode 0640 and leaves no temporary. Text is validated (profile + reference
   scan) before any write. SPDX pair/line logic moved verbatim into `spdx.rs`;
   `files::create_new` is the old `create` body with the directory as a
   parameter; `new_change` tests (lifecycle, scenarios, cargo-workplan `new`) pass.
6. **Documents.** workplan-document-model.md: only bindings, three PROPOSED
   blocks (Contract, Usage, Verification) and the regenerated matrix; record
   `affects` gains `#usage`, `#verification`. okf-profile.md: bindings and
   additions inside its existing PROPOSED blocks. Architecture-page sentence now
   says only a misspelt claim marker or unknown `raf:` word is refused and a
   malformed `raf:generated:` comment is text — true (T1 advisory 1 closed).
   CHANGELOG entries accurate.
7. **Mutation (6).** SHA padding `>= 56` -> `> 56`: killed (unit vectors). Length
   field dropped from the encoding: killed (`each_kind_hashes_its_documented_encoding`).
   Singleton rule disabled: killed (closed-list kinds test). `create_new` ->
   create+truncate: killed (`an_existing_file_is_refused_and_left_byte_identical`).
   `claim-evidence-missing` dropped: killed (kinds test). `replace` as
   `fs::write` over the target: **SURVIVED** the whole lib + records suite (killed
   by the scratch read-only-directory test).
8. **Hygiene.** New files carry SPDX; no `unsafe`, no `#![feature]`; the one
   `#[allow]` (`many_single_char_names`) has a reason; no tracker numbers, process
   wording or attribution in added lines; no Cargo.toml change (no dependency).
9. **Gates** (scratch tree, `CARGO_BUILD_JOBS=6`): `make fmt`, `clippy`, `doc`,
   `deny`, `reuse`, `msrv`, `issue-refs` all rc 0; `cargo test --workspace
   --locked` rc 0 (102 binaries, 1716 tests); `docs_contract` + `repo_gates` rc 0;
   `cargo test -p raf-okf -p raf-workplan --release --no-fail-fast` 971 passed (a
   first parallel release run hit `Text file busy` once in the untouched
   `sync.rs` example test; it passes on rerun); `make docs-sync` twice: no change
   either time.

## Findings

### REQUIRED

1. **Record item not true as built.** `work/changes/workplan-document-model.md`
   item "The graph holds the records and their edges to the capabilities,
   sections, symbols and evidence they name." The graph has no edge from a record
   to the symbols (or sections, records) of its `inputs`; only `for`, claim,
   subject and `scope` edges exist. Task 3 owns the graph edges, so either add
   input edges (record -> symbol/section/record input, with a fixture) or reword
   the item to the edges built and move any unbuilt part to a tracked record (D14).
2. **Atomic replace is untested.** The mutant replacing stage-then-rename with an
   in-place `fs::write` survives every test. Add a fixture that kills it, e.g.
   `replace` in a directory that refuses new files fails with `RecordError::Write`,
   leaves the target byte-identical and leaves no temporary file (and/or the
   target's inode changes on success).

### ADVISORY

1. Section digests normalise CRLF twice (`section_digest` already does, then
   `field` again), so a section with `x\r\r\ny` digests equal to `x\r\ny`, while
   record and symbol digests keep them apart. Harmless in practice; normalise once
   or state it.
2. The kind and name are NUL-terminated, not length-prefixed; the
   "no two different sequences of fields encode to the same bytes" sentence holds
   because kinds and names never contain NUL (slugs, resolved paths). Worth one
   clause in the module doc.
3. Any file in `work/coverage`, including a `.keep` or the temporary of an
   interrupted `replace`, is `singleton-other-file`. Consistent with the rule;
   the fix text could mention a leftover temporary.
4. Release parallel run hit `Text file busy` once in
   `sync::the_docs_sync_example_writes_the_expected_files_once_and_nothing_on_a_second_run`
   (pre-existing, file untouched); a flake to watch.

Round-one verdict: BLOCKED (REQUIRED 1 and 2).

## Round two: recheck of the fixes

The staged diff now has 33 files, +4395/-190. It was re-applied to a scratch
worktree byte-identical to `git diff --cached`, which was removed afterwards.
The reviewed worktree was not modified.

* **R1 fixed.** `EdgeKind::Input`, `Edge::Input`, `InputEdge`, `InputTarget`
  (Section / Symbol definitions / Record), `Graph::inputs`, `Graph::inputs_of`.
  A section input resolves only to a capability that exists and a valid slug; a
  symbol input resolves through the source index to its definitions; a record
  input resolves only to a loaded evidence id. An unresolved input is no edge
  and no finding.
  * The item "edges to the capabilities, sections, symbols and evidence they
    name" is now true as built.
  * Contract, Verification, bindings, the generated matrix and CHANGELOG state
    the edges.
  * New fixture
    `each_input_of_a_record_that_resolves_is_an_edge_and_one_that_does_not_is_no_finding`.
  * Mutants: dropping every input edge is killed (that fixture and
    `the_graph_has_every_kind_of_edge`), and so is dropping the input symbols
    from the nodes.
* **Symbol node semantics widened.** `Node::Symbol` and `NodeKind::Symbol` now
  also cover an item named by a record input, and their docs say so.
  * `item_paths` gains those items; it is used only to sort.
  * `Graph::nodes()` has no other caller in any crate's source.
  * `code_of`, `capabilities_of`, `tests_for` and `bound_items` still read
    bindings only, so no other query changes.
  * Whole raf-workplan suite green.
  * `cargo run -q -p cargo-workplan -- check` on this repository is
    byte-identical to HEAD and exits 0.
* **R2 fixed.** `replace_renames_a_new_file_over_the_target_and_never_writes_into_it`:
  * On success the target's inode changes.
  * Where permissions bind the user (probed, so it also works as root), a
    read-only directory gives `RecordError::Write`, the target stays
    byte-identical and no temporary file is left.
  * The round-one mutant (`fs::write` over the target) is now killed by it.
* **Advisories 1-3 addressed.**
  * Advisory 1: CRLF is replaced once. `field` no longer normalises.
    `Digest::record` and `symbol_of` call `lf()`, and `Digest::section` hashes
    the text `section_digest` already normalised.
    * My scratch test confirms `x\r\r\ny` and `x\r\ny` now differ for sections.
    * Section digests equal `sha256sum` of the encoding over the
      already-normalised text, and the record-digest/`sha256sum` sweep (lengths
      0..300, random texts, 1 MiB) still matches.
    * Mutant: dropping `lf()` is killed (`crlf_and_lf_give_the_same_digest`).
  * Advisory 2: the module doc states that no kind or name holds NUL.
  * Advisory 3: the fix text mentions a leftover temporary file.
* **Re-run of the six mutants:** all killed (SHA padding, length field,
  singleton rule, create replaced by overwrite, `claim-evidence-missing` dropped,
  non-atomic replace).
* **Hostile round trip and writer containment** (scratch tests re-run): all
  pass (symlinked `work/` and `work/evidence`, dangling link, file in the way,
  16-way concurrent create).
* **Hygiene:** added lines hold no `unsafe`, attribution or tracker tokens, and
  no dependency is added.
* **Gates:**
  * `make fmt`, `clippy`, `doc`, `deny`, `reuse`, `msrv`, `issue-refs`: rc 0.
  * `cargo test --workspace --locked --no-fail-fast`: 1718 passed, 0 failed.
  * `docs_contract` + `repo_gates`: rc 0.
  * raf-okf + raf-workplan release: 973 passed.
  * `make docs-sync` twice: no change either time.

### REQUIRED (round two)

None.

### ADVISORY (round two)

1. Advisory 4 (the `Text file busy` flake in the untouched sync example test)
   still stands. It did not recur this round.

VERDICT: PASS
