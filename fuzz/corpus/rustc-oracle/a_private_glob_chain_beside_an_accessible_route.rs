// oracle: agree
// probe demo self::L
use crate::top0::t::*;
pub mod top0 {
pub use crate::top0::u as d0;
pub mod s {
pub use crate::top0::d0::*;
}
pub mod u {
pub struct L {} impl L { pub const TAG: u32 = 1; }
}
pub mod t {
pub use crate::top0::o::*;
}
pub mod a {
pub(crate) use crate::top0::s::*;
pub mod p0 {
pub(crate) mod c0 {
pub struct L {} impl L { pub const TAG: u32 = 6; }
}
}
}
pub mod b {
pub mod m {
pub(crate) use crate::top0::a::*;
use crate::top0::a::p0::c0::*;
}
}
pub mod o {
pub use crate::top0::b::m::*;
pub(super) use crate::p0::c0::*;
}
}
pub mod p0 {
pub use crate::top0::a::*;
pub mod c0 {
pub use super::*;
}
}
