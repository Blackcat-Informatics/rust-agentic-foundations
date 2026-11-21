# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec4200000000000;

struct Triple @0xeec4200000000001 (A, B, C) {
  first @0 :A;
  second @1 :B;
  third @2 :C;
}

struct Outer @0xeec4200000000002 (A) {
  struct Middle @0xeec4200000000003 (B) {
    interface Reader @0xeec4200000000004 (C) {
      struct Record @0xeec4200000000005 {
        item @0 :C;
      }

      read @0 (first :A, second :B, third :C) -> Triple(A, B, C);
      ready @1 () -> ();
      shared @2 Record -> Record;
    }
  }
}

struct Arity @0xeec4200000000006 (A) {
  interface EnclosingOnly @0xeec4200000000007 {
    read @0 (item :A) -> ();
  }
}
