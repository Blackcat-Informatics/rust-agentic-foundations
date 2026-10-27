// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m1::s0 self::C
// probe demo::m2 crate::m2::m1::B
// probe demo::m0 self::C
// probe demo::m2::s0 crate::m1::s1::s1::C
// probe demo crate::A
// probe demo::m1 crate::C
// probe demo::m1::s1 s0::B
// probe demo m1::B
mod m2 {
pub mod s0 {
pub struct A {} impl A { pub const TAG: u32 = 11; }
pub const fn B() -> u32 { 10 }
}
pub(crate) use crate::m1::s0::A;
const fn C() -> u32 { 9 }
pub use crate::m1::*;
}
mod m0 {
use crate::m2::s0::*;
pub const fn A() -> u32 { 5 }
pub struct A {} impl A { pub const TAG: u32 = 4; }
}
mod m1 {
pub mod s0 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 8; }
use super::*;
pub use crate::B;
pub use super::*;
}
pub const fn B() -> u32 { 7 }
pub const fn C() -> u32 { 6 }
use self::s1::A;
pub mod s1 {
use crate::m1::s0::C;
pub use crate::*;
pub use crate::*;
}
}
pub use crate::m0::*;
pub use self::m0::A as B;
struct C {} impl C { pub const TAG: u32 = 2; }
struct A {} impl A { pub const TAG: u32 = 1; }
const fn C() -> u32 { 3 }
use self::m2::*;
