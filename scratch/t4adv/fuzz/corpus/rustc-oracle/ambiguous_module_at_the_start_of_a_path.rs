// oracle: agree
// probe demo::m2 s0::A
// probe demo::m2 s0::B
// probe demo::m2::s1 s0::A
// probe demo::m3 s0::A
pub mod m0 {
pub mod s0 {
pub struct A {} impl A { pub const TAG: u32 = 1; }
}
}
pub mod m1 {
pub mod s0 {
pub struct B {} impl B { pub const TAG: u32 = 2; }
}
}
pub mod m2 {
pub use crate::m0::*;
use crate::m1::*;
pub mod s1 {
use super::*;
}
}
pub mod m3 {
use crate::m2::*;
}
