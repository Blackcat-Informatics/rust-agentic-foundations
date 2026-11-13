// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared original-account query and scalar policy for embedded RDF schemas.

use super::{
    output::{Failure, QueryCause},
    resource::{Grant, Meter, Native, Scratch, Text},
};
use crate::ReportError as E;
use purrdf_lex::{
    allocation::{Admission, OrderedTable},
    control::compare_bytes,
};
use raf_rdf::{AdmittedGraph, AdmittedNode, AdmittedValue};

pub(super) type Graph<'m> = AdmittedGraph<Native<'m>>;
pub(super) type Node<'a> = AdmittedNode<'a>;
pub(super) type Value<'a> = AdmittedValue<'a>;

pub(super) struct Emitter<'b, 's, 'm, 'run> {
    pub(super) build: &'b mut raf_rdf::Construction<'s, 'm, Native<'run>, super::resource::Refusal>,
    pub(super) schema: Schema<'run>,
}
impl<'run> Emitter<'_, '_, '_, 'run> {
    pub(super) fn iri(
        &mut self,
        id: &str,
        field: &str,
        value: &str,
    ) -> Result<(), super::output::BodyFailure<'run>> {
        let property = self.schema.property(field)?;
        Ok(self
            .build
            .insert_iri(purrdf_core::RecordKind::Ordinary, id, &property, value)?)
    }
    pub(super) fn text(
        &mut self,
        id: &str,
        field: &str,
        value: &str,
    ) -> Result<(), super::output::BodyFailure<'run>> {
        let property = self.schema.property(field)?;
        Ok(self.build.insert_literal(
            purrdf_core::RecordKind::Ordinary,
            id,
            &property,
            value,
            raf_rdf::vocab::STRING,
        )?)
    }
    pub(super) fn display(
        &mut self,
        id: &str,
        field: &str,
        value: &impl std::fmt::Display,
        datatype: &str,
    ) -> Result<(), super::output::BodyFailure<'run>> {
        let property = self.schema.property(field)?;
        Ok(self.build.insert_display_literal(
            purrdf_core::RecordKind::Ordinary,
            id,
            &property,
            value,
            datatype,
        )?)
    }
    pub(super) fn number(
        &mut self,
        id: &str,
        field: &str,
        value: usize,
    ) -> Result<(), super::output::BodyFailure<'run>> {
        self.display(
            id,
            field,
            &purrdf_xsd::exact::Integer::from(value as u128),
            raf_rdf::vocab::INTEGER,
        )
    }
    pub(super) fn class(
        &mut self,
        id: &str,
        class: &str,
    ) -> Result<(), super::output::BodyFailure<'run>> {
        let class = Text::display(
            self.schema.meter,
            &format_args!("{}{class}", self.schema.vocab),
        )?;
        self.iri(id, "type", &class)
    }
}

pub(super) struct BorrowedSet<'m, 'a> {
    entries: OrderedTable<&'a str, ()>,
    grant: Grant<'m>,
}
impl<'m, 'a> BorrowedSet<'m, 'a> {
    pub(super) fn new(meter: &'m Meter) -> Self {
        Self {
            entries: OrderedTable::default(),
            grant: Grant::new(meter),
        }
    }
    pub(super) fn contains(&self, key: &str) -> Result<bool, Failure<'m>> {
        Ok(self
            .entries
            .try_get_by(|prior| self.grant.meter().compare(key, prior))?
            .is_some())
    }
    pub(super) fn insert(&mut self, key: &'a str) -> Result<(), Failure<'m>> {
        self.grant.memory(|memory| {
            let entry =
                self.entries
                    .prepare_insert_with_memory(key, memory, |left, right, memory| {
                        Ok::<_, super::resource::Refusal>(compare_bytes(
                            left.as_bytes(),
                            right.as_bytes(),
                            &mut |units| memory.admission_mut().checkpoint(units),
                        )?)
                    })?;
            if !entry.is_occupied() {
                entry.publish(());
            }
            Ok(())
        })?;
        Ok(())
    }
}

/// Native exact names and their independently granted payloads.
pub(super) struct Named<'m, T> {
    entries: OrderedTable<Text<'m>, T>,
    grant: Grant<'m>,
}
impl<'m, T> Named<'m, T> {
    pub(super) fn new(meter: &'m Meter) -> Self {
        Self {
            grant: Grant::new(meter),
            entries: OrderedTable::default(),
        }
    }
    pub(super) fn get(&self, key: &str) -> Result<Option<&T>, Failure<'m>> {
        Ok(self
            .entries
            .try_get_by(|prior| self.grant.meter().compare(key, prior))?)
    }
    pub(super) fn remember(&mut self, key: &str, value: T) -> Result<bool, Failure<'m>> {
        if self.get(key)?.is_some() {
            return Ok(false);
        }
        let meter = self.grant.meter();
        let key = Text::new(meter, key)?;
        self.grant.memory(|memory| {
            let entry =
                self.entries
                    .prepare_insert_with_memory(key, memory, |left, right, memory| {
                        Ok::<_, super::resource::Refusal>(compare_bytes(
                            left.as_bytes(),
                            right.as_bytes(),
                            &mut |units| memory.admission_mut().checkpoint(units),
                        )?)
                    })?;
            entry.publish(value);
            Ok(())
        })?;
        Ok(true)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Schema<'m> {
    pub(super) meter: &'m Meter,
    pub(super) vocab: &'static str,
    pub(super) label: &'static str,
}

impl<'m> Schema<'m> {
    pub(super) fn error(
        &self,
        variant: fn(String) -> E,
        text: &impl std::fmt::Display,
    ) -> Failure<'m> {
        Failure::report(self.meter, variant, text)
    }

    pub(super) fn property(&self, key: &str) -> Result<Text<'m>, Failure<'m>> {
        Ok(if key == "type" {
            Text::new(self.meter, raf_rdf::vocab::TYPE)?
        } else {
            Text::display(self.meter, &format_args!("{}{key}", self.vocab))?
        })
    }

    pub(super) fn get<'a>(&self, graph: &'a Graph<'m>, id: &str) -> Result<Node<'a>, Failure<'m>> {
        Failure::query(self.meter, |memory| Ok(graph.get_with_memory(id, memory)?))?
            .ok_or_else(|| self.error(E::Dangling, &id))
    }

    pub(super) fn one<'a>(&self, node: Node<'a>, key: &str) -> Result<Value<'a>, Failure<'m>> {
        let property = self.property(key)?;
        let (first, second) = Failure::query(self.meter, |memory| {
            let mut values = node.values_with_memory(&property, memory);
            Ok((values.next().transpose()?, values.next().transpose()?))
        })?;
        match (first, second) {
            (Some(value), None) => Ok(value),
            (None, _) => Err(self.error(E::Missing, &format_args!("{}:{key}", self.label))),
            _ => Err(self.error(E::Repeated, &format_args!("{}:{key}", self.label))),
        }
    }

    pub(super) fn values<'a>(
        &self,
        node: Node<'a>,
        key: &str,
    ) -> Result<Scratch<'m, Value<'a>>, Failure<'m>> {
        let property = self.property(key)?;
        let mut objects = Scratch::new(self.meter);
        Failure::query(self.meter, |memory| {
            for value in node.values_with_memory(&property, memory) {
                objects.push(value?)?;
            }
            Ok(())
        })?;
        Ok(objects)
    }

    pub(super) fn amount(&self, node: Node<'_>, key: &str) -> Result<usize, Failure<'m>> {
        let property = self.property(key)?;
        Failure::query(self.meter, |memory| {
            let mut amount = 0usize;
            for value in node.values_with_memory(&property, memory) {
                value?;
                amount = amount
                    .checked_add(1)
                    .ok_or(purrdf_lex::allocation::StorageError::SizeOverflow)?;
            }
            Ok(amount)
        })
    }

    pub(super) fn exists(&self, node: Node<'_>, key: &str) -> Result<bool, Failure<'m>> {
        let property = self.property(key)?;
        Failure::query(self.meter, |memory| {
            Ok(node
                .values_with_memory(&property, memory)
                .next()
                .transpose()?
                .is_some())
        })
    }

    pub(super) fn properties<'a>(
        &self,
        node: Node<'a>,
    ) -> Result<Scratch<'m, (purrdf_core::TermId, &'a str)>, Failure<'m>> {
        let mut properties = Scratch::new(self.meter);
        Failure::query(self.meter, |memory| {
            for value in node.property_records_with_memory(memory) {
                let (key, property, _) = value?;
                properties.push((key, property))?;
            }
            Ok(())
        })?;
        purrdf_lex::walk::try_sort_unstable_by(properties.as_mut_slice(), |left, right| {
            self.meter.work(1)?;
            Ok::<_, Failure<'m>>(left.0.cmp(&right.0))
        })?;
        properties.dedup_by(|left, right| left.0 == right.0)?;
        Ok(properties)
    }

    pub(super) fn nodes<'a>(
        &self,
        graph: &'a Graph<'m>,
    ) -> Result<Scratch<'m, (&'a str, Node<'a>)>, Failure<'m>> {
        let mut nodes = Scratch::new(self.meter);
        Failure::query(self.meter, |memory| {
            let mut seen = OrderedTable::default();
            graph.try_for_each_record_with_memory(memory, |node, subject, _, _, memory| {
                let entry =
                    seen.prepare_insert_with_memory(subject, memory, |left, right, memory| {
                        Ok::<_, QueryCause>(compare_bytes(
                            left.as_bytes(),
                            right.as_bytes(),
                            &mut |units| memory.admission_mut().checkpoint(units),
                        )?)
                    })?;
                if !entry.is_occupied() {
                    entry.publish(());
                    nodes.push((subject, node))?;
                }
                Ok::<_, QueryCause>(())
            })?;
            seen.release(memory)?;
            Ok(())
        })?;
        purrdf_lex::walk::try_sort_unstable_by(nodes.as_mut_slice(), |left, right| {
            self.meter.work(1)?;
            Ok::<_, Failure<'m>>(left.1.native_id().cmp(&right.1.native_id()))
        })?;
        Ok(nodes)
    }

    pub(super) fn iri<'a>(&self, value: Value<'a>) -> Result<Option<&'a str>, Failure<'m>> {
        Failure::query(self.meter, |memory| Ok(value.iri_with_memory(memory)?))
    }

    pub(super) fn text<'a>(&self, node: Node<'a>, key: &str) -> Result<&'a str, Failure<'m>> {
        let value = self.one(node, key)?;
        if let Some((text, datatype)) = value.literal()
            && self
                .meter
                .compare(datatype, raf_rdf::vocab::STRING)?
                .is_eq()
        {
            return Ok(text);
        }
        Err(self.error(
            E::Value,
            &format_args!("{}:{key}: string required", self.label),
        ))
    }

    pub(super) fn count(&self, node: Node<'_>, key: &str) -> Result<usize, Failure<'m>> {
        let value = self.one(node, key)?;
        if let Some((lexical, datatype)) = value.literal()
            && self
                .meter
                .compare(datatype, raf_rdf::vocab::INTEGER)?
                .is_eq()
        {
            self.meter.work(lexical.len())?;
            if let Ok(value) =
                purrdf_xsd::numeric::read_integer_typed(lexical, purrdf_xsd::XsdDatatype::Integer)
                && let Ok(count) = usize::try_from(value)
            {
                let canonical =
                    Text::display(self.meter, &purrdf_xsd::exact::Integer::from_i128(value))?;
                if self.meter.compare(&canonical, lexical)?.is_eq() {
                    return Ok(count);
                }
            }
        }
        Err(self.error(
            E::Value,
            &format_args!("{}:{key}: canonical usize required", self.label),
        ))
    }

    pub(super) fn boolean(&self, node: Node<'_>, key: &str) -> Result<bool, Failure<'m>> {
        let value = self.one(node, key)?;
        if let Some((lexical, datatype)) = value.literal()
            && self
                .meter
                .compare(datatype, raf_rdf::vocab::BOOLEAN)?
                .is_eq()
        {
            self.meter.work(lexical.len())?;
            if let Ok(value) = purrdf_xsd::simple::read_boolean(lexical)
                && self
                    .meter
                    .compare(lexical, if value { "true" } else { "false" })?
                    .is_eq()
            {
                return Ok(value);
            }
        }
        Err(self.error(
            E::Value,
            &format_args!("{}:{key}: canonical boolean required", self.label),
        ))
    }

    pub(super) fn reference<'a>(&self, node: Node<'a>, key: &str) -> Result<&'a str, Failure<'m>> {
        self.reference_as(node, key, "owned node")
    }

    pub(super) fn reference_as<'a>(
        &self,
        node: Node<'a>,
        key: &str,
        required: &str,
    ) -> Result<&'a str, Failure<'m>> {
        if let Some(iri) = self.iri(self.one(node, key)?)?
            && iri.starts_with('#')
        {
            return Ok(iri);
        }
        Err(self.error(
            E::Value,
            &format_args!("{}:{key}: {required} required", self.label),
        ))
    }

    pub(super) fn class<'a>(&self, node: Node<'a>) -> Result<&'a str, Failure<'m>> {
        self.class_named(node, self.label)
    }

    pub(super) fn class_named<'a>(
        &self,
        node: Node<'a>,
        label: &str,
    ) -> Result<&'a str, Failure<'m>> {
        if let Some(iri) = self.iri(self.one(node, "type")?)? {
            self.meter
                .work(iri.len().min(self.vocab.len()).saturating_add(1))?;
            if let Some(class) = iri.strip_prefix(self.vocab) {
                return Ok(class);
            }
        }
        Err(self.error(E::Class, &format_args!("{label} class required")))
    }

    pub(super) fn digest<'a>(&self, node: Node<'a>, key: &str) -> Result<&'a str, Failure<'m>> {
        let value = self.text(node, key)?;
        self.meter.work(value.len())?;
        if purrdf_hash::hex::decode_32_canonical(value).is_some() {
            return Ok(value);
        }
        Err(self.error(
            E::Value,
            &format_args!("{}:{key}: full lowercase BLAKE3 required", self.label),
        ))
    }

    pub(super) fn key<'a>(
        &self,
        kind: &str,
        parts: impl IntoIterator<Item = &'a str>,
    ) -> Result<Text<'m>, Failure<'m>> {
        Ok(Text::build(self.meter, |memory| {
            Ok(super::report::key_with_memory(kind, parts, memory)?)
        })?)
    }

    pub(super) fn numbered_key(
        &self,
        kind: &str,
        head: &[&str],
        numbers: &[usize],
        tail: &[&str],
    ) -> Result<Text<'m>, Failure<'m>> {
        let mut numeric = Scratch::new(self.meter);
        for number in numbers {
            numeric.push(Text::display(self.meter, number)?)?;
        }
        self.key(
            kind,
            head.iter()
                .copied()
                .chain(numeric.iter().map(Text::as_ref))
                .chain(tail.iter().copied()),
        )
    }
}
