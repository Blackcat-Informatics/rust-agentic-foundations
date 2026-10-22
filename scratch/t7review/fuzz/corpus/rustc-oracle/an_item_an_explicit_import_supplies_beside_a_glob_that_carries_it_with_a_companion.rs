// oracle: order
// probe demo::mm self::B
// probe demo::mm self::probe
pub mod m1 {
pub mod s1 {
pub struct B {} impl B { pub const TAG: u32 = 1; }
}
}
pub mod m2 {
pub struct B {} impl B { pub const TAG: u32 = 2; }
}
pub mod m0 {
pub use crate::m2::*;
use crate::m1::s1::*;
}
pub mod w {
pub use crate::m2::B;
}
pub mod mm {
pub use crate::m0::*;
pub use crate::w::*;
use self::B as probe;
}
