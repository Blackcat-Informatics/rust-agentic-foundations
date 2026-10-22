// oracle: agree
// probe demo crate::m::c::A
// probe demo crate::n::A
pub struct A {} impl A { pub const TAG: u32 = 1; }
mod m {
pub use crate::*;
pub mod c {
pub use super::A;
}
}
mod n {
pub use crate::m::c::*;
}
