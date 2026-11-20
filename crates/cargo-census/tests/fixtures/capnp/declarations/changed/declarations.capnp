# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec1000000000000;

using Dependency = import "/dependency.capnp";

struct Holder @0xeec1000000000003 {
  struct Moved @0xeec1000000000001 {
    payload :group {
      value @0 :UInt16;
    }
    struct Child @0xeec1000000000002 {
      name @0 :Text;
    }
    const limit :UInt16 = 16;
  }
}

enum Shape @0xeec1000000000004 {
  zero @0;
}

enum State {
  ready @0;
}

annotation marker(field) :Void;
using Alias = Holder.Moved.Child;

interface Fetcher @0xeec1000000000007 {
  read @0 (id :UInt64) -> (value :Holder.Moved.Child);
  imported @1 () -> (dependency :Dependency.Wanted);
}
