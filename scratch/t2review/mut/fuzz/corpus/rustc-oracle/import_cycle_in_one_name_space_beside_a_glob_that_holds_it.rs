// oracle: agree
// probe demo crate::m0::C
pub const fn C() -> u32 { 1 }
pub mod m1 {
pub use crate::*;
pub(crate) use crate::m0::C;
}
pub mod m0 {
pub use crate::m1::*;
pub struct C {} impl C { pub const TAG: u32 = 3; }
const fn C() -> u32 { 4 }
}
