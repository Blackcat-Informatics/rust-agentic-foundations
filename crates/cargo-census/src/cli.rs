// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Cargo and direct-executable argument shapes.

use clap::{Args as ClapArgs, Parser, Subcommand};
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo", disable_help_subcommand = true)]
enum Cargo {
    /// Inspect a repository's structural duplication policy.
    Census(Args),
}

#[derive(ClapArgs)]
#[command(version, arg_required_else_help = true, disable_help_subcommand = true)]
pub(crate) struct Args {
    /// Directory inside the workspace; ancestor manifests locate its root.
    #[arg(long, default_value = ".", global = true)]
    pub(crate) workspace: PathBuf,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Emit a migrated RDF ledger to stdout without modifying either input.
    Migrate {
        /// Explicit foreign TOML/JSON import, rather than older RDF ledger/index.
        #[arg(long)]
        legacy: bool,
        /// Named older ledger input.
        #[arg(long)]
        ledger: PathBuf,
        /// Canonical older RDF index (modern mode only).
        #[arg(long, required_unless_present = "legacy", conflicts_with = "legacy")]
        index: Option<PathBuf>,
        /// Foreign JSON baseline (required with --legacy).
        #[arg(long, required_if_eq("legacy", "true"), requires = "legacy")]
        baseline: Option<PathBuf>,
    },
    /// Record present debt through `Gate::write_debt`, refusing growth.
    Debt {
        /// Publish the exact present record.
        #[arg(long, required = true)]
        write: bool,
    },
}

pub(crate) fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Args, clap::Error> {
    let mut words: Vec<_> = args.into_iter().collect();
    if words.is_empty() {
        words.push("cargo-census".into());
    }
    if words.get(1).is_none_or(|word| word != "census") {
        words.insert(1, "census".into());
    }
    let Cargo::Census(args) = Cargo::try_parse_from(words)?;
    Ok(args)
}
