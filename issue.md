# Issue #375: okf: validate closed capability binding keys with native SHACL

State: OPEN   Repo: Blackcat-Informatics/rust-agentic-foundations   Forge: github

## Body

Validate the nested capability bindings map with native SHACL Core.
Derive the shape from the existing keys for code, tests and examples.
All three keys are required and the map is closed.

- Use the bounded header graph and prepared validator from #367. Include
  the binding-map focus only when the authored value is a map.
- Preserve one `BindingsKeys` finding at `bindings` when the authored map
  has any extra or missing key. Preserve its order among profile value
  findings and carry native failures through the existing typed diagnostic.
- Keep the existing wrong-shape, list and capability-state rules. Preserve
  the canonical RDF authoring round trip.
- Exercise an extra key, a missing key and a valid neighbor through the
  actual profile entry point. Verify that native key-set validation
  preserves the diagnosed locations and accepted cases.

Parent #41 retains other nested record shapes, value constraints and
remaining graph-wide validation.


## Comments (0)

