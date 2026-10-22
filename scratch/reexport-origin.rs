// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

#![allow(dead_code)]
mod gen {
    macro_rules! make { () => { pub struct Made; }; }
    make!();
}
mod api {
    mod gen {
        macro_rules! make { () => { pub struct Made; }; }
        make!();
    }
    #[cfg(not(absolute))]
    pub use gen::Made;
    #[cfg(absolute)]
    pub use ::gen::Made;
    #[cfg(root_origin)]
    pub fn witness(value: crate::gen::Made) -> Made { value }
    #[cfg(module_origin)]
    pub fn witness(value: self::gen::Made) -> Made { value }
}
pub use api::*;
