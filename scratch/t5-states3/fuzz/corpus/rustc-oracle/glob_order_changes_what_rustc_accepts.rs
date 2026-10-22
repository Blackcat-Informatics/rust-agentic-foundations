// oracle: order
// probe demo::m1 self::A
mod m0 {
use self::s1::*;
pub use super::*;
mod s1 {
pub const fn A() -> u32 { 5 }
}
}
pub mod m1 {
pub use crate::m0::A;
pub use crate::*;
}
pub struct A {} impl A { pub const TAG: u32 = 1; }
pub const fn A() -> u32 { 2 }
