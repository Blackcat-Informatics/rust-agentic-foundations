<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Counted selection policy

Normal form 1 uses a body floor of 20 value tokens, a literal tier of 30, and
shim bounds of 6 inclusive to 30 exclusive. These are conservative selection
decisions, with the following counted evidence and explicit tradeoffs. The
[selected RDF rows](data/census-selection.ttl) retain each corpus distribution and
each policy used in these tables directly from the tool output.

The `raf-census` example `selection` measures existing production forms. It parses
each source corpus once through `Tree` and `Workspace`, deduplicates source items
across target reaches, and collects written function and method bodies. Parameters
and body ranges come from that parse. `Tower::literal_values` applies the same
production literal and held-value passes to the existing literal-tier form on
demand. Candidate groups compare the resulting token sequences directly; they do
not invent normal-form fingerprints. Trait-implementation and `Self::` shim
exclusions match production grouping. A fixture test relates the default policy
to production prints and pins 261 fixtures, 3,900 tokens, 22 groups and 54 pairs.

Each corpus is grouped separately; counts are summed across corpora, without
cross-package-root pairs. A duplicate group has at least two members and a group
with `n` members contributes `n*(n-1)/2` pairs. A selected body can be a singleton.
All thresholds count value-form tokens. The tool reads no clocks.

## Corpus

The fixed source corpus comprises the five RAF crate roots and six cached
dependencies at the exact versions below. All target reaches and test bodies are
included; constants and statics are excluded from this function-body experiment.

| Corpus | Written bodies | Value tokens |
| --- | ---: | ---: |
| raf-okf | 724 | 78,509 |
| raf-source | 1,779 | 213,241 |
| raf-workplan | 2,635 | 277,449 |
| cargo-raf-workplan | 370 | 47,906 |
| raf-census | 417 | 50,664 |
| blake3 1.8.7 | 437 | 45,917 |
| purrdf-lex 3.0.1 | 1,238 | 125,081 |
| serde 1.0.229 | 2 | 296 |
| syn 3.0.8 | 3,559 | 238,082 |
| toml 1.1.8+spec-1.1.0 | 690 | 23,964 |
| quote 1.0.47 | 141 | 4,313 |

The separate synthetic corpus has 261 bodies and 124 labelled pairs: 79 equal and
45 distinct under the documented normal form. These fixtures deliberately cover
the delivered boundaries. They are regression labels, not an independent sample
of real-source semantics. Macro-generated bodies are not expanded: serde's two
written bodies illustrate this substantial coverage limit. Targets are indexed
syntactically, including conditional alternatives, rather than executed.

## Body floor

With literal tier 30 and shim bounds 6–29, the source counts are:

| Body floor | Selected | Body groups | Shim groups | Pairs | Labelled equal selected |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 11,126 | 287 | 49 | 1,187 | 66 |
| 8 | 10,189 | 249 | 49 | 907 | 41 |
| 12 | 9,682 | 200 | 49 | 681 | 37 |
| 16 | 9,370 | 180 | 49 | 605 | 30 |
| 20 | 9,009 | 155 | 49 | 393 | 23 |
| 24 | 8,780 | 141 | 49 | 372 | 16 |
| 30 | 8,344 | 125 | 49 | 319 | 13 |
| 40 | 7,480 | 87 | 49 | 247 | 8 |

Twenty is a conservative knee: moving from 16 to 20 removes 212 source pairs;
moving from 20 to 24 removes only 21 while losing another seven labelled equal
pairs. Inspection of the excluded 16-token groups finds repeated trait plumbing:
toml's `ValueSerializer::serialize_bool`, integer and string methods all call
`self.dst.value(v)?; Ok(self.dst)` (13 members, 78 pairs); its `KeySerializer`
methods call `self.dst.key(v.to_string())?; Ok(())` (11 members, 55 pairs).
These two overload families alone account for 133 of the 212 removed pairs.
Other inspected groups include syn iterator `size_hint` implementations and
generated `Hash::hash` field traversals. Their repeated syntax is real; suppressing
small generic plumbing is a policy choice, not a claim that it is semantically
distinct. Some useful short duplicates are deliberately missed. The existing body
floor option lets a caller make that sensitivity choice explicitly.

## Literal tier

With floor 20 and shim bounds 6–29:

| Literal tier | Source groups | Source pairs | Labelled equal selected | Labelled distinct merged |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 211 | 404 | 23 | 3 |
| 16 | 211 | 404 | 23 | 3 |
| 24 | 210 | 401 | 23 | 3 |
| 30 | 204 | 393 | 23 | 0 |
| 40 | 193 | 359 | 18 | 0 |
| 64 | 175 | 310 | 17 | 0 |

Thirty is the lowest sampled tier without a labelled distinct merge. Forty loses
five labelled equal pairs without improving that labelled refusal count. Concrete
tier-24 source merges include workplan list-parser tests with different input
strings and syn literal-debug implementations with different field labels. These
illustrate how early abstraction erases distinguishing data; they are inspected
examples, not a statistical precision estimate. Literal abstraction above the tier
intentionally groups code that differs in values, including test inputs.

## Shim bounds

With floor 20 and tier 30:

| Inclusive minimum / exclusive maximum | Source groups | Source pairs | Labelled equal selected |
| --- | ---: | ---: | ---: |
| 3 / 30 | 207 | 407 | 28 |
| 6 / 30 | 204 | 393 | 23 |
| 8 / 30 | 203 | 392 | 20 |
| 6 / 20 | 206 | 400 | 25 |
| 6 / 40 | 204 | 393 | 23 |

Lowering the minimum to three adds trivial result constructors: syn has a
five-member `Ok(())` group, accounting for ten pairs. Raising it to eight removes
three labelled equal pairs. Six keeps fixed-argument forwarding examples while
excluding the smallest constructors. Lowering the maximum to 20 reclassifies
forwarders as body candidates: for example the `Self::ALL` `from_name` methods
merge after local renaming although their shim keys are excluded. Raising it to
40 gains no source pairs and loses 13 selected bodies; it primarily reclassifies
larger expression bodies such as workplan test refusal helpers. Thirty therefore
preserves the useful short-forwarding range without extending it merely to cover
more expressions. The minimum and maximum are fixed parts of the normal form.

## Reproduction and limits

Run the example from the checkout root with arguments of the form
`label=tree-directory#package-root`. Omit `#package-root` for a standalone cached
crate. A RAF package requires the checkout Tree root so inherited workspace
manifest values remain visible. For example:

```sh
cargo run --locked -p raf-census --example selection -- \
  'raf-okf=.#crates/raf-okf' 'raf-source=.#crates/raf-source' \
  'raf-workplan=.#crates/raf-workplan' \
  'cargo-raf-workplan=.#crates/cargo-raf-workplan' 'raf-census=.#crates/raf-census' \
  blake3-1.8.7=/exact/cache/blake3-1.8.7 \
  purrdf-lex-3.0.1=/exact/cache/purrdf-lex-3.0.1 \
  serde-1.0.229=/exact/cache/serde-1.0.229 \
  syn-3.0.8=/exact/cache/syn-3.0.8 \
  toml-1.1.8+spec-1.1.0=/exact/cache/toml-1.1.8+spec-1.1.0 \
  quote-1.0.47=/exact/cache/quote-1.0.47 > selection.ttl
```

The output is RDF 1.2 Turtle: corpus sizes, token distributions, every combination
of eight floors, six tiers, three shim minima and three shim maxima (432 policies
per corpus), labelled outcomes, and concrete default/neighbour disagreement
groups. Load failures become explicit `s:Refusal` records and a failing process
status; the tool never replaces a refused sample. Missing dependencies must be
provided at the stated versions, not silently substituted. Cargo's locked registry
checksums identify the cached source versions. Source edits change the counts;
these tables describe this input corpus, not an invariant of arbitrary revisions.

The corpus is convenience-selected Rust source, heavily weighted toward parsing
libraries and tests. In the threshold experiment, only synthetic pairs carry
equality labels. Pair counts
measure review volume and sensitivity, not real-source precision or recall. The
inspected examples explain the selected tradeoffs; they do not establish a
universal optimum. Counting does not measure execution cost or latency.

## Review of returned groups in two consumer repositories

At the source revisions below, seven of twelve reviewed groups repeat the same
operation on the same kind of data. Four capture intentional similarities. One
needs a decision about ownership before it can be called a duplicate helper.
The unit counted here is a group, even when it has several members.

| Source revision | Scanned units | Body groups | Shim groups | Groups reviewed |
| --- | ---: | ---: | ---: | ---: |
| Katamari `7e10cac6747a6e7b6470401573702af8a24dd61c` | 19,389 | 46 | 45 | 6 |
| PurRDF `51c3c1534a80f58e7c99746575b0a70e63bed113` | 78,821 | 367 | 132 | 6 |

To reproduce the reports, build `cargo-raf-census` from RAF revision
`194030497cc1696eb58e23d2caad73a5f3c7e48c` and run the following against
clean source trees of the named revisions. Run the binary from the RAF build.
The command reads the consumer as data.

```sh
cargo-raf-census --workspace "$katamari_root" baseline --format ntriples > katamari.nt
cargo-raf-census --workspace "$purrdf_root" baseline --format ntriples > purrdf.nt
```

The SHA-256 digests of the resulting reports are
`b2a4e676cddc0ac7a345633a9293482cb2eea38b2059739043fde07f43b4db37` and
`6973b6216a54fa934eee3c75b49f1fdde5e39de98761c3fc38ef3517d769a95d`,
respectively. Each records normal form 1, report form 5 and a body floor of 20.
Within each repository and group kind, sort by fingerprint and take the first
three. Read every member at its reported path and line, including its signature,
inputs and surrounding tests. The prefixes below identify those groups in the
reports. Shared constants are outside this sample.

| Source | Kind and fingerprint prefix | Judgment | Evidence from members |
| --- | --- | --- | --- |
| Katamari | body `133e47f6` | Intentional similarity | Two partition tests use the same assertions. One supplies the intervals in order and the other reverses them to exercise sorting. |
| Katamari | body `165e689f` | Genuine duplication | `label_commit` and `commit_label` open the same transaction, stage the same label record and commit it with the same failure handling. |
| Katamari | body `204cfb6f` | Inconclusive | Both `objects` helpers filter on a predicate. The image metadata test returns owned strings, while the projection test borrows typed RDF terms. Sharing the traversal would require deciding which ownership contract callers need. |
| Katamari | shim `03519e6f` | Intentional similarity | `SoftPinDevice::new` and `PluggableDevice::new` set the same initial presence flag and device, but construct distinct test doubles for PIN and machine custody. |
| Katamari | shim `0609fbec` | Intentional similarity | `listed` extracts staged byte paths. `everything` extracts graph node names. Both collect cloned map keys into a set, an idiom with distinct domain meanings here. |
| Katamari | shim `08e0dbdc` | Genuine duplication | The two `said` helpers render the same `ProviderError` and apply the same failure assertion. |
| PurRDF | body `006b49f2` | Genuine duplication | `envelope` and `encode_envelope` encode the same three COSE parts with the same CBOR tag. |
| PurRDF | body `00d1c923` | Intentional similarity | The shape tests distinguish two spellings of one expression kind from two different expression kinds. They expect the same ambiguity diagnostic. |
| PurRDF | body `01b72d7c` | Genuine duplication | Both `bnode` helpers construct the same blank-node term and initialize every optional field identically. |
| PurRDF | shim `03eb112b` | Genuine duplication | The JSON record and retrieval tests parse fixture text into the same value with the same failure message. |
| PurRDF | shim `0ceb5d91` | Genuine duplication | All four `exact` helpers resolve the same exact-arithmetic environment and assert the same precondition. |
| PurRDF | shim `0f46eb95` | Genuine duplication | All five geometry helpers construct an XY coordinate, converting both integer components to the same rational type. |

### Known copies selected from source

A separate search started from Rust files under `crates/*/src`, excluding test
files and test directories. It matched functions with the same name and a
single-line signature, looking for identical nonempty body lines after trimming
whitespace through a same-indent closing brace within 35 lines. From those cross-crate matches, manual inspection selected two substantive
examples and a short accessor in each repository. The six pairs were fixed before looking up their locations in the
group membership rows. A hit requires **both** source locations in one group.
Paths below are relative to the consumer's `crates/` directory.

| Source | Function | First location | Second location | Result |
| --- | --- | --- | --- | --- |
| Katamari | `check_generation` | `katamari-index-fjall/src/lib.rs:1474` | `katamari-index-redb/src/lib.rs:780` | Body `5d505095`. Both views reject a snapshot from a revoked deletion generation. |
| Katamari | `repository_root` | `katamari-cas/src/bin/katamari-chunk-fixtures.rs:61` | `katamari-domain/src/bin/katamari-act-fixtures.rs:218` | Shim `f3061e84`. Both fixture generators canonicalize the same relative manifest path. |
| Katamari | `authority_epoch` | `katamari-canonical/src/lib.rs:326` | `katamari-domain/src/commit.rs:188` | Miss. Both read the epoch field directly. |
| PurRDF | `find` | `datalog/src/plan.rs:1061` | `rdf-core/src/canonical_labelling/reference.rs:800` | Body `3ef5802d`. Both halve the path to the union-find representative. |
| PurRDF | `eat_kw` | `shex/src/parser.rs:225` | `sparql-algebra/src/parser.rs:1323` | Body `c84e19cd`. Both advance the parser by one token only on a keyword match. |
| PurRDF | `limit` | `cli/src/fingerprint.rs:26` | `lex/src/allocation.rs:112` | Miss. Both decode a stored little-endian ceiling as `u128`. |

Four of the six source pairs occur together in returned groups. The controls
favor short, same-name, text-identical functions in different crates. They do
not measure recall for renamed copies, embedded blocks or macro expansions.
All twelve fingerprint-selected groups happen to contain test support or tests.
The equal quota per group kind also gives shims more weight than their share of
the PurRDF report. Neither count estimates repository-wide precision or recall.
The original consumer baseline files are unavailable, so historical parity and
foreign-ledger migration remain unmeasured by this sample.
