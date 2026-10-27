// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m0 crate::m3::s0::B
// probe demo::m1::s1 crate::m3::s0::C
// probe demo::m3 crate::m2::s0::B
// probe demo::m0::s0 crate::m3::s0::s0::A
// probe demo::m3 m0::A
// probe demo::m0::s1 s1::B
mod m0 {
mod s1 {
pub use crate::*;
struct B {} impl B { pub const TAG: u32 = 3; }
pub const fn A() -> u32 { 4 }
pub const fn B() -> u32 { 2 }
}
mod s0 {
pub(crate) use crate::m1::C;
use crate::A;
use crate::m3::s0::A as B;
}
pub(crate) use crate::m1::A as B;
pub use super::*;
pub(crate) struct A {} impl A { pub const TAG: u32 = 1; }
use crate::m3::*;
}
mod m3 {
pub mod s0 {
pub(crate) const fn A() -> u32 { 11 }
pub use crate::m0::*;
pub(crate) const fn C() -> u32 { 10 }
const fn B() -> u32 { 12 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 9; }
}
pub(crate) use crate::m2::*;
pub(crate) use crate::m1::s1::*;
}
mod m2 {
const fn A() -> u32 { 7 }
pub use crate::m0::*;
mod s0 {
struct B {} impl B { pub const TAG: u32 = 8; }
use crate::m1::s1::*;
use crate::m3::C;
pub(crate) use crate::m3::A;
}
}
pub mod m1 {
pub mod s0 {
pub(crate) use super::*;
}
pub use crate::*;
pub mod s1 {
pub const fn C() -> u32 { 6 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 5; }
use crate::*;
}
use crate::m2::A;
}
pub use crate::m3::s0::*;
