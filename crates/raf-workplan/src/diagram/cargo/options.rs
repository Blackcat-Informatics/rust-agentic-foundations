// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Declared documentation selection; construction grants no execution authority.

use crate::identifiers::Package;
use std::fmt;
use std::path::{Path, PathBuf};

/// Explicit package, feature and output selection for documentation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoDocOptions {
    packages: Vec<Package>,
    features: Vec<String>,
    no_default_features: bool,
    target_directory: Option<PathBuf>,
}

/// A malformed or oversized declared documentation selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoDocOptionsError(String);

impl fmt::Display for CargoDocOptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CargoDocOptionsError {}

impl CargoDocOptions {
    /// Validate at most 256 packages and qualified features without starting tools.
    ///
    /// # Errors
    /// Refuses empty/repeated packages, invalid qualified features and invalid paths.
    pub fn new(
        packages: &[&str],
        features: &[&str],
        no_default_features: bool,
        target_directory: Option<&Path>,
    ) -> Result<Self, CargoDocOptionsError> {
        if packages.is_empty() || packages.len() > 256 || features.len() > 256 {
            return Err(CargoDocOptionsError(
                "select 1 to 256 packages and at most 256 qualified features".into(),
            ));
        }
        let mut selected = Vec::new();
        for name in packages {
            let package =
                Package::new(name).map_err(|error| CargoDocOptionsError(error.to_string()))?;
            if selected.contains(&package) {
                return Err(CargoDocOptionsError(format!(
                    "package `{name}` is repeated"
                )));
            }
            selected.push(package);
        }
        let mut requested: Vec<String> = Vec::new();
        for feature in features {
            let (package, name) = feature
                .split_once('/')
                .ok_or_else(|| CargoDocOptionsError("features must name package/feature".into()))?;
            if !packages.contains(&package)
                || name.is_empty()
                || feature.len() > 4096
                || !name
                    .chars()
                    .all(|value| value.is_alphanumeric() || matches!(value, '_' | '-' | '+' | '.'))
                || requested.iter().any(|previous| previous == *feature)
            {
                return Err(CargoDocOptionsError(format!(
                    "invalid or repeated qualified feature `{feature}`"
                )));
            }
            requested.push((*feature).to_string());
        }
        if let Some(path) = target_directory {
            let text = path
                .to_str()
                .ok_or_else(|| CargoDocOptionsError("target directory must be UTF-8".into()))?;
            if text.is_empty() || text.len() > 4096 || text.chars().any(char::is_control) {
                return Err(CargoDocOptionsError(
                    "target directory must be nonempty, bounded and contain no controls".into(),
                ));
            }
        }
        Ok(Self {
            packages: selected,
            features: requested,
            no_default_features,
            target_directory: target_directory.map(Path::to_path_buf),
        })
    }

    /// The explicitly selected packages.
    #[must_use]
    pub fn packages(&self) -> &[Package] {
        &self.packages
    }
    /// The qualified requested features, in request order.
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
    /// Whether default features were excluded from the request.
    #[must_use]
    pub const fn no_default_features(&self) -> bool {
        self.no_default_features
    }
    /// The declared output locator, which grants no filesystem authority itself.
    #[must_use]
    pub fn target_directory(&self) -> Option<&Path> {
        self.target_directory.as_deref()
    }
}
