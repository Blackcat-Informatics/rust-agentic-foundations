# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xe7d7158fd3cfbd15;

struct Payload(T) {
  value @0 :T;
}

interface Parent {}

interface Inherited @0xf8e70893fb8da3a6 extends(Parent) {}

struct Outer(T) {
  interface Generic @0xa6615df804a45ef7 {}
}

interface GenericDerived @0xeec6000000000001 (T) extends(Parent) {}

interface GenericMethod @0xba6c33c22a5d3e73 {
  read @0 [T] () -> (value :T);
}

interface GenericEndpoint @0xadb4bfe48ad45105 {
  read @0 () -> Payload(Text);
}
