// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Load one workspace from disk, read-only, list the public surface of each
//! library target, resolve every path it lists, and print a digest of all
//! of it, so that the instructions a load of a real workspace costs can be
//! counted from outside and the answers of two trees compared.
//!
//! ```text
//! cargo build --release -p raf-source --example ws_cost
//! perf stat -e instructions:u target/release/examples/ws_cost <workspace root>
//! ```
//!
//! The workspace is loaded through a `DiskTree` at the root given, with
//! `"."` as the one root and the default options. For each library target,
//! in the order of the targets, its exported items are listed, sorted, each
//! with the path of the item it names and its kind, and each listed path is
//! resolved; a listing that is refused is written as its error. The last
//! line counts the libraries and the lines and gives the 64-bit FNV-1a
//! digest of the lines. A load that is refused, or a root that cannot be
//! opened, ends the run with a failure and prints no count. No clock is read.

use std::fmt::Write as _;
use std::process::ExitCode;

use raf_source::{DiskTree, TargetKind, Workspace};

/// The 64-bit FNV-1a digest of `text`.
fn digest(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn run(args: &[String]) -> Result<String, String> {
    let [root] = args else {
        return Err("usage: ws_cost <workspace root>".to_owned());
    };
    let tree = DiskTree::new(root).map_err(|error| format!("cannot open {root}: {error}"))?;
    let ws = Workspace::load(&tree, &["."])
        .map_err(|error| format!("the load of {root} is refused: {error}"))?;
    let mut out = String::new();
    let mut libraries = 0;
    let ids: Vec<_> = ws.target_ids().collect();
    for id in ids {
        if ws
            .target(id)
            .is_none_or(|target| target.kind != TargetKind::Lib)
        {
            continue;
        }
        libraries += 1;
        match ws.exported_items(id) {
            Ok(exports) => {
                let mut items: Vec<(String, String)> = exports
                    .items
                    .iter()
                    .map(|exported| {
                        let defined = ws
                            .item(exported.item)
                            .map_or_else(String::new, |item| item.path.to_string());
                        (
                            exported.path.clone(),
                            format!("{} = {defined} ({:?})", exported.path, exported.kind),
                        )
                    })
                    .collect();
                items.sort();
                for (path, line) in &items {
                    let answer = ws.resolve(path).map(|found| (found.path, found.via_glob));
                    writeln!(out, "{line}\n  -> {answer:?}").expect("write");
                }
            }
            Err(error) => writeln!(out, "refused {error}").expect("write"),
        }
    }
    Ok(format!(
        "libraries={libraries} lines={} digest={:016x}",
        out.lines().count(),
        digest(&out)
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("ws_cost: {error}");
            ExitCode::FAILURE
        }
    }
}
