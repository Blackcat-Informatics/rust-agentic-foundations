### Stage 2 fix R9 (`8ff3859`): changelog dedup

**G10 closed.** `cliff.toml` ports purrdf's per-release dedup (`rendered_entries`/`breaking_keys`) and the `\(\s*\)` cleanup. RAF's tag pattern, parsers and PR-number stripping are unchanged.

**Check.** I built a scratch clone with signed commits and no bypass: a two-parent merge whose branch commit repeats the squash subject.
- With the old config the bullet printed twice.
- With the new config it prints once, and is marked BREAKING when either commit has `!`.
- `git cliff --unreleased` on the real history gives rc 0 and output identical to before.

**Review:** `G-R9-review` gave PASS. One LOW advisory, accepted: when two entries of different types share both scope and subject, the breaking flag over-marks. It errs safe and matches purrdf.
