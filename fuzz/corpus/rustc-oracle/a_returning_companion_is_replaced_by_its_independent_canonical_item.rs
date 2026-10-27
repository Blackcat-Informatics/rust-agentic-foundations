// oracle: agree
// probe demo::m1::s0 crate::C
pub const fn C() -> u32 { 2 }
pub(crate) use self::m0::*;
mod m1 {
use super::*;
pub use crate::m2::*;
pub mod s0 {
}
}
mod m2 {
pub(crate) struct C {} impl C { pub const TAG: u32 = 13; }
pub mod s0 {
pub use crate::C;
}
}
pub use self::m1::*;
mod m0 {
pub struct C {} impl C { pub const TAG: u32 = 4; }
}
