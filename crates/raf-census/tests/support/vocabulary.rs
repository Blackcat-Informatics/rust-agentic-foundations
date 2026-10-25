// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Exact documented vocabulary coverage over actual native report emissions.

use purrdf_lex::allocation::{Memory, OrderedTable, Resident};
use raf_rdf::{Format, Graph, TermValue, vocab};

type Terms = OrderedTable<String, ()>;

pub(crate) fn assert_inventory(header: &str, reports: impl IntoIterator<Item = String>) {
    let document = include_str!("../../../../docs/capabilities/rdf-report-profile.md");
    let table = document
        .split(header)
        .nth(1)
        .expect("the shared profile declares the inventory")
        .split("\n\n")
        .next()
        .unwrap();
    let documented = terms(
        table
            .split('`')
            .enumerate()
            .filter(|(index, _)| index % 2 == 1)
            .map(|(_, term)| term.to_owned()),
    );
    let mut emitted = Terms::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for text in reports {
        let graph = Graph::from_text(&text, Format::NTriples).unwrap();
        for triple in graph.assertions() {
            for value in [&triple.subject, &triple.predicate, &triple.object] {
                let _ = value.visit_terms(|term| {
                    if let TermValue::Iri(iri) | TermValue::Literal { datatype: iri, .. } = term
                        && let Some(name) = iri.strip_prefix(vocab::REPORT)
                    {
                        emitted.insert(name.to_owned(), (), &mut memory).unwrap();
                    }
                    std::ops::ControlFlow::<()>::Continue(())
                });
            }
        }
    }
    assert_eq!(
        difference(&emitted, &documented),
        (Vec::<String>::new(), Vec::<String>::new()),
        "{header}: undocumented emissions and never-emitted declarations"
    );
    let term = emitted.iter().next().unwrap().0.clone();
    let undocumented = terms(
        documented
            .iter()
            .filter(|(name, ())| *name != &term)
            .map(|(name, ())| name.clone()),
    );
    assert!(difference(&emitted, &undocumented).0.contains(&term));
    let unused = terms(
        documented
            .iter()
            .map(|(name, ())| name.clone())
            .chain(std::iter::once("NeverEmitted".into())),
    );
    assert!(
        difference(&emitted, &unused)
            .1
            .contains(&"NeverEmitted".to_owned())
    );
    assert_eq!(
        difference(&emitted, &emitted),
        (Vec::<String>::new(), Vec::<String>::new())
    );
}

fn terms(values: impl IntoIterator<Item = String>) -> Terms {
    let mut table = Terms::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for value in values {
        table.insert(value, (), &mut memory).unwrap();
    }
    table
}

fn difference(emitted: &Terms, documented: &Terms) -> (Vec<String>, Vec<String>) {
    (
        emitted
            .iter()
            .filter(|(name, ())| documented.get(name).is_none())
            .map(|(name, ())| name.clone())
            .collect(),
        documented
            .iter()
            .filter(|(name, ())| emitted.get(name).is_none())
            .map(|(name, ())| name.clone())
            .collect(),
    )
}
