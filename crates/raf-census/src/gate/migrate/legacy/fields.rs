// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Application-field consumption over native TOML occurrence indices.

use super::super::UnmappedRow;
use crate::Error;
use purrdf_toml::{Document, Kind, Scalar};
use std::{collections::BTreeMap, rc::Rc};

pub(super) struct Fields<'d, 's> {
    pub(super) document: &'d Document<'s>,
    pub(super) at: String,
    // Only unclaimed immediate child indices, never a copied value tree.
    pub(super) table: Vec<usize>,
    headers: Rc<BTreeMap<usize, Vec<usize>>>,
}

impl<'d, 's> Fields<'d, 's> {
    pub(super) fn new(
        document: &'d Document<'s>,
        at: impl Into<String>,
        table: Vec<usize>,
    ) -> Self {
        let mut headers = BTreeMap::<usize, Vec<usize>>::new();
        for (index, statement) in document.statements().iter().enumerate() {
            if statement.header {
                headers.entry(statement.node).or_default().push(index);
            }
        }
        Self {
            document,
            at: at.into(),
            table,
            headers: Rc::new(headers),
        }
    }

    fn child(&self, at: String, table: Vec<usize>) -> Self {
        Self {
            document: self.document,
            at,
            table,
            headers: Rc::clone(&self.headers),
        }
    }

    pub(super) fn take(&mut self, key: &str) -> Option<usize> {
        let at = self.table.iter().position(|id| {
            self.document.nodes()[*id]
                .key
                .as_ref()
                .is_some_and(|name| name.decoded == key)
        })?;
        Some(self.table.remove(at))
    }

    pub(super) fn text(&mut self, key: &str, default: Option<&str>) -> Result<String, Error> {
        match self.take(key).map(|id| &self.document.nodes()[id].kind) {
            Some(Kind::Scalar(Scalar::String(text)))
                if default.is_some() || !text.trim().is_empty() =>
            {
                Ok(text.clone())
            }
            None if default.is_some() => Ok(default.unwrap_or_default().into()),
            _ => Err(self.bad(key, "a string is required")),
        }
    }

    pub(super) fn flag(&mut self, key: &str, default: bool) -> Result<bool, Error> {
        match self.take(key).map(|id| &self.document.nodes()[id].kind) {
            Some(Kind::Scalar(Scalar::Boolean(value))) => Ok(*value),
            None => Ok(default),
            _ => Err(self.bad(key, "a boolean is required")),
        }
    }

    pub(super) fn integer(&mut self, key: &str) -> Result<Option<u64>, Error> {
        match self.take(key).map(|id| &self.document.nodes()[id].kind) {
            None => Ok(None),
            Some(Kind::Scalar(Scalar::Integer(value))) => u64::try_from(*value)
                .map(Some)
                .map_err(|_| self.bad(key, "a nonnegative integer is required")),
            _ => Err(self.bad(key, "a nonnegative integer is required")),
        }
    }

    pub(super) fn table(&mut self, key: &str) -> Result<Self, Error> {
        let indices = match self.take(key) {
            None => Vec::new(),
            Some(id) if matches!(self.document.nodes()[id].kind, Kind::Table(_)) => {
                self.document.nodes()[id].children.clone()
            }
            Some(_) => return Err(self.bad(key, "a table is required")),
        };
        Ok(self.child(format!("{}.{}", self.at, key), indices))
    }

    pub(super) fn rows(&mut self, key: &str) -> Result<Vec<Self>, Error> {
        let Some(id) = self.take(key) else {
            return Ok(Vec::new());
        };
        let node = &self.document.nodes()[id];
        if !matches!(node.kind, Kind::Array | Kind::ArrayOfTables) {
            return Err(self.bad(key, "an array of tables is required"));
        }
        node.children
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let node = &self.document.nodes()[*id];
                if !matches!(node.kind, Kind::Table(_)) {
                    return Err(self.bad(key, "an array of tables is required"));
                }
                Ok(self.child(
                    format!("{}.{}[{index}]", self.at, key),
                    node.children.clone(),
                ))
            })
            .collect()
    }

    pub(super) fn strings(&mut self, key: &str) -> Result<Vec<String>, Error> {
        let value = self.take(key);
        strings(self.document, value, &format!("{}.{}", self.at, key))
    }

    pub(super) fn bad(&self, key: &str, reason: &str) -> Error {
        Error::Options(format!("{}.{}: {reason}", self.at, key))
    }

    pub(super) fn original(&self, id: usize) -> Result<String, Error> {
        let node = &self.document.nodes()[id];
        if !matches!(node.kind, Kind::Table(_) | Kind::ArrayOfTables) || node.children.is_empty() {
            return Ok(self.document.text()[node.span.clone()].into());
        }
        // An implicit or header-defined table's first token span is not its value.
        // Account its original descendant occurrences with the shared header index.
        let mut rows = Vec::new();
        self.child("value".into(), vec![id]).finish(&mut rows)?;
        Ok(rows
            .into_iter()
            .map(|row| format!("{}: {}", row.location, row.original))
            .collect::<Vec<_>>()
            .join("\n"))
    }

    pub(super) fn finish(self, unmapped: &mut Vec<UnmappedRow>) -> Result<(), Error> {
        let mut pending = purrdf_lex::walk::WorkList::<_, 32>::new();
        for id in self.table.into_iter().rev() {
            pending.try_push(id).map_err(Error::ImportStorage)?;
        }
        while let Some(id) = pending.pop() {
            let node = &self.document.nodes()[id];
            if let Some(headers) = self.headers.get(&id) {
                for index in headers {
                    let statement = &self.document.statements()[*index];
                    unmapped.push(UnmappedRow {
                        location: format!("{}{}:statement[{index}]", self.at, node.path),
                        original: self.document.text()[statement.span.clone()].into(),
                        reason: "unknown field declaration; no core or declared extension mapping"
                            .into(),
                    });
                }
            }
            let span = if node.children.is_empty() || matches!(node.kind, Kind::Scalar(_)) {
                node.span.clone()
            } else if let Some(key) = &node.key {
                key.span.clone()
            } else {
                node.span.start..node.span.start + 1
            };
            unmapped.push(UnmappedRow {
                location: format!("{}{}", self.at, node.path),
                original: self.document.text()[span].into(),
                reason: "unknown field or occurrence; no core or declared extension mapping".into(),
            });
            for child in node.children.iter().rev() {
                pending.try_push(*child).map_err(Error::ImportStorage)?;
            }
        }
        Ok(())
    }
}

pub(super) fn strings(
    document: &Document<'_>,
    value: Option<usize>,
    at: &str,
) -> Result<Vec<String>, Error> {
    let Some(id) = value else {
        return Ok(Vec::new());
    };
    let node = &document.nodes()[id];
    if !matches!(node.kind, Kind::Array | Kind::ArrayOfTables) {
        return Err(Error::Options(format!(
            "{at}: an array of strings is required"
        )));
    }
    node.children
        .iter()
        .map(|id| match &document.nodes()[*id].kind {
            Kind::Scalar(Scalar::String(text)) => Ok(text.clone()),
            _ => Err(Error::Options(format!(
                "{at}: an array of strings is required"
            ))),
        })
        .collect()
}
