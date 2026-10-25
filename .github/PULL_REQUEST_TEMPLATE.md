<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->
## Summary

<!-- What does this change, and why? -->

## Checklist

- [ ] Native Linux `make check` passes (fmt, clippy, tests, release tests, doc, deny, reuse, msrv, fuzz, licenses, issue-refs)
- [ ] `make hooks` is installed and no hook was bypassed
- [ ] Every new file carries an SPDX header (`uvx reuse@6.2.0 lint` is clean)
- [ ] Capability docs are reconciled: they state the delivered contract, no PROPOSED block or change record remains for finished work
- [ ] Any change to a census fingerprint bumps `normal_form` and its golden tests
- [ ] No issue or PR numbers, and no development-process wording, in any committed file
- [ ] Commit messages follow conventional commits
