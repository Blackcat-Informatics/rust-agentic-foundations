Strict mode for raf-workplan: forced coverage and harder gates (#16).

## Delivered on this branch
* Strict mode: the coverage record's `strictness` setting, the global `--strict` option, its floor, and the rules for a delivered capability.
* Claim coverage with the `raf:tested-by` marker, and `coverage claims`.
* The change lifecycle under strict mode, including the `reason` of a change item.
* raf-source: resolution from the modules of every target, the impl blocks of every target, and item reachability over contains, impl-of and uses edges.
* Reverse (line) coverage: the orphan-line index, its rendering, `coverage lines`, and the ceilings/exceptions of the coverage record.

## Not yet complete
* The reverse-coverage commit (`T6`) has not had an independent review yet.
* Still to do: the debt record and ratchet (R9), completing `cap:workplan-strict`, and the final gate sequence.

## Scope decision
At the user's direction, R18 (this repository turning on strict mode for its own docs) is out of scope here. In the user's words: "Making this repo work under strict is an additional, low-priority issue we may need to file. That's a non-goal for this issue". It is tracked in #37.

## Questions for the maintainer
* E4: the census semantics are used in place of "fail only on new ones". Is that acceptable?
* E2: four existing fixtures were updated. Are those updates acceptable?

The branch also carries a merge of local `main`, which brings in the rtk configuration (`.rtk/filters.toml` and a `CLAUDE.md` section) that is not yet on `origin/main`.
