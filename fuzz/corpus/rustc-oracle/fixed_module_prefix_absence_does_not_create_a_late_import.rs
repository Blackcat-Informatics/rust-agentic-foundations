// oracle: agree
// probe demo crate::m2::C
pub mod m2 {
use crate::m1::*;
pub use super::*;
}
use crate::m3::C;
pub use crate::m1::*;
mod m1 {
pub const fn C() -> u32 { 7 }
}
pub mod m3 {
pub struct C {} impl C { pub const TAG: u32 = 14; }
}
