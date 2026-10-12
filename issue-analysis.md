## Issue Analysis: #1

**Title:** Scaffold workspace, project docs and branding, plus spec-first capability docs
**State:** OPEN
**Repo:** Blackcat-Informatics/rust-agentic-foundations (github)
**Labels:** documentation, repo, dogfood, spec
**Milestone (per plan):** M0 Foundation
**Comments:** 0. **Linked PRs:** none. **Depends on:** none. **Blocks (per plan table):** shared crates (raf-okf + raf-source), the raf-workplan model, and transitively everything else.
**Evidence base:** `.stage/.../issue.md`, `brief.json`, `prior-art.md` (no ADRs, no linked issues, no defect-class trailers: first recorded instance); the approved plan `~/.claude/plans/i-would-two-create-snoopy-sifakis.md` (row 1 and "House conventions"); `INITIAL_PLANNING_PLAN.md` (248 lines, the only file at base `720262d`). The environment facts below were verified by running commands on 2026-10-11 in scratch dirs under `/tmp` (since deleted).

### Summary
Turn an empty repository into the RAF workspace: toolchain, lints, Makefile/hooks/CI gates, the house agent files, the purrdf-derived legal and community documents (re-branded, purrdf-only content removed), and the RAF brand (a toolbox service object on the shared cat-head-core). Most importantly for dogfooding, it writes 21 capability documents and 2 architecture documents in the OKF capability format that `raf-workplan` will later enforce. The 5 M2 capabilities are `considered`, each with a `work/changes/<slug>.md` record. The planning essay is dissolved into the Explanation sections and deleted. No crate code, beyond the minimum that makes `make check` meaningful (see A1).

---

### Verified environment facts (inputs to the checks)

| Fact | Evidence |
|---|---|
| `cargo` 1.100.0-nightly (2026-09-11), rustc nightly 2026-09-13. The machine policy pins every session to the floating nightly | `cargo --version`; comment in `logit-loom/rust-toolchain.toml` |
| MSRV toolchain 1.97.1 is installed in the separate validation rustup home. Run it with `rust-validate 1.97.1 <cmd>`; `cargo +1.97.1` intentionally fails on this machine | `ls ~/stage/packages/rustup-validation/toolchains`; `rust-validate` header |
| Present: `cargo-deny` 0.20.2, `rsvg-convert` 2.63.2, `git-cliff`, `actionlint`, `xmllint`, `shellcheck`, `yq` v4.54.1, `jq`, python 3.13 (has `tomllib`, no PyYAML) | `which` |
| **`reuse` is NOT installed.** `uvx reuse` works with network access (reuse 6.2.0). `uvx --offline reuse` fails (not cached) | `which reuse` -> missing; `uvx reuse --version` |
| **Local `cargo deny check` (advisories) FAILS.** The machine-wide git hook `hook.stage-signed-refs` (reference-transaction, `~/stage/git-hooks/signature-guard.py`) rejects the advisory-db fetch: "commit f58ccfe5... is unsigned". `cargo deny --offline check` passes against the cached db (last commit 2026-09-17) | ran both in a stub workspace |
| `origin` is already `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git` | `git remote -v` |
| `.stage/` in the worktree is untracked and NOT git-ignored. `reuse lint` checks untracked, non-ignored files, so it FAILS on `.stage/` | `git status --porcelain --ignored`; reuse test with an untracked `.stage/x.md` |
| purrdf `LICENSE-MULAN`, purrdf `LICENSES/MulanPSL-2.0.txt` and gmeow-gts `LICENSES/MulanPSL-2.0.txt` all hash to `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c` | `sha256sum` |
| purrdf `LICENSE-MIT` = `958c0e61...f000f3`, `LICENSE-APACHE` = `cfc7749b...523d30`, `LICENSES/CC-BY-4.0.txt` = `d557539df68e771cc1eedcc91d13f70fca930e508d11eedcafa4b15db49e3744` | `sha256sum` |
| The `cat-head-core` group is byte-identical in the purrdf, gmeow-ontology and gmeow-gts logos: 69 lines, SHA-256 `4ec35cf6551175dbcb7bed900ccf626aaf6623d73e29b39dc83c8468eb6a97fe` (awk extraction in R22) | hashed all three |
| purrdf SECURITY.md PGP block (BEGIN..END lines inclusive) SHA-256 = `6bb700ee9cc3d15b35231717654898b3f0c77897e4164124fd21c733ab5cfa62` | `sed -n '/BEGIN PGP/,/END PGP/p' \| sha256sum` |
| logit-loom has `AGENTS.md`, `rustfmt.toml`, `rust-toolchain.toml` and `CLA.md`, but **no** `.goals`, `.baseline`, `.deficiencies`, `deny.toml`, `clippy.toml` or `.githooks/`. Those exist in purrdf (`.goals`, `.baseline`, `.deficiencies`, `clippy.toml`, `.githooks/`) and lillith_memetics (`deny.toml`, `clippy.toml`) | `ls -a` survey |
| purrdf has no `REUSE.toml` and does not run `reuse`. The nearest template is `gmeow-gts/REUSE.toml` | `ls */REUSE.toml` |

### Cargo behaviour on an "empty workspace" (verified)

| Manifest | Result |
|---|---|
| Virtual manifest, `[workspace] members = []` | `cargo metadata` OK. `cargo clippy/test/doc/build` exit 101 "The manifest is virtual, and the workspace has no members". `cargo fmt --all --check` exits 1 "Failed to find targets". `cargo deny check` fails (`cargo metadata` error). **`make check` cannot pass.** |
| Virtual or rooted, `members = ["crates/*"]`, no `crates/` dir | exit 101 "failed to read crates/*/Cargo.toml" |
| **Root package** (non-virtual manifest, `publish = false`, `src/lib.rs` = one `//!` line), `members = []`, workspace lints inherited | fmt rc 0, clippy `-D warnings` rc 0, test rc 0, doc rc 0, `cargo deny --offline check` rc 0 (with a `[licenses] allow` list), `rust-validate 1.97.1 cargo check --workspace --all-targets` rc 0 |
| A `[workspace.package]` field that no member inherits (e.g. `authors`) | nightly cargo warns `cargo::unused_workspace_package_fields`. **rc stays 0** under `-D warnings`, because manifest warnings are not rustc warnings |
| `{ level = "allow", reason = "..." }` in `[workspace.lints.*]` | cargo warns "unused manifest key ... reason". Cargo lint tables have no `reason` field |

---

### Requirements and acceptance checks

All commands run from the worktree root. "->" gives the expected observation. The slug lists:

```sh
CAPS_M1C="census-normal-form census-ledger census-detectors census-patterns census-rule-api census-debt-ratchet census-reference-scan census-migration census-cli"
CAPS_M1W="okf-profile source-index workplan-document-model workplan-reconcile workplan-complete-gate workplan-generated-docs workplan-cli"
CAPS_M2="workplan-entity-history workplan-evidence-claims workplan-prose-assessment workplan-two-way-coverage workplan-current-rationale"
CAPS="$CAPS_M1C $CAPS_M1W $CAPS_M2"
```

#### Workspace

**R1. Workspace manifest**: edition 2024, resolver 3, `rust-version = "1.97.1"`, and `[workspace.package]` with authors `Blackcat Informatics Inc.`, license `MIT OR Apache-2.0 OR MulanPSL-2.0`, and the repository URL.
- `python3 -I -c 'import tomllib;d=tomllib.load(open("Cargo.toml","rb"));w=d["workspace"];p=w["package"];assert w["resolver"]=="3";assert p["edition"]=="2024";assert p["rust-version"]=="1.97.1";assert p["authors"]==["Blackcat Informatics Inc."];assert p["license"]=="MIT OR Apache-2.0 OR MulanPSL-2.0";assert p["repository"]=="https://github.com/Blackcat-Informatics/rust-agentic-foundations"'` -> exit 0. (`cargo metadata` does not expose `resolver`, so the TOML is read directly.)
- `cargo metadata --format-version 1 --no-deps | jq -r '.packages[]|[.name,.edition,.rust_version,.license,.repository]|@tsv'` -> every package shows `2024  1.97.1  MIT OR Apache-2.0 OR MulanPSL-2.0  https://github.com/Blackcat-Informatics/rust-agentic-foundations`.
- `cargo metadata --format-version 1 --no-deps 2>&1 >/dev/null | grep -c '^warning'` -> `0` (no unused `[workspace.package]` fields, no unused manifest keys).

**R2. `[workspace.lints]`**: rust `missing_docs`, `unreachable_pub` and `unsafe_code` are deny; clippy `all` and `pedantic` are warn; every allow carries a reason; no rustflags.
- `python3 -I -c 'import tomllib;l=tomllib.load(open("Cargo.toml","rb"))["workspace"]["lints"];r=l["rust"];c=l["clippy"];assert all(r[k]=="deny" for k in ("missing_docs","unreachable_pub","unsafe_code"));assert all(c[k]["level"]=="warn" and c[k]["priority"]==-1 for k in ("all","pedantic"))'` -> exit 0.
- `awk '/"allow"/{if(prev !~ /^[[:space:]]*#/){print NR": allow without a reason comment"; bad=1}} {prev=$0} END{exit bad}' Cargo.toml` -> exit 0. The reason has to be a TOML comment, because cargo ignores a `reason` key (verified).
- Code-level allows: `git grep -nE '#!?\[allow\(' -- '*.rs' | grep -v 'reason *='` -> no output. Recommended: clippy `allow_attributes_without_reason = "deny"`, so the lint set enforces this itself.
- No rustflags: `! test -e .cargo/config.toml || ! grep -q rustflags .cargo/config.toml` -> exit 0; `git grep -nE 'RUSTFLAGS|rustflags' -- Makefile .github .githooks Cargo.toml` -> no output.
- Each member opts in: `grep -A1 '^\[lints\]' Cargo.toml` -> `workspace = true` (repeat for each member manifest).

**R3. `rust-toolchain.toml`**: floating nightly with clippy and rustfmt; no `#![feature]`.
- `python3 -I -c 'import tomllib;t=tomllib.load(open("rust-toolchain.toml","rb"))["toolchain"];assert t["channel"]=="nightly";assert {"clippy","rustfmt"}<=set(t["components"])'` -> exit 0 (channel exactly `nightly`, no date).
- `git grep -n '#!\[feature' -- '*.rs'` -> no output.

**R4. `rustfmt.toml`, `clippy.toml`, `deny.toml`.**
- `for f in rustfmt.toml clippy.toml deny.toml; do test -s $f || echo MISSING $f; done` -> no output.
- `cargo fmt --all --check` -> exit 0, with no "unknown configuration option" warnings on stderr.
- `grep -E '^msrv' clippy.toml` -> no output, or exactly `msrv = "1.97.1"`. Clippy already reads `rust-version`, and a different value triggers a mismatch warning. purrdf's `msrv = "1.98"` must not be copied.
- `cargo deny --offline check licenses bans sources` -> `bans ok, licenses ok, sources ok`. `[licenses] allow` must contain `MIT`, `Apache-2.0` and `MulanPSL-2.0`: the lillith_memetics template has no `[licenses]` section and fails with "license is not explicitly allowed" (verified). Strip memetics' substrate-specific `allow-git` and `skip` lists.

**R5. `Makefile` `check` target** (fmt, clippy `-D warnings`, test, doc, deny, `reuse lint`).
- `make -n check` -> output contains each of: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` with `-D warnings`, `cargo test --workspace`, `cargo doc --workspace`, `cargo deny`, `reuse lint`.
- `make check` -> exit 0 (see R33).
- Recommended additions:
  - An MSRV step, from the plan's Verification line ("... `cargo deny check`, and the MSRV build"); see A2.
  - The issue-reference grep from R36, so that acceptance criterion is enforced on every run and not checked once.

**R6. `.githooks/pre-commit` checks the staged index; `make hooks` installs it.**
- `test -x .githooks/pre-commit && git ls-files -s .githooks/pre-commit | cut -c1-6` -> `100755`.
- `shellcheck .githooks/pre-commit` -> exit 0.
- `make hooks && git config --get core.hooksPath` -> `.githooks`.
- Index semantics, tested in a throwaway clone:
  - Stage a badly formatted `src/lib.rs`, then fix the working tree without staging -> `git commit` is refused.
  - Stage a good file and leave a bad unstaged edit -> the commit succeeds.
  - Use purrdf's pattern: snapshot the index with `git checkout-index --all --prefix=$snapshot/`.
- The machine-wide config hooks (`stage-hygiene-commit`, `stage-signed-refs`, the attribution commit-msg/pre-push guards) still run alongside `core.hooksPath`. They must not be disabled.

**R7. `.github/workflows/ci.yaml`** with jobs fmt, clippy, test, doc, deny and msrv.
- `test -f .github/workflows/ci.yaml && actionlint .github/workflows/ci.yaml` -> exit 0.
- `yq '.jobs|keys' .github/workflows/ci.yaml` -> covers fmt, clippy, test, doc, deny, msrv (as separate jobs or named steps).
- The msrv job pins its own toolchain:
  - `yq '.jobs.msrv' .github/workflows/ci.yaml | grep -E '1\.97\.1'` -> a match. The job's rustup toolchain env must explicitly name 1.97.1 (purrdf's pattern), because `rust-toolchain.toml` says nightly and would otherwise win.
  - It also needs a step asserting that `rustc --version` starts with `rustc 1.97.1` (purrdf's pattern).
- The workflow file is named `ci.yaml`, not logit-loom's `ci.yml`.
- CI is green: after push, `gh pr checks <PR>` -> every check passes (R33).

**R8. `AGENTS.md`, `CLAUDE.md`, `.goals`, `.baseline`, `.deficiencies` from the house templates.**
- `for f in AGENTS.md CLAUDE.md .goals .baseline .deficiencies; do test -s $f || echo MISSING $f; done` -> no output.
- `.baseline` keeps the issue-number ban and the no-deferrals rule: `grep -c 'issue numbers' .baseline` -> >= 1; `grep -ciE 'DEF+ERRALS' .baseline` -> >= 1.
- `.deficiencies` is the notice plus marker, with no entries: `sed -n '/--- ENTRIES BELOW THIS LINE ---/,$p' .deficiencies | tail -n +2 | grep -c .` -> `0`.
- `.goals` is adapted, not copied: `grep -nE 'RDF|PYTHON|WASM|wasm' .goals` -> no output. purrdf's `.goals` has "RDF1.2 ONLY", "PYTHON-SURFACE" and "MAXIMAL PORTABILITY ... wasmable", none of which apply to RAF (A5).
- `CLAUDE.md` defers to `AGENTS.md`: `grep -c 'AGENTS.md' CLAUDE.md` -> >= 1.

**R9. Update `origin` to the renamed repository URL.**
- `git remote get-url origin` -> `ssh://git@ssh.github.com:443/Blackcat-Informatics/rust-agentic-foundations.git`. **This is already true**, so verify it, record it as done, and do not change it.

#### Project documents (adapted from purrdf)

**R10. `LICENSING.md`**: code under `MIT OR Apache-2.0 OR MulanPSL-2.0`, docs under CC-BY-4.0, the commercial reservation, trademarks, CLA, copyright notice.
- `grep -E '^## ' LICENSING.md` -> includes `Open-source terms`, `Proprietary / commercial licensing`, `Trademarks`, `Contributions` and `Copyright notice`. It does not include `Unicode data compiled into published crates`. The third-party and notices sections remain only with RAF-relevant text.
- `grep -c 'licensing@blackcatinformatics.ca' LICENSING.md` -> >= 1. `grep -c 'CC-BY-4.0' LICENSING.md` -> >= 1. `grep -cE 'Contributor License Agreement|CLA' LICENSING.md` -> >= 1.
- `grep -nE 'Unicode-3.0|Python|WebAssembly|wasm|npm|PyO3|GTS|corpora|libpurrdf' LICENSING.md` -> no output.
- Copy hazards in purrdf's text:
  - Line 35 (`` `SPDX-License-Identifier: CC-BY-4.0`; ``) makes `reuse lint` report "Invalid SPDX License Expressions" (reproduced).
  - Line 233 (`... AND Unicode-3.0`) would add a missing-license error.
  - Fix: wrap prose mentions of the tag in `REUSE-IgnoreStart`/`REUSE-IgnoreEnd`, or reword them.

**R11. `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-MULAN` (byte-pinned), `LICENSES/CC-BY-4.0.txt`.**
- `sha256sum LICENSE-MULAN` -> `eb7a1d713eb919b146787629e22e4c975cb701f529a65d4d7e0fcd417558bf1c`.
- `sha256sum LICENSES/CC-BY-4.0.txt` -> `d557539df68e771cc1eedcc91d13f70fca930e508d11eedcafa4b15db49e3744`.
- `cmp LICENSE-APACHE ~/Active/purrdf/LICENSE-APACHE` -> identical. `LICENSE-MIT` keeps purrdf's text, with the copyright line naming Blackcat Informatics Inc.
- **Implied R11b:** `LICENSES/MIT.txt`, `LICENSES/Apache-2.0.txt` and `LICENSES/MulanPSL-2.0.txt` must also exist. Reproduced: with only the root `LICENSE-*` files, `reuse lint` reports "Missing licenses: Apache-2.0, MIT, MulanPSL-2.0", because reuse ignores root `LICENSE-*` files. Check: `cmp LICENSE-MULAN LICENSES/MulanPSL-2.0.txt` -> identical.

**R12. `docs/LICENSING.zh-Hans.md`** with names substituted.
- `grep -c 'PurRDF\|purrdf' docs/LICENSING.zh-Hans.md` -> `0`. `grep -c 'RAF' docs/LICENSING.zh-Hans.md` -> >= 1.
- purrdf-only sentences removed: `grep -nE 'Unicode|YAML|XML|Python|npm|WASM|C 发行包' docs/LICENSING.zh-Hans.md` -> no output. purrdf's paragraphs 3-4 cover Unicode tables, the YAML emitter, XML test data and Python/npm/WASM/C packages, so name substitution alone does not satisfy the "remove purrdf-only content" rule (A6).
- These link targets exist: `LICENSE-MULAN`, `LICENSING.md`, `docs/LICENSING-BACKTRANSLATION.md`.

**R13. `docs/LICENSING-BACKTRANSLATION.md`**: a new record for the changed Chinese source.
- The recorded source hash matches: the first 64-hex value in the record == `sha256sum docs/LICENSING.zh-Hans.md | cut -d' ' -f1`.
- The recorded byte count N in "Source: `docs/LICENSING.zh-Hans.md` (N UTF-8 bytes)" == `wc -c < docs/LICENSING.zh-Hans.md`.
- The recorded MulanPSL hash is `eb7a1d71...bf1c`, and the record has a "Complete independent backtranslation" section.
- Provenance cannot be checked mechanically. The backtranslation must come from an independent model run that sees ONLY `docs/LICENSING.zh-Hans.md`: not the English original, the licences, the history, or this analysis. Its timestamps must be real (risk K3).

**R14. `CONTRIBUTING.md`**: RAF design constraints (normal-form stability, dogfood gates, the workplan completion gate). It keeps the toolchain/MSRV, semver, conventional-commit, PR-checklist and contribution-licensing sections, plus conduct (per the plan).
- `grep -ciE 'normal.form' CONTRIBUTING.md` -> >= 1. `grep -ciE 'dogfood|self-gate' CONTRIBUTING.md` -> >= 1. `grep -ciE 'completion gate|change complete' CONTRIBUTING.md` -> >= 1.
- `grep -E '^## ' CONTRIBUTING.md` -> includes design constraints, development/toolchain (mentions `1.97.1`), versioning (semver), conventional commits, "Before you open a pull request", "Licensing of contributions" and "Conduct".
- `grep -nE 'Benchmarks|conformance|wasm|Python|corpora' CONTRIBUTING.md` -> no output.

**R15. `CODE_OF_CONDUCT.md`, with the name changed.**
- `diff <(sed 's/PurRDF/RAF/g; s/purrdf/raf/g' ~/Active/purrdf/CODE_OF_CONDUCT.md) CODE_OF_CONDUCT.md` -> no differences beyond the SPDX header lines. The plan says "copied verbatim with the name changed". Its `## Attribution` section credits the source of the code-of-conduct text, which is expected content.

**R16. `SECURITY.md`**: the same process, address and PGP key, with the attack surface rewritten.
- `sed -n '/-----BEGIN PGP PUBLIC KEY BLOCK-----/,/-----END PGP PUBLIC KEY BLOCK-----/p' SECURITY.md | sha256sum` -> `6bb700ee9cc3d15b35231717654898b3f0c77897e4164124fd21c733ab5cfa62`.
- `grep -c 'security@blackcatinformatics.ca' SECURITY.md` -> >= 2.
- Each attack-surface item is named: `for t in 'Rust source' TOML YAML '#\[path\]' 'reference scan' subprocess cargo git 'crates.io'; do grep -qiE "$t" SECURITY.md || echo "MISSING $t"; done` -> no output.
- `grep -nE 'SPARQL|RDF|GTS|PyO3|CBOR|zstd|libpurrdf|PyPI' SECURITY.md` -> no output. The "Supported versions" table reflects RAF (A16).

**R17. `.github/ISSUE_TEMPLATE/{bug_report,feature_request,config}.yml` and `PULL_REQUEST_TEMPLATE.md`.**
- `for f in bug_report feature_request config; do yq '.' .github/ISSUE_TEMPLATE/$f.yml >/dev/null || echo BAD $f; done` -> no output.
- `grep -rniE 'purrdf|W3C|SPARQL|python|wasm' .github/ISSUE_TEMPLATE .github/PULL_REQUEST_TEMPLATE.md` -> no output.
- The PR template checklist mentions `make check` and the workplan obligation (capability docs reconciled).

**R18. `CITATION.cff`, `cliff.toml`, `CHANGELOG.md`.**
- `uvx cffconvert --validate -i CITATION.cff` -> valid. This needs network; offline, use a `yq` parse plus the required keys `cff-version`, `message`, `title`, `authors`.
- `git cliff --config cliff.toml --unreleased >/dev/null` -> exit 0.
- `test -s CHANGELOG.md` -> exit 0, and `grep -c 'PurRDF' CHANGELOG.md` -> `0`.

**R19. SPDX headers on every file; `REUSE.toml`; `reuse lint` passes.**
- `uvx reuse lint` (or `reuse lint` once installed) -> exit 0, "your project is compliant with version 3.3 of the REUSE Specification". Its summary shows "Files with copyright information: N / N", "Missing licenses: 0" and "Invalid SPDX License Expressions: 0".
- Header convention: code `MIT OR Apache-2.0 OR MulanPSL-2.0`, documents `CC-BY-4.0`. `git grep -L 'SPDX-License-Identifier: CC-BY-4.0' -- '*.md'` -> lists only files deliberately under the code licence (decide per A13).
- `REUSE.toml` annotates the files that cannot carry headers: `Cargo.lock`, `docs/social-preview.png`, `.gitignore`, `.goals`, `.baseline`, `.deficiencies`, and `CITATION.cff` if it has no comment header. Template: `~/Active/gmeow-gts/REUSE.toml`.
- Implied: `.gitignore` must ignore `/.stage/` and `/target`, or `reuse lint` fails locally (R40).

**R20. purrdf names and purrdf-only content removed** (vendored corpora, Unicode tables, bindings, GTS, Node).
- `git grep -ni 'purrdf' -- . ':!docs/BRAND.md'` -> no output. BRAND.md may name `purrdf/docs/purrdf-logo.svg` as the family source.
- `git grep -nwE 'GTS|PyO3|npm|Node|Unicode-3\.0|corpora|wasm-pack' -- '*.md' '*.yml' '*.yaml' '*.cff'` -> no output.

#### Branding

**R21. `docs/BRAND.md`**: name **RAF (Rust Agentic Foundations)**; machine ids lowercase `raf-*` / `cargo-*`; positioning, tagline, repository description; family system; colour tokens; asset rebuild command.
- `grep -c 'RAF (Rust Agentic Foundations)' docs/BRAND.md` -> >= 1.
- `grep -E '^## ' docs/BRAND.md` -> includes Name, Positioning, Tagline, Repository description, Family system, Colour tokens and Logo assets.
- `grep -c 'service-toolbox' docs/BRAND.md` -> >= 1. `grep -c 'rsvg-convert -w 1280 -h 640' docs/BRAND.md` -> >= 1.
- The family colour tokens are unchanged: `for c in '#111214' '#fffdf5' '#ffffff' '#ea4335' '#4285f4' '#fbbc05' '#34a853'; do grep -q "$c" docs/BRAND.md || echo MISSING $c; done` -> no output.

**R22. Family system**: `cat-head-core` reused verbatim; the service object is `service-toolbox` (a toolbox with a wrench), with the accent colours isolated to that group.
- The core is verbatim: `awk '/<g id="cat-head-core">/{f=1;d=0} f{print; d+=gsub(/<g[ >]/,"&"); d-=gsub(/<\/g>/,"&"); if(d==0) exit}' docs/raf-logo.svg | sha256sum` -> `4ec35cf6551175dbcb7bed900ccf626aaf6623d73e29b39dc83c8468eb6a97fe`. The same command gives this value on all three family logos, and it only matches if indentation is preserved.
- `grep -c 'id="service-toolbox"' docs/raf-logo.svg` -> `1`. `grep -c 'id="service-triple"' docs/raf-logo.svg` -> `0`.
- The four accents appear only inside `service-toolbox`:
  ```sh
  python3 -I - docs/raf-logo.svg <<'PY'
  import sys, xml.etree.ElementTree as ET
  A={"#ea4335","#4285f4","#fbbc05","#34a853"}; t=ET.parse(sys.argv[1]); bad=[]
  def walk(e, inside):
      inside = inside or e.get("id")=="service-toolbox"
      blob=" ".join(v.lower() for v in e.attrib.values())
      if not inside and any(a in blob for a in A): bad.append(e.get("id") or e.tag)
      for c in e: walk(c, inside)
  walk(t.getroot(), False); print(bad); sys.exit(1 if bad else 0)
  PY
  ```
  -> prints `[]`, exit 0. Also, `grep -niE 'ea4335|4285f4|fbbc05|34a853' docs/raf-logo.svg` matches only lines inside the group, so no `<style>` rule leaks an accent.
- `xmllint --noout docs/raf-logo.svg docs/social-preview.svg` -> exit 0.
- `<title>`/`<desc>` describe a cat holding a toolbox with a wrench: `grep -ci 'toolbox' docs/raf-logo.svg` -> >= 2; `grep -ci 'wrench' docs/raf-logo.svg` -> >= 1.

**R23. `docs/raf-logo.svg`, `docs/social-preview.svg`, `docs/social-preview.png`** (PNG rendered with `rsvg-convert -w 1280 -h 640`).
- `file docs/social-preview.png` -> `PNG image data, 1280 x 640`.
- Reproducible on this machine: `rsvg-convert -w 1280 -h 640 docs/social-preview.svg -o /tmp/sp.png && cmp /tmp/sp.png docs/social-preview.png` -> identical. Font fallback differs between machines, so this is not a CI check (K6).
- `grep -c 'RAF' docs/social-preview.svg` -> >= 1. `grep -ciE 'purrdf|RDF 1.2' docs/social-preview.svg` -> `0`.

**R24. `README.md`**: the logo, what each crate does, and the dev-dependency usage snippets.
- `grep -c 'docs/raf-logo.svg' README.md` -> >= 1.
- `for c in raf-okf raf-source raf-census cargo-census raf-workplan cargo-workplan; do grep -q "$c" README.md || echo MISSING $c; done` -> no output.
- `grep -c 'dev-dependencies' README.md` -> >= 1, and the snippets include `raf_census::Gate::workspace` and `raf_workplan::Gate::workspace` (the plan's "How projects use them as dev-dependencies").
- The README states that the crates are designed but not yet available; it must not present planned behaviour as shipped (A17). `grep -ciE 'designed|planned|not yet published' README.md` -> >= 1.

#### Spec-first capability documents (dogfood)

**R25. Frontmatter schema**: every `docs/capabilities/<slug>.md` has exactly the keys `type: Capability`, `id: cap:<slug>`, `title`, `description`, `state`, `bindings` and `depends`.
```sh
for s in $CAPS; do f=docs/capabilities/$s.md
  test -f $f || { echo "MISSING $f"; continue; }
  [ "$(head -1 $f)" = "---" ] || echo "NOFM $f"
  k=$(yq --front-matter=extract 'keys|sort|join(",")' $f)
  [ "$k" = "bindings,depends,description,id,state,title,type" ] || echo "KEYS $f: $k"
  [ "$(yq --front-matter=extract '.type' $f)" = Capability ] || echo "TYPE $f"
  [ "$(yq --front-matter=extract '.id' $f)" = "cap:$s" ] || echo "ID $f"
  [ "$(yq --front-matter=extract '.bindings|keys|sort|join(",")' $f)" = "code,examples,tests" ] || echo "BINDINGS $f"
done
ls docs/capabilities/*.md | wc -l
```
-> no output from the loop, and the count is `21` (no stray non-capability files; see A10 on `index.md`).
- The frontmatter starts at line 1, so SPDX goes in YAML comments inside it (verified: reuse accepts this and `yq` ignores the comments; A13).

**R26. Fixed sections**, exactly these and in this order: Purpose, Contract, Usage, Examples, Constraints, Explanation, Verification.
```sh
want='## Purpose|## Contract|## Usage|## Examples|## Constraints|## Explanation|## Verification|'
for s in $CAPS; do f=docs/capabilities/$s.md
  got=$(grep -E '^## ' $f | tr '\n' '|'); [ "$got" = "$want" ] || echo "SECTIONS $f: $got"
  awk '/^## /{if(h!="" && n==0) print FILENAME": empty "h; h=$0; n=0; next} h!="" && NF{n++} END{if(n==0) print FILENAME": empty "h}' $f
done
```
-> no output: exactly 7 H2 sections in order, none empty.

**R27. M1 census docs, `state: designed`**: the 9 slugs in `CAPS_M1C`.
- `for s in $CAPS_M1C; do [ "$(yq --front-matter=extract .state docs/capabilities/$s.md)" = designed ] || echo $s; done` -> no output.
- Plan decisions are carried over. Spot greps, each expected to match:

  | File | Must contain |
  |---|---|
  | `census-normal-form.md` | `normal_form = 1`; `min_tokens` default 20; `SHIM_MAX_TOKENS` 30; `MIN_SHIM_TOKENS` 6; BLAKE3 with the `<kind>:` + first-8-bytes hex form; kinds `body`, `shim`, `table`, `rule` |
  | `census-cli.md` | exit codes 0/1/2; the subcommands `check`, `baseline`, `index`, `job`, `count`, `fingerprint`, `debt --write`, `migrate`, `dump-ledger` |
  | `census-ledger.md` | `deny_unknown_fields`, `[[job]]`, `[[distinct]]`, `[job.ext]`, `LedgerExtension`, the staleness checks |
  | `census-detectors.md` | all 14 built-in detector ids |
  | `census-patterns.md` | the 6 matcher forms (`call`, `path`, `macro`, `type`, `literal_regex`, `tokens`) and `rule:<id>` |
  | `census-rule-api.md` | the `Rule` trait signature; the negative-fixture rule |
  | `census-debt-ratchet.md` | `census-debt.json`; `debt --write` |
  | `census-reference-scan.md` | `cargo metadata`; `[census] reference` |
  | `census-migration.md` | `migrate --old-baseline ... --ledger ...` |

**R28. M1 workplan docs, `state: designed`**: the 7 slugs in `CAPS_M1W`.
- The same state loop over `$CAPS_M1W` -> no output.
- Spot greps, each expected to match:

  | File | Must contain |
  |---|---|
  | `workplan-document-model.md` | the four states; `> [!PROPOSED change:`; `work/changes/`; the Change keys; the outcomes `implemented`, `dropped`, `moved:` |
  | `workplan-generated-docs.md` | `<!-- raf:generated:`; `docs sync` |
  | `workplan-reconcile.md` | "obligation" |
  | `workplan-complete-gate.md` | the five completion conditions; that the change file is deleted |
  | `workplan-cli.md` | `check`, `change new`/`impact`/`reconcile`/`complete`, `docs sync`/`preview`, `status`, `verify` |
  | `source-index.md` | `#[path]`; the `use`-alias index; `Tree` (Disk/Memory) |
  | `okf-profile.md` | closed key sets |

**R29. M2 docs, `state: considered`, each with a `work/changes/<slug>.md` record**: the 5 slugs in `CAPS_M2`.
```sh
for s in $CAPS_M2; do c=docs/capabilities/$s.md; w=work/changes/$s.md
  [ "$(yq --front-matter=extract .state $c)" = considered ] || echo "STATE $c"
  test -f $w || { echo "MISSING $w"; continue; }
  [ "$(yq --front-matter=extract .type $w)" = Change ] || echo "TYPE $w"
  [ "$(yq --front-matter=extract .id $w)" = "change:$s" ] || echo "ID $w"
  yq --front-matter=extract '.changes[]' $w | grep -qx "cap:$s" || echo "CHANGES $w"
  yq --front-matter=extract '.contract_effect' $w | grep -qxE 'none|changes' || echo "EFFECT $w"
  [ "$(yq --front-matter=extract '.items|length' $w)" -gt 0 ] || echo "ITEMS $w"
  yq --front-matter=extract '.affects[]' $w | grep -vE "^cap:$s#(purpose|contract|usage|examples|constraints|explanation|verification)$" && echo "AFFECTS $w"
  grep -q "\[!PROPOSED change:$s\]" $c || echo "NOPROPOSAL $c"
done
```
-> no output. `ls work/changes/*.md | wc -l` -> `5`, or more if A8 adds M1 records.
- Every PROPOSED marker names an existing change: `git grep -ohE '\[!PROPOSED change:[a-z0-9-]+\]' -- docs | sed -E 's/.*change:([a-z0-9-]+)\]/\1/' | sort -u | while read c; do test -f work/changes/$c.md || echo "ORPHAN $c"; done` -> no output.
- Plan content is carried over:
  - entity-history mentions `git log --follow` and `-L`;
  - evidence-claims mentions persisted evidence and which statements depend on which result;
  - prose-assessment mentions invalidation when inputs change;
  - two-way-coverage mentions code->docs and tests->claims;
  - current-rationale mentions architecture pages and "conditions to revisit".

**R30. `docs/architecture/normal-form.md` and `docs/architecture/okf-profile.md`**, stating the current shared decisions.
- `test -s docs/architecture/normal-form.md && test -s docs/architecture/okf-profile.md` -> exit 0.
- `normal-form.md` contains `normal_form = 1`, the thresholds (20 / 30 / 6), the fingerprint definition, the 4 kinds, the version-bump rule and the golden-test rule.
- `okf-profile.md` defines the frontmatter profile and closed key sets for Capability and Change (and Architecture, A9), the `outcome` rule (A11), and where the SPDX comment lives (A13).
- Capabilities link to them: `grep -l 'architecture/normal-form.md' docs/capabilities/census-*.md` -> at least `census-normal-form.md`. `grep -l 'architecture/okf-profile.md' docs/capabilities/*.md` -> at least `okf-profile.md` and `workplan-document-model.md`.
- Every relative Markdown link under `docs/` resolves:
  ```sh
  for f in $(git ls-files 'docs/*.md'); do grep -oE '\]\([^)#: ]+\)' $f | sed -E 's/^\]\(//; s/\)$//' | while read l; do test -e "$(dirname $f)/$l" || echo "$f -> $l"; done; done
  ```
  -> no output.

**R31. Move the content of `INITIAL_PLANNING_PLAN.md` into these documents and remove the file.**
- `test ! -e INITIAL_PLANNING_PLAN.md` -> exit 0. `git diff --name-status origin/main...HEAD -- INITIAL_PLANNING_PLAN.md` -> `D	INITIAL_PLANNING_PLAN.md`.
- `git grep -n 'INITIAL_PLANNING_PLAN'` -> no output.
- Coverage map: each essay section must land somewhere. Check each key phrase with `git grep -il '<phrase>' -- docs` -> at least one file.

  | Essay section | Destination (Explanation unless noted) | Key phrase |
  |---|---|---|
  | Organizing principle | `workplan-document-model`, README positioning | `Plan changes to the documented system` |
  | 1. No mental amendments; the reconcile operation | `workplan-reconcile` | `amendment` |
  | 2. A permanent home; lifecycle table; roadmap | `workplan-document-model` (states), `workplan-cli` (`status`), `workplan-generated-docs` (catalogue) | `permanent`; `roadmap` |
  | 3. Design in the shape of docs; 7 sections; promotion needs reconciliation | `okf-profile`, `workplan-document-model`, `workplan-complete-gate` | `requires reconciliation` |
  | 4. Documentation obligations; `complete`; `contract_effect: none`; generated vs explanatory prose | `workplan-reconcile`, `workplan-complete-gate`, `workplan-generated-docs`, `workplan-prose-assessment` | `obligation`; `contract_effect` |
  | 5. Current rationale vs historical deliberation; architecture pages; entity history | `workplan-current-rationale`, `workplan-entity-history`, `docs/architecture/*` | `current rationale`; `historical deliberation` |
  | 6. Graph; two-way checks; evidence -> claims; the GMEOW Grounding Contract | `workplan-two-way-coverage`, `workplan-evidence-claims`, `source-index` | `Grounding` |
  | 7. Scenario A/B | Verification of `workplan-complete-gate` / `workplan-reconcile` | `Scenario A`; `Scenario B` |

- Essay text that must be rewritten, not copied: `CHANGE-17` (-> a slug); "implemented, deferred, or dropped" (-> `implemented | dropped | moved:change:<slug>`); the conversational voice ("Your second point", "I would") -> current-tense contract. R36-R38 check this.

**R32. `depends` integrity.**
- Every target exists: `for f in docs/capabilities/*.md; do yq --front-matter=extract '.depends[]' $f | while read d; do test -f docs/capabilities/${d#cap:}.md || echo "$f -> $d"; done; done` -> no output.
- No cycles: `for f in docs/capabilities/*.md; do s=$(basename $f .md); echo "$s $s"; yq --front-matter=extract '.depends[]' $f | sed "s/^cap://; s/^/$s /"; done | tsort >/dev/null` -> no "input contains a loop" message.

#### Acceptance criteria

**R33. `make check` passes on the empty workspace, and so does CI.**
- `make check` -> exit 0 locally, given the A1 resolution plus A14 (deny) and A15 (reuse).
- `gh pr checks <PR>` -> all checks pass, and the msrv job log shows `rustc 1.97.1`.
- `cargo metadata --format-version 1 --no-deps | jq '.packages|length'` -> `1` (only the A1 root package; no `raf-*` crate stubs).

**R34. `reuse lint` is clean.** Same as R19. Run it in a tree where `git status --porcelain --ignored` shows `.stage/` as ignored (`!!`), not untracked (`??`).

**R35. Every listed capability exists with all 7 sections, and with resolved, current-tense text where the state allows.**
- The R25 and R26 loops -> no output.
- Unresolved markers in `designed` docs: `for s in $CAPS_M1C $CAPS_M1W; do grep -nEi 'TBD|TODO|FIXME|\?\?\?|open question|to be decided' docs/capabilities/$s.md; done` -> no output.
- `considered` docs may carry open questions, but only in a clearly labelled form (inside the PROPOSED block, or an "Open questions" list), never as statements of current behaviour.
- Tense: `grep -nwE 'will|would|shall|should' docs/capabilities/*.md` should hit only inside `> [!PROPOSED` blocks or M2 docs. The grep finds candidates; a reviewer judges them.

**R36. No issue numbers in repo docs (the `.baseline` rule); identities are slugs.**
- `git grep -nE '(^|[^[:alnum:]&/])#[0-9]{1,5}\b|/(issues|pull)/[0-9]+|\bCHANGE-[0-9]+|\bPR ?#[0-9]+' -- . ':!*.png'` -> no output.
  - Validated: the regex catches `Per #928`, `.../issues/31` and `CHANGE-17`.
  - It does NOT match the colour tokens `#111214` or `#fff`, or Markdown `##` headings.
  - It reports nothing on purrdf's BRAND.md, SECURITY.md, LICENSING.md, CONTRIBUTING.md or its logo.
- Process-reference phrasing. purrdf's `scripts/check-issue-refs.py` also bans "the issue", "this issue", "this branch", "the plan's", "Task N" and "AC1", all under the `.baseline` rule against referencing development. Check: `git grep -niE "\b(this|the) issue('s)?\b|this branch|the plan('s)?\b|\btask [0-9]+|\bAC[0-9]+\b" -- '*.md' ':!.baseline'` -> no output. Issue-template wording such as "Security issue" lives in `.yml` files and is out of this grep's scope.

#### Implied requirements (plan conventions section and house rules)

**R37. No deferrals.**
- `git grep -niE 'defer|follow-?up|left for future|future work' -- . ':!.baseline'` -> at most one expected hit: `workplan-document-model.md` stating that deferral is not an outcome. Any other hit is a defect.
- Change outcomes are only `implemented | dropped | moved:change:<slug>`: `yq --front-matter=extract '.items[].outcome // ""' work/changes/*.md | grep -vE '^(|implemented|dropped|moved:change:[a-z0-9-]+)$'` -> no output.

**R38. Slug identities.**
- Every `id` matches `^(cap|change):[a-z0-9]+(-[a-z0-9]+)*$` and equals its file stem: `for f in docs/capabilities/*.md work/changes/*.md; do i=$(yq --front-matter=extract .id $f); [ "${i#*:}" = "$(basename $f .md)" ] || echo $f; done` -> no output.

**R39. No attribution text** (the plan says "No attribution text anywhere", and the user's global attribution policy applies).
- The machine-wide commit-msg and pre-push attribution guards (`stage-attribution-commit`, `stage-attribution-push`) accept every commit on the branch: `git push` succeeds without a guard refusal.
- `git grep -niE 'generated (by|with)'` -> no output. The file name `CLAUDE.md` is a house file, not attribution.

**R40. `.gitignore`.** Not listed in the issue, but R19 and R34 need it.
- `git check-ignore -q .stage/brief.json && echo ok` -> `ok`. `/target` is ignored too.
- `.gitignore` carries an SPDX header or is annotated in `REUSE.toml`.

**R41. `Cargo.lock` is committed.** `--locked` in make check and CI, and deterministic cargo-deny results, depend on it.
- `git ls-files Cargo.lock` -> `Cargo.lock`, and REUSE.toml covers it.

**R42. The root stub stays minimal and does not pre-empt the shared-crates work.**
- The root package has `publish = false`, no dependencies, and a single documented `src/lib.rs` with an SPDX header.
- `cargo metadata --format-version 1 --no-deps | jq '.packages[0].dependencies|length, .packages[0].publish'` -> `0` and `[]`.

**R43. Brand casing**: "RAF" in prose, lowercase only in machine ids.
- `git grep -nwE 'Raf|rAF' -- '*.md'` -> no output.

**R44. Signed, conventional commits.** The machine-wide `stage-signed-refs` hook rejects unsigned refs, and `cliff.toml` depends on conventional subjects.
- `git log --show-signature origin/main..HEAD` -> a good signature on every commit.
- `git log origin/main..HEAD --format=%s | grep -vE '^(feat|fix|docs|chore|build|ci|refactor|test|style|perf)(\([a-z0-9-]+\))?!?: '` -> no output.

---

### Dependencies

#### Depends On
- None.

#### Blocks (from the plan table)
- The shared crates raf-okf + raf-source (they need the workspace, the lints, and the `okf-profile`/`source-index` specs).
- The raf-workplan model (it needs these capability and change documents as its first corpus).
- Transitively: census core, ledger, detectors and CLI; the workplan lifecycle; scenarios A/B and the self-gate; the M2 essay work.

#### Related sources
- `~/Active/purrdf`: document and brand sources.
- `~/Active/logit-loom`: lints, toolchain, `AGENTS.md`, `CLA.md`.
- `~/Active/gmeow-gts/REUSE.toml` and `~/Active/lillith_memetics/deny.toml`: templates.
- The gmeow-ontology Grounding Contract, which the essay cites.

### Context from Discussion
- The issue has no comments. Decisions come from the plan:
  - `raf-*` libraries with `cargo-*` binaries; normal form v1;
  - slugs, never numbers; outcomes `implemented | dropped | moved:<change-slug>`;
  - the change file is deleted on `complete`;
  - M1 workplan scope, with the rest of the essay as M2 `considered` docs.
- Rejected approaches (from the plan):
  - ADR/PRD amendment chains (the essay's core critique);
  - numeric ids like `CHANGE-17`;
  - a "deferred" outcome;
  - rustflags-based lints (a logit-loom comment explains that rustflags leak into sibling path dependencies).

### Ambiguities and proposed resolutions

| # | Ambiguity | Evidence | Proposed resolution |
|---|---|---|---|
| A1 | **"`make check` passes on the empty workspace".** A member-less virtual workspace makes fmt, clippy, test, doc and deny fail, and a `crates/*` glob with no match fails too | verified table above | Make the root manifest a **non-virtual root package**:<br>- name it e.g. `raf-dogfood` or `rust-agentic-foundations`;<br>- `publish = false`, no dependencies, `src/lib.rs` with one `//!` line, `[lints] workspace = true`;<br>- inherit every `[workspace.package]` field so the nightly unused-field lint stays quiet;<br>- `members = []` for now.<br>This matches the plan's root-level `tests/self_workplan.rs`, which needs a root package anyway. It creates no `raf-*` crate, so it does not pre-empt the shared-crates issue.<br>Rejected: stub `crates/raf-okf` etc. (pre-empts that work and yields empty published names); skipping cargo steps when there are no members (violates the no-optionality rule). |
| A2 | MSRV: the issue's `make check` list omits msrv, but CI has an msrv job and the plan's Verification includes "the MSRV build" | issue vs plan lines 168 and 177 | Add `rust-validate 1.97.1 cargo check --workspace --all-targets --locked` to `make check` (the toolchain is installed locally; verified). The CI msrv job pins its rustup toolchain env to 1.97.1. Document `rust-validate` in CONTRIBUTING, since other contributors may not have it. |
| A3 | `reuse lint` is in `make check` but not in the CI job list. "So does CI" implies parity | issue Scope vs Acceptance | Add a reuse step to CI (`fsfe/reuse-action` or `pipx run reuse lint`). Otherwise CI cannot catch a missing header. |
| A4 | Root `LICENSE-*` files only, but REUSE requires `LICENSES/<id>.txt` | reproduced "Missing licenses" | Ship both (R11b). `LICENSES/MulanPSL-2.0.txt` must be byte-identical to `LICENSE-MULAN`. |
| A5 | The issue says "house templates" and the plan says "copied from logit-loom" for `.goals`, `.baseline`, `.deficiencies`, `deny.toml`, `clippy.toml` and `.githooks`, but logit-loom has none of them | `ls -a` survey | Take `.goals`, `.baseline`, `.deficiencies`, `.githooks` and `clippy.toml` from purrdf, and `deny.toml` from lillith_memetics, then adapt them.<br>- `.goals` needs a real rewrite: drop RDF 1.2, the Python surface and WASM portability.<br>- Keep `.baseline` as purrdf's text: it has the issue-number ban, which lillith_decodes' variant lacks. |
| A6 | The Chinese doc is "with names substituted", but its source also has purrdf-only sentences (Unicode tables, the YAML emitter, XML data, Python/npm/WASM/C packages) | purrdf `docs/LICENSING.zh-Hans.md` paragraphs 3-4 | - Delete those sentences, and reduce the distribution sentence to Cargo packages only.<br>- Keep every other sentence byte-identical to purrdf apart from the name, so little new, unreviewed Chinese is written.<br>- Then produce a fresh independent backtranslation record (R13). |
| A7 | LICENSING mentions "the project CLA". purrdf has no `CLA.md`; logit-loom has one | `ls` | Keep the CLA text inside LICENSING.md and CONTRIBUTING.md, as purrdf does (issue scope). Adding a `CLA.md` adapted from logit-loom (with the three-licence expression) is optional. Ask before adding unlisted files. |
| A8 | M1 change records. The scenarios/self-gate issue expects to "complete the M1 change records", but this issue only creates M2 records | plan row 9; build step 6 | Decision needed. Recommended: also create `work/changes/<slug>.md` for each M1 capability now (`contract_effect: changes`, items = the designed claims). Then the `designed` text is backed by a tracked change, and later completion has something to complete. Otherwise a later issue must create them retroactively. |
| A9 | Architecture-doc frontmatter is undefined: Capability and Change are the only defined types | plan's Document model | Define `type: Architecture` (`id: arch:<slug>`, `title`, `description`) in `docs/architecture/okf-profile.md` and use it on both architecture pages, or use no frontmatter. Either way, record the choice in okf-profile, so the later `check` has a rule to apply. |
| A10 | `docs/capabilities/index.md` (the generated catalogue) appears in the plan, but generation belongs to the workplan model work | plan's Generated blocks | Do not create it now. A hand-written index would be a non-Capability file in a closed directory, and stale by construction. |
| A11 | How to write a `considered` doc (PROPOSED blocks or plain text), and Change `items` that have no outcome yet | plan: "Outside those blocks, the text states the current contract only"; the complete gate requires "every item to have an outcome" | - M2 docs: Purpose and open questions in labelled plain text; the candidate contract inside `> [!PROPOSED change:<slug>]` blocks that reference the existing change.<br>- Pending items carry `claim` only; `outcome` is added at completion.<br>- okf-profile must declare `outcome` optional until completion, so the closed-key check does not reject pending items. |
| A12 | `bindings` for designed and considered docs, whose symbols don't exist yet | Capability schema | Use `bindings: {code: [], tests: [], examples: []}`. Never list future symbols: the later binding-resolution check would fail on every one of them. |
| A13 | Where the SPDX header goes in frontmatter docs, and which licence each doc gets | reuse test | - Frontmatter docs: `# SPDX-FileCopyrightText:` and `# SPDX-License-Identifier: CC-BY-4.0` as YAML comments inside the frontmatter (verified compliant). Other docs use HTML comments.<br>- purrdf puts SECURITY.md and the logos under the code licence. Either follow it or move both to CC-BY-4.0; decide once and record it in REUSE.toml comments. |
| A14 | `cargo deny check` (advisories) cannot fetch locally: the machine-wide signature-guard hook blocks the advisory-db update | reproduced | Do not bypass the hook. Two options:<br>(a) `make check` runs `cargo deny --offline check` against the cached db, and CI runs `cargo deny check` with fetch.<br>(b) The memetics pattern: `make check` runs `cargo deny check bans licenses sources`, and advisories run in a separate `make audit` and in CI.<br>Prefer (a): all four checks stay local. Refreshing the cache is the user's decision. |
| A15 | `reuse` is not installed, and the house rule is hard failure with no optional tools | `which reuse` | `make check` invokes `reuse lint` and fails hard when it is missing, with an install hint (`pipx install reuse`, or `uvx reuse`). Installing the tool is the user's call; the agent must not silently skip the step. `uvx reuse lint` works today with network access. |
| A16 | SECURITY's "same process" includes a supported-versions table, but nothing is released | purrdf's `0.1.x` table | Keep the table, with RAF's pre-release status (e.g. latest `0.x` only) and no purrdf versions. |
| A17 | README "what each crate does", but none of the crates exists yet | the essay: planned work must not read as available behaviour | Describe each crate with its designed status and link to its capability docs. Put the usage snippets under a "Planned usage" heading or mark them not yet published. Do not include the README into a crate's rustdoc until the APIs exist; it would become a failing doctest. |
| A18 | `okf-profile` exists both as a capability and as an architecture page | the issue lists both | The architecture page holds the shared, current profile decision: keys, closed sets, types. The `okf-profile` capability holds the `raf-okf` crate's reader/writer/validator contract and links to the page. One source of truth per fact (essay section 5). |
| A19 | logit-loom's lint set also has `unused_qualifications = "warn"` and clippy allows for `module_name_repetitions` and `must_use_candidate` | logit-loom `Cargo.toml` | Copy them, each allow with a preceding `#` reason comment (R2). Add `allow_attributes_without_reason = "deny"` to enforce the reason rule in code. |

### Risk Assessment

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| K1 | Local `make check` fails on deny (signature-guard hook) or reuse (not installed), so "passes" cannot be demonstrated | HIGH | HIGH | A14 option (a) and A15. Verify `cargo deny --offline check` and `uvx reuse lint` before claiming a pass. Never disable the hook. |
| K2 | `reuse lint` fails on files outside the deliverable (`.stage/`, a `target` symlink, untracked agent directories) | HIGH | MED | Ignore `/.stage/` and `/target` in `.gitignore`. Run reuse from a clean `git status`. |
| K3 | The backtranslation's provenance is weak or fabricated (the agent that wrote the English also translates back; invented timestamps) | MED | HIGH | Spawn a context-isolated translator that gets only the zh-Hans file. Record real UTC timestamps, the byte count and the SHA-256 at read time. Re-run whenever the Chinese source changes (the R13 hash check catches drift). State plainly that it is a model backtranslation, not legal review. |
| K4 | The logo's cat-head-core is altered by reformatting (editor or SVG optimiser), or an accent leaks through a shared `<style>` or `class` | MED | MED | The R22 hash and isolation script. Keep accent fills as attributes inside `service-toolbox`. Do not run svgo. Check that the `mouth-bite` overlay (a separate group above the service object) still reads correctly over the toolbox handle. |
| K5 | Capability docs present `designed` behaviour as delivered, or carry deferral or process language copied from the essay (`CHANGE-17`, "deferred", "I would") | HIGH | HIGH | The R35-R38 greps. Write from the plan's decisions in current-tense contract voice, and move the essay's rationale into Explanation. |
| K6 | `social-preview.png` differs between machines (font fallback for "DejaVu Sans") | MED | LOW | Render locally with the documented command and commit the PNG. Byte-compare only locally (R23); do not gate CI on a re-render. |
| K7 | `-D warnings` does not catch nightly cargo manifest warnings (unused workspace fields, unknown keys) | MED | LOW | The third R1 check. Optionally, add a `make check` step that fails on `^warning:` in `cargo metadata` stderr. |
| K8 | The CI msrv job silently runs nightly because `rust-toolchain.toml` takes precedence | MED | MED | Pin the msrv job's rustup toolchain env to 1.97.1 and add a `rustc --version` assertion step (R7). |
| K9 | Scope: 21 capability docs, 2 architecture docs and about 30 project files in one PR, so review fatigue hides schema drift | HIGH | MED | Run the R25, R26, R29 and R32 loops mechanically before review. Commit the slices separately (workspace, legal, brand, capabilities). |
| K10 | The pre-commit hook is too slow, or does not check the index faithfully | LOW | MED | Use purrdf's `checkout-index` snapshot pattern. Keep only fast gates in the hook (fmt, reuse, the issue-ref grep); clippy and tests stay in `make check` and CI. |
| K11 | The closed-key frontmatter chosen now conflicts with the later `raf-okf` implementation (SPDX comment placement, `outcome` optionality) | MED | MED | Record these decisions explicitly in `docs/architecture/okf-profile.md` (A9, A11, A13), so the crate implements the doc rather than the other way round. |
| K12 | The `stage-signed-refs` hook rejects unsigned commits at commit or push time | MED | LOW | Use the normal signed-commit flow. Never use `--no-verify`. |

### Technical Notes

#### Affected areas (all new, except the deleted essay)
- Root build and tooling: `Cargo.toml`, `Cargo.lock`, `src/lib.rs` (A1), `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `Makefile`, `REUSE.toml`, `.gitignore`.
- Root licences: `LICENSE-{MIT,APACHE,MULAN}`, `LICENSES/{CC-BY-4.0,MIT,Apache-2.0,MulanPSL-2.0}.txt`.
- Root documents: `LICENSING.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CITATION.cff`, `cliff.toml`, `CHANGELOG.md`, `README.md`.
- Root house files: `AGENTS.md`, `CLAUDE.md`, `.goals`, `.baseline`, `.deficiencies`.
- `.githooks/pre-commit`, `.github/workflows/ci.yaml`, `.github/ISSUE_TEMPLATE/*.yml`, `.github/PULL_REQUEST_TEMPLATE.md`.
- `docs/`: `BRAND.md`, `raf-logo.svg`, `social-preview.{svg,png}`, `LICENSING.zh-Hans.md`, `LICENSING-BACKTRANSLATION.md`, `architecture/{normal-form,okf-profile}.md`, `capabilities/<21 slugs>.md`.
- `work/changes/<5 M2 slugs>.md`, plus M1 records if A8 is accepted.
- Deleted: `INITIAL_PLANNING_PLAN.md`.

#### Potential challenges
- The 9 census capability docs draw their substance only from the plan file outside the repo: thresholds, ledger schema, detector list, CLI. After this PR the repo docs are the only durable record, so every plan decision must carry over (R27 spot checks).
- Turning a conversational essay into contract-voice Explanation sections without losing its rationale or importing its banned vocabulary.
- Drawing an original toolbox-and-wrench service object in the 1024 viewBox, under the shared mouth overlay. purrdf's service object occupies roughly x 290-710, y 620-830.

### Related PRs
- None (no PR references issue #1 yet).
