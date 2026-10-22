// oracle: agree
// probe demo crate::A
// probe demo::m::c crate::A
// probe demo::m self::A
// probe demo crate::m::A
pub mod x {
pub struct A {} impl A { pub const TAG: u32 = 1; }
}
pub mod p {
pub struct A {} impl A { pub const TAG: u32 = 2; }
}
use crate::m::*;
pub use crate::x::*;
pub mod m {
pub use super::*;
use crate::p::*;
pub mod c {
}
}
