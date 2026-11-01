// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed instrumentation locators for an explicitly selected Rustdoc operation.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

/// Explicit Rustdoc delegate, observing driver and owned receipt directory.
///
/// This deliberately selects the documentation compiler. It does not discover
/// ambient Cargo configuration or attest any file merely from its locator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RustdocObservationOptions {
    driver: PathBuf,
    delegate: PathBuf,
    receipts: PathBuf,
    assembly: Option<(PathBuf, String)>,
}

/// An unsupported or ambiguous Rustdoc instrumentation locator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustdocObservationOptionsError(String);

impl fmt::Display for RustdocObservationOptionsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for RustdocObservationOptionsError {}

impl RustdocObservationOptions {
    pub(crate) const MARKER: &'static str = "RAF_RUSTDOC_OBSERVER";
    pub(crate) const DELEGATE: &'static str = "RAF_RUSTDOC_DELEGATE";
    pub(crate) const RECEIPTS: &'static str = "RAF_RUSTDOC_RECEIPTS";
    pub(crate) const ASSEMBLY: &'static str = "RAF_RUSTDOC_ASSEMBLY";
    pub(crate) const ASSEMBLY_DIGEST: &'static str = "RAF_RUSTDOC_ASSEMBLY_DIGEST";

    /// Select exact absolute driver, delegate and new receipt-directory locators.
    ///
    /// Filesystem identities are admitted by the documentation operation itself.
    ///
    /// # Errors
    /// Refuses relative, empty, non-UTF-8, control-bearing or oversized locators,
    /// and an immediately recursive driver/delegate selection.
    pub fn new(
        driver: &Path,
        delegate: &Path,
        receipts: &Path,
    ) -> Result<Self, RustdocObservationOptionsError> {
        for (name, path) in [
            ("driver", driver),
            ("delegate", delegate),
            ("receipts", receipts),
        ] {
            let text = path.to_str().ok_or_else(|| {
                RustdocObservationOptionsError(format!("Rustdoc {name} locator must be UTF-8"))
            })?;
            if !path.is_absolute() || text.len() > 4096 || text.chars().any(char::is_control) {
                return Err(RustdocObservationOptionsError(format!(
                    "Rustdoc {name} locator must be absolute, at most 4096 bytes and contain no controls",
                )));
            }
        }
        if driver == delegate {
            return Err(RustdocObservationOptionsError(
                "Rustdoc driver and delegate must differ".into(),
            ));
        }
        Ok(Self {
            driver: driver.into(),
            delegate: delegate.into(),
            receipts: receipts.into(),
            assembly: None,
        })
    }

    /// Select a closed, content-addressed assembly plan for pre-HTML includes.
    ///
    /// This opts into include generation and strict required-link diagnostics;
    /// it does not establish emitted link destinations or publish documentation.
    ///
    /// # Errors
    /// Refuses a nonabsolute, non-UTF-8, oversized or control-bearing locator.
    pub fn with_assembly(
        mut self,
        plan: &Path,
        digest: purrdf_core::ContentDigest,
    ) -> Result<Self, RustdocObservationOptionsError> {
        let text = plan.to_str().ok_or_else(|| {
            RustdocObservationOptionsError("assembly plan locator must be UTF-8".into())
        })?;
        if !plan.is_absolute() || text.len() > 4096 || text.chars().any(char::is_control) {
            return Err(RustdocObservationOptionsError(
                "assembly plan locator must be absolute, bounded and contain no controls".into(),
            ));
        }
        self.assembly = Some((plan.into(), digest.to_string()));
        Ok(self)
    }

    /// Exact optional assembly-plan locator and native content digest.
    #[must_use]
    pub fn assembly(&self) -> Option<(&Path, &str)> {
        self.assembly
            .as_ref()
            .map(|(path, digest)| (path.as_path(), digest.as_str()))
    }

    /// Driver executable deliberately supplied to Cargo as `RUSTDOC`.
    #[must_use]
    pub fn driver(&self) -> &Path {
        &self.driver
    }

    /// Exact documentation compiler delegated to by the observing driver.
    #[must_use]
    pub fn delegate(&self) -> &Path {
        &self.delegate
    }

    /// New owned directory receiving successful native invocation receipts.
    #[must_use]
    pub fn receipts(&self) -> &Path {
        &self.receipts
    }

    pub(crate) fn environment(&self) -> Vec<(&'static str, &OsStr)> {
        let mut values = vec![
            ("RUSTDOC", self.driver.as_os_str()),
            (Self::MARKER, OsStr::new("1")),
            (Self::DELEGATE, self.delegate.as_os_str()),
            (Self::RECEIPTS, self.receipts.as_os_str()),
        ];
        if let Some((path, digest)) = self.assembly() {
            values.push((Self::ASSEMBLY, path.as_os_str()));
            values.push((Self::ASSEMBLY_DIGEST, OsStr::new(digest)));
        }
        values
    }
}
