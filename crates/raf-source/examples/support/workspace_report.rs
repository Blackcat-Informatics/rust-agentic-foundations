// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The actual workspace surface report shared by the native load consumers.

use raf_source::{TargetKind, Workspace};
use std::fmt::Write as _;

pub(crate) fn report(ws: &Workspace) -> String {
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
    format!(
        "libraries={libraries} lines={} digest={:016x}",
        out.lines().count(),
        purrdf_hash::fnv::fnv1a64(out.as_bytes())
    )
}
