---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:okf-profile
title: "OKF profile reader and writer"
description: "The raf-okf crate: parse OKF Markdown, validate the RAF profile with closed key sets, and write documents back deterministically."
state: implementing
bindings:
  code:
    - raf_okf::Document
    - raf_okf::Body
    - raf_okf::Profile
    - raf_okf::Index
    - raf_okf::is_capability_index
    - raf_okf::Span
    - raf_okf::RafDocument
    - raf_okf::EvidenceRecord
    - raf_okf::AssessmentRecord
    - raf_okf::CoverageRecord
    - raf_okf::Claim
    - raf_okf::EvidenceRecord::new
    - raf_okf::EvidenceRecord::to_document
    - raf_okf::AssessmentRecord::new
    - raf_okf::AssessmentRecord::to_document
    - raf_okf::CoverageRecord::new
    - raf_okf::CoverageRecord::to_document
    - raf_okf::Number::compare
    - raf_okf::Number::exceeds
  tests:
    - raf_okf::test_crate::profile::bindings_by_state_designed_and_considered_bind_nothing_delivered_binds_code_and_tests
    - raf_okf::test_crate::refusals::a_document_without_a_fence_on_line_one_is_refused
    - raf_okf::test_crate::refusals::a_frontmatter_that_is_not_a_block_mapping_is_refused
    - raf_okf::test_crate::refusals::a_billion_laughs_document_is_refused_at_the_first_anchor
    - raf_okf::test_crate::refusals::nesting_over_thirty_two_levels_is_refused_before_descent
    - raf_okf::test_crate::refusals::a_text_of_exactly_one_mebibyte_parses_and_one_more_byte_is_refused
    - raf_okf::test_crate::refusals::a_leading_byte_order_mark_is_refused_at_line_one_column_one
    - raf_okf::test_crate::refusals::a_nul_in_the_body_is_kept_and_renders_back
    - raf_okf::test_crate::refusals::a_frontmatter_fence_line_with_a_trailing_space_is_not_a_fence
    - raf_okf::test_crate::validate_cost::a_change_at_the_cap_with_maximal_affects_and_changes_validates_within_the_bound
    - raf_okf::test_crate::validate_cost::a_capability_at_the_cap_with_many_sections_and_blocks_validates_within_the_bound
    - raf_okf::test_crate::profile::every_required_key_of_every_type_is_reported_when_missing
    - raf_okf::test_crate::profile::an_unknown_key_is_reported_at_that_key
    - raf_okf::test_crate::profile::an_unknown_or_foreign_key_is_reported_on_every_type
    - raf_okf::test_crate::profile::a_nested_unknown_key_is_reported_in_bindings_and_in_a_change_item
    - raf_okf::test_crate::profile::the_key_resource_has_its_own_rule_on_every_type
    - raf_okf::test_crate::profile::an_unknown_type_is_one_violation_at_the_type_key
    - raf_okf::test_crate::profile::a_state_is_one_of_the_four
    - raf_okf::test_crate::profile::bindings_have_exactly_the_keys_code_tests_and_examples_each_a_list
    - raf_okf::test_crate::profile::affects_entries_have_the_form_a_section_of_the_seven_and_a_capability_in_changes
    - raf_okf::test_crate::profile::items_need_a_claim_known_keys_and_a_known_outcome
    - raf_okf::test_crate::profile::a_document_with_three_faults_returns_three_violations
    - raf_okf::test_crate::profile::a_split_link_is_reported_at_its_line_and_a_one_line_link_is_not
    - raf_okf::test_crate::profile::a_proposed_block_is_valid_only_inside_a_section_of_a_capability
    - raf_okf::test_crate::schema::each_type_lists_its_keys_in_the_order_of_the_architecture_page
    - raf_okf::test_crate::render::every_tracked_document_renders_to_its_own_bytes
    - raf_okf::test_crate::render::each_fixture_document_renders_to_its_own_bytes_and_validates
    - raf_okf::test_crate::render::a_parsed_document_equals_its_rendering_read_back
    - raf_okf::test_crate::render::keys_out_of_order_render_in_table_order_and_then_stay_put
    - raf_okf::test_crate::render::comments_attach_to_the_next_key_and_trailing_comments_stay_on_their_line
    - raf_okf::test_crate::render::tags_render_sorted_and_stay_sorted
    - raf_okf::test_crate::splice::two_generated_blocks_change_only_their_byte_ranges
    - raf_okf::test_crate::splice::overlapping_spans_are_refused_and_the_body_is_unchanged
    - raf_okf::test_crate::splice::a_malformed_result_is_refused_and_the_body_is_unchanged
    - raf_okf::test_crate::index::a_generated_only_index_is_accepted_with_either_header_form
    - raf_okf::test_crate::index::prose_outside_the_blocks_is_refused_with_its_line
    - raf_okf::test_crate::index::document_parse_of_index_text_fails_at_line_one
    - raf_okf::test_crate::index::only_the_capability_index_path_is_the_index
    - raf_okf::test_crate::markers::markers_inside_a_fence_produce_no_blocks
    - raf_okf::test_crate::markers::every_misspelling_of_the_proposed_marker_is_refused_with_its_line
    - raf_okf::test_crate::markers::each_generated_marker_error_names_its_line
    - raf_okf::test_crate::links::every_link_form_is_read_with_its_target_anchor_and_line
    - raf_okf::test_crate::links::a_link_split_across_lines_is_recorded_and_not_read
    - raf_okf::test_crate::links::a_destination_or_title_of_exactly_4096_bytes_is_read_and_one_more_is_refused
    - raf_okf::test_crate::links::targets_that_leave_the_root_or_are_empty_or_absolute_are_refused
    - raf_okf::test_crate::body_structure::block_nesting_of_sixty_four_levels_parses_and_sixty_five_is_refused
    - raf_okf::test_crate::body_structure::indented_code_is_ordinary_text_so_links_and_markers_in_it_count
    - raf_okf::test_crate::body_structure::a_fenced_block_exposes_its_content_and_whether_it_is_closed
    - raf_okf::test_crate::body_structure::a_fenced_block_prints_and_compares_without_the_body_around_it
    - raf_okf::test_crate::body_structure::only_atx_headings_are_headings_and_setext_headings_are_not
    - raf_okf::test_crate::body_structure::headings_of_every_level_carry_github_anchors_with_repeats_numbered
    - raf_okf::test_crate::alloc_bound::refused_documents_stay_within_eight_times_their_size
    - raf_okf::test_crate::placeholder_lines::reports_each_token_in_prose_with_its_line
    - raf_okf::test_crate::no_io_in_source::the_crate_sources_name_neither_the_file_system_nor_the_process_module
    - raf_okf::test_crate::usage_docs::the_examples_block_validates_between_fences_and_resource_is_rejected
    - raf_okf::test_crate::repo_markdown::no_tracked_markdown_file_holds_a_split_link_or_a_malformed_block
    - raf_okf::test_crate::schema::only_coverage_is_a_singleton_and_only_the_three_records_are_records
    - raf_okf::test_crate::records::a_valid_evidence_record_validates_to_its_typed_value
    - raf_okf::test_crate::records::a_value_over_its_limit_is_judged_on_its_text_and_not_its_nearest_float
    - raf_okf::test_crate::records::numbers_compare_exactly_by_sign_whole_part_and_fraction
    - raf_okf::test_crate::records::a_valid_assessment_validates_to_its_typed_value
    - raf_okf::test_crate::records::a_valid_coverage_record_validates_and_an_empty_scope_opts_out_visibly
    - raf_okf::test_crate::records::a_verdict_is_one_of_the_verdicts_of_its_type
    - raf_okf::test_crate::records::by_is_human_or_agent_and_never_a_name
    - raf_okf::test_crate::records::a_digest_is_sixty_four_lowercase_hex_digits
    - raf_okf::test_crate::records::a_revision_is_a_full_commit_id
    - raf_okf::test_crate::records::an_exclusion_names_an_item_or_a_module_prefix_below_a_crate_root
    - raf_okf::test_crate::records::an_exclusion_has_a_reason_with_no_placeholder
    - raf_okf::test_crate::records::a_list_that_holds_each_entry_once_refuses_a_repeat
    - raf_okf::test_crate::records::coverage_is_one_record_at_its_one_path
    - raf_okf::test_crate::records::a_record_sits_in_its_directory_and_its_id_matches_its_file
    - raf_okf::test_crate::records::a_record_has_no_title_description_or_body
    - raf_okf::test_crate::claims::a_claim_marker_names_its_evidence_and_supports_the_paragraph_above_it
    - raf_okf::test_crate::claims::a_claim_may_rest_on_several_records
    - raf_okf::test_crate::claims::a_claim_after_a_list_item_supports_that_item_only
    - raf_okf::test_crate::claims::markers_in_fences_code_spans_and_proposed_blocks_are_not_read
    - raf_okf::test_crate::claims::every_other_raf_comment_is_refused_with_its_line
    - raf_okf::test_crate::claims::a_malformed_claim_marker_is_refused_with_its_line
    - raf_okf::test_crate::claims::a_claim_marker_with_no_statement_directly_above_is_refused
    - raf_okf::test_crate::claims::claims_in_a_document_carry_whole_text_lines
    - raf_okf::test_crate::render::record_keys_out_of_order_render_in_table_order_and_validate
    - raf_okf::test_crate::record_writer::a_built_evidence_record_renders_to_a_record_that_reads_back_equal
    - raf_okf::test_crate::record_writer::a_built_assessment_and_coverage_record_read_back_equal
    - raf_okf::test_crate::record_writer::every_record_fixture_reads_back_equal_after_writing
    - raf_okf::test_crate::record_writer::the_keys_are_written_in_the_order_of_the_key_table
    - raf_okf::test_crate::record_writer::a_value_the_profile_refuses_is_written_and_refused_when_it_is_read_back
    - raf_okf::test_crate::claims::a_raf_generated_comment_in_another_case_or_spacing_is_text
    - raf_okf::test_crate::claims::a_raf_comment_split_over_lines_is_a_marker_error_and_other_split_comments_are_text
    - cargo_workplan::test_crate::check::a_raf_comment_split_over_lines_fails_check_and_other_split_comments_pass
    - raf_okf::test_crate::validate_cost::an_evidence_record_at_the_cap_with_maximal_values_and_limits_validates_within_the_bound
  examples:
    - raf_okf::EvidenceRecord::to_document
    - raf_okf::Claim
depends: []
---

# OKF profile reader and writer

## Purpose

`raf-okf` is the one reader and writer of RAF documents. Everything that touches a
capability, change or architecture page, the workplan gate and the repository
tooling alike, goes through it, so the profile is interpreted in a single place and
a document means the same thing to every tool.

## Contract

The facts of the profile, which are the document types, the closed key sets, the
value rules, the body rules and the syntax of the PROPOSED blocks, the generated
blocks and the claim markers, are stated
once, in [the RAF profile of OKF](../architecture/okf-profile.md). `raf-okf`
implements that page and adds no rule of its own. This document states only what the
crate does with it.

**Parse.** `Document::parse` takes the text of one file and returns the frontmatter
and a body model. It fails with a positioned error when the file has no frontmatter
fence on line 1, when the YAML does not parse, or when the YAML is not a mapping.
SPDX lines are YAML comments inside the frontmatter and are preserved. Comments are
kept with the key they belong to: a full-line comment or blank line attaches to the
key that follows it, a trailing ` # comment` stays on its line, and comments after
the last key stay last. The frontmatter accepts the subset of YAML stated in
[the RAF profile of OKF](../architecture/okf-profile.md#frontmatter-subset) and
refuses the rest. Numbers are read as strings.

**Resource bounds.** Documents are read from sources the reader may not control, so
`Document::parse` bounds its input and fails with a positioned error and never a
panic, abort or unbounded allocation:

- the text is at most 1 MiB;
- YAML nesting is at most 32 levels, and the check runs before any recursive
  descent can overflow the stack;
- YAML anchors, aliases and merge keys are refused, because the closed key schema
  has no use for them and an alias expansion can grow without bound;
- Markdown block nesting (quotes and lists) is at most 64 levels;
- a link destination or title is at most 4096 bytes, not counting the delimiters of a
  title or of an angle-bracket destination. This bounds the link scanner, which
  would otherwise rescan a long paragraph for every `](`. A longer part is a
  positioned error, never a link that is silently not read.

**Document types.** The profile has six types: the documents `Capability`, `Change`
and `Architecture`, which people write, and the records `Evidence`, `Assessment` and
`Coverage`, which state a result in their frontmatter. `DocType::ALL` lists them.

**Validate.** `Profile::validate` checks a parsed document and its path against the
closed key set of its `type` and returns a typed result: the typed document
(`RafDocument`, with `State`, `Bindings`, `Affects` entries, `ChangeItem` entries and
their `Outcome`), a record as `RafDocument::Evidence`, `RafDocument::Assessment` or
`RafDocument::Coverage`, holding an `EvidenceRecord`, an `AssessmentRecord` or a
`CoverageRecord` with a `RecordHeader`, since a record has no `title` and no
`description`, or the list of every violation, not only the first. Each violation
names the file, the rule and, for a frontmatter rule, the key; a body rule names the
section or the line instead. The rules are:

- the key set comes from one table per type, in the order of the architecture page. A
  key outside the table, a missing required key, and the base key `resource` (a rule of
  its own, which names the key set of the type) are violations. The optional keys
  `tags` and `timestamp` may be absent;
- a missing `type`, or one outside `DocType::ALL`, is the only violation reported for
  that document, because no key set applies;
- a document of a known type sits directly in the directory of its type;
- the `id` carries the prefix of its type and a kebab-case slug equal to the file name
  without `.md`; `title` and `description` are non-empty strings;
- `state` is an enumeration, and `bindings` is a map with exactly `code`, `tests` and
  `examples`, each a list. A `designed` or `considered` capability binds nothing, and a
  `delivered` one binds at least one `code` and one `tests` entry;
- `depends` and `changes` hold capability ids (`changes` at least one), `affects`
  entries are parsed into a capability id and one of the seven sections, and name a
  capability listed in `changes`; `contract_effect` is `none` or `changes`; `items`
  entries are parsed into a non-empty claim and an optional outcome
  (`implemented`, `dropped` or `moved:change:<slug>`). `Outcome` and `LinkTarget`
  are exhaustive enums, because the profile closes both sets: a consumer matches
  every variant with no fallback arm, and a new variant is a breaking change;
- `tags` is a list of non-empty strings with no duplicate, and `timestamp` is an RFC
  3339 date and time;
- a Capability body holds the seven sections in order, none empty; a Change body holds
  a non-empty `Summary`; an Architecture body is not empty; a record body is empty. A
  section is empty when nothing is left after its HTML comments and generated blocks
  are removed;
- a PROPOSED block appears only in a Capability and only inside a section;
- a claim marker appears only in a Capability: one in any other document is a
  violation;
- no prose line, title, description or item claim holds a placeholder token, and
  no link is split across lines. `Body::placeholder_lines` returns the
  placeholder tokens of any body, with their lines, by the same rule, so a caller
  that reads Markdown without frontmatter applies the rule `Profile::validate`
  applies; fenced code, code spans and HTML comments are not read.

The records follow the key table and value rules of the architecture page:

- evidence and an assessment share the trace keys `subject`, `inputs`, `verdict`
  and `by`. The subject is parsed into an `EvidenceSubject` (`command` or
  `binding:<path>`) or a `SectionRef`, each input row into an `Input` with its
  `InputKind` and a digest of 64 lowercase hexadecimal digits, the verdict into
  an `EvidenceVerdict` or an `AssessmentVerdict`, and `by` into an `Actor`, which
  is `human` or `agent` and never the name of a person;
- evidence names the capabilities it covers in `for`, takes `symbol` inputs
  only, and states `command` exactly when its subject is `command`; `revision`
  is a full commit id; `values` and `limits` hold `Number`s, each limit of
  evidence that passes names a value, and evidence that passes holds no value
  above its limit, compared exactly on the decimal texts (`Number::compare`);
- an assessment holds the `section` input of its subject;
- the coverage record is `coverage:default` at `work/coverage/default.md`. Its
  `scope` holds distinct crate package names, and each exclusion an
  `ExclusionItem` below a crate root with a non-empty reason; a crate root is
  refused;
- `DocType`, `InputKind`, `Actor`, `EvidenceVerdict`, `AssessmentVerdict`,
  `EvidenceSubject` and `ExclusionItem` are exhaustive enums, because the profile
  closes each set.

Checks that need other documents, such as whether an id exists or whether a graph is
acyclic, belong to [the document model](./workplan-document-model.md); `raf-okf`
validates one document at a time. Prose that talks about future work is not a
violation: the crate takes no position on tense.

**Body model.** The body is parsed into a structure that tools query instead of
re-scanning text. `Document::parse` builds the body of a document through
`Body::parse`, which also reads Markdown that has no frontmatter, such as a README: a
leading `---` fence closed by a later `---` line is skipped, and otherwise the whole
text is the body. Every span the model returns carries both line numbers and byte
offsets, in the coordinates of the whole text. Line ranges are 1-based and half-open,
as are byte ranges, so a span on line `n` alone has the lines `n..n+1`. The model
holds:

- headings of levels 1 to 6 outside fenced code, each with its GitHub anchor (repeats
  numbered `-1`, `-2`), and a check whether an anchor exists;
- second-level sections, each with its heading, its slug and a span that starts on the
  line after the heading and ends before the next second-level heading;
- links, each with its target, its anchor, its line and its form: inline link, image,
  reference definition, `href` attribute or `src` attribute. Links inside fenced
  code, code spans of any backtick length and HTML comments are not links. A target
  resolves against the path of the linking document to an external target, a
  fragment, or a path inside the repository, and a target that climbs above the root
  or is an absolute path is an error, never clamped. A link inside a block quote reads
  the same on every quoted line;
- split links: an inline link or image whose text or parentheses span a line break is
  not read as a link and is recorded with its line, so that a split link is a
  violation;
- fenced code blocks, matched by character and length and recognised inside block
  quotes and lists, so markers inside a fence are never read as markers. Each block
  has its span, its info string, its content (`FencedBlock::content`, the text between
  the fence lines as written) and whether a closing fence ends it
  (`FencedBlock::is_closed`); a block that is never closed runs to the end of the body;
- generated blocks, each with its kind, its outer span (markers included) and its
  content span (markers excluded);
- PROPOSED blocks, each with the change slug it names, its span, its marker line, its
  unquoted content and the section that holds it;
- prose lines: the lines outside fenced code and HTML comments with code spans
  removed, each with its section and whether it lies in a PROPOSED or generated block;
- claims: `Body::claims` returns each claim marker outside fenced code, code spans
  and PROPOSED blocks as a `Claim`: the evidence slugs it names, its line, its marker
  span, the span of the paragraph or list item it supports, and its section.

A malformed PROPOSED marker in any spelling, an unclosed generated block, a nested
generated block and an end marker without a start are parse errors with line numbers.
So are block quotes and list items nested over 64 levels and, outside fenced code,
code spans and PROPOSED blocks, a malformed claim marker, a claim marker with no
statement directly above it, and a `raf:` comment that is neither a `raf:generated:`
comment nor a claim marker, so a misspelt claim marker never reads as a plain comment.
A marker is one line: an HTML comment that `<!--` opens at the end of a line, outside
its code spans, and whose first non-blank text on the lines below is `raf:` in any
letter case and not `raf:generated:`, is the same error at the line of `<!--`,
`okf-parse-malformed-claim` for `raf:claim` and `okf-parse-unknown-marker` for
another word, so a claim marker split over lines never reads as a plain comment
either. A multi-line `raf:generated:` comment, an SPDX header written over several
lines and every other multi-line comment are plain comments. `Body::replace_spans`
applies non-overlapping byte-range edits in one pass and reads the model again; it
is all or nothing, and it fails with a typed error for a range outside the body, a
range inside a character, two ranges that overlap, or a result that no longer
parses. `Document` and `Index` delegate to it.

**Write.** `Document::render` writes a document back. The writer is deterministic
and round-trips: parsing the output yields an equal document, and rendering a
document that was produced by `render` returns identical bytes. Key order follows
the order of the profile table: the keys of the document's type come first in table
order, then any other keys in input order, and a document of an unknown type keeps
input order. The top-level `tags` list is put in sorted order when a document is
parsed or built, so it is written sorted and a document equals the reading of its own
rendering; every other list keeps author order. Comments travel with the key they are attached to, and the body is written
unchanged apart from the replaced spans that a caller supplies, such as a generated
block. Two documents are equal when their keys, compared as a map, have equal values
and equal attached comments, and their bodies hold the same bytes.

**Writing records.** `EvidenceRecord::new`, `AssessmentRecord::new` and
`CoverageRecord::new` build a record with its header (`RecordHeader::new`), from
`Input::new`, `SectionRef::new`, `Measure::new` and `Exclusion::new`; the
constructors judge nothing. `EvidenceRecord::to_document`,
`AssessmentRecord::to_document` and `CoverageRecord::to_document` write a record as
a document with its keys in key-table order, its lists and measures in record
order, numbers unquoted as written, an empty list as `[]`, absent optional keys left
out, and an empty body. `Profile::validate` reads the rendering back as an equal
record when the values satisfy the profile, and refuses it with the violations
otherwise; a value the frontmatter cannot hold is a `ConstructError`.
`EvidenceVerdict::name`, `AssessmentVerdict::name` and `EvidenceSubject::text` give
the values of the keys.

**Index handling.** `docs/capabilities/index.md` is the one navigation file in the
capability directory. `is_capability_index` is true for that path and false for any
other, including `docs/index.md`. `Index::parse` accepts a text only when apart from an
SPDX header comment (one line, or several between `<!--` and `-->` lines, each tag with
a value) every non-blank line lies inside a generated block and each marker starts its
own line with no indentation. Prose, an indented or mid-line marker, an empty header
tag, a header after a block, an unclosed block and a text with no block are errors with
a line. The text of an index never parses as a Capability document: `Document::parse`
fails on it at line 1. `Index::render` returns the text, and `Index::replace_spans`
replaces a block and refuses an edit that leaves anything but an index.

## Usage

```rust
use raf_okf::{Document, Profile, RafDocument};

const TEXT: &str = "\
---
type: Architecture
id: arch:example
title: \"Example\"
description: \"One line.\"
---

# Example

What this page says.
";

let document = Document::parse(TEXT).expect("the text is a document");
assert_eq!(document.render(), TEXT);
match Profile.validate(&document, "docs/architecture/example.md") {
    Ok(RafDocument::Architecture(page)) => assert_eq!(page.header.title, "Example"),
    Ok(other) => panic!("not an architecture page: {other:?}"),
    Err(violations) => panic!("{violations:?}"),
}
```

The text comes from the caller, so the block needs no file access. A caller that rewrites
a span of the body reads the span from the body model, for example
`document.body().generated_blocks()[0].content()`, then calls
`document.replace_spans(&[(span.clone(), new_text)])` and `document.render()`.

## Examples

A frontmatter block the profile accepts, for a capability with nothing built:

```yaml
type: Capability
id: cap:example
title: "Example"
description: "One line."
state: designed
bindings:
  code: []
  tests: []
  examples: []
depends: []
```

Between `---` fences, with the seven sections as its body, at the path
`docs/capabilities/example.md`, the block validates. The same block with
`resource: https://example.org/x` added is rejected with a violation at the key
`resource`, whose message names the `resource` key and the closed key set of
`Capability`.

## Constraints

- The crate reads and writes text only. It touches no file and runs no process, so
  a caller supplies the document text as strings.
- Input is bounded to 1 MiB, 32 YAML levels, 64 Markdown block levels and 4096 bytes
  per link destination or title, and YAML anchors and aliases are refused. A bound
  violation is a parse error.
- Only ATX headings (`#` marks) and fenced code blocks are modelled. A setext heading
  is not a heading, and indented code is read as ordinary text, so a marker or a link
  in it counts; write code that holds a marker or a link in a fence.
- Keys are exactly those of the profile. There is no pass-through for extra keys.
- A document is valid or invalid. There is no warning level.
- The writer preserves comments in the frontmatter but does not reflow the body.
- The crate takes no position on tense or content. Checks on prose belong to the
  reconcile operation.

## Explanation

The profile is deliberately narrower than its parent format. A base that requires one
key and leaves the rest open suits a wide ecosystem; a toolchain that must answer
"is this document complete" needs the opposite, so the key sets are closed and every
key is required, except the two base keys `tags` and `timestamp`, which carry no
fact the toolchain depends on and are optional. A missing required key then means a
missing fact rather than an optional one, and a misspelled key is an error rather
than a silent no-op.

The body model exists so that no other crate scans Markdown. A line that begins with
the PROPOSED marker inside a code fence is documentation of the syntax, not a
proposal, and only a parser that tracks fences can tell the difference. Putting that
rule in one crate keeps the gate, the generator and the reconcile operation agreeing.

The writer is deterministic because generated blocks are rewritten in files that
people also edit. If rendering could reorder keys or alter whitespace, every sync
would produce noise in the diff, and a clean check could not be told from a stale one.

## Verification

- A parse fixture per failure: no frontmatter, malformed YAML, a non-mapping
  frontmatter.
- A validation fixture per type for each missing key, each extra key, the key
  `resource`, and an unknown `type`; each asserts the named key and rule.
- A value fixture per rule: `state` outside its set, `bindings` missing a key,
  an `affects` entry with an unknown section, an `items` outcome outside
  `implemented`, `dropped` and `moved:change:<slug>`.
- Bound fixtures: a document of 1 MiB plus one byte, frontmatter nested 33 levels
  deep, a quote nested 65 levels deep, and a link destination or title of 4097 bytes
  each fail with a positioned error, and documents at the limit (a destination or
  title of 4096 bytes included) parse.
- An alias fixture: frontmatter with an anchor and an alias (a "billion laughs"
  document) fails fast with a positioned error naming the anchor, and the process
  allocates no more than the input size times a small constant.
- A body fixture where a PROPOSED marker and a generated marker appear only inside a
  fence and produce no blocks, and fixtures for each marker error.
- Refusal fixtures for a leading byte order mark (no fence at line 1, column 1) and
  for a frontmatter fence line with a trailing space; a NUL in the body is kept and
  renders back.
- A cost fixture: a change at the 1 MiB cap with the most `changes` and `affects`
  entries it holds, and a capability with the most sections and generated blocks,
  validate within 100 ms in a release build.
- A round-trip property over the fixture documents: `render(parse(x))` equals `x`
  for canonical input, and `parse(render(d))` equals `d`.
- An index fixture: a generated-block index is accepted and a hand-written one is
  rejected.
- A record fixture per rule, each beside an accepted neighbour: a verdict outside
  the set of each type, a `by` that names a person, a malformed digest, a short
  commit id, an empty exclusion reason, a crate-root exclusion, a repeated scope
  entry, a record in another directory or under another file name, and a
  coverage record under another slug.
- Claim fixtures: a claim with one record and with several, after a paragraph
  and after a list item; markers inside a fence, a code span and a PROPOSED block
  are not read; an unknown `raf:` comment is refused; a `raf:generated:` comment
  keeps its own rules; each malformed and each unattached marker is refused with
  its line.
- The record fixtures render to their own bytes and validate, and a record with
  its keys out of order renders them in table order.
- A record of each type built with the constructors, and each record fixture,
  written by `to_document` reads back as an equal record with its keys in table
  order; empty lists and absent optional keys are written as the profile reads
  them; a value the profile refuses is refused on reading back, and a value the
  frontmatter cannot hold by the writer.
- A `raf:generated:` comment in another letter case or spacing is text, beside
  an unknown `raf:` comment that is refused.
- Split-marker fixtures of the body model: `<!--`, `raf:claim evidence=x` and `-->`
  on three lines, the same with a blank line and `  RAF:Claim` below `<!--`, and a
  comment opened after a statement on its line are each a malformed claim marker at
  the line of `<!--`, and `<!--`, `raf:foo`, `-->` is an unknown marker; beside them
  a split `raf:generated:` comment in two spellings, an SPDX header over several
  lines, an ordinary multi-line comment, an empty one and one opened in a code span
  are plain, and nothing is read in fenced code or a PROPOSED block. Through
  `check`, the three-line claim marker and the one with a blank line in a
  capability are `okf-parse-malformed-claim` and the `raf:foo` comment
  `okf-parse-unknown-marker`, each exit 1 with nothing written, and the split
  `raf:generated:` comment, the SPDX header and the ordinary comment exit 0.
- Cost fixture: an evidence record at the 1 MiB cap with the most `values` and
  `limits` validates within 100 ms in a release build.
- The documentation examples of `EvidenceRecord::to_document` (a record built,
  written, parsed and validated back to an equal record) and of `Claim` (a claim
  marker read with its evidence, line, section and statement) run as the bound
  examples.
