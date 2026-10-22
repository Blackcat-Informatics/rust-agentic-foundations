// oracle: order
// probe demo::m1 self::A
struct A {} impl A { pub const TAG: u32 = 1; }
mod m0 {
use crate::m2::*;
pub use crate::m3::*;
}
mod m3 {
pub(crate) use crate::m2::s0::*;
}
mod m2 {
mod s1 {
pub(crate) use crate::m3::*;
}
use crate::m2::s1::*;
pub use crate::*;
pub mod s0 {
pub struct A {} impl A { pub const TAG: u32 = 8; }
pub const fn A() -> u32 { 9 }
}
}
mod m1 {
pub(crate) use crate::m0::*;
}
