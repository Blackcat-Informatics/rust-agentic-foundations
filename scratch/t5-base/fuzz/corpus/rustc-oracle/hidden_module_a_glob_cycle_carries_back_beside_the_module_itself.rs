// oracle: agree
// probe demo::m3::s0 crate::s0::C
// probe demo crate::s0::C
// probe demo::m0::s0 self::s0::C
// probe demo::m2 self::s0::C
pub mod m3 {
pub mod s0 {
pub use crate::m0::s0::*;
}
}
mod m2 {
use crate::m1::*;
pub(crate) use super::*;
pub const fn C() -> u32 { 8 }
}
pub mod m0 {
pub mod s0 {
pub use crate::m2::*;
pub(crate) use super::*;
}
}
pub mod m1 {
pub mod s0 {
}
}
pub use crate::m3::s0::*;
