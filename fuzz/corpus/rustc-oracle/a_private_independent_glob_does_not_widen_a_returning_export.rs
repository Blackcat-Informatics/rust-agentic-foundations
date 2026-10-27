// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m2 crate::m2::C
pub(crate) struct C {} impl C { pub const TAG: u32 = 1; }
mod m1 {
pub struct C {} impl C { pub const TAG: u32 = 8; }
}
pub mod m0 {
use crate::*;
use crate::m1::*;
pub use crate::m2::*;
}
pub mod m2 {
pub(crate) use crate::m0::*;
pub(crate) use crate::*;
const fn C() -> u32 { 11 }
}
