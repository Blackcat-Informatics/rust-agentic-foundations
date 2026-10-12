---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Capability
id: cap:workplan-current-rationale
title: "Current rationale"
description: "Shared architecture pages that state resolved decisions, and the conditions to revisit them."
state: considered
bindings:
  code: []
  tests: []
  examples: []
depends: [cap:workplan-document-model, cap:workplan-entity-history]
---

# Current rationale

## Purpose

Removing archaeology from the documentation must not remove the reasons that still
matter. This capability defines where current rationale lives: in the Explanation of a
capability for a decision it owns, and in an architecture page for a decision that
several capabilities share. It also defines the conditions to revisit, which say what
would justify reopening a decision.

Open questions:

- Whether a conditions-to-revisit list is a required heading on every architecture
  page or only on pages that record a decision.
- Whether a capability links to an architecture page from its Explanation only, or from
  any section.
- How a check recognises a condition that has been met.

## Contract

> [!PROPOSED change:workplan-current-rationale]
> **Current rationale and historical deliberation.** The documents keep one and Git
> keeps the other.
>
> | Current documentation retains | Git retains |
> | --- | --- |
> | Why the active design exists. | The sequence of proposals that preceded it. |
> | Constraints that still govern it. | Constraints that were removed. |
> | Alternatives worth knowing about today. | Every alternative ever discussed. |
> | Conditions that would justify revisiting it. | Previous assessments of those conditions. |
>
> **Architecture pages.** A decision shared by several capabilities has a page in
> `docs/architecture`. The page states the resolved current decision, as the
> [normal form](../architecture/normal-form.md) page does, and each capability that
> depends on the decision links to it.
>
> **Conditions to revisit.** A page or an Explanation that records a significant
> decision ends it with a list of conditions to revisit. Each condition is a statement
> that, if it became true, would justify reopening the decision.
>
> **Checks.** Links from a capability to an architecture page resolve. An architecture
> page that no document links to is reported. A page that records a decision and has no
> conditions to revisit is reported.

## Usage

> [!PROPOSED change:workplan-current-rationale]
> - A capability author who makes a decision shared with another capability writes the
>   decision once in an architecture page and links to it from each Explanation.
> - A reader who wants the path to a decision runs the
>   [history command](./workplan-entity-history.md) on the page.
> - `cargo workplan check` reports unlinked pages and decision pages without
>   conditions to revisit.

## Examples

> [!PROPOSED change:workplan-current-rationale]
> A rationale sentence in the current tense, kept with the contract it explains:
>
> > The parser reports incomplete input at stream completion because an incomplete
> > token may become valid when the next chunk arrives.
>
> and the matching conditions to revisit at the end of the decision:
>
> - The parser gains a mode in which incomplete tokens are never valid.
> - Input arrives with explicit token boundaries.

## Constraints

> [!PROPOSED change:workplan-current-rationale]
> - An architecture page states a resolved decision. It does not narrate how the
>   decision was reached.
> - Rationale keeps only constraints that still govern. A constraint that was removed
>   leaves the page.
> - Rationale sits next to the contract it explains. It is not collected in a separate
>   history document.

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

> [!PROPOSED change:workplan-current-rationale]
> - A fixture with an architecture page linked from a capability passes.
> - A fixture page that no document links to is reported by name.
> - A fixture decision page without conditions to revisit is reported by name.
> - A capability link to a missing page, or to a missing heading, fails the link check.
