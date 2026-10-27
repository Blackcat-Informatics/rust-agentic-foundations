// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m2 crate::m2::C
pub struct A {} impl A { pub const TAG: u32 = 3; }
pub(crate) const fn C() -> u32 { 2 }
mod m1 {
pub struct C {} impl C { pub const TAG: u32 = 8; }
pub struct A {} impl A { pub const TAG: u32 = 6; }
pub const fn C() -> u32 { 9 }
pub(crate) struct B {} impl B { pub const TAG: u32 = 7; }
pub(crate) use crate::m0::*;
pub use crate::m0::*;
}
pub mod m0 {
use super::*;
pub(crate) use crate::m1::A;
use crate::m1::*;
pub use crate::m2::*;
pub const fn B() -> u32 { 5 }
pub struct B {} impl B { pub const TAG: u32 = 4; }
}
pub mod m2 {
pub(crate) use crate::m0::*;
const fn C() -> u32 { 11 }
pub(crate) const fn B() -> u32 { 10 }
pub(crate) use super::*;
}
pub(crate) struct C {} impl C { pub const TAG: u32 = 1; }
