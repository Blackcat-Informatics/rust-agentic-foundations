// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Preserve library and bounded host execution failures at the CLI boundary.

use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub(crate) enum Error {
    Census(raf_census::Error),
    Embedded(raf_census::embedded::Error),
    Source(raf_source::LoadError),
}

impl Error {
    pub(crate) const fn exit_code(&self) -> u8 {
        if matches!(
            self,
            Self::Census(raf_census::Error::RegistrationRequired { .. })
        ) {
            2
        } else {
            1
        }
    }
}

impl From<raf_census::Error> for Error {
    fn from(error: raf_census::Error) -> Self {
        Self::Census(error)
    }
}

impl From<raf_census::embedded::Error> for Error {
    fn from(error: raf_census::embedded::Error) -> Self {
        Self::Embedded(error)
    }
}

impl From<raf_source::LoadError> for Error {
    fn from(error: raf_source::LoadError) -> Self {
        Self::Source(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Census(error) => error.fmt(f),
            Self::Embedded(error) => error.fmt(f),
            Self::Source(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Census(error) => Some(error),
            Self::Embedded(error) => Some(error),
            Self::Source(error) => Some(error),
        }
    }
}
