// oracle: agree
// probe demo::m2::s0 self::B
mod m0 {
pub mod s0 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 6; }
}
}
mod m2 {
pub mod s1 {
pub use crate::m0::s0::*;
use crate::m1::*;
}
pub mod s0 {
pub use crate::m2::s1::*;
}
pub struct B {} impl B { pub const TAG: u32 = 9; }
}
mod m1 {
use crate::m2::*;
pub use crate::m0::s0::*;
}
