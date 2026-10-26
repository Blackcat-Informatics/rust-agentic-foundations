// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Borrowed authoring values enter the native interner after one budget pass.

use std::sync::Arc;

use purrdf_core::{RdfDataset, RdfDatasetBuilder, RdfLiteral, TermId};

use super::{Budget, DatasetError as E, INTEGER, STRING, TYPE, descendants, identity, property};
use crate::{Comments, Document, ItemValue, ListForm, Map, ScalarStyle, Value};

pub(super) fn admit(document: &Document) -> Result<(), E> {
    let mut admitted = Budget::default();
    admitted.node()?;
    admitted.text(document.body().text())?;
    count_map(&mut admitted, document.frontmatter().map())?;
    for text in document.frontmatter().trailing_comments() {
        admitted.node()?;
        admitted.text(text)?;
    }
    Ok(())
}

pub(super) fn document(document: &Document) -> Result<Arc<RdfDataset>, E> {
    let root = identity(document)?;
    let mut writer = Writer {
        store: RdfDatasetBuilder::new(),
        prefix: descendants(&root),
        next: 0,
        budget: Budget::default(),
    };
    let root = writer.store.intern_iri(&root).map_err(E::native)?;
    writer.class(root, "Document")?;
    writer.text(root, "body", document.body().text())?;
    let (map, _) = writer.map(document.frontmatter().map())?;
    writer.put(root, "frontmatter", map)?;
    for (ordinal, text) in document
        .frontmatter()
        .trailing_comments()
        .iter()
        .enumerate()
    {
        let comment = writer.comment(ordinal, text)?;
        writer.put(root, "trailingComment", comment)?;
    }
    writer.store.freeze().map_err(E::native)
}

fn count_comments(budget: &mut Budget, comments: &Comments) -> Result<(), E> {
    for text in comments.leading() {
        budget.node()?;
        budget.text(text)?;
    }
    if let Some(text) = comments.trailing() {
        budget.text(text)?;
    }
    Ok(())
}

fn count_map(budget: &mut Budget, map: &Map) -> Result<(), E> {
    budget.node()?;
    for entry in map.entries() {
        budget.node()?;
        budget.text(entry.key())?;
        count_comments(budget, entry.comments())?;
        count_value(budget, entry.value())?;
    }
    Ok(())
}

fn count_value(budget: &mut Budget, value: &Value) -> Result<(), E> {
    match value {
        Value::Scalar(value) => {
            budget.node()?;
            budget.text(value.text())?;
        }
        Value::Map(map) => count_map(budget, map)?,
        Value::List(list) => {
            budget.node()?;
            for item in list.items() {
                budget.node()?;
                count_comments(budget, item.comments())?;
                match item.value() {
                    ItemValue::Scalar(value) => {
                        budget.node()?;
                        budget.text(value.text())?;
                    }
                    ItemValue::Map(map) => count_map(budget, map)?,
                }
            }
        }
    }
    Ok(())
}

struct Writer {
    store: RdfDatasetBuilder,
    prefix: String,
    next: usize,
    budget: Budget,
}

impl Writer {
    fn class(&mut self, subject: TermId, role: &str) -> Result<(), E> {
        self.budget.node()?;
        let predicate = self.store.intern_iri(TYPE).map_err(E::native)?;
        let role = self.store.intern_iri(&property(role)).map_err(E::native)?;
        self.budget.triple()?;
        self.store
            .push_quad(subject, predicate, role, None)
            .map_err(E::native)
    }

    fn node(&mut self, role: &str) -> Result<TermId, E> {
        let id = self
            .store
            .intern_iri(&format!("{}{}", self.prefix, self.next))
            .map_err(E::native)?;
        self.next += 1;
        self.class(id, role)?;
        Ok(id)
    }

    fn put(&mut self, subject: TermId, predicate: &str, object: TermId) -> Result<(), E> {
        self.budget.triple()?;
        let predicate = self
            .store
            .intern_iri(&property(predicate))
            .map_err(E::native)?;
        self.store
            .push_quad(subject, predicate, object, None)
            .map_err(E::native)
    }

    fn text(&mut self, subject: TermId, predicate: &str, text: &str) -> Result<(), E> {
        let value = self
            .store
            .intern_literal(RdfLiteral::typed(text, STRING))
            .map_err(E::native)?;
        self.put(subject, predicate, value)
    }

    fn ordinal(&mut self, subject: TermId, ordinal: usize) -> Result<(), E> {
        let value = self
            .store
            .intern_literal(RdfLiteral::typed(ordinal.to_string(), INTEGER))
            .map_err(E::native)?;
        self.put(subject, "ordinal", value)
    }

    fn comment(&mut self, ordinal: usize, text: &str) -> Result<TermId, E> {
        let id = self.node("Comment")?;
        self.ordinal(id, ordinal)?;
        self.text(id, "text", text)?;
        Ok(id)
    }

    fn comments(&mut self, subject: TermId, comments: &Comments) -> Result<(), E> {
        for (ordinal, text) in comments.leading().iter().enumerate() {
            let comment = self.comment(ordinal, text)?;
            self.put(subject, "leadingComment", comment)?;
        }
        if let Some(text) = comments.trailing() {
            self.text(subject, "trailing", text)?;
        }
        Ok(())
    }

    fn map(&mut self, map: &Map) -> Result<(TermId, TermId), E> {
        let id = self.node("Map")?;
        let first = map
            .entries()
            .first()
            .ok_or_else(|| E::shape("mapping", "empty mapping"))?
            .key();
        let mut entries: Vec<_> = map.entries().iter().collect();
        purrdf_lex::walk::try_sort_unstable_by(&mut entries, |a, b| {
            Ok::<_, E>(a.key().cmp(b.key()))
        })?;
        let mut head = id;
        for (ordinal, entry) in entries.into_iter().enumerate() {
            let child = self.node("Entry")?;
            if entry.key() == first {
                head = child;
            }
            self.ordinal(child, ordinal)?;
            self.text(child, "key", entry.key())?;
            self.comments(child, entry.comments())?;
            let value = self.value(entry.value())?;
            self.put(child, "value", value)?;
            self.put(id, "entry", child)?;
        }
        Ok((id, head))
    }

    fn scalar(&mut self, value: &crate::Scalar) -> Result<TermId, E> {
        let id = self.node("Scalar")?;
        self.text(id, "text", value.text())?;
        self.text(
            id,
            "style",
            match value.style() {
                ScalarStyle::Plain => "plain",
                ScalarStyle::SingleQuoted => "single",
                ScalarStyle::DoubleQuoted => "double",
            },
        )?;
        Ok(id)
    }

    fn value(&mut self, value: &Value) -> Result<TermId, E> {
        match value {
            Value::Scalar(value) => self.scalar(value),
            Value::Map(map) => self.map(map).map(|(id, _)| id),
            Value::List(list) => {
                let id = self.node("List")?;
                self.text(
                    id,
                    "form",
                    match list.form() {
                        ListForm::Flow => "flow",
                        ListForm::Block => "block",
                    },
                )?;
                for (ordinal, item) in list.items().iter().enumerate() {
                    let child = self.node("Item")?;
                    self.ordinal(child, ordinal)?;
                    self.comments(child, item.comments())?;
                    let value = match item.value() {
                        ItemValue::Scalar(value) => self.scalar(value)?,
                        ItemValue::Map(map) => {
                            let (value, head) = self.map(map)?;
                            self.put(child, "headEntry", head)?;
                            value
                        }
                    };
                    self.put(child, "value", value)?;
                    self.put(id, "item", child)?;
                }
                Ok(id)
            }
        }
    }
}
