// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

extern crate self as renamed;
pub fn local_fn() {}
pub fn encode(_: u8) {}
mod qualified_value_binding {
    pub fn check() {
        let std = ();
        std::mem::drop(std);
    }
}
mod qualified_value_import {
    pub fn check() {
        use crate::local_fn as std;
        std::mem::drop(0_u8);
    }
}
mod same_name_standard_import {
    pub fn check() {
        use std::mem::drop as std;
        std(0_u8);
    }
}
mod dependency_type_head {
    pub fn check() {
        use crate::local_fn as renamed;
        renamed::encode(0);
    }
}
mod separate_import_namespaces {
    mod types {
        pub mod chosen {
            pub fn encode(_: u8) {}
        }
    }
    mod values {
        pub fn chosen() {}
    }
    pub fn check() {
        use self::types::chosen;
        use self::values::chosen;
        chosen::encode(0);
        chosen();
    }
}
mod outer_value_binding {
    mod types {
        pub struct Chosen {}
    }
    pub fn check() {
        let chosen = || {};
        {
            use self::types::Chosen as chosen;
            chosen();
        }
    }
}
mod qualified_macro_head {
    pub fn check() {
        macro_rules! std {
            () => {};
        }
        std::println!("probe");
        std!();
    }
}
mod bare_macro_shadow {
    pub fn check() {
        macro_rules! println {
            ($($arg:tt)*) => {};
        }
        println!("probe");
    }
}
mod bare_value_shadow {
    pub fn check() {
        use std::mem::drop as local;
        let local = || {};
        local();
    }
}
mod genuine_type_shadow {
    pub fn check() {
        mod std {
            pub mod mem {
                pub fn drop(_: u8) {}
            }
        }
        std::mem::drop(0);
    }
}
mod genuine_type_import {
    mod local {
        pub fn encode(_: u8) {}
    }
    pub fn check() {
        use self::local as renamed;
        renamed::encode(0);
    }
}
fn main() {}
