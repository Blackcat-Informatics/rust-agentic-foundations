// oracle: agree
// probe demo crate::m0::A
// probe demo crate::m0::B
// probe demo::m0 self::B
mod m1 {
pub struct A {} impl A { pub const TAG: u32 = 1; }
}
mod m0 {
pub use crate::m1::*;
use self::s0::*;
mod s0 {
pub struct B {} impl B { pub const TAG: u32 = 2; }
}
}
