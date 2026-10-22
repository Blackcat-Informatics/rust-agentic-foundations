// oracle: agree
// probe demo::m2 self::B
pub use crate::m2::*;
pub mod m1 {
pub struct C {} impl C { pub const TAG: u32 = 3; }
pub struct B {} impl B { pub const TAG: u32 = 4; }
}
mod m2 {
use crate::m0::s1::B;
pub const fn C() -> u32 { 6 }
}
pub mod m0 {
pub use crate::m1::*;
pub mod s0 {
pub use crate::m0::s1::*;
pub use crate::m2::*;
}
pub mod s1 {
use crate::m2::C;
pub(crate) use super::*;
pub use crate::m0::s0::C as B;
}
}
