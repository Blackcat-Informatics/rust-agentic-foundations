// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo crate::C
struct C {} impl C { pub const TAG: u32 = 1; }
mod m2 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 10; }
pub(crate) use crate::m3::*;
pub use crate::m0::*;
pub mod s1 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 13; }
use crate::m2::s0::*;
use crate::*;
}
use super::C;
pub mod s0 {
pub use crate::m0::*;
pub const fn B() -> u32 { 11 }
struct B {} impl B { pub const TAG: u32 = 12; }
}
pub use crate::m3::*;
}
mod m3 {
pub(crate) const fn A() -> u32 { 15 }
pub(crate) const fn C() -> u32 { 16 }
pub(crate) struct A {} impl A { pub const TAG: u32 = 14; }
use crate::m2::s1::*;
}
mod m1 {
mod s0 {
pub(crate) use crate::m1::B;
use crate::m2::s1::*;
const fn A() -> u32 { 6 }
pub(crate) const fn C() -> u32 { 5 }
}
pub mod s1 {
pub const fn C() -> u32 { 8 }
pub struct C {} impl C { pub const TAG: u32 = 7; }
use crate::m3::*;
pub struct B {} impl B { pub const TAG: u32 = 9; }
}
pub struct A {} impl A { pub const TAG: u32 = 4; }
pub struct B {} impl B { pub const TAG: u32 = 2; }
use self::s1::*;
pub(crate) use super::*;
const fn B() -> u32 { 3 }
}
mod m0 {
use crate::m1::s1::*;
use crate::m2::*;
pub use crate::m2::*;
}
pub use crate::m2::s0::*;
