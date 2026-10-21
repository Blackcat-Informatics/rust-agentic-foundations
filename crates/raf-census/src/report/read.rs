// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Iterative parser of the closed Turtle 1.2 report profile.
use super::{
    Baseline, Index, ReportError as E,
    shape::{self, Graph, Node, Term},
    vocab, write,
};
use crate::Error;
use purrdf_lex::terminals;
use std::collections::BTreeMap;

struct Parser<'a> {
    text: &'a str,
    at: usize,
    prefixes: BTreeMap<String, String>,
}
impl Parser<'_> {
    fn error(&self, reason: impl Into<String>) -> E {
        let head = &self.text[..self.at];
        E::Syntax {
            line: head.bytes().filter(|b| *b == b'\n').count() + 1,
            column: head.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            reason: reason.into(),
        }
    }
    fn ws(&mut self) {
        self.at = terminals::skip_ws(self.text.as_bytes(), self.at);
    }
    fn take(&mut self, token: &str) -> bool {
        self.ws();
        if self.text[self.at..].starts_with(token) {
            self.at += token.len();
            true
        } else {
            false
        }
    }
    fn need(&mut self, token: &str) -> Result<(), E> {
        if self.take(token) {
            Ok(())
        } else {
            Err(self.error(format!("expected {token}")))
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.ws();
        self.text[self.at..].chars().next()
    }
    fn quoted(&mut self, close: char) -> Result<String, E> {
        let mut out = String::new();
        loop {
            let Some(c) = self.text[self.at..].chars().next() else {
                return Err(self.error("unterminated term"));
            };
            if c == close {
                self.at += c.len_utf8();
                return Ok(out);
            }
            if c == '\\' {
                let tail = &self.text.as_bytes()[self.at..];
                if matches!(tail.get(1), Some(b'u' | b'U')) {
                    let (value, len) = terminals::decode_uchar(tail)
                        .map_err(|e| self.error(format!("invalid UCHAR: {e:?}")))?;
                    out.push(value);
                    self.at += len;
                } else if close == '"' {
                    let value = tail
                        .get(1)
                        .and_then(|b| terminals::echar_value(*b))
                        .ok_or_else(|| self.error("invalid ECHAR"))?;
                    out.push(value);
                    self.at += 2;
                } else {
                    return Err(self.error("IRI escape must be UCHAR"));
                }
            } else {
                if (close == '>' && terminals::is_iriref_forbidden(c))
                    || (close == '"' && matches!(c, '\n' | '\r'))
                {
                    return Err(self.error("forbidden character in term"));
                }
                out.push(c);
                self.at += c.len_utf8();
            }
        }
    }
    fn word(&mut self) -> Result<String, E> {
        self.ws();
        let start = self.at;
        while let Some(c) = self.text[self.at..].chars().next() {
            if c.is_whitespace()
                || matches!(
                    c,
                    ';' | ','
                        | '~'
                        | '{'
                        | '}'
                        | '<'
                        | '>'
                        | '"'
                        | '['
                        | ']'
                        | '('
                        | ')'
                        | '#'
                        | '@'
                        | '\''
                        | '\\'
                        | '|'
                        | '^'
                )
            {
                break;
            }
            if c == '.' {
                break;
            }
            self.at += c.len_utf8();
        }
        if start == self.at {
            return Err(self.error("construct outside the report profile"));
        }
        Ok(self.text[start..self.at].into())
    }
    fn iri(&mut self) -> Result<String, E> {
        if self.take("<") {
            if self.text[self.at..].starts_with('<') {
                return Err(self.error("explicit triple terms are outside the report profile"));
            }
            return self.quoted('>');
        }
        let word = self.word()?;
        let (prefix, local) = word
            .split_once(':')
            .ok_or_else(|| self.error("IRI or prefixed name expected"))?;
        if !prefix.chars().all(terminals::is_pn_chars)
            || (!local.is_empty()
                && (!local
                    .chars()
                    .next()
                    .is_some_and(terminals::is_pn_local_start)
                    || !local.chars().all(|c| terminals::is_pn_chars(c) || c == ':')))
        {
            return Err(self.error("invalid unescaped prefixed name"));
        }
        let namespace = self
            .prefixes
            .get(prefix)
            .ok_or_else(|| self.error(format!("unknown prefix {prefix}")))?;
        Ok(format!("{namespace}{local}"))
    }
    fn object(&mut self) -> Result<Term, E> {
        match self.peek() {
            Some('"') => {
                self.at += 1;
                Ok(Term::String(self.quoted('"')?))
            }
            Some('0'..='9') => {
                let word = self.word()?;
                Ok(Term::Integer(
                    word.parse()
                        .map_err(|_| self.error("unsigned integer expected"))?,
                ))
            }
            Some('t' | 'f') => {
                let before = self.at;
                let word = self.word()?;
                match word.as_str() {
                    "true" => Ok(Term::Boolean(true)),
                    "false" => Ok(Term::Boolean(false)),
                    _ => {
                        self.at = before;
                        Ok(Term::Iri(self.iri()?))
                    }
                }
            }
            _ => Ok(Term::Iri(self.iri()?)),
        }
    }
    fn predicate(&mut self) -> Result<String, E> {
        self.ws();
        if self.text[self.at..].starts_with("a ") || self.text[self.at..].starts_with("a\n") {
            self.at += 1;
            return Ok("type".into());
        }
        let iri = self.iri()?;
        if let Some(local) = iri.strip_prefix(vocab::CENSUS) {
            Ok(local.into())
        } else {
            Err(E::UnknownPredicate(iri))
        }
    }
    fn properties(&mut self, graph: &mut Graph, subject: &str, annotation: bool) -> Result<(), E> {
        loop {
            let predicate = self.predicate()?;
            loop {
                let object = self.object()?;
                graph
                    .entry(subject.into())
                    .or_default()
                    .entry(predicate.clone())
                    .or_default()
                    .push(object.clone());
                if self.take("~") {
                    if annotation {
                        return Err(self.error("nested annotation"));
                    }
                    let reifier = self.iri()?;
                    if graph.contains_key(&reifier) {
                        return Err(E::Repeated(format!("{reifier}: named reifier")));
                    }
                    let Term::Iri(unit) = object else {
                        return Err(self.error("match object must be a unit IRI"));
                    };
                    let p = format!("{}{predicate}", vocab::CENSUS);
                    let mut value = Node::new();
                    value.insert(
                        "reifies".into(),
                        vec![Term::Triple(subject.into(), p, unit)],
                    );
                    graph.insert(reifier.clone(), value);
                    self.need("{|")?;
                    self.properties(graph, &reifier, true)?;
                    self.need("|}")?;
                }
                if !self.take(",") {
                    break;
                }
            }
            if !self.take(";") {
                return Ok(());
            }
        }
    }
}
pub(super) fn parse(text: &str) -> Result<(Graph, BTreeMap<String, String>), E> {
    let mut p = Parser {
        text,
        at: 0,
        prefixes: BTreeMap::new(),
    };
    if !p.take("VERSION") {
        return Err(E::Version("missing VERSION".into()));
    }
    p.need("\"")?;
    let version = p.quoted('"')?;
    if version != "1.2" {
        return Err(E::Version(version));
    }
    p.need("BASE")?;
    p.need("<")?;
    let base = p.quoted('>')?;
    let hash = base
        .strip_prefix(vocab::BASE)
        .ok_or_else(|| p.error("BASE must name the census content digest"))?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(p.error("BASE requires 64 lowercase hexadecimal digits"));
    }
    while p.take("PREFIX") {
        let prefix = p.word()?;
        let prefix = prefix
            .strip_suffix(':')
            .ok_or_else(|| p.error("prefix label must end with colon"))?;
        p.need("<")?;
        let namespace = p.quoted('>')?;
        if p.prefixes.insert(prefix.into(), namespace).is_some() {
            return Err(p.error("repeated prefix"));
        }
    }
    let mut graph = Graph::new();
    while p.peek().is_some() {
        let subject = p.iri()?;
        if graph.contains_key(&subject) {
            return Err(E::Repeated(format!("subject {subject}")));
        }
        p.properties(&mut graph, &subject, false)?;
        p.need(".")?;
    }
    Ok((graph, p.prefixes))
}
fn finish(
    text: &str,
    canonical: &str,
    prefixes: &BTreeMap<String, String>,
    nf: u32,
    kind: &str,
) -> Result<(), E> {
    let expected = write::prefixes(nf, kind)
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect::<BTreeMap<_, _>>();
    if prefixes != &expected {
        return Err(E::Inconsistent("prefix namespaces and normalForm".into()));
    }
    let without_base = |text: &str| {
        text.split_inclusive('\n')
            .enumerate()
            .filter_map(|(i, line)| (i != 1).then_some(line))
            .collect::<String>()
    };
    let body = without_base(text);
    let wanted = without_base(canonical);
    if body != wanted {
        let line = body
            .split_inclusive('\n')
            .zip(wanted.split_inclusive('\n'))
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| {
                body.split_inclusive('\n')
                    .count()
                    .min(wanted.split_inclusive('\n').count())
            })
            + 1;
        return Err(E::NotCanonical {
            line: if line > 1 { line + 1 } else { line },
        });
    }
    let base = text
        .lines()
        .nth(1)
        .ok_or_else(|| E::Missing("BASE line".into()))?;
    let declared = base
        .strip_prefix("BASE <")
        .and_then(|s| s.strip_suffix('>'))
        .and_then(|s| s.strip_prefix(vocab::BASE))
        .ok_or(E::NotCanonical { line: 2 })?;
    let computed = write::digest(&body);
    if declared != computed {
        return Err(E::Digest {
            declared: declared.into(),
            computed,
        });
    }
    Ok(())
}
pub(super) fn baseline(text: &str) -> Result<Baseline, Error> {
    let (graph, prefixes) = parse(text)?;
    let report = shape::baseline(&graph)?;
    let canonical = write::render(&graph, report.normal_form, "baseline")?;
    finish(text, &canonical, &prefixes, report.normal_form, "baseline")?;
    Ok(report)
}
pub(super) fn index(text: &str) -> Result<Index, Error> {
    let (graph, prefixes) = parse(text)?;
    let report = shape::index(&graph)?;
    let canonical = write::render(&graph, report.normal_form, "index")?;
    finish(text, &canonical, &prefixes, report.normal_form, "index")?;
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_token_outside_the_profile_names_its_line_and_column() {
        for text in ["_:b", "[]", "()", "# comment", "'''x'''", "<<("] {
            let mut p = Parser {
                text,
                at: 0,
                prefixes: BTreeMap::new(),
            };
            assert!(matches!(
                p.object(),
                Err(E::Syntax {
                    line: 1,
                    column: _,
                    reason: _
                })
            ));
        }
    }

    fn absolute(value: &str, base: &str) -> String {
        if value.is_empty() || value.starts_with('#') {
            format!("{base}{value}")
        } else {
            value.into()
        }
    }
    fn nt_term(value: &Term, base: &str) -> String {
        let mut out = String::new();
        match value {
            Term::Iri(value) => {
                purrdf_lex::term_syntax::write_iri(&absolute(value, base), &mut out);
            }
            Term::String(value) => purrdf_lex::term_syntax::write_literal(
                value,
                &format!("{}string", vocab::XSD),
                None,
                None,
                &mut out,
            ),
            Term::Integer(value) => purrdf_lex::term_syntax::write_literal(
                &value.to_string(),
                &format!("{}integer", vocab::XSD),
                None,
                None,
                &mut out,
            ),
            Term::Boolean(value) => purrdf_lex::term_syntax::write_literal(
                if *value { "true" } else { "false" },
                &format!("{}boolean", vocab::XSD),
                None,
                None,
                &mut out,
            ),
            Term::Triple(s, p, o) => {
                out.push_str(purrdf_lex::term_syntax::TRIPLE_TERM_OPEN);
                out.push(' ');
                for (i, value) in [s, p, o].iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    purrdf_lex::term_syntax::write_iri(&absolute(value, base), &mut out);
                }
                out.push(' ');
                out.push_str(purrdf_lex::term_syntax::TRIPLE_TERM_CLOSE);
            }
        }
        out
    }
    /// Pinned expansions were produced by the independent purrdf-rdf 3.0.1
    /// parser with no caller base, then sorted as ground N-Triples.
    #[test]
    fn the_graph_of_each_example_is_its_pinned_triple_set() {
        for (text, wanted) in [
            (
                include_str!("../../tests/fixtures/reports/baseline.ttl"),
                include_str!("../../tests/fixtures/reports/baseline.nt"),
            ),
            (
                include_str!("../../tests/fixtures/reports/index.ttl"),
                include_str!("../../tests/fixtures/reports/index.nt"),
            ),
            (
                include_str!("../../tests/fixtures/reports/awkward.ttl"),
                include_str!("../../tests/fixtures/reports/awkward.nt"),
            ),
        ] {
            let (graph, _) = parse(text).expect("profile parses");
            let base = text
                .lines()
                .nth(1)
                .expect("BASE")
                .strip_prefix("BASE <")
                .expect("BASE")
                .strip_suffix('>')
                .expect("IRI");
            let mut triples = Vec::new();
            for (subject, node) in &graph {
                for (key, values) in node {
                    let predicate = match key.as_str() {
                        "type" => format!("{}type", vocab::RDF),
                        "reifies" => format!("{}reifies", vocab::RDF),
                        _ => format!("{}{key}", vocab::CENSUS),
                    };
                    for value in values {
                        triples.push(format!(
                            "{} {} {} .\n",
                            nt_term(&Term::Iri(subject.clone()), base),
                            nt_term(&Term::Iri(predicate.clone()), base),
                            nt_term(value, base)
                        ));
                    }
                }
            }
            triples.sort();
            assert_eq!(triples.concat(), wanted);
        }
    }
}
