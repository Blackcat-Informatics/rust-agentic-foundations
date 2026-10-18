// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Loading a crate in every order of its declarations.

use raf_source::{Namespace, ResolveError, Workspace};

/// Every order of `items`.
pub(crate) fn permutations(items: &[String]) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for position in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(position);
        for mut tail in permutations(&rest) {
            tail.insert(0, head.clone());
            all.push(tail);
        }
    }
    all
}

/// What `module` reads `path` as in `ns` when it writes it in a `use`: the
/// canonical path, or the kind of refusal.
pub(crate) fn read_as(ws: &Workspace, module: &str, path: &str, ns: Namespace) -> String {
    match ws.resolve_from_in(module, path, ns) {
        Ok(found) => found.path,
        Err(ResolveError::NotFound(_)) => "not found".to_owned(),
        Err(ResolveError::Ambiguous(ids)) => format!(
            "ambiguous {:?}",
            ids.iter()
                .map(|id| ws.item(*id).expect("item").path.to_string())
                .collect::<Vec<_>>()
        ),
        Err(other) => format!("{other:?}"),
    }
}

/// What [`read_as`] answers for a name two items answer to.
pub(crate) fn ambiguous_between(first: &str, second: &str) -> String {
    format!("ambiguous [\"{first}\", \"{second}\"]")
}

/// A question to a reader: the module, the `use` path and the name space.
pub(crate) type Read = (&'static str, &'static str, Namespace);

/// What the canonical path `path` resolves to in the workspace: the path of
/// the item, or the kind of refusal.
pub(crate) fn resolved_as(ws: &Workspace, path: &str) -> String {
    match ws.resolve(path) {
        Ok(found) => found.path,
        Err(ResolveError::NotFound(_)) => "not found".to_owned(),
        Err(ResolveError::Ambiguous(ids)) => format!(
            "ambiguous {:?}",
            ids.iter()
                .map(|id| ws.item(*id).expect("item").path.to_string())
                .collect::<Vec<_>>()
        ),
        Err(other) => format!("{other:?}"),
    }
}

/// Load the crate `load` builds from every order of its top-level modules,
/// `others` and the `modules` of `(header, declarations)`, with the
/// declarations inside each of those modules in every order too, and collect
/// what `answer` says about each. Asserts that the answers are the same in
/// each, and returns them and the number of loads.
pub(crate) fn in_every_order(
    load: fn(&str) -> Workspace,
    others: &[&str],
    modules: &[(&str, &[&str])],
    answer: impl Fn(&Workspace) -> Vec<String>,
) -> (Vec<String>, usize) {
    let mut expected: Option<Vec<String>> = None;
    let mut loads = 0;
    let insides: Vec<Vec<Vec<String>>> = modules
        .iter()
        .map(|(_, inner)| permutations(&inner.iter().map(|d| (*d).to_owned()).collect::<Vec<_>>()))
        .collect();
    let mut choice = vec![0; modules.len()];
    loop {
        let mut blocks: Vec<String> = others.iter().map(|b| (*b).to_owned()).collect();
        for (at, (header, _)) in modules.iter().enumerate() {
            blocks.push(format!(
                "{header} {{ {} }}",
                insides[at][choice[at]].join(" ")
            ));
        }
        for order in permutations(&blocks) {
            let results = answer(&load(&order.join("\n")));
            match &expected {
                None => expected = Some(results),
                Some(first) => assert_eq!(&results, first, "{order:?}"),
            }
            loads += 1;
        }
        // The next combination of orders inside the modules.
        let mut at = 0;
        while at < choice.len() {
            choice[at] += 1;
            if choice[at] < insides[at].len() {
                break;
            }
            choice[at] = 0;
            at += 1;
        }
        if at == choice.len() {
            break;
        }
    }
    (expected.expect("loaded"), loads)
}
