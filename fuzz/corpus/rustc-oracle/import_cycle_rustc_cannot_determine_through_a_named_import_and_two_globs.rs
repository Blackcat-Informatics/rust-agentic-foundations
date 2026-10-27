// oracle: agree
// probe demo::m1::s1 crate::m2::A
pub mod m0 {
pub use crate::m1::s0::*;
}
pub mod m2 {
pub(crate) use crate::m1::s1::*;
use crate::m0::*;
}
mod m1 {
pub mod s1 {
pub use crate::m0::A;
}
pub mod s0 {
pub const fn A() -> u32 { 4 }
}
}
