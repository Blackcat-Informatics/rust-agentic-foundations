**Those should become two hard requirements: the current design must be directly readable, and completed work must leave usable documentation.**

Your second point changes where I would put the centre of this system. I would revise my earlier separation between `work/features/` and generated feature documentation: **the persistent feature or capability should have its home in the actual documentation. Planning describes how that capability will change.**

The organizing principle could be:

> **Plan changes to the documented system. Implement those changes. Reconcile the documentation with the result. Let Git retain the previous states.**

That addresses both the archaeology problem and the gap between design and documentation.

## 1. A reader should never have to apply amendments mentally

This is the failure we should explicitly design out:

> “The parser behaves as described in ADR-0012, except for the changes in ADR-0028, with the error-handling amendment in PRD-0041, although the streaming section was superseded last month.”

The current documentation should instead state the complete contract that applies:

> “The parser accepts input incrementally, preserves incomplete tokens between calls, and reports incomplete input when the caller finishes the stream. Recovery from malformed input is unsupported.”

The historical sequence remains available through Git. Reading it becomes a deliberate investigation into how the design evolved.

**Every accepted change must be incorporated into the current account of the system.** An amendment can be an input to that operation, but it should not become a permanent instruction that every future reader must remember to apply.

Normal links between current topics remain useful. A parser page can link to the current error model. The important distinction is that the reader is following the structure of the system, rather than reconstructing its chronology.

### This has a consequence for the CLI

A command that accepts a change should identify and update the affected current statements. It should not merely append “supersedes X” and consider the job finished.

That gives us a concrete operation:

**Reconcile this change into the current model and documentation.**

The reconciliation might update a contract, remove an obsolete constraint, revise an example, change a code binding, and invalidate some evidence. The CLI should understand those as consequences of the same change.

## 2. Give each capability a permanent documentation home

For example:

```text
docs/capabilities/streaming-parser.md
```

That could be an OKF document with a stable capability identity and ordinary, readable Markdown.

It persists through the life of the capability:

| While the work is… | The same capability record contains… |
|---|---|
| Being considered | Purpose, desired behaviour, constraints, open questions, proposed criteria. |
| Being designed | The candidate contract, selected approach, rationale, planned examples. |
| Being implemented | The evolving contract, actual code bindings, test bindings, remaining gaps. |
| Delivered | The supported contract, usage, examples, limitations, current rationale, verification links. |
| Being revised | Its current account plus an explicitly identified proposed change. |

The document has a continuing identity. The work items that create or change it are temporary execution concerns.

This also makes the roadmap simpler:

- A planned capability appears as upcoming work.
- An implemented capability with unresolved verification appears with that gap.
- A delivered capability appears in the documentation catalogue.
- Later improvements are changes to that capability.

There is no need for a completed PRD to remain beside a separate, partially updated description of what was built.

For a change developed on a branch, the candidate documentation can evolve beside the candidate code. The branch diff then shows both what the implementation changes and what users or maintainers will be told about it.

## 3. Design should begin in the shape of the eventual documentation

This is probably the most important practical change.

If the initial design is written entirely as a proposal to an approval committee, turning it into useful documentation requires another writing project. That second project is easy to postpone.

Instead, planning should start drafting the material the finished capability will need:

- **Purpose:** what problem it solves and where it fits.
- **Contract:** what behaviour callers can rely on.
- **Usage:** how to use it.
- **Examples:** representative inputs, outputs, and workflows.
- **Constraints:** limits, prerequisites, and unsupported cases.
- **Explanation:** why it works this way and which tradeoffs matter.
- **Verification:** which checks substantiate which claims.

Early versions can clearly mark uncertainty and intended behaviour. As implementation proceeds, the same material gets resolved and corrected.

For the streaming-parser example:

| Design material | What it should become |
|---|---|
| “We need to support input arriving in arbitrary chunks.” | A documented behavioural guarantee and linked tests. |
| “Should incomplete input return an error immediately?” | An explicit lifecycle/error contract and a concise explanation of the chosen behaviour. |
| “Maybe expose `push()` and `finish()`.” | Actual API references and a compiling usage example. |
| “Recovery might be possible.” | A resolved statement of supported recovery behaviour, or a documented limitation and separately tracked future work. |
| “Add tests for truncation.” | Named test bindings and observed verification evidence. |

**Promotion to documentation requires reconciliation, not just a status change.**

The implementation may have changed the API. An intended guarantee may have proved impractical. An initially attractive option may have been dropped. Those outcomes need to be reflected in the current text before the work is considered complete.

## 4. Make documentation an explicit part of completing a change

A work item should identify the documentation it creates or changes when it is planned.

Something conceptually like:

```text
Change: Add incremental parsing

Changes capability:
  Streaming parser

Documentation affected:
  Behaviour contract
  Usage example
  Error-handling explanation
  Verification matrix
```

That is much stronger than a final checkbox saying “update docs.”

The CLI could provide operations along these lines:

```sh
cargo workplan change impact CHANGE-17
cargo workplan change reconcile CHANGE-17
cargo workplan docs preview CHANGE-17
cargo workplan verify CHANGE-17
cargo workplan change complete CHANGE-17
```

The exact grammar can come later. The important behaviour is that `complete` checks whether the change has been incorporated into the current documentation.

Depending on the change, that could require:

- affected contracts updated;
- proposed behaviour resolved into implemented, deferred, or dropped outcomes;
- code and test references resolving;
- relevant examples compiling or running;
- obsolete pending-change material removed;
- documentation assessments current;
- generated indices and status views synchronized.

Small internal changes should be able to record that they have no effect on the documented contract. The tool should apply obligations appropriate to the change, rather than requiring a new tutorial for every refactor.

### Mechanical generation and explanatory writing have different jobs

The crate can deterministically produce reference sections, relationship tables, status summaries, links, indices, and verification matrices.

Explanations and instructional prose need authorship and assessment. Humans and agents can do that work using the same structured context, and the crate can record what they assessed.

I would make the reconciliation command produce a precise list of documentation obligations and the evidence needed to resolve them. An agent then has a concrete writing task:

> “The final API differs from the proposed API in these two places. Update the usage example and lifecycle explanation. The documented memory bound is still unsupported by evidence.”

That makes documentation work specific and reviewable.

## 5. Preserve the reasons that still matter

Eliminating archaeology should not erase useful rationale.

The current documentation should explain significant decisions where that helps someone maintain the system:

> “The parser reports incomplete input at stream completion because an incomplete token may become valid when the next chunk arrives.”

That explanation belongs with the current contract.

The full deliberation—earlier proposals, discarded names, intermediate designs, reversals—belongs in history.

I would make this a distinction between **current rationale** and **historical deliberation**:

| Current documentation should retain | Git can retain |
|---|---|
| Why the active design exists. | The sequence of proposals that preceded it. |
| Constraints that still govern it. | Constraints that were removed. |
| Alternatives worth knowing about today. | Every alternative ever discussed. |
| Conditions that would justify revisiting it. | Previous assessments of those conditions. |

For a decision shared by several capabilities, it can have a current architecture page that those capabilities reference. That page should itself express the resolved current decision.

This is where an entity-aware history command becomes valuable: someone can ask how a capability or decision evolved without requiring everyone else to carry that history in their working context.

## 6. Use the graph to keep intention, implementation, and documentation aligned

We should preserve the distinctions from the earlier discussion:

- a design has been accepted;
- code implementing it exists;
- tests address its criteria;
- executions provide evidence;
- the documentation describes the supported result.

Those facts often become true at different times.

A planned feature should remain visibly planned until its implementation and documentation justify a stronger description. Likewise, an accepted design should not automatically turn future-tense prose into a statement about available behaviour.

For a given repository revision and supported configuration, the tool should be able to produce **one complete current account**, with unresolved conflicts or gaps explicitly identified.

The graph then gives us checks in both directions:

| Direction | Useful question |
|---|---|
| Documentation → code | Where is this documented capability implemented? |
| Code → documentation | Where is this supported behaviour explained? |
| Documentation → tests | Which tests address this guarantee? |
| Tests → documentation | Which documented guarantees does this test support? |
| Change → documentation | Which explanations, examples, and contracts must change? |
| Evidence → claims | Which current statements depend on this result? |

GMEOW’s current [Grounding Contract](https://github.com/Blackcat-Informatics/gmeow-ontology/blob/a631f73e6e39b4493d295bcd83b4deec80637f54/docs/GROUNDING.md) already describes a similar two-way duty: design promises need an accounted-for realization, and functionality that has been built needs the documentation to follow it. This crate could make that discipline available to an ordinary Rust repository.

A particularly useful consequence is that **documentation maintenance becomes work the system can identify**. If a referenced API disappears, an example stops compiling, or a criterion changes, the affected documentation gets an explicit obligation.

## 7. Turn your frustrations into the crate’s own acceptance tests

I would give the crate two flagship scenarios.

### Scenario A: several redesigns, one readable present

A fixture repository changes the same capability several times. Decisions are amended and reversed; interfaces and constraints evolve.

At the end:

- the current documentation directly states the active contract;
- its links resolve to current explanations;
- obsolete proposals are absent from the current reading path;
- the relevant rationale remains;
- each previous state can be recovered from Git.

The test is whether a reader needs to reconstruct the changes to understand what applies now.

### Scenario B: planning produces actual docs

A feature begins as an intended capability with open questions. Implementation changes some of the original choices.

The completion workflow must leave:

- an accurate capability explanation;
- an actual usage example;
- current API references;
- explicit limitations;
- feature-to-test traceability;
- no unresolved proposal text presented as established behaviour.

A document merely existing would not satisfy that scenario.

**This gives the crate a stronger purpose: every completed change should improve the repository’s current explanation of itself.** The work process continually produces the documentation, and Git carries the history that would otherwise accumulate in the reader’s way.
