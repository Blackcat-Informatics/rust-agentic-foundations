// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The RAF profile: the schema table and the value checks that need no
//! document.

mod schema;
mod timestamp;

pub use schema::{DocType, KeySpec, ValueKind};
pub use timestamp::{TimestampError, validate_timestamp};
