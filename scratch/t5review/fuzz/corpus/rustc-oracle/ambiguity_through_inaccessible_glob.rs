// oracle: agree
// probe demo::m0 crate::m1::s0::s0::B
const fn B() -> u32 { 2 }
pub mod m1 {
pub(crate) use crate::*;
pub mod s0 {
pub use crate::m2::*;
pub const fn B() -> u32 { 7 }
}
}
pub mod m2 {
pub mod s0 {
pub use crate::m0::*;
}
}
pub mod m0 {
pub(crate) use crate::m1::*;
use crate::m1::s0::*;
}
