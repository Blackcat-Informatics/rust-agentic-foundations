// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m1::s0 crate::m1::s0::m2::B
// probe demo self::A
// probe demo::m0 crate::m0::m0::B
// probe demo::m1 m1::C
pub const fn B() -> u32 { 3 }
pub mod m1 {
pub(crate) use super::*;
pub mod s0 {
pub use crate::*;
}
pub const fn A() -> u32 { 8 }
pub struct B {} impl B { pub const TAG: u32 = 9; }
pub struct A {} impl A { pub const TAG: u32 = 7; }
}
pub const fn A() -> u32 { 2 }
struct A {} impl A { pub const TAG: u32 = 1; }
mod m2 {
pub struct A {} impl A { pub const TAG: u32 = 10; }
use crate::m1::s0::B;
pub(crate) use crate::m0::*;
}
mod m0 {
pub struct A {} impl A { pub const TAG: u32 = 4; }
pub(crate) const fn C() -> u32 { 6 }
pub use crate::m1::B;
pub use crate::m1::*;
pub(crate) struct C {} impl C { pub const TAG: u32 = 5; }
}
