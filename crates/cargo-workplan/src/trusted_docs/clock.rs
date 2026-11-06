// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The explicit trusted documentation operation's sole direct-child watchdog.

#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "only this private module reads a clock and pauses for the explicit trusted documentation watchdog; no test asserts clock behavior"
)]

use std::time::{Duration, Instant};

pub(super) struct Watchdog {
    started: Instant,
    limit: Duration,
}
impl Watchdog {
    pub(super) fn start(build: bool) -> Self {
        Self {
            started: Instant::now(),
            limit: Duration::from_secs(if build { 1800 } else { 300 }),
        }
    }
    pub(super) fn expired(&self) -> bool {
        self.started.elapsed() >= self.limit
    }
    pub(super) fn pause() {
        std::thread::sleep(Duration::from_millis(10));
    }
}
