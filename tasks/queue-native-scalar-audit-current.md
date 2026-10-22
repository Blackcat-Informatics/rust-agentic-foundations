<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Native scalar callers and exact subset adapters

The current adapters delegate supplied numeric and hexadecimal operations to the
pinned PurRDF revision. RAF retains its domain lexical, width and error rules.

- OKF record SHA-256 digests use native `decode_32_canonical`; full Git identifiers
  use native `nibble_canonical` behind the exact 40/64-byte guard. OKF now directly
  depends on the pinned native hash crate. Workplan's commit pattern uses the same
  native classifier.
- OKF debt counts retain unsigned digits and accepted leading zeroes. Zero padding
  is removed before native typed nonnegative Integer parsing; significant width
  is checked before arbitrary-precision work and the final usize conversion is
  checked. Actual record fixtures include zero, padded zero, leading zeroes, the
  host maximum and exact one-past-host-maximum refusal.
- Workplan's private unsigned-count adapter delegates to native typed Integer,
  preserving Rust's accepted optional plus and zero padding for libtest counts.
  Git major/minor retain their stricter digit-only spelling. Missing/non-digit
  patch remains zero as documented; a digit sequence above u32 now fails with
  VersionUnreadable instead of being silently replaced by zero. Public rustdoc
  states this overflow refusal. The real parser tests cover both neighbors.
- OKF scalar constructors use native `parse_double` for numeric quoting, with a
  lexical adapter preserving case-insensitive Rust infinity/NaN aliases and the
  existing explicit YAML aliases. Existing scalar constructor/read-back tests
  exercise aliases, exponent, dot forms and neighboring plain strings.
- Census table classification derives lower/upper spellings using native hex
  encoders into one fixed stack buffer. No allocation, copied alphabet or local
  per-call alphabet construction remains. RAF still chooses its TableKind.

## Actual execution

All commands below returned terminal exit 0; they are focused qualification, not
whole-delivery acceptance.

- `queue-native-count-workplan-current.log`: 273 workplan unit tests.
- `queue-native-scalar-okf-current.log`: profile 37, records 36, render 22 tests.
- `queue-native-record-one-past.log`: all 36 record tests with the final exact
  one-past-usize fixture.
- `queue-native-table-callers.log`: census groups 23 and units 31 tests.
- `queue-native-scalar-final-clippy.log`: all targets of OKF, workplan and census.

Earlier `queue-okf-native-record-hex.log` and
`queue-okf-native-record-scalars.log` contain corrected compile failures from
initial API selection; neither is success evidence. The final native APIs use
Option for decode_32_canonical and the crate-root XsdDatatype reexport.

The prior CLI scalar-caller record is `tasks/queue-cli-native-scalars.md`.
Assessment currency, remaining meaningful claim ownership, actual delivered-state
adoption, lifecycle completion and the mandatory full gate remain unfinished.
