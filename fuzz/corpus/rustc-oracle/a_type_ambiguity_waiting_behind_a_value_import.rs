// oracle: agree
// probe demo::r self::Y
pub mod r {
pub use crate::m::*;
pub use self::B as Y;
}
pub mod m {
use crate::p::*;
use crate::f::B;
pub use crate::q::*;
}
pub mod p {
pub struct B {} impl B { pub const TAG: u32 = 1; }
}
pub mod q {
pub struct B {} impl B { pub const TAG: u32 = 2; }
}
pub mod f {
pub const fn B() -> u32 { 3 }
}
