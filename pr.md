## Summary

`cargo census init` creates the canonical starter ledger at the root that `check` discovers. The ledger selects the entire workspace and grants no debt. The library exposes the same operation as `Gate::init_ledger`.

Initialization verifies workspace source paths and publishes through the existing Linux directory lock and exclusive file creation path. It preserves existing ledger and debt bytes, including when the workspace already contains duplicates. The consumer guide now installs the revision of the guide being read and walks through the first check.

## Verification

- The two focused `delivery` cases cover initialization from a nested directory, a busy writer, a clean check, a planted duplicate, existing policy and debt, a missing Rust source and Git administration.
- The `raf-census` debt example exercises existing whole-body and locked debt publication, including its busy-writer diagnostic.
- Clippy passed for the `raf-census` and `cargo-census` production targets with warnings denied.
- Workplan completed the capability change. The contract, usage and dependent assessments were renewed against their current inputs.

Closes #380.
