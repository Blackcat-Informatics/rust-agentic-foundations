// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed application-field consumption after the existing TOML parser.

use super::super::UnmappedRow;
use crate::Error;
use toml::{Table, Value};

pub(super) struct Fields {
    pub(super) at: String,
    pub(super) table: Table,
}

impl Fields {
    pub(super) fn new(at: impl Into<String>, table: Table) -> Self {
        Self {
            at: at.into(),
            table,
        }
    }

    pub(super) fn take(&mut self, key: &str) -> Option<Value> {
        self.table.remove(key)
    }

    pub(super) fn text(&mut self, key: &str, default: Option<&str>) -> Result<String, Error> {
        match self.take(key) {
            Some(Value::String(text)) if default.is_some() || !text.trim().is_empty() => Ok(text),
            None if default.is_some() => Ok(default.unwrap_or_default().into()),
            _ => Err(self.bad(key, "a string is required")),
        }
    }

    pub(super) fn flag(&mut self, key: &str, default: bool) -> Result<bool, Error> {
        match self.take(key) {
            Some(Value::Boolean(value)) => Ok(value),
            None => Ok(default),
            _ => Err(self.bad(key, "a boolean is required")),
        }
    }

    pub(super) fn integer(&mut self, key: &str) -> Result<Option<u64>, Error> {
        match self.take(key) {
            None => Ok(None),
            Some(Value::Integer(value)) => u64::try_from(value)
                .map(Some)
                .map_err(|_| self.bad(key, "a nonnegative integer is required")),
            _ => Err(self.bad(key, "a nonnegative integer is required")),
        }
    }

    pub(super) fn table(&mut self, key: &str) -> Result<Self, Error> {
        match self.take(key) {
            None => Ok(Self::new(format!("{}.{}", self.at, key), Table::new())),
            Some(Value::Table(value)) => Ok(Self::new(format!("{}.{}", self.at, key), value)),
            _ => Err(self.bad(key, "a table is required")),
        }
    }

    pub(super) fn rows(&mut self, key: &str) -> Result<Vec<Self>, Error> {
        match self.take(key) {
            None => Ok(Vec::new()),
            Some(Value::Array(rows)) => rows
                .into_iter()
                .enumerate()
                .map(|(index, value)| match value {
                    Value::Table(table) => {
                        Ok(Self::new(format!("{}.{}[{index}]", self.at, key), table))
                    }
                    _ => Err(self.bad(key, "an array of tables is required")),
                })
                .collect(),
            _ => Err(self.bad(key, "an array of tables is required")),
        }
    }

    pub(super) fn strings(&mut self, key: &str) -> Result<Vec<String>, Error> {
        strings(self.take(key), &format!("{}.{}", self.at, key))
    }

    pub(super) fn bad(&self, key: &str, reason: &str) -> Error {
        Error::Options(format!("{}.{}: {reason}", self.at, key))
    }

    pub(super) fn finish(self, unmapped: &mut Vec<UnmappedRow>) {
        for (key, value) in self.table {
            unmapped.push(UnmappedRow {
                location: format!("{}.{}", self.at, key),
                original: value.to_string(),
                reason: "unknown field; no core or declared extension mapping".into(),
            });
        }
    }
}

pub(super) fn strings(value: Option<Value>, at: &str) -> Result<Vec<String>, Error> {
    match value {
        None => Ok(Vec::new()),
        Some(Value::Array(values)) => values
            .into_iter()
            .map(|value| match value {
                Value::String(text) => Ok(text),
                _ => Err(Error::Options(format!(
                    "{at}: an array of strings is required"
                ))),
            })
            .collect(),
        _ => Err(Error::Options(format!(
            "{at}: an array of strings is required"
        ))),
    }
}
