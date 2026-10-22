// oracle: agree
// probe demo::m2::s0 self::A
pub mod m0 {
pub use crate::m3::*;
pub use crate::m1::s0::*;
pub use crate::m1::s1::*;
}
pub use crate::m0::*;
mod m1 {
pub mod s1 {
pub(crate) use crate::*;
use crate::m2::*;
}
pub mod s0 {
pub use crate::m1::*;
}
}
mod m2 {
pub mod s1 {
}
mod s0 {
}
}
mod m3 {
pub mod s1 {
}
}
