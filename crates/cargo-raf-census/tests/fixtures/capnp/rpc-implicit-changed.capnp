# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec4400000000000;

struct Pair @0xeec4400000000001 (Left, Right) {
  left @0 :Left;
  right @1 :Right;
}

interface Reader @0xeec4400000000002 (Outer) {
  read @0 [First, Second] (source :Outer, owner :Outer, pair :Pair(Second, Outer))
       -> Pair(Second, Outer);
  write @1 [Value] Pair(Outer, Value) -> (value :Value);
  marker @2 [Unused, Added] () -> ();
}

struct Stored @0xeec4400000000003 {
  reader @0 :Reader(Text);
}
