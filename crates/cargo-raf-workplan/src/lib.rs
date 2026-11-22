// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Check the capability documents in a Rust repository with Workplan.
//!
//! Install the command with `cargo install --locked cargo-raf-workplan`.
//! Run `cargo raf-workplan check` at the repository root to verify bindings
//! and assessments. Run `cargo raf-workplan status` to inspect the current
//! state or `cargo raf-workplan --help` to list commands and options.
//!
//! These checks inspect source and documents without running the repository's
//! tests or examples. Use the [Workplan guide] to set up capability documents,
//! regenerate their catalogue and reconcile changes. Use the [`raf-workplan`
//! library] for Rust integrations.
//!
//! [Workplan guide]: https://github.com/Blackcat-Informatics/rust-agentic-foundations/blob/main/WORKPLAN-HOWTO.md
//! [`raf-workplan` library]: https://docs.rs/raf-workplan
