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
| cargo-workplan | 370 | 47,906 |
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
  'cargo-workplan=.#crates/cargo-workplan' 'raf-census=.#crates/raf-census' \
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
libraries and tests. Only synthetic pairs carry equality labels. Pair counts
measure review volume and sensitivity, not real-source precision or recall. The
inspected examples explain the selected tradeoffs; they do not establish a
universal optimum. Counting does not measure execution cost or latency.
