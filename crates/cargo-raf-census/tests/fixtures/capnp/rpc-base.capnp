# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xd128dbd816a8f941;

struct Payload @0xf7d9fae829b17495 {
  value @0 :UInt32;
}

interface ByteReader @0xcad554910d7a67ab {
  read @0 (size :UInt32 = 0) -> (payload :Payload, ready :Bool);
  status @1 () -> (ready :Bool);
  watch @2 () -> ();
  finish @3 () -> ();
}
