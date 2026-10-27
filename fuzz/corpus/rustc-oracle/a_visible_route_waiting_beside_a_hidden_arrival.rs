// oracle: agree
// probe demo self::K
pub mod top {
pub mod t {
pub struct K {} impl K { pub const TAG: u32 = 2; }
}
pub mod u {
pub struct K {} impl K { pub const TAG: u32 = 1; }
}
pub use self::u as uu;
pub mod o {
pub use crate::top::b::m::*;
}
pub mod a {
pub(super) use super::s::*;
}
pub mod b {
pub mod m {
pub use crate::top::a::*;
pub use super::*;
pub use crate::top::t::*;
}
use super::s::*;
}
pub mod s {
pub use crate::top::uu::*;
}
}
pub use crate::top::o::*;
pub use crate::top::t::*;
