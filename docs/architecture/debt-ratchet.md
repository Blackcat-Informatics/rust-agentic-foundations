---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:debt-ratchet
title: "The debt ratchet"
description: "The one rule by which a RAF tool records the findings a repository carries on adoption and lets that record only shrink."
---

# The debt ratchet

A repository that adopts a RAF gate usually already holds what the gate refuses:
duplicated helpers for the census, unexplained items and claims without a test for
the strict mode of the workplan. A debt ratchet records that present state so the
gate passes at once, then fails on anything new and forces the record down as the
debt is paid. Two tools carry one: [the census debt ratchet](../capabilities/census-debt-ratchet.md)
and the debt of [the workplan's strict mode](../capabilities/workplan-strict.md). This
page states the rule they share, once; each capability states its own keys. The OKF
shape of this page is stated in [the profile](./okf-profile.md).

## Decision

A debt record is owned by one tool and holds only that tool's debt. Its entries are
keyed, and the record always equals the debt the repository carries now:

- a present entry that the record does not hold is growth, and growth fails the
  check;
- a recorded entry that is not present, or a count below its record, fails the check
  too, until the write lowers the record, so the record never carries slack;
- the write takes the present debt. It refuses to record growth into an existing
  record, and it refuses while the repository has a finding that cannot be debt;
- the write records growth in exactly two cases: it creates a missing record, which
  is how a repository adopts the ratchet, and it writes over a record made for
  another version of what the keys mean. A tool whose keys carry no version, as the
  workplan's do not, has the first case alone;
- the write has two entry points that follow one rule, a `debt --write` command and
  `Gate::write_debt`, which returns a `DebtWrite`, and it is never called from a
  test;
- equal debt gives byte-equal records: the entries in a stable order and the text
  ending with one line break;
- the control is review. Every creation of a record, and every write over one made
  for another version, is read in its diff as a statement of the debt the repository
  accepts.

What can be debt is closed by the tool: a finding that the tool's default rules
raise is never debt, so a record can only hold what the ratchet was adopted for.

## Shared and separate

The two tools share the rule above and the names of its entry points. They share no
file and no code. Each tool is adopted on its own as a dev-dependency, so a shared
file would make each read and preserve the other's schema, and a version change of
one tool's keys, such as a new census normal form, would invalidate the other's debt.
The census records its debt in `census-debt.json` beside its ledger; the workplan
records its strict debt in the OKF record `work/debt/default.md`, beside the authored
coverage record. Each record keeps its tool's keys: census groups and job copy counts,
and workplan findings by kind and subject with orphan line counts by item.

## Conditions to revisit

- Both ratchets are built, and their compare and write code is the same apart from
  the keys, so one shared implementation would be read by two real consumers.
- A repository needs one record that both tools write, such as a single review of all
  adopted debt.
- A tool needs a debt that may grow without a review of its diff, which the rule
  refuses.
