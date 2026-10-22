<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Registry fixture and allocator ownership recheck

Narrow independent read-only source review. No product edits, builds, probes or
tests were performed. This rechecks the unstable native registry fixture
identified in queue-typed-command-recheck.md and the bounds allocator ownership
change. The prior four runtime repairs are not reopened.

**The registry repair is honest controlled-input testing.** In
`registry.rs::recorded_binding_outcomes`, the helper creates actual source tests,
marks the successful test ignored for the unresolved neighbor, and invokes the
built evidence-record command. CARGO points at an executable fixture wrapper
whose complete body is `exec "$RAF_REGISTRY_CARGO" "$@" 2>/dev/null`.
RAF_REGISTRY_CARGO comes from the existing real_cargo helper. Thus this changes
the child Cargo stderr stream at the process boundary, before the actual
executor constructs its domain result. It forwards argv unchanged, preserves
Cargo exit status through exec, and retains actual test stdout. It does not
rewrite the resulting graph, fabricate an outcome, parse/replace volatile
message substrings or change production rendering/error behavior.

The failed native pin now records the actual invocation, exit 101 and proof
counts `0 passed, 1 failed, 0 ignored`; the domain `outcome` remains failed.
The ignored neighbor still obtains unresolved from the real binding path.
Suppressing all child Cargo stderr means this fixture does not exercise
propagation of compiler/build diagnostics. That is an explicit fixture input
limit, not a claim that those diagnostics are unnecessary in production. A
compile or list failure cannot silently satisfy the existing exact golden:
its invocation/reason/proof result differs. The separate ordinary CLI
failed/ignored/passed binding tests continue to cover actual outcomes without
this registry-only stderr control.

`observe` still compares complete native Turtle bytes, native N-Triples bytes,
reparsed equivalent graphs and sorted independent-parser pins. Registry
extraction still includes triple-term interiors, exact documented terms and
the both-directions emitted-term equality. Splitting assessment and binding
producer helpers changes fixture organization, not semantic assertions. The
old unstable capture is retained under raw/nondeterministic-binding-fixture;
it is not presented as a passing current native pin.

Inspected `queue-controlled-binding-registry-pins.log` and
`queue-review-current-registry.log` each show the selected actual registry test
passed. The final independent MIME replay log lists each actual .ttl/.nt input
and independently parsed triple count; the owner reports its terminal exit 0
and pair/pin equality. This reviewer read those logs and source but did not
rerun the parser or assert a new terminal status from the final printed line.
The owner's stated coverage is 28 native pairs and 152 command terms.

**Allocator ownership is genuine.** The diff in `raf-okf/tests/bounds.rs` moves
CURRENT/PEAK into AtomicUsize fields of Counting, initializes those fields on
the actual global ALLOCATOR, and changes GlobalAlloc alloc/dealloc/realloc to
use self.current/self.peak. `peak_of_parse` reads actual ALLOCATOR.live_bytes,
resets its actual peak, parses the body, reads peak and drops the body.
Therefore the test's ordinary references reach the allocator type whose real
methods own and update those counters. There is no fake read, no inserted
source-only use for the role checker, no schema/gate exception and no added lint
allowance. The existing scoped unsafe permission remains unchanged.

Atomic ordering, ceiling, forward-to-System requests, growth/shrink accounting,
test shapes and peak assertion are unchanged by the ownership refactor. The
test still compares the two actual parses with equal unit counts, and the
long-heading peak against the short-heading baseline plus the explicit
heading/allowance terms. It is a finite allocation regression, not a universal
parser memory proof. This narrow change does not independently establish every
underlying allocator failure path or arbitrary concurrent-measurement property.

`queue-adoption-owned-allocator.log` shows the actual bounds test passed;
`queue-review-fidelity-clippy-5.log` shows the reported check command reached a
successful finish. No independent execution was performed here. No additional
actionable defect was found in these two scoped repairs. Full qualification,
adoption, final parent review and authorized delivery remain separate.
