// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m3 crate::m0::s1::A
// probe demo::m0 crate::m3::C
// probe demo::m2::s0 crate::m0::s1::m3::B
// probe demo::m3 s0::B
// probe demo::m1 s0::B
pub(crate) use crate::m1::*;
mod m2 {
mod s0 {
pub use crate::m1::*;
pub(crate) const fn B() -> u32 { 7 }
}
pub(crate) const fn C() -> u32 { 6 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 5; }
}
pub mod m0 {
mod s1 {
pub const fn A() -> u32 { 4 }
}
pub struct C {} impl C { pub const TAG: u32 = 2; }
use crate::m2::*;
pub const fn C() -> u32 { 3 }
mod s0 {
}
pub(crate) use self::s1::A;
}
pub struct C {} impl C { pub const TAG: u32 = 1; }
mod m3 {
mod s0 {
pub use crate::m3::*;
pub struct B {} impl B { pub const TAG: u32 = 8; }
}
use crate::m1::A as C;
pub use crate::m1::*;
}
pub mod m1 {
pub use crate::m0::*;
pub(crate) use crate::m0::*;
}
