// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The memory of a parsed body grows with the input, not with the product of
//! the line count and the length of a heading.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use raf_okf::Body;

/// The allocator refuses above this, so a regression ends in an allocation
/// failure instead of exhausting the host.
const CEILING: usize = 1 << 30;

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every request is forwarded unchanged to `System`; the counters only
// observe sizes.
#[allow(
    unsafe_code,
    reason = "a counting global allocator must implement the unsafe GlobalAlloc trait"
)]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let now = CURRENT.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
        if now > CEILING {
            CURRENT.fetch_sub(layout.size(), Ordering::SeqCst);
            return std::ptr::null_mut();
        }
        PEAK.fetch_max(now, Ordering::SeqCst);
        // SAFETY: the caller upholds the `GlobalAlloc::alloc` contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        CURRENT.fetch_sub(layout.size(), Ordering::SeqCst);
        // SAFETY: `ptr` came from `alloc` with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old = layout.size();
        let now = if new_size >= old {
            CURRENT.fetch_add(new_size - old, Ordering::SeqCst) + (new_size - old)
        } else {
            CURRENT.fetch_sub(old - new_size, Ordering::SeqCst) - (old - new_size)
        };
        if now > CEILING {
            CURRENT.fetch_sub(new_size.saturating_sub(old), Ordering::SeqCst);
            return std::ptr::null_mut();
        }
        PEAK.fetch_max(now, Ordering::SeqCst);
        // SAFETY: the caller upholds the `GlobalAlloc::realloc` contract.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const DOCUMENT_BYTES: usize = 1024 * 1024;
const LONG_HEADING: usize = 60 * 1024;
const ALLOWANCE: usize = 64 * 1024;

/// A body of `DOCUMENT_BYTES` bytes with a heading of `heading_bytes` bytes,
/// whose remaining lines are the repeated `unit`. The line count depends on
/// the unit and the body size only, so a long and a short heading give the
/// same number of lines.
fn document(heading_bytes: usize, unit: &str) -> (String, usize) {
    let mut text = String::with_capacity(DOCUMENT_BYTES);
    text.push_str("## ");
    text.push_str(&"h".repeat(heading_bytes));
    text.push('\n');
    let units = (DOCUMENT_BYTES - LONG_HEADING - 8) / unit.len();
    for _ in 0..units {
        text.push_str(unit);
    }
    (text, units)
}

/// The most the parse held above what was live before it, with the result
/// still alive at the end.
fn peak_of_parse(text: &str) -> usize {
    let before = CURRENT.load(Ordering::SeqCst);
    PEAK.store(before, Ordering::SeqCst);
    let body = Body::parse(text).expect("the document is a body");
    let peak = PEAK.load(Ordering::SeqCst);
    drop(body);
    peak - before
}

fn assert_linear(unit: &str) {
    let (long, long_units) = document(LONG_HEADING, unit);
    let (short, short_units) = document(1, unit);
    assert_eq!(
        long_units, short_units,
        "the two documents have the same shape"
    );
    let sections = 1;
    let peak_short = peak_of_parse(&short);
    let peak_long = peak_of_parse(&long);
    let allowed = peak_short + 4 * LONG_HEADING * sections + ALLOWANCE;
    assert!(
        peak_long <= allowed,
        "a {LONG_HEADING} byte heading over {long_units} units peaked at {peak_long} bytes, \
         against {peak_short} for a 1 byte heading (allowed {allowed})"
    );
    eprintln!("unit {unit:?}: peak(long) = {peak_long}, peak(short) = {peak_short}");
}

/// The peak is process-wide, so the measurements share one test and run in
/// turn.
#[test]
fn a_long_heading_over_many_lines_stays_linear() {
    assert_linear("some prose text\n");
    assert_linear("> [!PROPOSED change:x]\n> t\n");
}
