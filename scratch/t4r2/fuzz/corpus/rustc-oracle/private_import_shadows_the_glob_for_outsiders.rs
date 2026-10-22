// oracle: agree
// probe demo::m1 self::A
// probe demo::m0 self::A
mod m1 {
pub use crate::m0::*;
}
mod m2 {
pub const fn C() -> u32 { 8 }
pub const fn A() -> u32 { 6 }
}
mod m0 {
pub use crate::m2::*;
use crate::m2::C as A;
}
