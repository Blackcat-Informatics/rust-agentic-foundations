// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Run embedded structural matching over a helper copied into a larger function.
//!
//! The scratch workspace holds `helper`, and `larger` contains the same three
//! `let` statements and tail under renamed locals, with an unrelated earlier
//! local in front. A neighbouring variant changes one operator and yields no
//! witness. The production run loads the workspace from disk, analyzes it and
//! returns each verified region pair with its original span. The integration
//! test that includes this file asserts those spans. Consumer code is inspected
//! as data and never executed.

use std::{error::Error, fs, ops::Range, path::Path};

use purrdf_testkit::TempDir;
use raf_census::embedded::{Options, Run};
use raf_source::{DiskTree, LoadOptions, Workspace};

/// `larger` holds a verbatim copy of `helper`'s three `let` statements and tail.
pub(crate) const COPY_SOURCE: &str = "\
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

/// The same shape with one operator changed in the copied statements.
pub(crate) const NEIGHBOUR_SOURCE: &str = "\
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
    let remainder = amount + value / 2;
    let sum = remainder * amount + value;
    sum % 11 + sum
}

fn log(_: u32) {}
";

/// One side of a verified pair, owned so that the scratch workspace can go.
#[derive(Debug)]
pub(crate) struct Side {
    /// The name of the first indexed owner of the physical function.
    pub(crate) function: String,
    /// The original, half-open byte range of that function's body.
    pub(crate) body: Range<usize>,
    /// The original, half-open byte range of the verified region.
    pub(crate) bytes: Range<usize>,
    /// The inclusive line range of the verified region.
    pub(crate) lines: std::ops::RangeInclusive<usize>,
    /// The original text of the verified region.
    pub(crate) text: String,
}

/// A verified pair of regions, as the production run returns them.
#[derive(Debug)]
pub(crate) struct Pair {
    pub(crate) left: Side,
    pub(crate) right: Side,
}

/// Write `source` into a scratch crate under the repository target area, run the
/// production embedded analysis over it and return every verified pair.
pub(crate) fn witnesses(source: &str) -> Result<Vec<Pair>, Box<dyn Error>> {
    let parent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/embedded-examples");
    fs::create_dir_all(&parent)?;
    let scratch = TempDir::new_in(&parent)?;
    let root = scratch.path();
    fs::create_dir(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )?;
    fs::write(root.join("src/lib.rs"), source)?;

    let tree = DiskTree::new(root)?;
    let workspace = Workspace::load_with(&tree, &["."], &LoadOptions::default())?;
    let run = Run::analyze(&workspace, &Options::default())?;
    Ok(run
        .witnesses()
        .map(|witness| {
            let side = |region: &raf_census::embedded::Region<'_>| Side {
                function: region.function().owners()[0].name.to_string(),
                body: region.function().body(),
                bytes: region.bytes(),
                lines: region.lines(),
                text: region.text().to_owned(),
            };
            Pair {
                left: side(witness.left()),
                right: side(witness.right()),
            }
        })
        .collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    for (label, source) in [("copy", COPY_SOURCE), ("neighbour", NEIGHBOUR_SOURCE)] {
        let pairs = witnesses(source)?;
        println!("{label}: {} verified pair(s)", pairs.len());
        for pair in &pairs {
            for side in [&pair.left, &pair.right] {
                println!(
                    "  {} body {:?} bytes {:?} lines {:?}",
                    side.function, side.body, side.bytes, side.lines
                );
                println!("{}", side.text);
            }
        }
    }
    Ok(())
}
