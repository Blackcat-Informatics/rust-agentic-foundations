// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m1 self::C
// probe demo::m2::s1 super::A
// probe demo::m1 crate::m2::s1::A
// probe demo::m2 crate::m1::A
// probe demo m0::A
pub mod m2 {
pub mod s1 {
pub(crate) const fn B() -> u32 { 7 }
pub use super::*;
pub(crate) struct B {} impl B { pub const TAG: u32 = 6; }
pub(crate) struct A {} impl A { pub const TAG: u32 = 5; }
}
pub mod s0 {
pub use crate::m0::*;
pub use crate::*;
}
pub const fn A() -> u32 { 3 }
pub use crate::m0::*;
pub(crate) struct B {} impl B { pub const TAG: u32 = 2; }
struct C {} impl C { pub const TAG: u32 = 4; }
}
mod m0 {
pub use crate::m2::s1::A;
}
pub(crate) use crate::m2::s0::A;
pub mod m1 {
pub(crate) const fn A() -> u32 { 1 }
use crate::m2::s0::*;
use crate::m2::s0::*;
pub use super::*;
}
