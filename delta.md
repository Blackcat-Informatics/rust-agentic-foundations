# Delta and merging reads for the artifact-evidence change

## 9259237f to f94a6788 (fix for the independent review)

- Read: crates/raf-workplan/src/assessment.rs, artifact.rs, digest.rs, obligation.rs, obligation/trace.rs, strict.rs.
- assessment.rs: only symbol rows of model-linked evidence become assessment inputs. Artifact rows are covered by the record digest. The invariant comment matches the code.
- artifact.rs: a missing, non-regular, symbolic-link-loop, outside-root or oversized file leaves the input stale. Any other read failure is a refusal, not a changed input.
- digest.rs, obligation.rs, trace.rs: the refusal is carried as a typed error; reconcile returns ReconcileError::ArtifactRefused, and the strict check gives it its own fix text.
- strict.rs: the refusal kinds are kept apart in the fix text.

## f94a6788 to 5da1f0b6 (CI failure fix)

- Read: diagram/semantic.rs, crates/raf-source/src/tree/disk/capture.rs (rustdoc links), crates/raf-okf/src/profile/records.rs (drive-designator refusal).
- semantic.rs passes the inner symbol or artifact error to DiagramError::data, so the caller's downcast to SymbolDigestError works. The sync test's budget of zero is set on purpose in the test and is unchanged.
- The drive-designator check refuses a segment such as `C:` in an artifact path; a negative fixture sits beside the accepted neighbour.

## Net merging reading at 5da1f0b6

- raf-source: Tree::read_bytes is a trait method with a default that refuses with an unsupported-operation error. The disk and memory trees implement it. The source-set tree delegates. The committed Git tree delegates to its inner tree, which refuses. Read.
- raf-okf: InputKind::Artifact is added to every match; artifact names are closed (no empty, dot, dot-dot or drive-prefixed segment, no backslash or control character, no leading or trailing space). Evidence takes symbol and artifact rows; reviews take symbols only; assessments take no artifact row. Read.
- raf-workplan: Workplan::load digests each cited artifact once through the tree passed to load; the artifact digest is SHA-256 over the framed field `artifact NUL path NUL length NUL bytes` (digest.rs, ContentHasher). graph.rs adds no edge for an artifact. staged.rs delegates reads to the inner tree after the staged text. Read.
- The `docs` tree passed to load is the tree the caller gives Workplan::load; the documented usage passes the repository root, so repository-root artifacts resolve.

## Observed, not changed

- Doc comment line widths in records.rs and capture.rs exceed the usual width; rustfmt does not reflow comments, and the gates pass.
- A Windows drive-designator refusal is applied on Linux too. The primary target is Linux; the refusal rejects only colon-prefixed segments such as `C:`.
