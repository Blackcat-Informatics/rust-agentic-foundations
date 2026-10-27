// oracle: agree
// probe demo::m1::s1 crate::m1::s0::B
pub mod m2 {
pub(crate) use crate::m1::s1::*;
pub struct B {} impl B { pub const TAG: u32 = 9; }
pub mod s0 {
pub use crate::m2::*;
use crate::m0::*;
}
}
pub(crate) use crate::m2::s0::B;
pub mod m0 {
pub use crate::*;
pub mod s0 {
pub(crate) const fn B() -> u32 { 2 }
}
use crate::m1::*;
}
mod m1 {
pub(crate) struct B {} impl B { pub const TAG: u32 = 5; }
pub mod s1 {
pub use crate::m0::s0::*;
}
mod s0 {
pub use crate::*;
}
}
