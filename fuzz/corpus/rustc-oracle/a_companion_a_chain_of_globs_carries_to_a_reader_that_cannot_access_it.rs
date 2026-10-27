// oracle: order
// probe demo::m3::s0 crate::m1::B
// probe demo::m3 crate::m1::B
use crate::m2::s0::*;
pub mod m1 {
pub mod s0 {
pub struct B {} impl B { pub const TAG: u32 = 8; }
}
pub(crate) use crate::m3::*;
}
mod m3 {
pub(crate) use super::*;
pub mod s0 {
}
}
mod m0 {
pub mod s0 {
use crate::m2::s1::*;
pub use crate::m1::s0::*;
}
}
pub mod m2 {
use crate::m0::s0::*;
pub mod s0 {
use super::*;
pub use crate::m1::s0::*;
}
pub mod s1 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 12; }
}
}
