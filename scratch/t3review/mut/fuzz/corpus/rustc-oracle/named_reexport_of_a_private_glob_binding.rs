// oracle: agree
// probe demo::n self::A
// probe demo::m self::A
pub mod x {
pub struct A {} impl A { pub const TAG: u32 = 1; }
}
mod m {
use crate::x::*;
pub const fn A() -> u32 { 2 }
pub mod c {
pub use super::A;
}
}
mod n {
pub use crate::m::c::*;
}
