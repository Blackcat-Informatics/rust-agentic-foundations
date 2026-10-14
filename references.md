# Referenced artifacts — issue #7

Everything the issue names, checked against disk. Facts only: whether a
file is a viable port source, or may be relicensed, is decided in the plan.
Paths resolve against the main checkout, then against a sibling checkout
only when that repo is named (first path component, or in the prose just
before the reference). "not here; exists in sibling(s)" is NOT resolved.

## Paths

| Reference | Exists | Resolved in | Kind | Lines | Licence header |
|---|---|---|---|---|---|
| `docs/capabilities/index.md` | NO | repo | — | — | — |
| `docs/` | yes | repo | dir | — | — |

### History: `docs/`

- `5bcd7411b fix(raf-source): index verbatim use groups and bodiless items`
- `bf08e459e feat(raf-source)!: seal the public types consumers bind to`
- `d8aba1e0c fix(raf-source): validate resolve paths and refuse globs that name nothing`
- `df71a428c fix(raf-source): match items_under by whole segment and check binding roles`
- `be71d40f9 fix(raf-source): bound glob scope tables with a byte budget`
- `71bd2c747 fix(raf-source): start first-line spans after a byte-order mark`
- `20b59c5d1 test(raf-source): exercise caps, spawn failure and source text through a load`
- `655834b9c fix(raf-source): harden manifest matching and error text`
- `bd6d3aa7a fix(raf-source): refuse an empty root list`
- `a26e75d76 build(fuzz): reach overflow-sized inputs and fuzz the OKF reader and workspace load`

## Identifiers (files containing each at HEAD)

- `title`: 82
- `description`: 83
- `state`: 85
- `depends`: 53
- `changes`: 67
- `status`: 26
- `check`: 100
- `delivered`: 30

## Workspace packages named

- `raf-source`

## URLs

None.
