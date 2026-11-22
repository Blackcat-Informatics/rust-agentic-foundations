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
    back @0 :Nested;
  }
}

struct Packet @0xf1804b9b18ae6505 {
  title @0 :Text = "packet";
  counter @1 :UInt32 = 42;
  state @2 :State = active;
  payload @3 :Payload = (numbers = [1, 2]);
  union {
    empty @4 :UInt16;
    selected @5 :Bool;
  }
  options :group {
    retries @6 :UInt16 = 2;
  }
  boxed @7 :Box(Data);
}

enum State @0xe70e425d8d63c329 {
  ready @0;
  active @1;
  retired @2;
}

struct Unrelated {
  ignored @0 :Text;
}
