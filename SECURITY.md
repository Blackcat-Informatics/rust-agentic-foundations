<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# RAF Security Policy

RAF (Rust Agentic Foundations: the `raf-okf`, `raf-source`, `raf-census` and
`raf-workplan` libraries and the `cargo-census` and `cargo-workplan` binaries)
is maintained by Blackcat Informatics® Inc. We take security reports seriously
and ask that vulnerabilities be reported privately so they can be investigated
and fixed responsibly.

## Supported versions

RAF is pre-release: nothing is published yet. Once the first release ships,
the latest `0.x` minor release is the supported one.

| Version | Supported |
| --- | --- |
| Latest `0.x` minor | Yes |
| Earlier `0.x` minors | No |

## Reporting a vulnerability

Do not open a public GitHub issue for a security vulnerability.

Instead:

- email <security@blackcatinformatics.ca>
- include `SECURITY` in the subject line
- name the affected crate(s) or binary and the version(s)
- describe the vulnerability, impact, and affected versions
- provide reproduction steps, a proof-of-concept input (a Rust source tree,
  ledger, capability document or manifest), or patches when possible

## Trust model

The checked workspace is **trusted build input**. RAF runs as a dev-dependency or
as a cargo subcommand inside the workspace it checks, and it executes that
workspace's cargo configuration exactly as `cargo build` or `cargo test` would:
`.cargo/config.toml` (including `build.rustc` and `build.rustc-wrapper`),
`rust-toolchain.toml`, `build.rs` and procedural macros all take effect when RAF
spawns `cargo`. Running RAF on a checkout therefore carries the same trust as
building that checkout. RAF is not a sandbox, and running it on a checkout you
would not build is out of scope.

Parsing is hardened regardless. The in-process parsers read Rust, TOML, YAML and
Markdown that the reader may not control. The aim is that a hostile file is refused
with a typed error, and that a failure never passes: a run ends in an error, a correct
result or an abort, and never in success with a wrong or partial result. The
guarantees are these, and no wider:

- **Containment.** A file is read only if it lies inside the canonical workspace
  root. The one exception is the reference scan, which reads the sources of
  reference packages by design; each of those files must lie inside the package
  directory that `cargo metadata` reports for that package. Symlinks that leave the
  root or package, directory-symlink loops and non-regular files are refused. A file
  read, a directory listing and an entry lookup each verify their handle against the
  root before the use and again after it, and on every platform a file's contents
  are read only through a handle verified to lie under the root. How the handle is
  verified depends on the platform:
  - On Linux a candidate is first taken as an `O_PATH` reference, and the kernel's
    own path for it must lie under the root before it is opened for reading or
    listed, through that descriptor path. An outside candidate is never opened for
    reading or listing; only an `O_PATH` reference is taken, which performs no I/O,
    device open or FIFO rendezvous.
  - On other Unix systems (macOS and the BSDs) the path is walked beneath a
    descriptor of the root, one `openat` per component, never following a symlink
    and never resolving a path by name; an in-root symlink is read and its target
    walked in its place. A walk outside the root may only descend back into the
    root, never climb, so a link cannot leave the root and come back through `..`:
    `../root/real.rs` from the root and `/…/root/../root/real.rs` are refused there,
    while Linux and Windows follow them, so a tree holding such links loads on
    Linux and Windows and not on macOS or the BSDs.
    Outside the root only directories are opened, and an outside non-directory is
    never opened. The file is read, the directory listed and the entry looked up
    through the walked handle; a directory renamed out of the root during the walk
    or the use is refused by the check after the use.
  - On Windows the handle is held without delete sharing and the path the system
    gives for it (`GetFinalPathNameByHandleW`) must lie under the root. An outside
    file can be opened in a race and is then refused before any read. A listing and
    an entry lookup go by the handle's verified path between the two checks, so a
    swap made and undone between them can return the names of an outside directory
    or the kind of an outside path, never their contents. Windows also refuses a
    path segment that is a reserved device name, in any case and with any
    extension, that ends in a dot or a space, or that holds a colon.
  - Whether an outside path exists. The other-Unix walk answers `OutsideRoot`
    whenever a path or a link leads outside the root, whether the outside path
    exists or not, except for a link whose target descends from outside into the
    root itself (such as an absolute target that spells the root through another
    path), which succeeds only when that outside path exists. Linux and Windows
    canonicalise first, and canonicalisation follows a link where it leads, so it
    tells the two apart: a link the tree holds, or a directory swapped for one
    during a call, that ends at an outside path is `OutsideRoot` when the path
    exists and `NotFound` when it does not, and one that passes an outside
    directory and climbs back into the root reads the in-root file when the
    directory exists and is `NotFound` when it does not.

  A Linux host runs the Linux strategy natively and the other two through a test
  override. On macOS and Windows hosts the native strategies are run only by the CI
  jobs that test the crate there; for FreeBSD they are compiled and not run.
- **Bounds.** Input size and nesting depth are bounded. For Rust source the source
  index reads each file under a byte cap, runs a pre-scan that refuses a file over a
  budget (delimiter nesting, nested `<`, prefix operators, closure bars, `else if`
  chains, an additive count of the parser's recursion over every open delimiter level,
  whose limit keeps an admitted file within half of the parse stack, and a token count, 2000000 by default, which bounds the memory of parsing
  one file at its peak at 1664 bytes per token, about 3.1 GiB, and which a caller on
  a smaller host lowers) with a typed error naming the file and the budget, bounds
  the module walk (128 nested modules and 262144 module walks per load, because a
  file reached by several module paths is walked once per path) and discovery (128
  directories below a root), keeping a stack of its own for each walk and for the
  search that keys the trait of an impl through imports and globs, reads and
  parses each file once whatever spellings reach it (files are told apart by device
  and inode, or on Windows by the path of the handle), and bounds what one load
  reads, keeps in its index and keeps as facts (256 MiB, 512 MiB and 256 MiB by
  default, 1 GiB in all).
  The scope tables that `use` declarations and globs build are bounded by no budget:
  a glob-heavy input can exhaust memory (one module of 3000 functions globbed by 3000
  others, 166 KB of source, peaks at 7.3 GiB). The source index parses on a
  dedicated 64 MiB stack on a thread named after the file, and is fuzzed through a
  target (`parse_source`) whose committed corpus `cargo test` replays in child
  processes, in debug and in release. These bound memory outside the scope tables
  and the stack that parsing an admitted file takes; the stack bound rests on an
  audit of the parser's recursion and on measured costs. A residual stack overflow
  aborts the run, so the gate fails and never passes, and a file ends in a typed
  error, a correct and complete index, or an abort, never in success with a wrong or
  partial index. Report
  an overflow as a robustness bug: under the trust model above its impact is a failed
  run and not a breach. The parsers set no time bound.
- **Subprocesses.** Every spawned `cargo` or `git` process runs with no shell and
  takes its arguments as an argv vector, and `cargo` runs with `--frozen`. A value
  taken from a document reaches a process only after resolution against the loaded
  documents or the source index and a stated pattern, and then in one of three
  forms: after `--`; inside a single `<rev>:<path>` or `-L<start>,<end>:<path>`
  argument; or, for a test name that the test harness reads, as a validated filter
  after `-- --exact --`.

Each spec states its own bounds and exact invocations.

## Security surface

The most relevant security surface for this project is **reading untrusted
files**. The following surfaces apply; `raf-okf` and `raf-source` implement the
parsing and path-traversal surfaces, and the rest apply as the other crates are built:

- **Rust source parsing** — `syn` parsing of untrusted `.rs` files; resource
  exhaustion from pathological nesting, very large files, or deeply recursive
  expressions, and panics on malformed input where a typed error is required.
- **TOML, YAML and Markdown parsing** — ledger and manifest TOML, capability
  and change frontmatter (YAML), and Markdown bodies; resource exhaustion
  (anchors and aliases, deep nesting, huge documents) and any input that is
  accepted where the strict closed-key schema must reject it.
- **Path traversal** — `#[path]` attributes, `mod` resolution and explicit
  `Cargo.toml` target paths that escape the workspace root, symlinks that point
  outside the scanned tree, non-regular files such as FIFOs, directory-symlink
  loops, and relative links in capability documents that resolve outside the
  repository. Every path is canonicalised and must lie inside the canonical workspace
  root, or inside the reported package directory for a reference package.
- **Reference-package scanning** — package resolution through
  `cargo metadata --frozen` and the sources of reference packages read from the
  local Cargo registry or git checkouts; a hostile manifest that redirects the
  scan. Resolution executes the workspace's cargo configuration (see the trust
  model above).
- **Subprocesses** — the `cargo` and `git` processes the gates spawn: argument
  injection through branch names, paths, package names or document fields,
  and unintended inheritance or leakage of the environment.
- **Supply chain** — the crates published on crates.io, their dependency tree,
  and the release automation that will publish them (none is published yet).

## What to expect

- acknowledgment within 48 hours
- initial triage within 7 days
- coordinated remediation and disclosure after validation

Resolution timelines depend on severity, exploitability, and release constraints, but
we aim to address confirmed issues as quickly as practical.

## Responsible disclosure process

1. Report the vulnerability privately.
2. Maintainers validate and triage the report.
3. A fix is developed, reviewed, and tested (across affected crates).
4. A release or advisory is prepared.
5. Public disclosure follows after users have had reasonable time to update.

## Reporter credit

For each vulnerability report resolved in the last 12 months, public release notes or
security advisories should credit the reporter unless the reporter requests anonymity
or private handling. If multiple reporters contributed to a confirmed issue, credit
each reporter who wants public recognition.

Reporter credit should use the name, handle, or organization supplied by the reporter.
If a reporter requests anonymity, release notes and advisories should state that the
issue was reported privately without naming them.

There are currently no publicly disclosed project vulnerabilities resolved in the last
12 months.

## Security updates

- watch the repository for releases and advisories
- keep dependencies current
- update to the latest supported release when fixes are published

## Contact

- Email: <security@blackcatinformatics.ca>
- PGP key for encrypted reports:
- Key fingerprint (RSA 4096, created 2016-12-26, expires 2026-12-24):
  `52E5 4B13 1C29 7D1E AB14  3BAA 0C54 057C 8F9A 0090`
- The key's user ID is `Blackcat Informatics Inc. (Secure Inbound Key)
  <secure@blackcatinformatics.ca>`. It names the `secure@` address, not the
  `security@` contact address above, so a keyring lookup by the contact address does
  not match this key. Select the key by fingerprint, and check the fingerprint
  against this page before encrypting.

```text
-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: GnuPG v2

mQINBFhhjUABEADg4mASErImePxCj0Ri8v08Axa1D1gnWPQBqtJW+P6OpQRuRXw0
KSeoeUipPmhJ2chK+rlCeocxO+1y0t7nkx5v7T20s3tF8rfpyQR4zX5h9C+ghi6r
LuZ3LIpBG9TLVALw8YpplMBXhbkIE0PftDYqt14mIFmK9tBO8fyWyPmaowEzbWIU
xOheaKQYzvU3RbiVPafWR5yqyiJQf+aBiAaAYPttfyiwOiKu9Aj6SvwssaGWci5Z
msVv5nLQuuZ0jE0M5jZupwmf/guBjCVE9pDs5k0i881otIQHjL8zzE5KtXKwpWAf
iAQkuKNktl+hc5GMeU2Ppu2GuK9zTm3WHtWyz5QUIsdz4rpGB/HZ10zymdHHqF0v
28RviJg8AFDFsJkVl275NLdt3PB4dIs6DGNholIG+R+LG6mmrG6mBhATJHVuFXpc
dM411h5gwl+X7ECW/VklcJgGRV+YVhdgRm8x5zGNSawxuXT2ksFXitgBpXGETCo9
wZv3s3nIximCV6n4J8bCbJtInt77e03fKzPMesG8UKCN0Ttkeu20lLD/maPPJlkX
xpq9jJi66j9dYIsK+1BXINOB2EgYvWApkXbh7cMiLScZIVJKlcFC9am+eWerRFP6
wcakBxhRjgrmlRYgytTc7oudMNvmzNtUhmAxOEM2MC640Bgss2D8O4isqQARAQAB
tE5CbGFja2NhdCBJbmZvcm1hdGljcyBJbmMuIChTZWN1cmUgSW5ib3VuZCBLZXkp
IDxzZWN1cmVAYmxhY2tjYXRpbmZvcm1hdGljcy5jYT6JAj8EEwEIACkFAlhhjUAC
GwMFCRLMAwAHCwkIBwMCAQYVCAIJCgsEFgIDAQIeAQIXgAAKCRAMVAV8j5oAkEqV
EADIwZHhD6Mdz7mVMfhcuoICvstJFr+GpP1zS/RHo0Xok5TgXhsZ4bP/A5BKYhkl
HoDT74pD9/bBplSQ/Cadg92nJCbPqQGkxZmHIteckoucKYayBZrOFEM/IwCft+R7
//TKHvYSwRqxFwo8LVOSH3/g1EI6d9zTQT/pDsRLdlDJUUK2sQVRrvkPACX5UJ4e
TveI8fUB51OVMQO73/27n/n5EMEt0B8+iBNjOIVJAImku/ZCyO4MJrUPYttz0E1P
B3w+9PwIOEb+EIZpFXFLWrsXBkwi3vHlwph1wvkPb2df+GIGkbPm4R+uQttzzV39
hlM805dFWhuE31RycH7PXgf4ZKw6YPwGjCmc0DrJgtMyrFB/rZNhNdl9DBVbIsLu
wXPZXwbMCViE+SPnLzMj5CjF1rB1Zp0WGBzrJ+IetLmTRthOIsL0ZMUKy31FEwW4
78BsVC3qCO+FaNRFwKwqCZdKs3Crnjb4TxZekf8sCi9sR5kHi9qEIAFJHh37Gfvb
u5LjZjhSTMNMCDBcvXVTrXmjxnJCMToc9AnpO8h4B+7hy7c+Ap6Pm/1UCrBdIPJ4
boWDSB1PVlZB3i3zRZ1YpU7FGX3XV7GbhYTS4r1rdo2nCNR+x+T+rugecrsd6yx/
T/5Q93Xgse0u2dQpiVeJGPQ/3pfvgT5kkIcRMEFrPApSh4hGBBARAgAGBQJYYY3M
AAoJEG9qKpCuDPLKBrsAoI9He4iNT6VLDp9DPSx3oK2gHe77AJ9Tk8oNAOsbKi+Y
a8/F0PWus+BoB4heBBARCAAGBQJYYY70AAoJEGwuemycFiRHe9QA/0EggxNwARzt
etCoenhIkBV4CrauHctataqBHE2zH1z2AQDKUeyAeCC2gKMLCoMlx+pgFSHV8ybN
LGA6/h5/4QPDZbkCDQRYYY1AARAAsRhXRchRyPsWV8rNFSkuhY6P+slHmFH1fvBE
41LkRWgQKMnUQK3Qr06tNoGHDkyZ15Haq6e/8RKoTjTOFF/uxeAmZrq1ZItfwuqv
gIpQvg+3uFNo8dccH0BWQZDKCHmUnoVFP8rW19ltW4qQ3QqvkiP2nKMJTp79T3/7
FYw9Kz4omt2+evhYiirkOTSCDYNFHsWh9JPdW/atzEZrKajNh4+6kq8dgqPjEv5P
UdhQsSb5iY408BykRHug9a1Zrm1rBsqSfESmd2v/Uc6EJ4a0Mv5xcVMulklijCeS
oYb5okS0yFh+q/+OjHthh7b+EMLi3m690cg+UYBLQS8Pzrr70D0FANKO1lSpGeQT
S4wqTjmb68fgeGEeteL2smgWa/oDOYcRmgiYP3Xkcf4c6Fb3aPwblYMsV9VNVD9H
y00l3F5uNLHZhj8N+aPGEyAwndc0WYSpC+x3HQMY52JBO78SJKVNFNtR58z02TyO
TtfAsY5rVrPUgnMYi10xaGdo/3GdhMVoWKp62xFqtasmgM563K+PM+JpQiq0JZkg
nIA5MtiHo+IEB/9xB61PGd4xU4XBl81pH8HDgUvARlUCIjysodwgc9QWILYXt7jB
j6BAK9V3RXLwvLEPX4fG2wlyfqJZ3BTcUIBWYjpP5X+uGwFZSpyV2GB8hkC0hFKx
jMcG1z8AEQEAAYkCJQQYAQgADwUCWGGNQAIbDAUJEswDAAAKCRAMVAV8j5oAkEkc
D/wNPwFwKJRKncoQP6KFgmgdLtxjfYGTMKrdTTJOXxRwcdSkma3PypbP+IT37MdR
WWM5qfBLNlw78kG+TmFRh2Mw+hZta8MKVhzJIBoxR0c18bvpig/TCBA8wRnrvFbx
OEXoEYxgtO1ORbzx/ifq6B47qFoPQu05XhQvNTKhdEtBROeZYP6qj/pnSy4u8g8w
Ds6LDBJiIUOgXH8kjU6psujoTYhrK+uKuMiHoaZt3kdoSDdC7+6iFpkpzuRbFi3w
3E7ZX+7XpwmKs21pKbzwSDTHKJ8fHnuq6sgzAiAy4dF8wp3dPIShaQ8qgSXrUblH
3GmV+VReBmzQNFElQz7zZRDwjpScQK6VwS/PA/rY+28N4ZiFruh4hqX917zttYNf
qL+AeU7BXe9VtTdvKyOwsdS/ayX0NeriPSxReZlBPgoG9/SEX+hyki9n7lS8eJby
46DbMBJafy9zErhP8ni0fO8+Q9gvtriAyo/ozwlSYxr6iu5VG8NJwZF8N/gzbx+6
jmyGBkMW5wHhJjlyy7SiZ/gg4Sb59vNLjbhQTJOB9DcCCWRHDZXR2avsJjP35YOQ
XE4dvUx/JNzvuZ/nkLMnuVf+feQJsvc+kLNV1K2sFGffpC/ZdBkU0lz5oLfqTtAM
1k2Eu+FYVJiyxA6fujgY65hx/hj/qZZJeuBTNgfWwiTn/A==
=fCTf
-----END PGP PUBLIC KEY BLOCK-----
```

For non-security questions or bug reports, use the normal public issue or discussion channels.
