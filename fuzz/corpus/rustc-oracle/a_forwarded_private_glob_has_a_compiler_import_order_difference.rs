// oracle: order
// Reversing imports changes the compiler Type verdict; the index stays stable.
// probe demo::m0 self::C
// probe demo::m0 crate::m2::s0::m1::A
// probe demo::m1::s0 crate::m1::s0::s0::A
// probe demo::m1::s1 self::A
// probe demo m2::A
const fn B() -> u32 { 3 }
pub struct C {} impl C { pub const TAG: u32 = 1; }
const fn C() -> u32 { 2 }
pub mod m0 {
pub use crate::m2::s0::*;
pub use crate::m1::s0::*;
pub mod s0 {
pub struct C {} impl C { pub const TAG: u32 = 4; }
pub const fn A() -> u32 { 5 }
pub struct A {} impl A { pub const TAG: u32 = 6; }
}
}
pub mod m1 {
use crate::m0::*;
use crate::m1::s1::*;
pub mod s0 {
pub(crate) const fn A() -> u32 { 8 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 9; }
pub(crate) struct A {} impl A { pub const TAG: u32 = 7; }
pub(crate) const fn C() -> u32 { 10 }
}
pub mod s1 {
pub use crate::m1::s0::*;
pub use super::*;
}
}
pub mod m2 {
use crate::m1::s0::*;
pub(crate) use crate::m1::s0::*;
pub mod s0 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 12; }
use crate::m2::*;
pub const fn C() -> u32 { 15 }
pub struct B {} impl B { pub const TAG: u32 = 14; }
pub(crate) const fn A() -> u32 { 13 }
pub use crate::m2::s1::*;
}
pub(crate) const fn A() -> u32 { 11 }
mod s1 {
use crate::m1::s1::*;
const fn B() -> u32 { 16 }
pub use crate::m1::s1::*;
}
pub use crate::m1::s1::*;
}
