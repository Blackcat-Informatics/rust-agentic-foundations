// oracle: documented
// probe demo::m1::s0 crate::m0::C
struct C {} impl C { pub const TAG: u32 = 2; }
pub mod m0 {
use crate::m1::*;
pub use crate::m1::s0::*;
}
mod m1 {
pub struct C {} impl C { pub const TAG: u32 = 7; }
pub mod s0 {
pub use super::*;
pub use crate::*;
}
}
