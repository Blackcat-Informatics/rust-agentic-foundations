# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xcd8cb54000000000;

struct Single @0xcd8cb54000000001 (Value) {
  value @0 :Value;
}

struct Pair @0xcd8cb54000000002 (First, Second) {
  first @0 :First;
  second @1 :Second;
}

struct Scope @0xcd8cb54000000003 (Value) {
  struct Nested @0xcd8cb54000000004 {
    value @0 :Value;
  }
}

interface Reader @0xcd8cb54000000005 (Outer) {
  omitted @0 () -> Single(AnyPointer);
  explicit @1 Single -> Pair(AnyPointer, Outer);
  trailing @2 Pair(Outer, AnyPointer) -> ();
  nested @3 () -> Scope(AnyPointer).Nested;
  shared @4 Scope(AnyPointer).Nested -> Single(Pair(AnyPointer, Text));
}
