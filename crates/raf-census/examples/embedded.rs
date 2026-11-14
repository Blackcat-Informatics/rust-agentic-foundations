// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Run embedded structural matching over a helper copied into a larger function.
//!
//! The scratch workspace holds `helper`, and `larger` contains the same three
//! `let` statements and tail under renamed locals, with an unrelated earlier
//! local in front. The production run loads the workspace from disk, analyzes it
//! and prints each verified region pair with its original span. Consumer code is
//! inspected as data and never executed.

use std::{error::Error, fs};

use purrdf_testkit::TempDir;
use raf_census::embedded::{Options, Run};
use raf_source::{DiskTree, LoadOptions, Workspace};

const SOURCE: &str = "\
pub fn helper(input: u32) -> u32 {
    let scaled = input * 3 + 7;
    let shifted = scaled - input / 2;
    let total = shifted * scaled + input;
    total % 11 + total
}

pub fn larger(value: u32) -> u32 {
    let earlier = 9;
    log(earlier);
    let amount = value * 3 + 7;
    let remainder = amount - value / 2;
    let sum = remainder * amount + value;
    sum % 11 + sum
}

fn log(_: u32) {}
";

fn main() -> Result<(), Box<dyn Error>> {
    let scratch = TempDir::new_in(std::env::temp_dir())?;
    let root = scratch.path();
    fs::create_dir(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )?;
    fs::write(root.join("src/lib.rs"), SOURCE)?;

    let tree = DiskTree::new(root)?;
    let workspace = Workspace::load_with(&tree, &["."], &LoadOptions::default())?;
    let run = Run::analyze(&workspace, &Options::default())?;

    let witnesses = run.witnesses().collect::<Vec<_>>();
    assert!(
        !witnesses.is_empty(),
        "the copied helper must be verified as an embedded clone"
    );
    for witness in &witnesses {
        let (left, right) = (witness.left(), witness.right());
        println!(
            "{}:{:?} lines {:?} <-> {}:{:?} lines {:?}",
            left.function().file().path.as_str(),
            left.bytes(),
            left.lines(),
            right.function().file().path.as_str(),
            right.bytes(),
            right.lines(),
        );
        println!("left:\n{}", left.text());
        println!("right:\n{}", right.text());
    }
    Ok(())
}
