// oracle: agree
// probe demo::m0::s0 s0::B
pub mod m0 {
pub mod s0 {
use crate::m1::s0::*;
pub use crate::*;
}
}
use self::m1::*;
mod m1 {
pub mod s0 {
use crate::m0::*;
pub struct B {} impl B { pub const TAG: u32 = 7; }
pub use crate::m2::s0::*;
}
}
pub mod m2 {
pub mod s0 {
pub(crate) use crate::m0::s0::*;
}
}
