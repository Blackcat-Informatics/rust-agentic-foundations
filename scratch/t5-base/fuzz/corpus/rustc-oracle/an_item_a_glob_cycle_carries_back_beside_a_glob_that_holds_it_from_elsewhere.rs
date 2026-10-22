// oracle: agree
// probe demo::m3::s1 crate::m0::A
pub mod m1 {
pub const fn A() -> u32 { 7 }
}
pub mod m0 {
pub use crate::m3::s1::*;
use crate::m3::*;
}
mod m3 {
pub mod s0 {
pub const fn A() -> u32 { 15 }
}
use crate::m3::s0::*;
pub(crate) use crate::m0::*;
pub mod s1 {
pub use crate::m1::*;
}
}
