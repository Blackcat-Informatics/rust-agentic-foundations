# Issue #367: okf: validate closed top-level header keys with native SHACL

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Validate closed top-level OKF header keys through native SHACL Core in
`Profile::validate`. Derive each document type’s shape from the existing
`KeySpec` table. Use a bounded header graph with one focus and one row per
authored key. Document-type dispatch precedes shape selection.

- Enforce the closed key set and the presence of required fields through
  PurRDF’s prepared validator. Keep the parser’s unique-key invariant.
- Map typed validation results back to the existing unknown, missing,
  and forbidden-resource rules. Preserve authored order for unknown keys
  and schema order for missing keys. A native refusal is an actionable
  document finding.
- Qualify required-key removal, accepted neighbors, unknown and foreign
  keys, the `resource` diagnostic, and stable mixed diagnostic order on
  the real profile entry point. Verify the admitted repository corpus
  retains its canonical RDF authoring round trip.

Parent #41 retains nested record shapes, value constraints, complete
profile reconciliation, and the remaining graph-wide validation.


## Comments (0)

