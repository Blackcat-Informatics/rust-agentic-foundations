// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo crate::m1::B
pub const fn B() -> u32 { 1 }
mod m2 {
pub(crate) use crate::B;
}
mod d {
pub struct B {} impl B { pub const TAG: u32 = 3; }
pub const fn B() -> u32 { 9 }
}
mod a {
pub use crate::b::B;
pub use crate::d::*;
}
mod b {
pub use crate::a::B;
}
pub mod m1 {
use crate::m2::B;
pub use crate::a::*;
}
