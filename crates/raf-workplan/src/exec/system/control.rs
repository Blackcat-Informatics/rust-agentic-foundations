// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed private control tags shared by the embedded helper and its owner.

pub(crate) const READY: u8 = 1;
pub(crate) const FINISHED: u8 = 2;
pub(crate) const SPAWN_ERROR: u8 = 3;
pub(crate) const WAIT_ERROR: u8 = 4;
pub(crate) const MAX_ERROR: usize = 4096;
