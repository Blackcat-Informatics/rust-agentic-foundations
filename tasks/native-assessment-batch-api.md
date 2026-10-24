<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native assessment batch mechanics

Static public-API assessment only. No helper compiled/executed, dependencies or
product files changed, current assessments rewritten or gates weakened.

cargo-workplan/src/assess.rs:27 opens Gate and loads Workplan on every CLI call,
then records just one AssessmentRequest at 69. Repeating that command rebuilds
the complete Source workspace. Existing public native APIs support an owner-
reviewed explicit batch without a product change: Gate::workspace(root).open()
once, Opened::workplan() once, and Workplan::record_assessment for each exact
request against that loaded source/document snapshot.

## Compiling-shaped stage-only helper core

Use the existing native tool harness with its existing raf-workplan and raf-okf
crate bindings. Actor and AssessmentVerdict are public raf_okf types, not public
reexports from raf_workplan::assessment. Do not introduce a new third-party
dependency or fabricate enum values. If an existing harness lacks that direct
crate binding, this is an explicit harness/API import prerequisite, not a reason
to rewrite native record syntax; no dependency edit is performed by this review.

```rust
use std::{error::Error, io, path::Path};
use raf_okf::{Actor, AssessmentVerdict};
use raf_workplan::{Gate, assessment::AssessmentRequest, record::RecordOptions};

fn batch(
    root: &Path,
    reviewed: &[(&str, AssessmentVerdict, Actor)],
) -> Result<(), Box<dyn Error>> {
    if reviewed.is_empty() {
        return Err(io::Error::other("explicit reviewed subject list is empty").into());
    }
    let opened = Gate::workspace(root).open()?;
    let plan = opened.workplan();
    if !plan.findings().is_empty() {
        return Err(io::Error::other(format!(
            "native document load findings: {:?}", plan.findings()
        )).into());
    }
    let request_for = |subject: &str, verdict, by| {
        AssessmentRequest::new(subject, verdict, by).options(
            RecordOptions::new()
                .copyright("2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>")
                .licence("CC-BY-4.0")
        )
    };
    for (index, &(subject, verdict, by)) in reviewed.iter().enumerate() {
        if reviewed[..index].iter().any(|(previous, _, _)| *previous == subject) {
            return Err(io::Error::other(format!("duplicate subject {subject}")).into());
        }
        let context = plan.assess_context(subject)?;
        // The owner has reviewed this exact authored prose and its native
        // symbol/evidence inputs. Emit context for review before this write
        // mode, rather than treating its successful resolution as acceptance.
        let _ = context;
        let request = request_for(subject, verdict, by);
        plan.assessment_record(&request)?; // all requests judged before writes
    }
    for &(subject, verdict, by) in reviewed {
        let request = request_for(subject, verdict, by);
        let written = plan.record_assessment(root, &request)?;
        println!("{written}"); // stage helper output, not a product flow
    }
    // Load new record files, keeping opened's already built Source workspace.
    let refreshed = opened.workplan();
    if !refreshed.findings().is_empty() {
        return Err(io::Error::other(format!(
            "native refreshed load findings: {:?}", refreshed.findings()
        )).into());
    }
    let states = refreshed.list_assessments();
    for &(subject, verdict, by) in reviewed {
        let request = request_for(subject, verdict, by);
        let mut matching = states.iter().filter(|state| state.subject == request.subject);
        let Some(state) = matching.next() else {
            return Err(io::Error::other(format!(
                "written assessment not loaded: {}", request.subject
            )).into());
        };
        if matching.next().is_some() || !state.currency.is_current()
            || state.verdict != request.verdict || state.by != request.by
        {
            return Err(io::Error::other(format!(
                "written assessment is not exact/current: {}", request.subject
            )).into());
        }
    }
    Ok(())
}
```

The call site supplies a literal list of subjects and individually reviewed
verdicts/actors; it must not derive that list from list_assessments, reconcile
obligations, files or stale/current status. A separate context-only mode can
print plan.assess_context(subject) for that same explicit list in one open.
Successful parsing/context collection never supplies a verdict. Source,
capability prose/bindings, evidence and inventory edits must be settled before
opening and remain unchanged throughout owner review/write. A change to those
inputs requires reopening and review, not blindly accepting a changed digest.

## Why repeated native writes are safe within the frozen snapshot

assessment.rs::inputs_now (706-783) includes the authored section, selected bound
code symbols and evidence records named by claims. Its record inputs are
evidence:<slug>, not assessment records. record_assessment (978-1014) writes only
the selected work/assessments record through native create/replace, performing
directory/symlink checks and atomic per-file publication; it does not alter a
capability, source symbol or evidence record. Therefore one assessment write
does not invalidate the next subject's loaded inputs in this batch.

The original Workplan model does not mutate/reload its assessment list after a
write; list_assessments on that old model reports old loaded records. Calling
opened.workplan() reloads documents from the same DiskTree and reuses the same
Workspace. The refresh above checks new records with native currency policy
without another Gate::open/Source rebuild. It does not refresh concurrently
changed Rust source; that is why the freeze condition is substantive.

Native assessment_record supplies exact input digests and validates the record
and RecordOptions before writes. Native record_assessment keeps existing SPDX,
profile, containment and replacement behavior; no handmade Markdown/frontmatter,
digest, timestamp or fallback is needed. Setting Gate::record_options alone is
not sufficient: assessment SPDX comes from each AssessmentRequest.options, as
the actual CLI demonstrates.

All failures propagate nonzero. Preflight avoids known request errors producing
a partial batch, but multiple record writes are not one transaction: a later
I/O failure can leave earlier valid native records written. Report the exact
successful writes and failing subject, preserve them, and stop. Do not roll back
or hide failures, skip unknown subjects, downgrade missing evidence/symbols or
continue with a partial-success claim. Native lifecycle/complete/currency gates
still execute through their ordinary required paths; batching is mechanics,
not approval, reconciliation, test execution or completion.

## Tracked-path and new-record visibility clarification

Exact reread confirms public Opened::workplan document discovery is not filtered
by Git's captured path set. Opened retains a plain DiskTree plus separate
tracked/symlink vectors (gate.rs:421-426). Its workplan() passes that plain tree
to Workplan::load_with (470), while load::scan_directory calls docs.list
(load.rs:63,108-112) and read_file calls docs.read (167-180), with no tracked
membership filter. Thus both existing tracked replacements and new untracked
assessment files are visible to a fresh opened.workplan() document load through
this exact public path. Native create/replace still verifies actual file kinds.

The tracked snapshot DOES remain frozen for link judgment: Opened::links builds
LinkContext from self.tracked (gate.rs:459-465), and links.rs:302 rejects links
outside that universe. Staging a new file after Gate::open does not update the
old Opened vector. If the delivery needs newly staged paths to become link
targets, the caller must stage them explicitly and perform a fresh Gate::open;
the helper must hard-report that prerequisite rather than fabricate tracked
paths, use LinkContext::of_tree or silently relax link validation. Routine
replacement of existing tracked assessment files needs no new link universe.

The owner's observed lifecycle reconciliation requiring git add is retained as
an observation about that execution, not explained away by this source read.
A Git-filtered Tree wrapper, index-backed gate, cached Workplan or different
native harness may have another visibility boundary. The batching helper must
use the exact reviewed public Opened path above; if its actual caller supplies
a tracked-only tree, new records must be explicitly staged and its document
snapshot refreshed before acceptance. Do not infer record discovery from file
existence or a successful write: retain the native refreshed state checks and
hard-fail when any selected record is absent. No staging, writes to product or
compilers were performed for this clarification.
