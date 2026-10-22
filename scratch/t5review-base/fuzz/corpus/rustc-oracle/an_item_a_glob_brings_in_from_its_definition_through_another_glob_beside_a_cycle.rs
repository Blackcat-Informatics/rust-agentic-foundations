// oracle: agree
// probe demo::m2::s1 crate::B
pub use crate::m1::s0::*;
pub mod m1 {
pub mod s0 {
pub use crate::m1::*;
}
pub(crate) const fn B() -> u32 { 1 }
}
use crate::m2::s1::*;
mod m2 {
pub const fn B() -> u32 { 6 }
pub mod s1 {
pub use crate::*;
use crate::m2::*;
}
}
