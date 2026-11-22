# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xe91447cf636b013a;
using Leaf = import "capture-leaf.capnp".Leaf;
struct Middle { nested @0 :Leaf; }
