# Constraints review (independent, single concern) — condensed record

Run read-only during plan mode; full report returned inline. VERDICT: NEEDS-REVISION (11 required, 10 advisory).

Required: 1 superset port imported L3 flat-first lookup, gmeow cfg evaluation and `seen` dedupe; 2 further precedent fallbacks (read().ok(), list swallowing, unreadable manifest, unparseable cfg, skipped targets, first-wins aliases, super above root, `_ => {}` foreign items, cfg(not(unix)) skip); 3 directory symlinks must error, not be skipped; 4 fuzz crate must carry identical workspace lints and MSRV check; 5 fuzz run must not write into tracked tree (fuzz/.gitignore, scratch corpus dir); 6 per-task --locked, msrv, deny, staged lockfile check; 7 Usage doctests vs no-fs scanner and equality enforcement; 8 no plan labels / precedent names in committed files; 9 subset and placeholder rules must be stated on the architecture page / in the profile; 10 missing negative fixtures list; 11 binding test must use the staged gate root.
Advisory: 12 drop unsafe fallback (cap has max_allocated); 13 containment race vs SECURITY.md; 14 every gate list; 15 docs current per commit; 16 fuzz deny/lockfile/fmt pinning; 17 crash input vs issue-ref gate; 18 writer must keep parsed style and refuse unreadable escapes; 19 commit types; 20 golden set from git ls-files; 21 toml without `unbounded`.

Dispositions: plan.md section "F. Review dispositions".
