// oracle: agree
// probe demo::m1 crate::m1::m0::C
// probe demo::m3 crate::m3::C
// probe demo::m3 crate::m3::B
// probe demo::m3 self::B
// probe demo::m1::s0 crate::m3::C
// probe demo m1::B
pub const fn B() -> u32 { 1 }
mod m0 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 3; }
pub(crate) struct C {} impl C { pub const TAG: u32 = 2; }
pub(crate) const fn B() -> u32 { 4 }
use crate::m1::s0::*;
use super::*;
pub(crate) struct A {} impl A { pub const TAG: u32 = 5; }
}
mod m2 {
pub(crate) use super::B;
use crate::m3::*;
use crate::m1::s0::C;
}
pub mod m1 {
pub use crate::m1::s0::*;
use crate::m3::*;
pub mod s0 {
pub struct C {} impl C { pub const TAG: u32 = 7; }
const fn B() -> u32 { 6 }
}
use crate::m2::B;
pub(crate) use crate::m0::*;
}
pub mod m3 {
pub const fn A() -> u32 { 8 }
}
