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

struct Scope @0xcd8cb54000000003 (Value, Spare) {
  struct Nested @0xcd8cb54000000004 {
    value @0 :Value;
  }
}

interface Reader @0xcd8cb54000000005 (Outer) {
  omitted @0 () -> Single;
  explicit @1 Single(AnyPointer) -> Pair(Outer, AnyPointer);
  trailing @2 Pair(Outer, AnyPointer) -> ();
  nested @3 () -> Scope.Nested;
  shared @4 Scope.Nested -> Single(Pair(AnyPointer, AnyPointer));
}
