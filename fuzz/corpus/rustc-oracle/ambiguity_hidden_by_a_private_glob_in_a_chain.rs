// oracle: agree
// probe demo::m0::s0 self::A
mod m3 {
use crate::m2::*;
pub(crate) use super::*;
}
pub mod m0 {
pub mod s0 {
pub(crate) use crate::m3::*;
}
}
pub mod m2 {
pub struct A {} impl A { pub const TAG: u32 = 15; }
}
pub(crate) struct A {} impl A { pub const TAG: u32 = 3; }
