<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Current acceptance

PR 57 is open for issue 4 at head 3521144. Consolidated report-form-3 completion
passed; normal form 1 remains unchanged. The implementation passed complete
make check in closure-final-make-check.log and integrated current main with normal
hooks. Fresh CI then found today's ctutils advisory. The exact two-lockfile 0.4.4
correction passed both deny gates, all 195 Census/CLI consumers and normal signed
hooks. Full make check on that patched lock is recorded in ci-57-ctutils-make-check.log;
fresh hosted deny passes. Hosted Linux subsequently exhausted runner disk during
binding verification; exact check annotations are retained. Job log downloads
returned HTTP 404. The workflow-only repair retains line-table debug information
and disables incremental caches in the three native test jobs; commands and
assertions remain unchanged. Independent review is in tasks/ci-storage-review.md;
the normal commit hook passed (document contract, 76 repository gates and assessed
self-workplan). The repair is committed as 3521144. Hosted rerun must establish sufficient storage.
The patched-lock full run passed fmt, all-target Clippy, workspace/release tests,
docs, deny, REUSE, MSRV and all five 5,000-run fuzz targets. It exited 2 at the
cross-target prerequisite check: the current managed toolchain lacked all three
required targets. Installed those exact targets on its existing dated toolchain
without changing the compiler. The remaining `make cross-check licenses issue-refs`
passed with exit 0 in raw/ci-57-remaining-gates.log: all three cross targets and
both 76-case repository selectors. Earlier completed targets are reused;
the original invocation is not relabeled exit 0.
No merge or issue closure is claimed. Historical failures and superseded source
scopes remain below.

Final pre-merge verdict: all twelve hosted checks pass on 3521144. Every host
accounts for all 2,816 bindings: Linux 2,814 pass / 2 not-applicable, macOS
2,802 / 14, Windows 2,620 / 196, zero failures. The three complete job logs are
raw/ci-57-final-{linux,macos,windows}-job.log. Final paginated feedback capture is
empty for reviews, inline/issue comments and review threads. Main remains
d957883; clean merge-tree is 9e4788795fc8333a173b4ff67fd58f448b5cd0b2. Required
acceptance is satisfied; ghprsq publication and verified closure/cleanup remain.

| Acceptance | Result and scope |
|---|---|
| Production callers and lint | `ci-57-ctutils-make-check.log`: patched-lock workspace all-target Clippy passed with warnings denied; the complete workspace and release test phases passed. Source/locks are unchanged by the final workflow-only repair. |
| Runtime foundation | `closure-report-form-runtime-fixed.log`: 195 executed Census and actual CLI tests passed on layout 3, superseding the older 52-case foundation receipt. |
| Settled deltas | Settings boundary, custom variant registration, foreign pattern retention and invalid pattern identifiers each executed successfully in `closure-*-runtime.log`; these extend the foundation cases. |
| Public examples | Current bound tests/examples passed in `closure-layout3-complete.log`; the workspace test phase also passed. The earlier standalone Rustdoc receipt qualified layout 2 only. |
| Review | `tasks/layout-detectors-review.md`: independent requirements, callers and failure-path review; reported defects corrected and relevant deltas reviewed. |
| Current hosted macOS | `raw/ci-57-final-macos-job.log`: job 120389632544 passed on 3521144. Actual no-slug verification accounts for 2,816 bindings: 2,802 pass, 14 platform-not-applicable, zero failures. Linux and Windows remain pending. |
| Current hosted Linux | `raw/ci-57-final-linux-job.log`: job 120389632678 passed on 3521144. Actual no-slug verification accounts for 2,816 bindings: 2,814 pass, 2 platform-not-applicable, zero failures. Only Windows remains pending. |
| Native preference recheck | Inspected clean `~/Active/purrdf` main `ea8b21c5e`: workspace dependencies and public lex/JSON/testkit modules still supply no TOML reader or general durable file-replacement API. Foreign TOML decoding remains the explicitly documented complete `toml` parser boundary. RAF's debt writer owns its filesystem publication policy; the native testkit benchmark store only writes its own estimate schema. Existing RDF, JSON, regex, numeric and scratch callers retain native APIs. |
| Current upstream TOML boundary | Rechecked clean main `df1a0e515`: native-codec module/export inventory has no TOML profile. The only TOML-specific source is helper-census's private, binary-only strict subset parser, which refuses valid complete-TOML forms and supplies no reusable public API. Upstream issue 685 remains open. This does not replace the complete foreign-input parser contract. |
| Lifecycle | `closure-layout3-complete.log`: native completion passed with exit 0 for ledger, debt, migration and the affected normal-form report contract; removed the record and rewrote relationships. Prior version-2 receipts are historical. |
| Earlier qualification failures | Original full runs exposed the old CLI package list, missing architecture shape/version reconciliation, then the old library package list. All are corrected; original raw logs remain unchanged. |
| Assessment currency | The six stale normal-form assessments were refreshed; current assessments were preserved. The coverage Verification prose received one refresh after its package count changed. Native completion and final workspace self-workplan both passed with all 133 current and accepted. |
| Package and root gates | `closure-package-coverage.log`: actual nine-package item/test/line coverage case passed. `closure-package-root-gates.log`: document-contract, all 72 repository gates and self-workplan passed, exit 0. All assessments current and accepted. |
| Dependency policy | The qualified run passed workspace tests, release tests and docs, then refused purrdf-jsonschema's additional Unicode terms. Its pinned 3.0.1 Unicode 17 table headers, inventory and notice were inspected; root/fuzz configs now admit Unicode-3.0 for exactly that package. Both native deny commands passed, followed by REUSE and all 72 repository gates. No Rust implementation changed. |
| Full qualification | `closure-final-make-check.log`: `make check` passed with exit 0: fmt, warnings-denied all-target Clippy, workspace/release tests, docs, deny, REUSE, workspace/fuzz MSRV, all five 5,000-run fuzz targets, three cross targets and both repository selectors. |

The second full gate (`closure-make-check-fixed.log`) passed the corrected
nine-package code/test coverage case and all Census Gate/debt/migration runtime
groups, then failed the architecture shape-table assertion. The architecture's
explicit vocabulary/shape version rule requires REPORT_FORM 3 for counted debt;
HEAD already published its version-2 closed shape even without an executable debt
reader. Independent review confirmed this obligation. Ordinary readers remain
strict; the native migration contract supports the implemented report layout,
while foreign TOML/JSON import remains the historical dialect route. No previously
executable native RDF ledger/debt reader is being removed.

Current layout qualification: `closure-report-form-runtime-fixed.log` executed
195 Census and actual CLI tests with zero failures, ignored or filtered cases.
The first layout-3 focused run refused two positive fixture builders still pinned
to version 2; both now use the public REPORT_FORM. Native report fixtures and 15
canonical fuzz carriers were projected through Graph/Builder. Original malformed
and older-layout negative seeds remain. Temporary generators exist only in stage
raw for replay, not the product source. Registry oracle pins retain original term
spellings with explicit version/BASE substitution; no fresh registry parse was run
or claimed. Independent layout review found that provenance wording, now corrected.

The package-list audit found exactly four repository scope inventories: the
coverage record, CLI coverage test, library coverage test and root source-index
test. All now include cargo-census. Synthetic fixtures and the historical counted
corpus remain untouched. The coverage verification prose now says nine; its one
changed assessment was refreshed. README points to the canonical capability
catalogue and accurately describes the delivered Gate and partial CLI. No product
API changed in these inventory repairs; layout-3 completion remains applicable.

# Historical qualification snapshots

Issue 4 remains incomplete. No commit, push, PR or completion claim exists.

Implemented locally: immutable exact-set debt groups, positive per-job counts,
canonical Turtle/N-Triples through the existing native RDF builder/parser,
closed debt shapes, version-aware comparison and explicit growth/shrink results.
The census collector can reuse an already loaded source workspace, so ledger
resolution need not trigger a second source load.

The closed RDF ledger reader and immutable policy view are now implemented
locally, including settings, jobs, variants, exact distinct sets and extension
node retention. Forbidden-name compilation uses purrdf-jsonschema's public ECMA
API at the existing native revision. Full offline Cargo metadata resolution
completed with only that one additional package in Cargo.lock. Rule, RuleHit and
RuleFixtures admission now use the existing bounded source parser for both case
sets. These additions compile and have focused runtime acceptance below. The capability
document carries the explicit proposed RDF policy/TOML authoring contract.

Source review found a nested-group comparison defect. The correction makes
missing exact rows visible and permits subset shrinkage only when the recorded
superset itself disappeared. The reviewer confirmed that correction by source
inspection; adjacent API wording was then aligned. See
tasks/debt-foundation-review.md.

Rustfmt completed. `cargo check -p raf-census --tests --locked -j 1` completed
with exit 0 (census-compile-fixed.log). Its first attempt correctly refused a
const comparison of references; dereferencing the primitive operands repaired
it without unstable features or lint changes. Current host inspection showed
about 40 GiB available, so the compile used its own normal managed build
directory and the shared compiler cache rather than waiting behind the
platform command. No broad qualification has been launched.
The production Gate and debt-write path are now implemented and compile
(gate-compile-fixed.log and gate-write-compile.log, exit 0). The disk path uses
native root discovery, bounded Tree reads and the existing workspace collector.
Registered rules execute admitted fixtures and ownership-checked real hits.
Alias resolution, file-qualified variants, documented source anchors, exact-set
sanctions and non-debt write refusals share one evaluation. A current-version
record may only shrink; missing/version-changed records are explicit adoption.
The writer stages one complete canonical record and renames it into place.

The focused command cargo test -p raf-census --test census_gate --test debt
--test rules --locked -j 1 completed with exit 0: six Gate, nine debt and four
rule cases, no ignored or filtered tests (gate-focused.log). The standalone
debt example then completed with exit 0, exercising real disk/root discovery,
registered Rule execution, adoption, byte equality, growth refusal, shrinking
job counts, non-debt refusal and version replacement. Its first attempt exposed
that the native TempDir unit-test constructor does not cover copied example
executables; using its explicit new_in API fixed the intended scratch boundary.
No runtime gates or cache policy were weakened. See debt-writer-example*.log.
The architecture lookup for a reusable
atomic-file API timed out after 300 seconds; direct inspection of native lex
and rdf-core identified no suitable general public file writer. The host file
publication boundary uses standard filesystem primitives, with cleanup failures
reported, and does not duplicate RDF storage, parsing or canonicalization.

The independent foundation review found a physical ownership mismatch and a
whole-workspace scan per rule hit. One indexed source-site mapping now serves
home/entry/variant classification and real Rule ownership. All eight Gate
integration cases passed (gate-physical-owners-final.log), including a shared
file reached through two modules, its reexported home, rule ownership through
the noncanonical reach, and a wrong-file variant refusal. Every variant source
site must match its detector; one valid alternative cannot sanction another.
The repeated-symbol cfg projection now uses counted canonical symbols. Ordinary
one-count debt retains its spelling; counts above one have native RDF
MemberMultiplicity nodes. Explicit distinct declarations must match resolved
physical cardinality, and alias overlap is refused. The affected Gate/debt
command passed nine plus ten cases (census-counted-acceptance.log), including
two same-symbol cfg definitions, an explicit sanction, a third-copy refusal,
both native formats and count reduction. The same independent reviewer confirmed
all three original findings resolved by source inspection, with no new concrete
delta defect. Debt keys duplication relations; equal projected
relations coalesce rather than claiming an injective physical-source archive.
The preceding all-target Census Clippy command completed with warnings denied
and exit 0 (census-clippy-complete.log). The count-model all-target Clippy check
also passed (census-counted-clippy.log). A focused runtime case refused redundant
count-one nodes and unowned multiplicity assertions, with the valid counted
graph accepted; that case and its affected Clippy target both passed
(census-counted-refusal*.log). Broad repository gates have not run for the
unfinished delivery.

Still required: remaining staleness coverage,
complete declarative-pattern and built-in detector execution, reference loading,
full Gate/debt-write acceptance including the real binary path,
production callers, capability
reconciliation and actual lifecycle completion, mandatory gates, publication and
sequential merge. CLI delivery remains mapped to the existing CLI issue; its
requirements are not counted as delivered here.

Modern RDF-ledger/index migration is implemented through the native graph and
source ownership paths. Actual production compile and four focused runtime
cases passed: divergent member hashes, cross-key replacement collisions,
retained unmapped assertions/comments and version/report-kind refusals.
All-target Census Clippy passed with warnings denied. The independent migration
review found no concrete delta defect and records its coverage limits in
tasks/migration-index-review.md. The final six focused migration cases passed
with exit 0 (migration-index-complete-runtime.log), including both cfg shim
sites of one symbol and oversized split output refused beside a same-size
successful mapping. The affected integration target's final Clippy also passed
(migration-index-complete-clippy.log). CLI writing and whole migration
capability completion remain unfinished.

Legacy import now accepts bounded complete TOML through the existing toml crate
at the foreign-input boundary, and bounded baseline JSON through native PurRDF
JSON/Record APIs. It maps old hashes through member symbols, moves all three
repository dialects into owned extension assertions, and preserves unknown or
unprojectable values as explicit UnmappedRow results and escaped review comments.
Missing or multiline summaries remain explicit unmapped work; output with no
unmapped rows must pass the actual unified Ledger reader before returning.
Native WorkList supplies the fallible depth frontier, including real spill
coverage; native JSON preserves metadata array order and duplicates. No local
TOML grammar or generic profile was introduced. The five focused runtime cases
passed (migration-legacy-runtime-fixed.log), including exact depth neighbours,
both byte caps, malformed/ambiguous input refusals and all three dialect shapes.
Independent source review in tasks/migration-legacy-review.md confirmed the
invalid-success summary finding and native frontier correction resolved.
All-target Census Clippy passed with warnings denied after the settled importer
changes (ledger-final-clippy-fixed.log). The same check covers the shared
16 MiB debt read/write boundary and explicit stale entry/fingerprint/variant
fixtures; their latest runtime receipt is collected separately below.
The settled Gate/debt/import runtime command passed 16 + 12 + 5 cases, with no
ignored or filtered cases (ledger-staleness-bound-runtime.log). This establishes
the explicit stale entry/fingerprint/variant neighbours and early oversized-debt
refusal; the writer size branch has independent source review rather than a
large generated runtime fixture.

Independent source review identified a shared migration completeness defect:
one surviving member could conceal another member's failed resolution or
missing current fingerprint. The mapper now retains the whole original row as
unmapped and reports the gap. Six affected RDF-index cases passed after the
correction (migration-member-completeness.log), including a partial-loss case;
the legacy target also exited 0 (migration-legacy-member-completeness.log).
The reviewer corrected the earlier permissive reading explicitly in
tasks/migration-index-review.md and confirmed both this fix and the debt cap.

Five byte-layout detector recognizers are being adapted from the existing
private PurRDF helper-census implementation into the intended shared RAF core.
They are syntax recognition, not duplicated arithmetic/encoding implementations;
no public reusable native detector API exists in the inspected binary module.
Gate assigns actual source ownership, shipping status and home-package exemption.
Early caller compilation found the expected syn 3 non-Copy operator-token
boundary and borrowing fixed it. Runtime and independent source review are
pending; the full fourteen-detector and declarative-pattern contracts remain open.

The detector production delta now passes all nineteen Gate/migration integration
cases (detector-production-runtime.log), with no ignored or filtered cases.
The seven implemented IDs are the five byte-layout detectors plus hex-table and
json-pointer-escape. The two fact-backed rules reuse existing collected values;
fact-only jobs do not parse source again. Each has a real positive/negative and
shipping/all-target/home-package boundary. Independent source review in
tasks/layout-detectors-review.md found donor overmatching in direct masks,
unrelated zero counts and rendered format order, plus a native numeric reuse gap.
All were corrected. The review's two residual cases, compound assignment and
implicit format precision, are also corrected and independently source-confirmed.
Exact-literal JSON Pointer semantics are explicit in the capability proposal.
All-target Census Clippy passed with warnings denied after extracting unchanged
rule admission into its own function (detector-production-clippy-fixed.log).
The nineteen-case runtime result predates only that mechanical extraction;
its computation is unchanged. Rustfmt and git diff --check both passed.
No broad gates, commit, publication or complete detector/provider claim exists.
Rule diagnostics now select the ledger's configured home, including its public
reexport spelling. The recognizers no longer hard-code a competing replacement.
The focused five-detector runtime case passed with that required diagnostic
(detector-configured-home-runtime-fixed.log), and all-target Census Clippy passed
(detector-configured-home-clippy.log). Its first assertion expected the physical
canonical symbol rather than the configured public entry; only that assertion
was corrected to the explicitly chosen public API. Source classification still
uses actual physical ownership. No extra broad runtime suite was repeated.

Next root implementation: remaining hex-format-loop, hex-pair-radix,
hex-digit-radix, std-default-hasher and the three job-context-dependent detectors;
then complete declarative patterns/reference/provider consumers using the verified
source APIs from the existing source-foundation lane. Original requirements and
normal qualification remain open.

Three more base16 syntax detectors now execute through the same bounded parse,
format-field scanner and native integer-value adapter. The focused target passed
(hex-detector-runtime.log), including qualified u8 paths, wider conversions,
actual template fields versus payload/escaped braces, named-function loop-state
reset, target scope and home-package boundaries. All-target Clippy passed
(hex-detector-clippy.log). Independent delta review found no concrete regression
and retains the written-name/type-resolution limit in the existing review.

Module homes/entry points now expand actual source descendants through the public
Source index and physical Owners mapping; distinct/variant resolution remains
strictly unit-based. Home packages come from actual source targets, avoiding an
empty units_of(module) lookup. All 21 Gate integration cases passed
(module-home-runtime.log), including reexported module classification and the
native detector's correct home-package exemption. All-target Census Clippy passed
(module-home-clippy.log). Independent source review confirmed these ownership
boundaries without asserting a new all-reaches package policy. Full Census,
contextual detectors, provider rules, CLI and capability completion remain open.

PurRDF issue 685 requests a reusable TOML authoring profile. Native RDF is the
policy core; no local TOML parser or substitute generic codec has been added.

Three job-context detectors now execute with job-local evidence and the shared
bounded native name-pattern matcher. All 23 Gate cases pass
(raw/context-detector-runtime.log), including empty strings, namespace refusal
beside valid literal initializers, job isolation and sole-name scope. All-target
Census Clippy passes with warnings denied (raw/context-detector-clippy.log).
Independent review confirmed the safe public initializer accessor and rejection
of every unavailable/empty namespace candidate. No normal-form hash changed.
Thirteen built-ins are locally qualified; std-default-hasher, patterns, reference,
provider, CLI and full capability acceptance remain unfinished.
