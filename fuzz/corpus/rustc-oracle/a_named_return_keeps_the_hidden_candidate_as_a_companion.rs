// oracle: agree
// probe demo::m1 self::A
pub mod m0 {
pub use crate::m0::s1::*;
mod s0 {
use crate::*;
pub(crate) use crate::m1::s0::*;
}
use self::s0::*;
pub mod s1 {
pub(crate) struct A {} impl A { pub const TAG: u32 = 5; }
}
}
pub mod m1 {
pub mod s0 {
pub(crate) use crate::m0::A;
}
use crate::m1::s0::*;
}
pub(crate) struct A {} impl A { pub const TAG: u32 = 2; }
