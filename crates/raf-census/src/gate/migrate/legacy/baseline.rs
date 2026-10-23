// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Foreign baseline records decoded by the native bounded JSON reader.

use std::collections::{BTreeMap, BTreeSet};

use purrdf_lex::json::{
    self, Value,
    record::{DecodeError, FromJson, Record},
};

use super::super::{MigrationInput, input};
use crate::{Error, MAX_LEDGER_BYTES};

pub(super) type Historical = BTreeMap<String, BTreeSet<String>>;

pub(super) fn read(source: MigrationInput<'_>) -> Result<Historical, Error> {
    let decoded = || {
        if source.text.len() > MAX_LEDGER_BYTES {
            return Err(Error::Options(format!(
                "foreign baseline exceeds {MAX_LEDGER_BYTES} bytes"
            )));
        }
        let limits = json::Limits {
            max_depth: 32,
            max_values: MAX_LEDGER_BYTES as u64,
            max_string_bytes: MAX_LEDGER_BYTES,
            unique_members: true,
        };
        let value = json::read_slice(source.text.as_bytes(), limits).map_err(Error::ImportJson)?;
        let (bodies, shims) = decode(&value).map_err(Error::ImportRecord)?;
        let mut historical = BTreeMap::new();
        for (kind, groups) in [("body", bodies), ("shim", shims)] {
            for group in groups {
                let hex = group
                    .fingerprint
                    .strip_prefix(kind)
                    .and_then(|text| text.strip_prefix(':'))
                    .ok_or_else(|| {
                        Error::Options(format!(
                            "legacy {kind} group has wrong fingerprint {:?}",
                            group.fingerprint
                        ))
                    })?;
                let bytes = purrdf_hash::hex::decode_canonical(hex).map_err(|error| {
                    Error::Options(format!(
                        "legacy fingerprint {:?}: {error}",
                        group.fingerprint
                    ))
                })?;
                if !matches!(bytes.len(), 8 | 32)
                    || group.members.is_empty()
                    || group
                        .members
                        .iter()
                        .any(|member| member.0.trim().is_empty())
                {
                    return Err(Error::Options(
                        "legacy groups require an 8- or 32-byte digest and nonempty member symbols"
                            .into(),
                    ));
                }
                let members = group.members.into_iter().map(|member| member.0).collect();
                if historical
                    .insert(group.fingerprint.clone(), members)
                    .is_some()
                {
                    return Err(Error::Options(format!(
                        "duplicate legacy fingerprint {:?}",
                        group.fingerprint
                    )));
                }
            }
        }
        Ok(historical)
    };
    decoded().map_err(|error| input(source.path, error))
}

fn decode(value: &Value) -> Result<(Vec<Group>, Vec<Group>), DecodeError> {
    let mut root = Record::new(value, "legacy census baseline")?;
    let _: usize = root.required("min_tokens")?;
    let _: usize = root.required("units")?;
    // Record::new has already established that this value is an object.
    let object = value.as_object().expect("record admitted an object");
    let bodies = if object.count("isomorphic_groups") > 0 {
        root.required("isomorphic_groups")?
    } else {
        root.required("body_groups")?
    };
    let shims = root.defaulted("shim_groups")?;
    let _: Vec<Value> = root.defaulted("hex_tables")?;
    let _: Vec<Value> = root.defaulted("shared_constants")?;
    root.deny_unknown()?;
    Ok((bodies, shims))
}

struct Group {
    fingerprint: String,
    members: Vec<Member>,
}

impl FromJson for Group {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        let mut row = Record::new(value, "legacy census group")?;
        let fingerprint = row.required("fingerprint")?;
        let members = row.required("members")?;
        let _: usize = row.required("tokens")?;
        let _: bool = row.defaulted("sanctioned")?;
        row.deny_unknown()?;
        Ok(Self {
            fingerprint,
            members,
        })
    }
}

struct Member(String);

impl FromJson for Member {
    fn from_json(value: &Value) -> Result<Self, DecodeError> {
        if let Some(symbol) = value.as_str() {
            return Ok(Self(symbol.into()));
        }
        let mut row = Record::new(value, "legacy census member")?;
        let symbol = row.required("symbol")?;
        let _: String = row.defaulted("package")?;
        let _: String = row.defaulted("file")?;
        let _: usize = row.defaulted("line")?;
        let _: bool = row.defaulted("shipping")?;
        row.deny_unknown()?;
        Ok(Self(symbol))
    }
}
