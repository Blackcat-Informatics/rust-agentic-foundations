# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec1000000000000;

using Dependency = import "/dependency.capnp";

struct Outer @0xeec1000000000001 {
  payload :group {
    value @0 :UInt16;
  }
  struct Child @0xeec1000000000002 {
    name @0 :Data;
  }
  const limit :UInt16 = 17;
}

struct Holder @0xeec1000000000003 {}
struct Shape @0xeec1000000000004 {}
struct Leaving @0xeec1000000000005 {}
struct Externalize @0xeec1000000000006 {}
struct Added @0xeec1000000000008 {}

enum State {
  ready @0;
}

annotation marker(field) :Void;
annotation obsoleteMarker(field) :Void;
const obsoleteLimit :UInt16 = 8;
using ChangedAlias = Outer.Child;

interface Reader @0xeec1000000000007 {
  read @0 (id :UInt64) -> (value :Outer.Child);
  imported @1 () -> (dependency :Dependency.Wanted);
}
