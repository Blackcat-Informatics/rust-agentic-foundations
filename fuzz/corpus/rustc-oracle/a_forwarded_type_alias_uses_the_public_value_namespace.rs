// oracle: agree
// probe demo::m3::s1 self::C
// probe demo::m1 crate::m1::s0::A
// probe demo::m3::s1 crate::m3::s0::s0::A
// probe demo::m2 crate::m3::s1::A
// probe demo::m1 s0::B
// probe demo m1::C
mod m1 {
mod s0 {
pub use crate::m3::s0::*;
use crate::m3::s1::A as B;
use crate::m3::s0::*;
}
pub mod s1 {
pub const fn C() -> u32 { 6 }
pub struct B {} impl B { pub const TAG: u32 = 7; }
pub struct C {} impl C { pub const TAG: u32 = 5; }
const fn A() -> u32 { 4 }
pub const fn B() -> u32 { 8 }
}
pub use crate::m3::s1::*;
use crate::m0::*;
}
pub mod m2 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 9; }
}
mod m3 {
pub const fn A() -> u32 { 11 }
pub struct A {} impl A { pub const TAG: u32 = 10; }
pub use crate::*;
pub mod s0 {
pub(crate) const fn B() -> u32 { 13 }
pub const fn A() -> u32 { 15 }
pub struct A {} impl A { pub const TAG: u32 = 14; }
pub(crate) use crate::m1::s1::*;
pub(crate) struct B {} impl B { pub const TAG: u32 = 12; }
}
pub mod s1 {
const fn A() -> u32 { 16 }
pub use crate::m2::*;
}
}
mod m0 {
pub(crate) const fn C() -> u32 { 2 }
pub(crate) struct C {} impl C { pub const TAG: u32 = 1; }
struct A {} impl A { pub const TAG: u32 = 3; }
}
