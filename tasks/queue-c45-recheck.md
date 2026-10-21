# C4/C5 fix recheck

VERDICT: PASS for the substantive reviewed corrections. Full delivery remains pending.

The census-normal-form Constraint now explicitly warns that Census::load also
invalidates proc_macro2 spans held by its caller on the same thread, including on
an error, and tells callers to finish using them first. The refreshed Constraint
assessment retains both YAML SPDX headers. The required review finding is resolved.

The fuzz self-check now requires the expected symbol, file, exact source line,
parameter binding and a function print, besides its original one-unit assertion.
That directly checks parse positions, parameter facts and body normalization.
The advisory is addressed without weakening the arbitrary-input paths.

Migration's designed change item now admits reports under census load budgets,
rather than incorrectly treating a per-file size cap as a bound on the complete
report. It separately preserves max_file_bytes. This retains the requirement and
does not claim the designed migration implementation exists.

Observed raw/queue-c4-fuzz-current.log ends with 'Done 5000 runs'; terminal wrapper
exit status remains for the coordinator to capture. The coordinator reports fuzz
clippy and make deny passed. At inspection queue-c45-gates.log shows repo_gates
68/68 passed and self_workplan started, with no final self_workplan result yet.
No builds or source edits were performed by this recheck. Normal hooks and all
remaining required qualification must run; full make check and C6 remain pending.
