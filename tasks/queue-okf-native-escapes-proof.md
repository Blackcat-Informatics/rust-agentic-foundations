<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native quoted escape ownership

The actual reader, inverse writer and representability checks use the public
six-member `ESCAPES` table. Its five shared decoded values now come from pinned
native `purrdf_lex::terminals::echar_value` at compile time, with explicit invariant
expects for the known ECHAR letters. The NUL extension remains RAF policy. Native
b, f and apostrophe escapes remain refused; there is no whole-YAML validation
pass, fallback parser or unused native result.

Session 84740 returned terminal exit 0 for all-target OKF caller compilation,
root/fuzz lock refresh, 29 actual refusal tests, 22 render tests, the bounds test
and warnings-denied all-target Clippy. Logs:

- `raw/queue-okf-native-escapes-callers.log`
- `raw/queue-okf-native-escapes-tests.log`
- `raw/queue-okf-native-escapes-clippy.log`
- `raw/queue-okf-native-escapes-fuzz-metadata.{json,log}`

The existing refused/valid escape owner now independently pins all six decoded
pairs rather than deriving both reader expectations and writer behavior only
from the implementation table. All three extra native ECHAR forms assert the
original UnknownEscape at line 2, column 5, beside a literal apostrophe neighbor.
Existing exact rendering/read-back and control-character refusal remain. The
bounds target here measures heading work, not a new universal frontmatter memory
proof; native constant evaluation adds no runtime parser allocation path.

The follow-on session 8165 starts with locked/offline fuzz metadata, then the
actual default check and production RDF completion. Those terminal receipts
remain separate. The new dependency is already in the audited root/fuzz closure;
both per-package lock lists now name native lex directly.

`queue-okf-native-yaml-boundary.md` records the independent source recheck and
the exact incompatible whole-YAML API boundary. The capability Explanation
retains that engineering rationale durably; the actual grammar, metadata,
refusal and resource promises remain in judged Contract/Constraints. No
upstream modification or general native YAML compatibility is claimed.
