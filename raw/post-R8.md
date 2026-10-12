### Stage 2 fix R8 (`cb90d10`): CI pins and merge-commit gates

**Gaps closed (G11 / S-L1, S-L2, G16 F4/F5/F9, S-L5)**
- **Pinned tools.**
  - `cargo install --locked cargo-deny@0.20.2` matches the local version.
  - uv is pinned to `0.13.0`, with `enable-cache: false`.
- **Toolchain action repin.** `dtolnay/rust-toolchain` is repinned to the master head `e2a55d2…`, which is never rebuilt. `git ls-remote` confirms it, and the compare against the input-handling fix `83d0610` reports `ahead`. Every job passes `toolchain:` explicitly.
- **Concurrency group.** Superseded PR runs are cancelled.
- **`.githooks/pre-merge-commit`.** It runs the fast gates. In a scratch clone with real signing, it refused a no-conflict merge that brought in a staged `#12`.
- **Dependabot.** Added for the pinned actions.
- **`$(ROOT)` quoting.** Already quoted in R6; verified with a path that contains a space.

**Checks run**
- `actionlint` rc 0.
- zizmor reports no findings (offline mode).
- `cargo deny --offline check` ok.
- reuse 81/81.
- The hook passed.

**Review:** `G-R8-review` gave PASS.
