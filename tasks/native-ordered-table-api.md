<!-- SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca> -->
<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Public native ordered storage boundary

Verified current PurRDF main 80d2731bdf19e79b22495359b4a01362e2edb07b and RAF's
locked 5db00aaae9f3abb521e98b8f79dbc1bebe5e3e43 source, not bot instructions.
Primary files: crates/lex/src/allocation.rs and allocation/ordered.rs.
The architecture graph query returned unrelated historical/artifact nodes and
does not establish these current APIs; current source is authoritative.

The existing RAF pin publicly exports allocation::OrderedTable<K,V>. It owns a
balanced AVL index in a native fallibly allocated slot array. Empty construction
and lookups allocate nothing. This supplies the indexed cache directly; no new
std container, generic tree, dependency upgrade or invented public API is needed.

Pinned capabilities and constraints:

- Default, len, is_empty; get/get_by/get_key_value_by; get_mut/get_mut_by.
  Comparator APIs use query-versus-stored-key ordering; they support borrowed
  spellings without allocating a key. Key/value lookups return borrowed payloads.
- insert(key,value,&mut Memory) requires K:Ord and returns Result<Option<V>,
  StorageError>; an occupied key replaces its value. reserve_insert admits the
  actual next backing layout independently. Refusal leaves existing associations
  unchanged. There is no pinned std-style vacant/occupied entry API or stable
  public slot locator. A miss can insert and then do another logarithmic lookup.
- ordered() is allocation-free canonical key order, double-ended, exact-size and
  fused. iter() and values_mut() use arena order, which is not key order. Preserve
  explicit declaration/output order separately and use ordered() when replacing
  BTreeMap iteration whose order is contractual.
- remove/remove_entry_by, pop_first/pop_last and borrowed-key range are available;
  vacant physical slots are reused. No ordinary std Entry API should be assumed.
- clear() keeps admitted capacity. storage_bytes() reports exact arena capacity;
  release(&mut Memory) destroys the arena before refund. try_clone(&mut Memory)
  is fallible, but its K/V clones must be shallow and allocation-free as stated
  by its contract; do not silently use allocating Vec/String clones as though
  the native arena had admitted their child allocations.
- Memory::new starts empty; Memory::resume borrows the same original Admission
  and already owned live-byte total. The same table's growth/release must retain
  that original account. Memory::reserve requests an exact replacement while the
  old array is still alive, allocates fallibly and releases old layout only after
  moving and destroying it. Resident supplies no application limit: do not use
  it to claim an existing external byte/slot grant was enforced.

Current main additionally exports OrderedEntry/OrderedSlot, prepare_insert with
a fallible comparator, admitted publish, and at_slot/next_slot/remove_slot. These
are not present in RAF's pin, so cache repair uses the already supplied pinned
API rather than silently assuming main's API or upgrading the entire dependency.

RAF cache policy remains byte-key selection, real-item/root selection, logical
entry/reference quotas and matching-work accounting. Native OrderedTable owns
the AVL implementation and physical allocation. Lookup/update work must be
charged logarithmically, not represented as constant merely because it is indexed.
