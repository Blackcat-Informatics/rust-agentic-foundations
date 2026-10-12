# Quality / edge-case review: tests/docs_contract.rs and the issue-refs, licenses and pre-commit gates

Head d6b6572, base 720262d. Read-only on the repo. All probes ran in throwaway copies under /tmp (since deleted). `git status` of the real worktree is clean.

## Reviewed scope

- `tests/docs_contract.rs` (1,799 lines): parser, checker and the 46 tests (1 real-tree test, 45 fixture tests).
- `docs/architecture/okf-profile.md` (the claimed rule set), plan.md decisions (docs_contract row, `issue-refs` pattern set, A9, A11, A12, adv. 11, adv. 12), issue.md acceptance criteria 3 and 4.
- `Makefile` targets `issue-refs`, `licenses`; `.githooks/pre-commit`; `.github/workflows/ci.yaml` (which gates run where).
- The real tree: 21 capabilities (5 considered, 16 designed), 21 change records, 2 architecture docs.

Method note: the first batch of probes ran `cargo test --test docs_contract --locked` in a copy (rc and failing test captured). Partway through, `/opt` (cargo slot storage) filled to 100% (no space left on device, unrelated to the repo), so the remaining probes compiled the same file with `CARGO_MANIFEST_DIR=<copy> rustc --edition 2024 --test tests/docs_contract.rs` and ran the binary. Same code, same `env!` root, same result semantics. Copies that needed git were re-initialised (the worktree `.git` file points at the real gitdir, so it was excluded) and files were `git add`ed only in the copy; no commit was attempted (the machine signature hook blocks unsigned commits and was not bypassed).

Baseline: copy of the head tree, 46 passed, rc 0. `make issue-refs` rc 0, `make licenses` rc 0, pre-commit hook rc 0.

## Verdict

The enforced rules that exist work on the real tree: every targeted mutation of section presence/order, frontmatter keys, ids, states, dangling depends, 1/2/3-cycles and self-cycles, state-to-body for considered/delivered, orphan PROPOSED, inline links and anchors, issue tokens and process wording is caught (table below). There is no CRITICAL finding. The gaps are all "silently passes" cases: the test does not pin the required capability set, does not check "resolved" text, and several documented rules or link forms are not enforced. Two of them (HIGH-1, MEDIUM-1) bear directly on acceptance criterion 3.

## Mutation table

Edits were applied to a fresh copy per row. "Caught" = rc 101 with the named violation. "MISSED" = rc 0, silently green.

| # | Edit | Expected | Observed |
|---|---|---|---|
| 1 | delete `## Usage` from census-cli | caught | caught: `missing section Usage` |
| 2 | swap Usage/Examples in census-cli | caught | caught: `sections out of order` |
| 3 | add `See #12` to a capability | caught | caught: issue number banned (line 120) |
| 4 | break a link in census-cli | caught | caught: `broken link ./censusX-debt-ratchet.md` |
| 5 | remove census-cli.md only | caught | caught (missing record flag plus broken links from 2 docs) |
| 6 | remove census-cli.md and its change record | caught | caught, only because 2 other docs link to it (broken links). See HIGH-1 |
| 7 | remove workplan-prose-assessment.md and its change record (no inbound links) | caught | **MISSED**, rc 0 |
| 8 | remove change record only | caught | caught: `requires an open change record` |
| 9 | remove `docs/capabilities` and `work/changes` entirely | caught | caught only via broken links from architecture docs/README. The "Capability >= 1" guard is conditional on the directory existing |
| 10 | remove both architecture docs / okf-profile only | caught | caught via inbound links (the test only needs >= 1 Architecture doc) |
| 11 | 2-cycle, true 3-cycle, self-dependency in `depends` | caught | caught: cycle path reported in each case |
| 12 | dangling `depends: [cap:nope]` | caught | caught |
| 13 | considered to delivered | caught | caught (PROPOSED present and open record) |
| 14 | considered with PROPOSED markers neutralised | caught | caught: `requires its candidate contract in PROPOSED blocks` |
| 15 | designed to considered with no PROPOSED | caught | caught |
| 16 | designed to delivered with open record | caught | caught |
| 17 | designed to implementing | accepted | accepted (valid) |
| 18 | real PROPOSED block added to a `designed` doc | profile table says contract "stated plainly" | MISSED, rc 0 (LOW-3) |
| 19 | `> [!proposed change:x]` or `>[!PROPOSED change:nonexistent]` in a designed doc | caught as malformed | **MISSED**, rc 0: marker not recognised at all (MEDIUM-3) |
| 20 | `## Usage` replaced by `### Usage`, `## Usage notes`, `## usage`, duplicate `## Usage`, extra `## Notes` | caught | all caught (exact-name match, no substring matching) |
| 21 | `## Usage ` (trailing space), `##  Usage` (two spaces) | accepted (CommonMark-equal) | accepted |
| 22 | a section's body replaced by `TBD`, by `<!-- later -->`, by `Usage will be documented later.` | caught by criterion 3 ("resolved, current-tense") | **MISSED**, rc 0 for all three (MEDIUM-1) |
| 23 | change record: `## Summary` renamed, bad `affects` section, id/file mismatch | caught | caught |
| 24 | change record outcome `moved:change:ghost` (no such change) | caught | **MISSED**, rc 0 (LOW-2) |
| 25 | extra non-md file in `docs/capabilities`, symlinked doc, `.MD` extension | n/a | not examined, rc 0 (LOW-6) |
| 26 | Capability-typed doc in `docs/other/` | type-in-wrong-dir | only caught incidentally (id/filename and links); LOW-6 |
| 27 | UTF-8 BOM before `---` in a capability | caught | caught as `missing frontmatter` |
| 28 | CRLF endings in a capability | accepted | accepted |
| 29 | frontmatter: unterminated `title: "abc`, unquoted `title: Built-in: census`, `description: >` (folded scalar), `title: >` | caught as malformed YAML | **MISSED**, rc 0 for all four (MEDIUM-2) |
| 30 | frontmatter: duplicate key, tab indent, unclosed `[...`, scalar bindings value, inline `# comment` on a value, empty title | caught | caught (the comment case is a strict false positive on valid YAML, LOW-5) |
| 31 | `bindings.code: [foo::Bar]` (symbol that does not exist) | profile: never name a nonexistent symbol | MISSED, rc 0; not enforceable today (LOW-4) |
| 32 | link forms: reference-style `[x][r]`/`[r]: ./missing.md`; HTML `<a href>`; HTML `<img src>` (README line 5 is the only one in the tree); two-line link | caught if links are enforced | **MISSED**, rc 0 (MEDIUM-4) |
| 33 | link forms: inline, image `![](..)`, title `"t"`, `<angle>`, `?query`, `%2D`-encoded path, link after inline code, broken link in README / CONTRIBUTING | caught | caught (`?query` and `%` forms are false positives against GitHub, strict direction) |
| 34 | broken link in `.github/PULL_REQUEST_TEMPLATE.md` | caught | MISSED (file is outside the scanned set, MEDIUM-4) |
| 35 | broken anchor (`#no-such`) to a capability or architecture doc, from a doc and from README | caught | caught |
| 36 | `#12` in src/lib.rs, Makefile, CITATION.cff, PULL_REQUEST_TEMPLATE.md | `make issue-refs` fails | `make issue-refs` rc 2 with file:line; docs_contract rc 0 (test is Markdown-only, by design, covered by the Makefile) |
| 37 | `#12`, `/pull/11` URL, `PR 11`, `this issue`, `task 5`, curly `the plan’s`, in README, SECURITY.md, AGENTS.md | caught | caught by both gates (curly apostrophe also caught) |
| 38 | `#123456`, `GH-12`, `issue 12`, `pull request 11` | per plan pattern set not covered | rc 0 in both gates (LOW-1) |
| 39 | `make licenses`: alter LICENSE-MULAN, alter LICENSES/MulanPSL-2.0.txt, alter/delete zh-Hans, delete the `Source SHA-256` line, delete LICENSE-MULAN, extra hash earlier on the line | caught | all caught with a named file; no silent pass. Altering LICENSE-MIT is not checked (plan only claims Mulan and zh-Hans) |
| 40 | pre-commit hook: `#12` staged but worktree clean | blocks | blocks (rc 1, `src/lib.rs:7`) |
| 41 | pre-commit hook: `#12` only in the worktree, index clean | passes | passes (index semantics confirmed) |
| 42 | pre-commit hook: staged capability missing `## Usage` | per plan AC3 enforced somewhere before CI | hook rc 0, commit would be allowed (MEDIUM-5, by documented design) |

## Findings

### HIGH-1: the required capability set is not pinned; deleting a document (with its change record) passes
- `tests/docs_contract.rs:1221-1245` (`repository_documents_satisfy_the_profile`), `:1239-1244`.
- The test checks whatever files exist. It never names the 21 slugs from the issue (9 census, 7 workplan M1, 5 M2), nor the two architecture pages, nor the 21 change records. The only existence assertions are `>= 1 Architecture` and, if `docs/capabilities/*` exists at all, `>= 1 Capability`.
- Mutation 7: removing `docs/capabilities/workplan-prose-assessment.md` and `work/changes/workplan-prose-assessment.md` leaves rc 0. Mutation 6 is caught only because two sibling docs happen to link to the removed file; any capability with no inbound link, or removal together with the links, passes. Renaming a capability slug consistently also passes.
- Consequence: acceptance criterion 3 ("Every capability listed above exists") has no executable enforcement. The test is the only gate; `make check` and CI just run it, so a later edit can silently drop a required capability, and so can anyone who removes the whole `docs/capabilities` directory together with links.
- Fix: add a constant list of the 21 slugs (and `normal-form`, `okf-profile` architecture ids), assert each file exists on the real tree, has the matching `id` and `state` family (9+7 `designed`, 5 `considered`, until a state change is made deliberately), and has its change record. Add a negative fixture (a repo missing one required slug). Make the `>= 1` guards unconditional. Note this pin must be updated when a capability is delivered or a record is completed; encode that in a single table so the update is one edit.

### MEDIUM-1: "resolved, current-tense text" is not enforced
- `tests/docs_contract.rs:588-592` (`check_section_layout`), `:425-440` (`banned_in_line`).
- A section passes if it holds any non-whitespace text. `TBD`, a lone HTML comment and `Usage will be documented later.` all pass (mutation 22). No scan for `TBD|TODO|FIXME|XXX` exists in the test, the Makefile or CI; plan row R35 only prescribes a one-off `git grep` at validation time. The tree is clean today (the grep I ran over docs, work and README found nothing), so this is a gate gap, not a present defect.
- Consequence: criterion 3's "resolved" half regresses silently. The future-tense half is inherently not mechanical, but the placeholder half is.
- Fix: in the test, reject placeholder tokens (`TBD`, `TODO`, `FIXME`, `XXX`, `to be written`, empty after stripping HTML comments) outside fences, and require each section to contain visible text after comment stripping. Add negative fixtures. Optionally add the same token scan to `make issue-refs` or a sibling target so it also runs in the hook and CI `gates` job.

### MEDIUM-2: malformed YAML is accepted by the hand-written frontmatter parser
- `tests/docs_contract.rs:90-101` (`unquote`), `:210-251` (`parse_fields`).
- Accepted with rc 0: `title: "abc` (unterminated quote, kept literally as `"abc`), `title: Built-in: census` (unquoted colon-space, invalid in real YAML), `description: >` and `title: >` (a folded-scalar indicator is read as the non-empty value `>`, so the "non-empty string" rule passes on an effectively empty value).
- Consequence: the test is the claimed implementation of the profile, but it is a superset of YAML. The future `raf-okf` parser and GitHub's renderer will reject or read these differently, so a doc can be green here and broken there. The empty-by-indicator case defeats the non-empty `title`/`description` rule.
- Fix: reject a scalar that starts with a quote but does not end with the matching quote; reject an unquoted scalar containing `: ` or starting with `>`, `|`, `&`, `*`, `!`, `%`, `@`, `` ` ``; reject a scalar equal to a block-scalar indicator. Add one negative fixture per form. (Inline `# comment` handling is the opposite problem, LOW-5.)

### MEDIUM-3: PROPOSED marker detection is case- and spacing-sensitive and fails open
- `tests/docs_contract.rs:829` (`trimmed.starts_with("> [!PROPOSED")`), `:52`.
- `> [!proposed change:x]`, `>[!PROPOSED change:x]`, `>  [!PROPOSED change:x]` and a marker inside a list item or nested quote are not recognised (mutation 19), so they are neither validated against `work/changes/<slug>.md` nor counted toward the `considered` rule or rejected from `delivered`. A `delivered` document can hold a mis-spaced PROPOSED block and pass; an orphan marker passes.
- Consequence: the state-to-body rule and the orphan check (okf-profile "PROPOSED blocks") are bypassable by a typo that a reader would still read as a proposal.
- Fix: detect any line whose quote-stripped content, case-insensitively, starts with `[!PROPOSED` (allow `>` followed by zero or more spaces) and route it through the same well-formed/malformed path so a near-miss is reported as malformed rather than ignored. Add negative fixtures for lowercase and no-space forms.

### MEDIUM-4: link checking covers inline Markdown links only, in a limited file set
- `tests/docs_contract.rs:886-905` (`link_targets`), `:450-457` (`is_scanned`), `:925-979`.
- Not checked (rc 0): reference-style links and definitions; `<a href>` and `<img src>` HTML (the README logo at `README.md:5` is the only HTML reference in the tree and is unchecked: renaming `docs/raf-logo.svg` leaves the test green); a link split across two lines; any Markdown outside top-level, `docs/` and `work/` (`.github/PULL_REQUEST_TEMPLATE.md` and the issue templates); anchors in non-Markdown targets (acceptable).
- Consequence: the profile says "every relative link resolves" without qualification; the test enforces a subset. The brand logo and a PR-template link can rot unnoticed.
- Fix: also scan `[label]: target` definitions, and `href="…"`/`src="…"` attributes; scan all tracked `*.md` (derive the file set the way `walk` already does, but drop the directory limit for link checks). If multi-line links are out of policy, state that in okf-profile ("links are single-line") so the rule matches the check.

### MEDIUM-5: the pre-commit hook does not run the document contract
- `.githooks/pre-commit:24-31`; `.github/workflows/ci.yaml:63` and `:144`.
- The hook runs fmt, reuse, licenses and issue-refs only. A staged capability with a missing section commits cleanly (mutation 42). The contract runs in `make check` and in CI's `cargo test --workspace`, so it cannot reach main, but the plan and AGENTS.md present these as commit-time gates for the issue-number and wording bans only, so this is an expected design limit, not a regression. It does mean criterion 3 is enforced at push/CI time, not commit time.
- Consequence: a broken doc is discoverable only after `make check` or CI. Low blast radius.
- Fix (optional, cheap): add `run docs-contract cargo test --test docs_contract --locked` against the snapshot (`--manifest-path "$snapshot/Cargo.toml"`; the test already reads `CARGO_MANIFEST_DIR`), or state in the hook header and AGENTS.md that docs structure is a CI-time gate. The test walks the working tree, not the index, so the snapshot form is the correct way to get commit semantics.

### LOW-1: issue-number pattern set has known blind spots (shared by Makefile and test)
- `Makefile:67-79`, `tests/docs_contract.rs:378-391`.
- `#123456` (6 digits, by the analysis's `{1,5}\b`), `GH-12`, `issue 12`, `pull request 11` pass both gates (mutation 38). Short hex colour tokens like `#000` and `#333` would be false positives; none are in the tree.
- Fix: widen to `#[0-9]+\b` is not needed unless 6-digit ids matter; add `\b(issue|pull request|GH-)[ -]?[0-9]+` case-insensitively if the "no issue numbers" rule is meant to cover prose forms. Decide and record the decision.

### LOW-2: `moved:change:<slug>` outcome target is not resolved
- `tests/docs_contract.rs:756-761`.
- Only the slug shape is checked; `moved:change:ghost` passes (mutation 24) though the profile defines the outcome as moving to a change. Fix: require `work/changes/<slug>.md` to exist (the cross-reference pass has the repo available).

### LOW-3: `designed` documents may contain PROPOSED blocks
- `tests/docs_contract.rs:1078-1108`.
- The profile table says `designed` states the designed contract plainly, and plan A11 says "nothing reads as delivered", but only `considered` (requires) and `delivered` (forbids) constrain PROPOSED blocks. Whether a designed capability may carry a PROPOSED block is not stated; decide, then either forbid it for `designed` or record that it is allowed. Also, "Outside PROPOSED blocks a document states the current contract only" has no check; it is unenforceable mechanically and could be removed from the claimed enforced set.

### LOW-4: `bindings` list contents and A12 are not enforced
- `tests/docs_contract.rs:605-623`.
- Only the key set is checked. A12 says the lists are empty for now and the profile says a list never names a nonexistent symbol; `code: [foo::Bar]` passes (mutation 31). Acceptable until the source index exists, but either assert empty lists now (the issue's tree has all 21 empty) or record that content is checked only by the future tool.

### LOW-5: strict parser false positives on valid YAML
- `tests/docs_contract.rs:84-101`, `:210-251`.
- A trailing `# comment` on a value, a `key:` value containing a `)` or `]` mixed with quotes, and `%`-encoded or `?query` link targets are reported as errors. This errs on the safe side, but it makes the claim "the test enforces YAML" imprecise; note it in the okf-profile or accept inline comments. No real doc triggers it.

### LOW-6: file-set and directory edge cases
- `tests/docs_contract.rs:460-468`, `:1173-1208`.
- A Capability/Change-typed document outside its directory (e.g. `docs/other/x.md`) is checked as a document but not flagged for location; non-`.md` files and `.MD` files in `docs/capabilities` are ignored; symlinks are skipped without a message; the loader reads the working tree (untracked files included), so a scratch file in `docs/` can satisfy a link locally that CI will not see. Fix: flag typed documents outside their directory, and flag any non-`.md` file in the typed directories.

### LOW-7: fixture coverage gaps (negative fixtures missing for implemented rules)
Rules that exist in code but have no failing fixture in the 45:
- duplicate section (`:579-581`), nested/unmatched generated markers and `generated end marker without a start` (`:845-865`), malformed generated marker, bad `id` prefix/kebab (`:538-554`), non-cap `depends`/`changes` entries (`:628-635`, `:661-668`), empty `changes`, `items` shape errors and unknown item key (`:742-768`), non-scalar `state`, empty `title`, self-dependency and 3-node cycle (the real-tree probes above show they work; the fixtures would pin them), anchors after a fenced region, and a section hidden in a fence.
- `rejects_sections_out_of_order` (`:1451-1464`) first half renames a section rather than reordering; only the second half tests order.
- `parses_quoted_and_block_forms` has no failing parse case for the MEDIUM-2 forms.
- Fix: add one fixture per bullet. None of these rules is currently broken on the real tree; this is regression protection.

### LOW-8 (out of focus, confirm): rust-version is 1.98.1, the issue specifies 1.97.1
- `Cargo.toml:25`, `Makefile:9,40-44`, issue.md workspace item 1 and plan A2/A15 say `1.97.1`. validation.md records `rust-validate 1.98.1`. If this is a deliberate user-approved change, record it where the issue text lives; otherwise it is an acceptance-criterion deviation. Not examined further here.

## Rules confirmed enforced on the real tree (no finding)

- Seven sections exact-name matched (no substring), in order, none duplicated, none empty; sections in fences ignored.
- Closed key sets for Capability/Change/Architecture, duplicate-key and unknown-key rejection, `resource` rejected, id equals file stem, bad `state`, `bindings` key set.
- `depends` resolution and acyclicity (1-, 2-, 3-cycles reported with path), `changes` resolution, `affects` slug-in-`changes` and section vocabulary.
- State rules: considered requires PROPOSED, delivered forbids PROPOSED and any open record, open states require a record, PROPOSED must name an existing record that lists the capability, only in Capability docs.
- Link and anchor resolution for inline links/images across top-level Markdown, `docs/` and `work/`, with GitHub-style anchors and duplicate-heading suffixes; links leaving the repo are rejected.
- `make issue-refs`: whole tracked tree, both pattern sets, error exit on grep failure, index vs worktree semantics correct in the hook. `make licenses`: every tampering variant fails with a named file; no silent skip on missing files or a missing recorded hash.
- CI runs `cargo test --workspace --locked` (includes the root package) and a `gates` job with `make licenses issue-refs`.
