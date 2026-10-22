// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! One normalized body and `PurRDF`'s native term emitter supply both formats.

use crate::{Error, Format, Graph, TermValue, term, vocab};
use purrdf_lex::allocation::{Memory, Resident};

struct Selection<'a> {
    graph: &'a Graph,
    rows: Vec<purrdf_core::QuadIds>,
    keys: std::collections::BTreeMap<purrdf_core::TermId, TermValue>,
}
impl<'a> Selection<'a> {
    fn new(graph: &'a Graph) -> Self {
        let rows: Vec<_> = graph.rows().collect();
        let mut keys = std::collections::BTreeMap::new();
        for row in &rows {
            keys.entry(row.o)
                .or_insert_with(|| graph.value(row.o).to_owned());
        }
        let mut selection = Self { graph, rows, keys };
        let iri = |id| match graph.dataset.resolve(id) {
            purrdf_core::TermRef::Iri(iri) => iri,
            _ => unreachable!("validated IRI role"),
        };
        selection.rows.sort_unstable_by(|a, b| {
            (graph.local(iri(a.s)), iri(a.p), &selection.keys[&a.o]).cmp(&(
                graph.local(iri(b.s)),
                iri(b.p),
                &selection.keys[&b.o],
            ))
        });
        selection
    }
    fn iri(&self, id: purrdf_core::TermId) -> &str {
        match self.graph.dataset.resolve(id) {
            purrdf_core::TermRef::Iri(iri) => iri,
            _ => unreachable!("validated IRI role"),
        }
    }
}

fn body(selection: &Selection<'_>, base: Option<&str>, out: &mut String) -> Result<(), Error> {
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for triple in &selection.rows {
        for (iri, owned) in [
            (selection.graph.local(selection.iri(triple.s)), true),
            (selection.iri(triple.p), false),
        ] {
            if let Some(base) = base
                && owned
                && term::local(iri)
            {
                out.push('<');
                purrdf_lex::iri_escape::push_escaped(base, out);
                purrdf_lex::iri_escape::push_escaped(iri, out);
                out.push('>');
            } else {
                purrdf_lex::term_syntax::write_iri(iri, out);
            }
            out.push(' ');
        }
        let value = &selection.keys[&triple.o];
        if let Some(base) = base {
            value
                .try_write_nested_with_memory(
                    out,
                    ["<<( ", " ", " )>>"],
                    |out, leaf, memory| {
                        if let TermValue::Iri(iri) = leaf
                            && term::local(iri)
                        {
                            out.push('<');
                            purrdf_lex::iri_escape::push_escaped(base, out);
                            purrdf_lex::iri_escape::push_escaped(iri, out);
                            out.push('>');
                            Ok(())
                        } else {
                            purrdf_core::write_term_value_with_memory(leaf, out, memory)
                        }
                    },
                    |out, text, _| {
                        out.push_str(text);
                        Ok(())
                    },
                    &mut memory,
                )
                .map_err(|error| Error::Value(format!("native term output: {error}")))?;
        } else {
            purrdf_core::write_term_value_with_memory(value, out, &mut memory)
                .map_err(|error| Error::Value(format!("native term output: {error}")))?;
        }
        out.push_str(" .\n");
    }
    Ok(())
}

fn normalized(selection: &Selection<'_>) -> Result<String, Error> {
    let mut out = String::from("VERSION \"1.2\"\n");
    body(selection, None, &mut out)?;
    Ok(out)
}

fn identity(text: &str) -> String {
    format!(
        "{}{}",
        vocab::BASE,
        purrdf_hash::hex::Lower(purrdf_hash::blake3::hash(text.as_bytes()).as_bytes())
    )
}

pub(crate) fn base(graph: &Graph) -> Result<String, Error> {
    Ok(identity(&normalized(&Selection::new(graph))?))
}

pub(crate) fn render(graph: &Graph, format: Format) -> Result<(String, String), Error> {
    let selection = Selection::new(graph);
    let mut normalized = normalized(&selection)?;
    let base = identity(&normalized);
    match format {
        Format::Turtle => {
            normalized.insert_str("VERSION \"1.2\"\n".len(), &format!("BASE <{base}>\n"));
            Ok((base, normalized))
        }
        Format::NTriples => {
            drop(normalized);
            let mut out = String::new();
            body(&selection, Some(&base), &mut out)?;
            Ok((base, out))
        }
    }
}
