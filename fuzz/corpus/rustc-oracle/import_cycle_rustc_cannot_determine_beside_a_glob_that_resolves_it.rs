// oracle: agree
// probe demo crate::s0::B
pub use self::m0::*;
pub mod m1 {
pub mod s0 {
pub(crate) use crate::m0::*;
}
}
pub(crate) use crate::m0::s1::B;
pub mod m0 {
pub mod s1 {
pub(crate) use crate::m0::*;
}
pub mod s0 {
use crate::m1::s0::*;
pub use crate::*;
}
pub(crate) const fn B() -> u32 { 5 }
}
