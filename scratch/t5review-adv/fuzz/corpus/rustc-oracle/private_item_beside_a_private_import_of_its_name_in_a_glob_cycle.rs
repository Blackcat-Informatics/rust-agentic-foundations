// oracle: agree
// probe demo::m2 crate::m0::B
// probe demo::m1 self::B
// probe demo::m0 self::B
pub mod m2 {
pub struct B {} impl B { pub const TAG: u32 = 6; }
pub const fn B() -> u32 { 7 }
}
mod m3 {
pub use crate::m1::*;
pub(crate) use crate::m2::*;
}
pub mod m0 {
struct B {} impl B { pub const TAG: u32 = 2; }
pub(crate) use crate::m1::*;
pub(crate) use crate::m3::*;
}
mod m1 {
use crate::m0::B;
use crate::*;
}
pub use self::m3::*;
