## Behavior

Run `cargo census check --package PACKAGE --example EXAMPLE` at the selected
`--workspace` root. The repository example owns its Gate registrations and locked
provider inputs. The host reuses the bounded frozen Cargo runner, validates the
complete native RDF check-v1 response and requires matching process status before
printing findings. Missing/broken targets, malformed/oversized/noncanonical or
wrong-version responses and exit disagreement fail with actionable bounded causes;
there is no built-in-only fallback or debt writing.

Native Graph storage retains every finding's order and multiplicity. The codec
uses shared native scalar/profile helpers and the existing canonical reader/writer.
A shared runnable Rule/Extension example and real Cargo fixture exercise the same
registration source. A real DiskTree fixture reads unchanged actual manifests and
complete lock, reaches the pinned public PurRDF Hasher through the CLI, accepts
frozen empty/nonempty outputs and reports an unlisted competitor.

## Validation

Current local qualification passed formatting, Clippy, whole-workspace tests,
Source release tests, docs, deny, REUSE, MSRV and five5000-run fuzz targets.
The full invocation then refused missing cross-target components. After installing
those components for the same compiler, all remaining unmodified gates passed:
three Source cross-compilation targets, licenses and issue references. The failed
invocation remains recorded as failed; unchanged earlier passes are reused.
Normal signed commit hooks passed on ad4c336, including document contracts,
76 repository gates and current self-workplan. Native change completion and
independent substantive review passed.

Seven new test entrypoints replace seven existing report entrypoints; every
prior assertion body remains executed. Focused reports pass19/19. Inventory is
1 designed,3 implementing,20 delivered and153 assessments/156 records. Hosted
checks are being collected before merge. Windows runtime remains unverified
under the explicit maintainer exception.

The repository target is trusted executable code, not a sandbox. Configuration is
caller-supplied evidence, not authenticated compiler-closure acquisition. Registered
writes, remaining queries, self-census, complete detectors/adoption and source
acquisition retain their parent requirements; #5/#6/#9/#12/#40/#53 remain open.

Closes #64
