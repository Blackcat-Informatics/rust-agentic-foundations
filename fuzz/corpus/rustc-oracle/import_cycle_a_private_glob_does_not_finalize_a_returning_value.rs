// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m2::s1 crate::m2::m2::B
// probe demo::m0 crate::m2::s0::B
// probe demo self::m0::C
// probe demo::m0 crate::m1::C
// probe demo::m0 crate::m2::m2::C
// probe demo::m0 s1::A
// probe demo m1::C
mod m1 {
pub use crate::m2::s0::*;
pub use crate::m3::B;
use crate::m0::C;
}
const fn A() -> u32 { 1 }
pub mod m2 {
pub mod s0 {
use crate::m1::*;
pub(crate) use crate::m0::*;
pub(crate) struct B {} impl B { pub const TAG: u32 = 5; }
struct A {} impl A { pub const TAG: u32 = 6; }
const fn C() -> u32 { 7 }
}
pub(crate) use crate::m1::*;
pub(crate) use super::*;
mod s1 {
}
}
mod m0 {
pub struct C {} impl C { pub const TAG: u32 = 2; }
mod s0 {
pub(crate) use super::C;
}
pub use crate::m1::B;
mod s1 {
const fn A() -> u32 { 4 }
struct A {} impl A { pub const TAG: u32 = 3; }
pub(crate) use crate::m2::s0::*;
use crate::*;
pub use crate::*;
}
pub(crate) use crate::m3::*;
}
mod m3 {
pub struct B {} impl B { pub const TAG: u32 = 8; }
pub struct A {} impl A { pub const TAG: u32 = 9; }
pub const fn A() -> u32 { 10 }
pub const fn B() -> u32 { 11 }
}
