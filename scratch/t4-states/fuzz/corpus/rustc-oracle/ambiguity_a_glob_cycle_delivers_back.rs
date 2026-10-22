// oracle: agree
// probe demo::m0 crate::A
use crate::m0::*;
mod m3 {
pub(crate) use self::s0::*;
mod s0 {
pub use crate::m0::s0::A;
}
}
pub mod m0 {
pub use super::*;
pub mod s0 {
pub const fn A() -> u32 { 4 }
}
use crate::m3::*;
}
pub mod m2 {
pub mod s1 {
pub const fn A() -> u32 { 18 }
}
}
pub use crate::m2::s1::*;
