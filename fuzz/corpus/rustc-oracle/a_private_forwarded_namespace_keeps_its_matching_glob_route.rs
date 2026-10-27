// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m3::s1 crate::m1::m0::C
// probe demo::m3::s0 crate::m0::C
// probe demo::m0::s0 crate::m1::s0::s0::C
// probe demo::m2 crate::m2::s1::C
// probe demo::m1 s0::B
// probe demo::m3 s0::B
pub mod m3 {
pub(crate) const fn C() -> u32 { 18 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 17; }
pub mod s0 {
use crate::m1::s0::*;
use crate::m0::s0::*;
pub(crate) use crate::m3::s1::*;
pub(crate) struct A {} impl A { pub const TAG: u32 = 20; }
pub(crate) struct B {} impl B { pub const TAG: u32 = 19; }
}
struct B {} impl B { pub const TAG: u32 = 16; }
mod s1 {
pub use crate::m1::B;
}
}
pub(crate) const fn A() -> u32 { 1 }
use self::m3::*;
pub mod m1 {
pub struct B {} impl B { pub const TAG: u32 = 5; }
pub use crate::*;
pub const fn B() -> u32 { 6 }
pub struct A {} impl A { pub const TAG: u32 = 7; }
pub mod s0 {
const fn A() -> u32 { 9 }
struct A {} impl A { pub const TAG: u32 = 8; }
}
}
pub mod m2 {
use crate::m1::s0::*;
pub(crate) const fn A() -> u32 { 12 }
pub mod s1 {
pub const fn B() -> u32 { 15 }
pub use crate::m1::*;
use crate::m3::s0::*;
use crate::m2::s0::C;
}
struct A {} impl A { pub const TAG: u32 = 10; }
pub const fn B() -> u32 { 11 }
mod s0 {
pub use crate::m3::*;
pub(crate) use crate::m0::A;
struct C {} impl C { pub const TAG: u32 = 13; }
pub use crate::m0::s0::*;
pub const fn B() -> u32 { 14 }
}
use crate::m2::s0::*;
}
pub mod m0 {
pub mod s0 {
pub use crate::m3::s0::*;
struct A {} impl A { pub const TAG: u32 = 4; }
}
const fn C() -> u32 { 2 }
pub struct B {} impl B { pub const TAG: u32 = 3; }
pub(crate) use crate::m2::*;
}
