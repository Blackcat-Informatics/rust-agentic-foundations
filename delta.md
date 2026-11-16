# Merging reading for the autolib library-target change

- Read: crates/raf-source/src/target.rs (the early return in `Context::library` when there is no `[lib]` and `package.auto("lib")` is false) and the new test in crates/raf-source/tests/discovery_targets.rs.
- The check is placed before the explicit-library logic, so an explicit `[lib]` is unaffected. Default discovery is unaffected because the early return needs the flag to be off.
- The docs and the renewed assessment were reviewed by the independent reviewer; the capability text states the same rule as the code.
