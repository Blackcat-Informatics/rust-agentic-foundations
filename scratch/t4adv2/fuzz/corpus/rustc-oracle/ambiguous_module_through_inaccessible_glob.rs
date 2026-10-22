// oracle: agree
// probe demo::m3::s1 crate::m1::s0::s0::A
mod m2 {
mod s1 {
pub use crate::*;
}
pub mod s0 {
use crate::m2::s1::*;
}
}
struct A {} impl A { pub const TAG: u32 = 1; }
mod m3 {
pub mod s0 {
pub struct A {} impl A { pub const TAG: u32 = 16; }
}
pub mod s1 {
}
}
pub mod m0 {
pub(crate) use crate::m2::s0::*;
pub(crate) use crate::m3::*;
}
mod m1 {
pub mod s0 {
pub use super::*;
use crate::m0::*;
}
}
