// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m3 crate::m0::s1::s0::A
// probe demo::m1::s0 crate::m0::s0::C
// probe demo::m2::s0 crate::m1::s1::s0::B
// probe demo::m1 crate::m2::A
// probe demo self::m2::A
// probe demo::m2::s0 crate::m1::s0::B
// probe demo::m2 m3::A
const fn A() -> u32 { 1 }
pub mod m1 {
pub use crate::m2::*;
pub mod s1 {
use crate::m3::s0::*;
use crate::m2::s0::*;
use crate::m0::s0::A;
pub use crate::m0::s1::C;
}
pub(crate) use crate::m0::s0::*;
pub mod s0 {
pub const fn B() -> u32 { 5 }
pub const fn A() -> u32 { 4 }
pub use crate::*;
pub struct A {} impl A { pub const TAG: u32 = 3; }
pub(crate) use crate::m3::*;
use crate::m0::s0::*;
}
}
pub use crate::m1::s1::*;
pub mod m3 {
pub use crate::m0::s1::*;
struct B {} impl B { pub const TAG: u32 = 11; }
use crate::m0::s1::*;
pub mod s0 {
pub use crate::m1::s1::*;
struct B {} impl B { pub const TAG: u32 = 12; }
const fn B() -> u32 { 13 }
}
}
pub mod m2 {
pub mod s0 {
use crate::*;
pub struct C {} impl C { pub const TAG: u32 = 9; }
pub(crate) struct B {} impl B { pub const TAG: u32 = 8; }
pub use crate::m1::s1::*;
pub const fn C() -> u32 { 10 }
}
pub use crate::m0::s0::*;
use crate::*;
pub const fn A() -> u32 { 7 }
pub const fn C() -> u32 { 6 }
}
pub struct C {} impl C { pub const TAG: u32 = 2; }
mod m0 {
pub mod s1 {
pub use crate::C;
}
pub use self::s0::*;
pub use crate::m0::s1::C;
pub(crate) use crate::m2::s0::*;
pub mod s0 {
pub use crate::m2::s0::*;
use crate::m1::s1::C;
pub(crate) use crate::m1::s0::*;
}
pub use crate::*;
}
