// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Boolean properties of a declaration that are stated only as true.

use super::E;
use crate::report::shape;
use raf_rdf::Node;

/// Read an optional boolean property that is stated only as `true`, because its
/// absence already means false. `subject` names the declaration in a refusal and
/// `advice` says what to do instead of stating `false`.
pub(super) fn true_only(
    node: &Node<'_>,
    key: &str,
    subject: &str,
    advice: &str,
) -> Result<bool, E> {
    let mut flags = node.values(&shape::property(key));
    match (flags.next(), flags.next()) {
        (None, _) => Ok(false),
        (Some(value), None) if shape::boolean_value(value) == Some(true) => Ok(true),
        (Some(_), None) => Err(E::Value(format!(
            "{subject}: {key} is stated only as true; {advice}"
        ))),
        _ => Err(E::Repeated(format!("{subject} {key}"))),
    }
}
