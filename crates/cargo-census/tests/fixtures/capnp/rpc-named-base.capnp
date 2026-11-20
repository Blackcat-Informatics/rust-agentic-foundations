# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec2000000000000;

annotation required @0xeec2000000000010 (field) :Void;

struct Payload @0xeec2000000000001 {
  struct Input @0xeec2000000000002 {
    count @0 :UInt64 = 0;
  }
}

struct Response @0xeec2000000000003 {
  value @0 :UInt64;
}

struct Alternate @0xeec2000000000004 {
  count @0 :UInt64;
}

interface Reader @0xeec2000000000005 {
  first @0 Payload.Input -> Response;
  second @1 Payload.Input -> (value :UInt64);
  third @2 (source :Text) -> Response;
}

struct Stored @0xeec2000000000006 {
  reader @0 :Reader;
}
