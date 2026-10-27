// oracle: agree
// probe demo m1::B
pub const fn B() -> u32 { 1 }
mod m0 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 3; }
pub(crate) const fn B() -> u32 { 4 }
}
mod m2 {
pub(crate) use super::B;
}
pub mod m1 {
use crate::m2::B;
pub(crate) use crate::m0::*;
}
