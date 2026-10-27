// oracle: agree
// probe demo::m1::s1 self::B
mod m2 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 9; }
}
mod m1 {
use crate::m2::B;
mod s1 {
pub use crate::m0::*;
pub(crate) const fn B() -> u32 { 7 }
}
pub use crate::m1::s1::*;
}
pub(crate) use crate::m1::B;
mod m0 {
pub(crate) use crate::B;
}
