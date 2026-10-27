// oracle: order
// The compiler resolves the type in one declaration order and reports E0659
// in another. The index retains stable bindings and an identical public surface.
// probe demo::m1::s0 crate::m3::C
// probe demo::m1 crate::m1::C
// probe demo::m2::s0 crate::m0::s1::B
// probe demo::m1 self::B
// probe demo::m1::s0 self::B
// probe demo::m2::s0 s0::B
// probe demo s1::B
pub mod m1 {
pub(crate) use crate::m0::*;
pub mod s1 {
struct C {} impl C { pub const TAG: u32 = 13; }
struct A {} impl A { pub const TAG: u32 = 14; }
pub use crate::m0::*;
const fn A() -> u32 { 15 }
}
pub mod s0 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 10; }
pub(crate) const fn B() -> u32 { 11 }
pub struct C {} impl C { pub const TAG: u32 = 12; }
}
pub(crate) use crate::m0::s1::*;
}
use crate::m0::s1::*;
pub mod m3 {
pub(crate) use crate::m1::s0::B;
struct C {} impl C { pub const TAG: u32 = 20; }
}
pub mod m0 {
pub mod s1 {
pub(crate) const fn B() -> u32 { 8 }
use crate::m1::s0::*;
pub use crate::m1::*;
pub use crate::m0::s0::*;
struct B {} impl B { pub const TAG: u32 = 9; }
}
pub mod s0 {
struct B {} impl B { pub const TAG: u32 = 5; }
use crate::m0::*;
pub struct C {} impl C { pub const TAG: u32 = 7; }
pub(crate) const fn B() -> u32 { 6 }
pub use crate::m0::*;
}
pub struct C {} impl C { pub const TAG: u32 = 4; }
const fn A() -> u32 { 3 }
}
const fn B() -> u32 { 2 }
pub const fn C() -> u32 { 1 }
pub mod m2 {
mod s0 {
use crate::m1::s1::*;
use crate::m1::s0::*;
pub const fn B() -> u32 { 19 }
pub(crate) use crate::*;
pub struct B {} impl B { pub const TAG: u32 = 18; }
}
struct A {} impl A { pub const TAG: u32 = 16; }
pub use crate::m0::s1::*;
pub const fn A() -> u32 { 17 }
pub(crate) use crate::m0::s1::B;
}
