// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

// oracle: agree
// probe demo::m1::s0 s0::B
mod m0 {
pub use crate::m1::*;
pub mod s0 {
pub use crate::*;
pub use crate::m1::s0::*;
}
}
pub mod m1 {
pub mod s0 {
pub(crate) use crate::m2::*;
use crate::m0::*;
pub(crate) const fn B() -> u32 { 8 }
}
pub(crate) struct B {} impl B { pub const TAG: u32 = 5; }
}
pub mod m2 {
pub(crate) use crate::m0::s0::B;
}
pub struct B {} impl B { pub const TAG: u32 = 1; }
