<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Remaining gate, example, literal and terminal native reuse

Current radix completion: binary/octal value accumulation is also removed.
Primary identified the public bounded yaml::read_with scalar route to native
radix_value. RAF validates every original byte as Rust digits/separators before
admitting at most 128 binary or 43 octal significant digits into a fixed stack
buffer; aliases are disabled, node budget is one and collection depth zero.
Native Number lexemes reach typed Integer. A preserved String is the pinned
resolver's overflow result; unexpected refusal/type/rewrite is an invariant
failure. Source suffix/radix grammar remains RAF's, not YAML grammar.

Session 91916 returned exit 0: all five literal units passed exact maximum,
one-past, padding and invalid-after-overflow neighbors; forms 29, immutable
goldens 4, groups 23 and units 31 passed; census/workplan all-target Clippy
passed. Logs: queue-census-native-all-radices-current.log,
queue-census-native-all-radix-callers.log and queue-native-radix-strict-clippy.log.
The earlier compile refusal in queue-census-native-all-radices.log records native
Value's custom Drop forbidding moving enum contents; the final path borrows
without cloning. The earlier statement that no public binary/octal value route
exists below is superseded by this source-backed compatible native route.

Current source replacements:

- tests/repo_gates.rs removes the complete copied SHA-256 K/H0/compress kernel.
  The actual licence/digest callers use native ContentDigest::of/to_hex.
  recorded_fingerprint uses native decode_32_canonical for the 64-byte value and
  either-case nibble for both boundaries. Uppercase neighbors remain rejected
  as boundaries; uppercase values remain noncanonical. Unicode punctuation and
  ordinary non-hex boundary neighbors remain accepted.
- registry_scan.rs delegates all nine numeric reads to bounded native typed
  Integer and checked u64/usize conversions. Its thin adapter retains the Rust
  unsigned lexical subset (+, leading zeros; no negative/space/exponent), skips
  zero padding before the arbitrary-precision reader, and bounds significant
  input width. Version remains RAF example policy, with the existing core width,
  numeric-versus-text prerelease comparison and build-metadata treatment. It is
  not a new SemVer parser. Actual counts, flags, byte suffixes, version precedence,
  max/one-past-width and padding neighbors exercise those callers.
- graph.rs records_named uses the existing native unsigned count adapter.
  claim_coverage.rs delegates hexadecimal key validation to native decode_canonical.
  The system runner test's process id/group fields use native Integer plus checked
  signed conversion. Actual FIFO/process-family execution exercises both fields.
- Census literal parsing retains Rust suffix, underscore and radix recognition.
  Decimal values use native typed Integer with at most 39 significant digits;
  hexadecimal values use native nibble/decode_32 with a fixed 64-byte stack buffer.
  Oversized valid integers retain syntax classification and their existing
  unabstracted spelling policy. The complete token stream is still validated
  after value overflow. The exact missing native API is a binary/octal Rust-literal
  reader; those two prefixes and checked radix accumulation remain local. No
  normal-form version, hash framing or golden is edited.
- escape.rs retains RAF control-character/line-formatting policy and uses public
  purrdf_text::unicode::is_bidi_control instead of a copied Unicode range list.
  The text package is pinned to the same native revision. Its real closure was
  inspected from Cargo metadata: only purrdf-text is newly added because its
  core/sparql-eval/hash/xsd/lex prerequisites were already present. The package's
  unicode_tables.rs retains Unicode attribution and Unicode-3.0; root/fuzz deny
  files contain the identical exact-version exception and LICENSING names it.

Root Cargo.lock records root dev dependencies core/hash, text through workplan,
and the source example's XSD dev dependency. The fuzz harness does not depend on
workplan, so text is correctly absent from its lock. Source-only dev dependencies
are likewise not dependencies of the fuzz harness's source path dependency.
Both actual offline metadata commands returned exit 0.

The AGENTS engineering rule already required native reuse including tests and
examples. It now explicitly narrows remaining adapters to RAF-specific policy or
an exact required contract missing from the public API, and distinguishes this
engineering rule from unimplemented Census provider enforcement.

Current evidence:

- queue-gate-example-native-compile.log: real registry example check, exit 0.
- queue-gate-native-hash-final.log: all 72 root gates passed before text adoption.
- queue-example-native-counts-current.log: all 8 registry example tests passed.
- queue-claim-native-hex.log: all 13 actual claim tests passed.
- queue-graph-native-count.log: all 5 graph units passed.
- queue-census-native-literal.log: all 4 literal units passed.
- queue-census-native-literal-callers.log: forms 29, immutable goldens 4, groups 23
  and units 31 passed. Existing normal-form bytes remain pinned.
- queue-gate-example-native-clippy.log: source/workplan/census all-target Clippy
  passed before adding text; the enclosing session 13436 returned exit 0.
- queue-native-text-callers-compile.log: actual CLI/workplan all-target check
  passed, enclosing session 91389 exit 0.
- queue-exec-native-pid-descendants.log: actual process-family caller passed (1).
  The adjacent queue-exec-native-pid-family.log selected a private helper and ran
  zero tests; it provides no execution proof. Earlier queue-exec-native-pids.log
  ran two Cargo watchdog fixtures, not the targeted direct family caller.
- queue-native-bidi-escape.log: both terminal/line-formatting tests passed.
- queue-native-text-root-gates.log: all 72 root gates passed.

Historical failures are preserved: initial example test used a '+' prerelease
spelling that Version treats as build metadata; the fixture was corrected to the
actual leading-zero numeric neighbor. The first workspace-wide Clippy found
three assert-is-empty style failures in the existing touched dependency gate;
exact assertions now use assert_eq/assert_ne without weakening them. Session
23781 returned 101 at that Clippy step, before deny/sync. Its root and bidi tests
above had completed successfully.

Session 12925 returned terminal exit 0: workspace all-target Clippy, both deny
policies, all 72 root gates and docs sync passed. Exact final logs are
queue-native-text-current-clippy-final.log, queue-native-text-deny.log,
queue-native-text-fuzz-deny.log, queue-native-text-root-gates-final.log and
queue-native-text-docs-sync.log. The fuzz policy intentionally mirrors the root
exception, as the repository gate requires; deny therefore warns that text's
exception is unmatched in that smaller graph, then reports all four checks okay.
No warning was suppressed or gate relaxed. git diff --check returned exit 0.

The measured 142/111 delivered-state diagnostic predates these edits and remains
historical; no new adoption, assessment, lifecycle or full-gate completion is
claimed.
