# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xd128dbd816a8f941;

struct Payload @0xf7d9fae829b17495 {
  label @0 :UInt32;
}

interface ByteReader @0xcad554910d7a67ab {
  fetch @0 (size :UInt32, offset :UInt32) -> (payload :Payload, ready :UInt16);
  watch @1 () -> ();
  status @2 () -> (ready :Bool);
}
