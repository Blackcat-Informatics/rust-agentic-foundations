// oracle: agree
// probe demo self::m2::B
// probe demo::m0::s0 self::B
pub mod m2 {
pub use crate::m0::s0::*;
}
pub mod m0 {
pub mod s0 {
pub use super::*;
}
struct B {} impl B { pub const TAG: u32 = 4; }
}
