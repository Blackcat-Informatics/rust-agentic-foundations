# Referenced artifacts — issue #16

Everything the issue names, checked against disk. Facts only: whether a
file is a viable port source, or may be relicensed, is decided in the plan.
Paths resolve against the main checkout, then against a sibling checkout
only when that repo is named (first path component, or in the prose just
before the reference). "not here; exists in sibling(s)" is NOT resolved.

## Paths

| Reference | Exists | Resolved in | Kind | Lines | Licence header |
|---|---|---|---|---|---|
| `docs/capabilities/index.md` | yes | repo | file | 35 | SPDX: CC-BY-4.0 |
| `Cargo.toml` | yes | repo | file | 56 | SPDX: MIT OR Apache-2.0 OR MulanPSL-2.0 |

### History: `docs/capabilities/index.md`

- `4dcdebc99 docs(capabilities): deliver the workplan-cli capability again`
- `f8a4e7dd5 docs(capabilities): deliver the workplan-entity-history capability again`
- `91804c0fd docs(capabilities): deliver the workplan-two-way-coverage capability again`
- `734a3aad7 docs(capabilities): deliver the workplan-prose-assessment capability again`
- `fd98b014a docs(capabilities): deliver the workplan-evidence-claims capability again`
- `f5a88ff9b docs(capabilities): deliver the workplan-complete-gate capability again`
- `113357b02 docs(capabilities): deliver the workplan-document-model capability again`
- `62d6c9bb1 docs(capabilities): deliver the workplan-generated-docs capability again`
- `11c7d0c0d docs(capabilities): deliver the source-index capability again`
- `92521b0a9 docs(capabilities): deliver the okf-profile capability again`

### History: `Cargo.toml`

- `c0685a4c2 feat(cargo-workplan): check, status and docs commands over raf-workplan`
- `b6dff1727 feat(raf-workplan): proposal, state, link and reference rules`
- `79cc6fb93 feat(raf-workplan): load documents into typed findings`
- `4e9c2f397 test(docs-contract): read documents through raf-okf and resolve bindings`
- `0a9d24352 feat(raf-source): bounded parsing on a dedicated stack with a fuzz target`
- `57d9a0614 feat(raf-source): file tree seam with canonical containment`
- `3f8697b35 feat(raf-okf): bounded frontmatter reader and deterministic writer`
- `a4eacdcfd build: scaffold RAF workspace`

## Identifiers (files containing each at HEAD)

- `delivered`: 85
- `bindings.examples`: 18
- `strict`: 41
- `Strictness`: 2
- `verify`: 47
- `dropped`: 63
- `moved`: 113
- `KINDS`: 18

## Workspace packages named

- `cargo-workplan`
- `raf-workplan`

## URLs

None.
