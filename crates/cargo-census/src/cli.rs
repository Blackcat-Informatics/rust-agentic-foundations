// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Cargo and direct-executable argument shapes.

use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use raf_workplan::exec::{ExampleName, Package};
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
    /// Inspect declared native ledger policy without executing it.
    DumpLedger {
        /// Native RDF 1.2 report carrier.
        #[arg(long, value_enum, default_value = "ntriples")]
        format: RdfFormat,
        /// Write the complete ledger here instead of stdout, relative to the caller.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Discover structural duplication without evaluating or writing policy.
    Baseline {
        /// Native RDF 1.2 report carrier.
        #[arg(long, value_enum, default_value = "ntriples")]
        format: RdfFormat,
        /// Write the complete report here instead of stdout, relative to the caller.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Read the complete native index from the repository's registered Gate.
    Index {
        /// Package containing the repository-owned index example.
        #[arg(long, value_parser = package)]
        package: Package,
        /// Example accepting the explicit index-v1 request.
        #[arg(long, value_parser = example)]
        example: ExampleName,
        /// Native RDF 1.2 report carrier.
        #[arg(long, value_enum, default_value = "ntriples")]
        format: RdfFormat,
        /// Write the complete report here instead of stdout, relative to the caller.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Inspect every classified physical match of one configured job.
    Job {
        /// Exact configured job identity.
        id: String,
        #[command(flatten)]
        target: QueryTarget,
    },
    /// Inspect one configured job's exact nonnegative copy count.
    Count {
        /// Exact configured job identity.
        id: String,
        #[command(flatten)]
        target: QueryTarget,
    },
    /// Inspect every physical occurrence and optional fingerprints of a symbol.
    Fingerprint {
        /// Exact indexed symbol; all matching source sites are retained.
        symbol: String,
        #[command(flatten)]
        target: QueryTarget,
    },
    /// Run the repository-owned registered check through its native RDF response.
    Check {
        /// Package containing the repository-owned check example.
        #[arg(long, value_parser = package)]
        package: Package,
        /// Example that emits the registered check-v1 response and matching exit.
        #[arg(long, value_parser = example)]
        example: ExampleName,
    },
    /// Emit a migrated RDF ledger to stdout without modifying either input.
    Migrate(MigrationArgs),
    /// Record present debt through `Gate::write_debt`, refusing growth.
    Debt {
        /// Publish the exact present record.
        #[arg(long, required = true)]
        write: bool,
        /// Repository package owning the registered debt writer.
        #[arg(long, value_parser = package, requires = "example")]
        package: Option<Package>,
        /// Example accepting the explicit versioned write request.
        #[arg(long, value_parser = example, requires = "package")]
        example: Option<ExampleName>,
    },
}

#[derive(ClapArgs)]
pub(crate) struct QueryTarget {
    /// Package containing the repository-owned index example.
    #[arg(long, value_parser = package)]
    pub(crate) package: Package,
    /// Example accepting the existing index-v1 request.
    #[arg(long, value_parser = example)]
    pub(crate) example: ExampleName,
    /// Native RDF 1.2 query carrier.
    #[arg(long, value_enum, default_value = "ntriples")]
    pub(crate) format: RdfFormat,
    /// Write the complete query here, relative to the caller, instead of stdout.
    #[arg(long)]
    pub(crate) out: Option<PathBuf>,
}

#[derive(ClapArgs)]
pub(crate) struct MigrationArgs {
    /// Explicit foreign TOML/JSON import, rather than older RDF ledger/index.
    #[arg(long)]
    pub(crate) legacy: bool,
    /// Named older ledger input for fingerprint migration or foreign import.
    #[arg(long, required_unless_present = "report_form")]
    pub(crate) ledger: Option<PathBuf>,
    /// Canonical older RDF index (fingerprint migration only).
    #[arg(long, required_unless_present_any = ["legacy", "report_form"], conflicts_with = "legacy")]
    pub(crate) index: Option<PathBuf>,
    /// Foreign JSON baseline (required with --legacy).
    #[arg(long, required_if_eq("legacy", "true"), requires = "legacy")]
    pub(crate) baseline: Option<PathBuf>,
    /// Convert exactly form 3 to the current layout, preserving native facts.
    #[arg(long, value_enum, requires_all = ["input", "format"], conflicts_with_all = ["legacy", "ledger", "index", "baseline"])]
    pub(crate) report_form: Option<UpgradeKind>,
    /// Named carrier to convert without source evaluation or input writes.
    #[arg(long, requires = "report_form")]
    pub(crate) input: Option<PathBuf>,
    /// Explicit syntax of both the original and converted carrier.
    #[arg(long, value_enum, requires = "report_form")]
    pub(crate) format: Option<RdfFormat>,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum UpgradeKind {
    Baseline,
    Index,
    Ledger,
    Debt,
}

impl UpgradeKind {
    pub(crate) const fn native(self) -> raf_census::ReportKind {
        match self {
            Self::Baseline => raf_census::ReportKind::Baseline,
            Self::Index => raf_census::ReportKind::Index,
            Self::Ledger => raf_census::ReportKind::Ledger,
            Self::Debt => raf_census::ReportKind::Debt,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum RdfFormat {
    Ntriples,
    Turtle,
}

impl RdfFormat {
    pub(crate) const fn native(self) -> raf_rdf::Format {
        match self {
            Self::Ntriples => raf_rdf::Format::NTriples,
            Self::Turtle => raf_rdf::Format::Turtle,
        }
    }
}

fn package(value: &str) -> Result<Package, String> {
    Package::new(value).map_err(|error| error.to_string())
}

fn example(value: &str) -> Result<ExampleName, String> {
    ExampleName::new(value).map_err(|error| error.to_string())
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
