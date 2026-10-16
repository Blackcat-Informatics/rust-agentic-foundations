---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-current-rationale
title: "Current rationale"
description: "Shared architecture pages that state resolved decisions, and the conditions to revisit them."
state: implementing
bindings:
  code:
    - raf_workplan::rationale
  tests:
    - raf_workplan::test_crate::rationale::a_capability_link_to_a_missing_page_or_heading_fails_the_link_check
    - raf_workplan::test_crate::rationale::a_decision_page_linked_from_a_capability_with_its_conditions_passes
    - raf_workplan::test_crate::rationale::a_decision_page_without_conditions_to_revisit_is_reported_by_name
    - raf_workplan::test_crate::rationale::a_hundred_thousand_headings_and_an_unterminated_fence_are_judged_in_bounded_time
    - raf_workplan::test_crate::rationale::a_link_from_a_capability_a_change_or_another_page_links_the_page
    - raf_workplan::test_crate::rationale::an_empty_conditions_list_is_reported_and_a_list_item_with_text_passes
    - raf_workplan::test_crate::rationale::an_explanation_decision_without_conditions_is_reported_and_with_them_passes
    - raf_workplan::test_crate::rationale::an_unlinked_page_is_reported_by_name_and_a_link_to_itself_does_not_count
    - raf_workplan::test_crate::rationale::headings_and_list_items_in_fenced_code_or_proposed_blocks_do_not_count
    - raf_workplan::test_crate::rationale::headings_match_exactly_at_their_level
    - raf_workplan::test_crate::rationale::no_page_is_reported_unlinked_while_a_document_that_may_link_it_does_not_load
    - raf_workplan::test_crate::rationale::the_findings_are_the_same_on_every_run_in_file_and_line_order
    - raf_workplan::test_crate::rationale::the_valid_fixture_links_its_page_from_gamma
    - raf_workplan::test_crate::rationale::this_repository_links_each_architecture_page_from_a_capability
    - raf_workplan::rationale::tests::a_line_without_a_marker_or_without_text_is_no_list_item
    - raf_workplan::rationale::tests::a_list_item_is_a_marker_a_space_and_text_with_a_letter_or_a_digit
  examples:
    - raf_workplan::rationale
depends: [cap:workplan-document-model, cap:workplan-entity-history]
---

# Current rationale

## Purpose

Removing archaeology from the documentation must not remove the reasons that still
matter. This capability defines where current rationale lives: in the Explanation of a
capability for a decision it owns, and in an architecture page for a decision that
several capabilities share. It also defines the conditions to revisit, which say what
would justify reopening a decision.

## Contract

**Current rationale and historical deliberation.** The documents keep one and Git
keeps the other.

| Current documentation retains | Git retains |
| --- | --- |
| Why the active design exists. | The sequence of proposals that preceded it. |
| Constraints that still govern it. | Constraints that were removed. |
| Alternatives worth knowing about today. | Every alternative ever discussed. |
| Conditions that would justify revisiting it. | Previous assessments of those conditions. |

**Architecture pages.** A decision shared by several capabilities has a page in
`docs/architecture`. The page states the resolved current decision, as the
[normal form](../architecture/normal-form.md) page does, and each capability that
depends on the decision links to it.

**Conditions to revisit.** A recorded decision carries a list of conditions to
revisit, under the headings the decision pages paragraph below names. Each condition
is a statement that, if it became true, would justify reopening the decision. An
architecture page that records no decision needs no such list.

**Decision pages.** A decision page is an architecture page with a `## Decision`
heading. It has a `## Conditions to revisit` heading, and the list under that heading
holds at least one item. A capability records a decision it owns in its Explanation,
under a `### Decision` subheading. A `### Conditions to revisit` subheading with at
least one list item follows it, inside the Explanation and before the next
`### Decision`.

**Headings and lists.** The headings are ATX headings: up to three spaces, one to six
`#` marks, then a space or a tab and the text. A setext heading, text underlined with
`=` or `-`, is not a heading of these rules, so a decision or a conditions heading
written that way does not count. A heading counts when its text after the `#` marks,
with the surrounding spaces and a closing run of `#` marks removed, is exactly
`Decision` or `Conditions to revisit`, case included, at the level given here. A
heading in fenced code does not count, and an unterminated fence runs to the end of
the document. A heading in a PROPOSED block does
not count until the block is resolved. The list of a conditions heading runs to the
next heading of the same or a higher level, or to the end of the section that holds
it. An item counts when it starts with `-`, `*` or `+`, or with a number followed by
`.` or `)`, then a space or a tab, and its text outside code spans and HTML comments
holds a letter or a digit. A line in a block quote is not an item, and neither is a line in a
PROPOSED block.

**Conditions are prose.** A condition is plain prose. No check reads what a condition
says or decides whether it has been met. A person judges that, with the
[history command](./workplan-entity-history.md) on the page or the capability.

**Checks.** `cargo workplan check` runs these rules over the loaded documents:

| Finding | Reported when |
| --- | --- |
| `architecture-page-unlinked` | No capability, change or other architecture page links to an architecture page. A link from the page to itself does not count. No page is reported while a capability, change or architecture document is present and fails to load, because that document may hold the link and its own findings already say what is wrong. |
| `decision-conditions-missing` | A decision page has no `## Conditions to revisit` heading (reported at its `## Decision`), or an architecture page has a `## Conditions to revisit` heading with no item (reported at that heading). |
| `explanation-conditions-missing` | A `### Decision` in an Explanation has no `### Conditions to revisit` after it, before the next decision and inside the Explanation (reported at the decision), or such a subheading has no item (reported at that subheading). |

A link counts when it is written anywhere in the body of the document, before its
first section or in any section, outside fenced code, code spans and comments, and
resolves to the page, with or without an anchor. A link from a capability to a missing
page is `link-broken`, and a link to a missing heading of a page is `anchor-broken`. A
broken link links nothing. Each finding names the page or the capability and gives
the fix. The rules read each heading and each line of a document once, and the
findings come in the order of the documents' paths and lines.

## Usage

- A capability author who makes a decision shared with another capability writes the
  decision once in an architecture page and links to it from each Explanation.
- A reader who wants the path to a decision runs the
  [history command](./workplan-entity-history.md) on the page.
- `cargo workplan check` reports unlinked pages, decision pages without conditions to
  revisit, and Explanation decisions without them.
- A capability author who records a decision the capability owns writes it under
  `### Decision` in the Explanation and ends it with `### Conditions to revisit`.

## Examples

A rationale sentence in the current tense, kept with the contract it explains:

> The parser reports incomplete input at stream completion because an incomplete
> token may become valid when the next chunk arrives.

and the matching conditions to revisit at the end of the decision:

- The parser gains a mode in which incomplete tokens are never valid.
- Input arrives with explicit token boundaries.

In a capability Explanation the rationale sentence sits under a `### Decision`
subheading. The two conditions follow it as the list under a
`### Conditions to revisit` subheading. A decision page uses the same two headings at
the second level, `## Decision` and `## Conditions to revisit`. If the conditions
list is empty, `cargo workplan check` reports the conditions heading.

## Constraints

- A decision an architecture page records is resolved. The page does not narrate
  how the decision was reached.
- Rationale keeps only constraints that still govern. A constraint that was removed
  leaves the page.
- Rationale sits next to the contract it explains. It is not collected in a separate
  history document.
- Conditions to revisit are plain prose. No check decides whether a condition has
  been met; a person judges it with the history command.
- The headings are matched by their exact text and level. A heading with other
  wording or at another level is not a decision or a conditions heading.

## Explanation

Eliminating archaeology must not erase useful rationale. The current documentation
explains significant decisions where that helps someone maintain the system, because an
explanation belongs with the current contract. The full deliberation, with earlier
proposals, discarded names, intermediate designs and reversals, belongs in Git.

The split follows who needs what. A maintainer needs to know why the active design
exists, which constraints still govern it, which alternatives are worth knowing about
and what would justify revisiting it. A historian needs the sequence. For a decision
that several capabilities share, one architecture page states the resolved decision and
those capabilities reference it, so the decision has a single current statement. The
entity-aware [history command](./workplan-entity-history.md) lets a reader ask how a
capability or a decision evolved without requiring every other reader to carry that
history in their working context.

## Verification

- A fixture with a decision page linked from a capability, with its conditions,
  passes.
- A fixture page that no document links to is reported by name. A link from the page
  to itself does not count. A link from a capability, a change or another page does.
- A fixture decision page without conditions to revisit is reported by name, and
  so is an empty conditions list. A list item with text passes.
- A capability Explanation with a `### Decision` and no `### Conditions to revisit`
  is reported, and with them it passes.
- Headings and list items in fenced code or in PROPOSED blocks do not count, and an
  unterminated fence runs to the end of the document.
- A capability link to a missing page, or to a missing heading, fails the link check.
- The findings are the same on every run and come in the order of files and lines.
  A page with a hundred thousand headings is judged in bounded time.
- Headings match exactly at their level; no page is reported unlinked while a
  document that may link it does not load; a list item is a marker, a space or a tab
  and text with a letter or a digit, and a line without a marker or without text is none.
- This repository links each architecture page from a capability, and the valid
  fixture links its page from a capability.
- The documentation example of the rules runs: a decision page whose conditions
  list is empty is `decision-conditions-missing`, and with one condition the
  repository checks clean.
