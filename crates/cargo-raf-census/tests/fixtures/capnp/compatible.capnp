# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xdbeeb3781948a347;

struct Box @0xd05283a9dc67c5f1 (T) {
  value @0 :T;
}

struct Payload @0xc5fa75a0c95aa56a {
  obsolete @0 :UInt32;
  numbers @1 :List(UInt16);
}

struct Outer @0xe4ddd1d087b36f8d (T) {
  struct Nested @0xc8a351b6b42dc6a5 {
    back @0 :Outer(T).Nested;
  }
}

struct Packet @0xf1804b9b18ae6505 {
  boxed @7 :Box(Data);
  state @2 :State = active;
  payload @3 :Payload = (numbers = [1, 2]);
  union {
    selected @5 :Bool;
    empty @4 :UInt16;
  }
  options :group {
    retries @6 :UInt16 = 2;
    note @8 :Text = "available";
  }
  counter @1 :UInt32 = 42;
  title @0 :Text = "packet";
  flags @9 :List(Bool) = [true, false];
  tail @10! :union {
    parked @11 :Void;
    number @12 :UInt32;
  }
}

enum State @0xe70e425d8d63c329 {
  active @1;
  ready @0;
  suspended @3;
  retired @2;
}

struct Unrelated {
  different @0 :UInt16;
}
