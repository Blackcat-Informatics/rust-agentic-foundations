// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! What the containers of a load allocate, as upper bounds derived from their
//! layout.
//!
//! Every charge to the item-bytes budget is built from these bounds, so it is
//! at least what the allocations it covers take, whatever the shape of the
//! input; none is an estimate measured on typical input. The bounds are those
//! of the standard library's containers:
//!
//! * A `Vec` grows by doubling from a first room of four elements (for
//!   elements of up to 1 KiB), so it holds at most twice its length once it
//!   has four, and four elements' room before. An element costs
//!   [`vec_slot`]; a list that starts with one element costs
//!   [`vec_first`] once.
//! * A `BTreeMap` built by insertion keeps every node but the root at least
//!   five entries full: a leaf holds eleven entries and an internal node also
//!   twelve edges. A map of up to eleven entries is one leaf, which the map
//!   costs once ([`btree_map`]). A larger one has at most n / 5 leaves and at
//!   most 1 + n / 25 internal nodes, which is at most n / 8 when n is twelve or
//!   more, so an entry costs a fifth of a leaf and an eighth of an internal
//!   node ([`btree_entry`]).
//! * A `HashMap` or `HashSet` keeps a power of two of buckets of one entry and
//!   one control byte each, at most 8/7 of its entries rounded up to a power
//!   of two after a doubling: an entry costs three buckets ([`hash_entry`])
//!   and a table its first eight buckets and a control group once
//!   ([`hash_map`]).
//! * A shared string (`Arc<str>`) is its text and a header of two counters.

use std::mem::size_of;

/// The room an element of a `Vec` of `T` takes at most once the list holds
/// four or more.
pub(crate) const fn vec_slot<T>() -> usize {
    2 * size_of::<T>()
}

/// The first room of a `Vec` of `T`: four elements.
pub(crate) const fn vec_first<T>() -> usize {
    4 * size_of::<T>()
}

/// The size of a leaf node of a `BTreeMap<K, V>`: a parent pointer, two
/// counters, eleven keys and eleven values, with padding.
const fn btree_leaf<K, V>() -> usize {
    16 + 11 * (size_of::<K>() + size_of::<V>()) + 8
}

/// The size of an internal node: a leaf and twelve edges.
const fn btree_internal<K, V>() -> usize {
    btree_leaf::<K, V>() + 12 * size_of::<usize>()
}

/// What one entry of a `BTreeMap<K, V>` adds at most, beyond what the map
/// costs once.
pub(crate) const fn btree_entry<K, V>() -> usize {
    btree_leaf::<K, V>().div_ceil(5) + btree_internal::<K, V>().div_ceil(8)
}

/// What a `BTreeMap<K, V>` costs once, whatever it holds: its first leaf.
pub(crate) const fn btree_map<K, V>() -> usize {
    btree_leaf::<K, V>()
}

/// What one entry of a hash table of `T` adds at most.
pub(crate) const fn hash_entry<T>() -> usize {
    3 * (size_of::<T>() + 1)
}

/// What a hash table of `T` costs once: eight buckets and a control group.
pub(crate) const fn hash_map<T>() -> usize {
    8 * (size_of::<T>() + 1) + 16 + 8
}

/// The allocation of a shared string of `len` bytes: its text and two
/// counters, rounded up to the counters' alignment.
pub(crate) const fn shared_text(len: usize) -> usize {
    (len + 2 * size_of::<usize>()).next_multiple_of(align_of::<usize>())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn a_shared_string_is_charged_its_padded_allocation() {
        assert_eq!(shared_text(0), 2 * size_of::<usize>());
        assert_eq!(shared_text(1), 2 * size_of::<usize>() + align_of::<usize>());
        assert_eq!(shared_text(8), 3 * size_of::<usize>());
        assert_eq!(shared_text(9) % align_of::<usize>(), 0);
    }

    #[test]
    fn a_map_of_one_entry_is_covered_by_what_a_map_costs_once() {
        // The leaf of a map of `u64` to `u64` holds eleven pairs of 16 bytes.
        assert_eq!(btree_leaf::<u64, u64>(), 16 + 11 * 16 + 8);
        assert!(btree_map::<u64, u64>() >= btree_leaf::<u64, u64>());
        // A map of up to eleven entries is one leaf; a larger one keeps at
        // most n / 5 leaves and 1 + n / 25 internal nodes. The charges cover
        // both for every n.
        for n in [1_usize, 5, 11, 12, 13, 24, 25, 100, 10_000] {
            let (leaves, internals) = if n <= 11 { (1, 0) } else { (n / 5, 1 + n / 25) };
            let held = leaves * btree_leaf::<u64, u64>() + internals * btree_internal::<u64, u64>();
            let charged = btree_map::<u64, u64>() + n * btree_entry::<u64, u64>();
            assert!(charged >= held, "{n}: {charged} < {held}");
        }
        let map: BTreeMap<u64, u64> = (0..12).map(|k| (k, k)).collect();
        assert_eq!(map.len(), 12);
    }

    #[test]
    fn a_list_is_charged_at_least_its_room() {
        for n in 1_usize..200 {
            let mut list: Vec<u64> = Vec::new();
            for k in 0..n {
                list.push(k as u64);
            }
            let held = list.capacity() * size_of::<u64>();
            let charged = vec_first::<u64>() + n * vec_slot::<u64>();
            assert!(charged >= held, "{n}: {charged} < {held}");
        }
    }

    #[test]
    fn a_hash_table_is_charged_at_least_its_buckets() {
        use std::collections::HashSet;
        for n in 1_usize..5000 {
            let set: HashSet<u64> = (0..n as u64).collect();
            // The buckets behind a capacity: a power of two of which seven
            // eighths are usable, and at least four.
            let capacity = set.capacity();
            let buckets = match capacity {
                0..4 => 4,
                4..8 => 8,
                _ => (capacity * 8 / 7).next_power_of_two(),
            };
            let held = buckets * (size_of::<u64>() + 1) + 16;
            let charged = hash_map::<u64>() + n * hash_entry::<u64>();
            assert!(charged >= held, "{n}: {charged} < {held}");
        }
    }
}
