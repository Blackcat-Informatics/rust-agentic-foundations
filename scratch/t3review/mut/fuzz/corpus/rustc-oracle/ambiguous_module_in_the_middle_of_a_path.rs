// oracle: agree
// probe demo::m0 crate::m0::s0::s0::B
pub mod m1 {
pub mod s0 {
pub const fn B() -> u32 { 13 }
}
}
pub mod m0 {
pub mod s0 {
pub use super::*;
const fn B() -> u32 { 5 }
pub(crate) use crate::m1::*;
}
}
