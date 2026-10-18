// oracle: agree
// probe demo crate::m1::A
// probe demo::m1 self::A
mod m0 {
pub struct A {} impl A { pub const TAG: u32 = 1; }
pub const fn A() -> u32 { 2 }
}
mod m1 {
pub use crate::m0::*;
}
