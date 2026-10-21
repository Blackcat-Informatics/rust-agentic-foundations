// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The version of the normal form and its gate.

use crate::Error;

/// The version of the normal form this crate implements. Any change to what a
/// fingerprint hashes bumps it.
pub const NORMAL_FORM: u32 = 1;

/// Accepts `declared` when it is [`NORMAL_FORM`].
///
/// # Errors
///
/// [`Error::Version`] for any other version; its message names both versions
/// and `cargo census migrate`.
pub fn check_normal_form(declared: u32) -> Result<(), Error> {
    if declared == NORMAL_FORM {
        Ok(())
    } else {
        Err(Error::Version {
            declared,
            implemented: NORMAL_FORM,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_implemented_version_is_one() {
        assert_eq!(NORMAL_FORM, 1);
    }

    #[test]
    fn the_implemented_version_passes() {
        assert!(check_normal_form(1).is_ok());
    }

    #[test]
    fn other_versions_are_refused_naming_both_and_the_migration() {
        for declared in [0, 2] {
            let message = check_normal_form(declared)
                .expect_err("a version other than 1 is refused")
                .to_string();
            assert!(
                message.contains(&format!("normal_form {declared}")),
                "{message}"
            );
            assert!(message.contains("normal_form 1"), "{message}");
            assert!(message.contains("cargo census migrate"), "{message}");
        }
    }
}
