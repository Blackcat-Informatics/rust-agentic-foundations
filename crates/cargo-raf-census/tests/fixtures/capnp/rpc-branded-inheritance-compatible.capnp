# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec4900000000000;

interface Plain @0xeec4900000000001 {}

struct Envelope @0xeec4900000000002 (Outer) {
  interface Source @0xeec4900000000003 (Item) {
    fetch @0 (key :Outer) -> (value :Item);
  }
}

interface Concrete @0xeec4900000000004 extends(Envelope(Text).Source(Data)) {}

struct Shelf @0xeec4900000000005 (Key) {
  interface Derived @0xeec4900000000006 (Item) extends(Envelope(Key).Source(Item), Plain) {}
}

interface Generic @0xeec4900000000007 (Unused) extends(Plain) {}

interface Base @0xeec4900000000008 (Item) {
  query @0 (value :Item, seed :UInt16 = 0) -> ();
}

interface Repeated @0xeec4900000000009 extends(Base(Data), Base(List(Text)), Base(Text)) {}
interface Left @0xeec490000000000a extends(Base(Text)) {}
interface Right @0xeec490000000000b extends(Base(Data)) {}
interface Diamond @0xeec490000000000c extends(Left, Right) {}
