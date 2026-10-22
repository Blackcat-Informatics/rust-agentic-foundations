// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The source of the `window-stats` crate: the library, its integration tests and
//! its example.

pub(crate) const W_LIB: &str = r"//! A window over the most recent values.

use std::collections::VecDeque;

/// The most recent values, up to a capacity.
#[derive(Debug, Clone)]
pub struct Window {
    capacity: usize,
    values: VecDeque<f64>,
}

impl Window {
    /// An empty window that keeps at most `capacity` values.
    ///
    /// ```
    /// let mut window = window_stats::Window::new(3);
    /// window.record(1.0);
    /// window.record(3.0);
    /// assert_eq!(window.mean(), Some(2.0));
    /// ```
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            values: VecDeque::new(),
        }
    }

    /// Records a value and drops the oldest one when the window is full.
    pub fn record(&mut self, value: f64) {
        if self.capacity == 0 {
            return;
        }
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    /// The mean of the values in the window, or `None` when it holds none.
    #[must_use]
    pub fn mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let count = self.values.len() as f64;
        Some(self.values.iter().sum::<f64>() / count)
    }

    /// The number of values in the window.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the window holds no value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
";

pub(crate) const W_TESTS: &str = r"use window_stats::Window;

#[test]
fn the_window_keeps_only_the_most_recent_values() {
    let mut window = Window::new(3);
    for value in [1.0, 2.0, 3.0, 4.0, 5.0] {
        window.record(value);
    }
    assert_eq!(window.len(), 3);
    assert_eq!(window.mean(), Some(4.0));
}

#[test]
fn the_mean_of_an_empty_window_is_none() {
    let window = Window::new(3);
    assert!(window.is_empty());
    assert_eq!(window.mean(), None);
}

#[test]
fn a_window_of_capacity_zero_keeps_nothing() {
    let mut window = Window::new(0);
    window.record(1.0);
    assert_eq!(window.len(), 0);
}

#[test]
fn a_nan_mean_recovers_after_its_value_is_evicted() {
    let mut window = Window::new(2);
    window.record(f64::NAN);
    window.record(2.0);
    assert_eq!(window.len(), 2);
    assert!(window.mean().expect("two stored values").is_nan());
    window.record(4.0);
    assert_eq!(window.len(), 2);
    assert_eq!(window.mean(), Some(3.0));
}
";

pub(crate) const W_ROLLING: &str = r#"fn main() {
    let mut window = window_stats::Window::new(3);
    for value in [2.0, 4.0, 6.0, 8.0] {
        window.record(value);
        println!("{value} -> {:?}", window.mean());
    }
}
"#;
