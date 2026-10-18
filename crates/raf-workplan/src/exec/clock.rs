// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The one place in this crate that reads a clock or waits on one.
//!
//! The runner measures how long a process has run and pauses between two looks
//! at it. Both go through [`Clock`], so the runner's decisions can be driven by
//! a clock that a test controls, and no other file has to name a clock at all.
//! [`WallClock`] is the real reading; it is the only implementation outside
//! tests.

use std::thread;
use std::time::{Duration, Instant};

/// A reading of the time since a run started, and a way to wait.
pub(super) trait Clock {
    /// The time since the clock started.
    fn elapsed(&self) -> Duration;

    /// Wait for this long before the next look.
    fn pause(&mut self, pause: Duration);
}

/// The clock of the machine: time since [`WallClock::start`], and a sleep.
pub(super) struct WallClock(Instant);

impl WallClock {
    /// Start counting now.
    pub(super) fn start() -> Self {
        Self(Instant::now())
    }
}

impl Clock for WallClock {
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }

    fn pause(&mut self, pause: Duration) {
        thread::sleep(pause);
    }
}
