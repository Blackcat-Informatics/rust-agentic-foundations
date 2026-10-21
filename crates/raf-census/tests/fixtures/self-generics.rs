// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Valid source neighbours for generic callee classification.

pub fn take<T>(_: u8, _: u8) {}

pub trait Factory {
    type Assoc<F>;

    fn kept() {
        take::<Self::Assoc<fn()>>(1, 2)
    }

    fn kept_arrow() {
        take::<Self::Assoc<fn() -> u8>>(1, 2)
    }

    fn go<'a: 'a>(value: &'a u8) -> u8 {
        *value
    }

    fn excluded<'a: 'a>(value: &'a u8) -> u8 {
        Self::go::<'a>(value)
    }

    fn generic<F>(value: u8) -> u8 {
        value
    }

    fn excluded_arrow(value: u8) -> u8 {
        Self::generic::<fn() -> u8>(value)
    }
}
