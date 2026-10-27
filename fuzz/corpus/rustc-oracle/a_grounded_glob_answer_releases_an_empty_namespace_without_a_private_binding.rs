// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m0::s0 s0::A
pub const fn A() -> u32 { 3 }
mod m2 {
pub use crate::m2::s1::*;
mod s1 {
pub const fn B() -> u32 { 10 }
struct C {} impl C { pub const TAG: u32 = 9; }
pub use crate::*;
}
pub mod s0 {
const fn A() -> u32 { 8 }
pub use crate::m0::s1::*;
pub use crate::m0::s0::*;
}
use crate::m2::s0::A;
}
pub mod m0 {
pub mod s1 {
use crate::m1::*;
pub(crate) use crate::m1::A as C;
}
pub(crate) struct A {} impl A { pub const TAG: u32 = 6; }
pub(crate) struct B {} impl B { pub const TAG: u32 = 5; }
pub const fn B() -> u32 { 4 }
pub mod s0 {
pub(crate) use crate::m2::A;
use crate::m0::*;
use crate::m0::s1::*;
}
}
pub(crate) struct B {} impl B { pub const TAG: u32 = 1; }
mod m1 {
pub const fn A() -> u32 { 7 }
pub mod s0 {
use crate::m0::B;
use crate::m0::s1::*;
pub use crate::m0::*;
}
}
pub(crate) struct A {} impl A { pub const TAG: u32 = 2; }
