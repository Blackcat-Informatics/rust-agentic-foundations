# C6 review-fix evidence

Current source remains uncommitted for the coordinator's independent review and
normal commit/push. This is implementer evidence, not an independent verdict.

- Review allocation findings: keyed unit membership replaces Vec scans; Copy
  counts accumulate by job in one pass. Sorting borrows validated symbol/file
  keys. Prefix namespaces are computed once per render. Writer and reader paths
  both render their already validated original graph instead of rebuilding it.
  Closed-shape validation, typed extraction and canonical-byte checks remain.
- Canonical-line R1: finish compares split_inclusive('\n') chunks, preserving CR
  and the final LF. The valid canonical neighbor reads; whole-file CRLF is refused
  with NotCanonical line 1 and a missing final LF names the actual last line.
  Focused test exits 0: raw/queue-c6-canonical-lines.log.
- Diagnostics: malformed RDF can decode a control into an IRI, so ReportError
  Display escapes diagnostic strings. A decoded ESC in a foreign predicate is
  refused without printing the raw control. Focused test exits 0:
  raw/queue-c6-diagnostics.log.
- After original-graph reader rendering, all 24 then-current report tests passed
  (raw/queue-c6-reports-final.log), with six profile/shape/oracle tests passing
  (raw/queue-c6-profile.log). The canonical-line test was added and passed after
  that whole focused report run, so the combined current count is 25 report tests;
  no claim that a single 25-test run has occurred.
- Root docs_contract/repo_gates/self_workplan passed after staging the old
  report.rs deletion and new report paths (raw/queue-c6-root-gates.log). An earlier
  run had explicitly refused the missing old tracked path; it was an index/file
  list transition, repaired by staging the intended replacement, without changing
  any gate. The final document/workplan recheck exited 0: no workplan findings,
  docs_contract 28/28 and self_workplan 1/1, after the diagnostic test binding
  and final generated-matrix sync. Final all-target clippy also exited 0.

The 5000-run report-reader fuzz campaign passed on the preceding implementation;
the later original-graph rendering optimization and line/error diagnostic changes
are covered by focused tests. The mandatory final make check, including its current
fuzz execution, remains pending after all #3 requirements. Full issue completion
also requires the width/corpus work mapped in acceptance-gaps-current.md.
