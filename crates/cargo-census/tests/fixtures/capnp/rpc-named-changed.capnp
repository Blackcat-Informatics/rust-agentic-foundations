# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xeec2000000000000;

annotation required @0xeec2000000000010 (field) :Void;

struct Payload @0xeec2000000000001 {
  struct Input @0xeec2000000000002 {
    count @0 :UInt64 = 9;
    token @1 :Text $required;
  }
}

struct Response @0xeec2000000000003 {
  value @0 :UInt64;
}

# This ID is the base descriptor's generated result for Reader.second.
struct NamedResult @0xb8f65465f7b65950 {
  value @0 :UInt64;
}

struct Alternate @0xeec2000000000004 {
  count @0 :UInt64;
}

struct Generic @0xeec2000000000007 (T) {
  renamed @0 :T;
}

struct Envelope @0xeec2000000000008 (T) {
  struct Body @0xeec2000000000009 (U) {
    first @0 :T;
    second @1 :U;
  }

  struct Stamp @0xeec200000000000a {
    value @0 :T;
  }
}

interface Reader @0xeec2000000000005 {
  first @0 Alternate -> Response;
  second @1 Payload.Input -> NamedResult;
  third @2 (source :Text) -> Response;
  fourth @3 Generic(Generic(Data)) -> Generic(Data);
  scoped @4 Envelope(Data).Body(Data) -> Envelope(Data).Body(Text);
  enclosingOnly @5 Envelope(Text).Stamp -> ();
}

interface GenericReader @0xeec200000000000b (First, Second) {
  struct Record @0xeec200000000000c {
    label @0 :First;
  }

  fetch @0 (value :Second) -> Generic(Second);
  inherited @1 Record -> Record;

  ready @2 () -> (ok :Bool);
}

interface ArityReader @0xeec200000000000d (First, Second) {
  ready @0 () -> ();
}

struct Stored @0xeec2000000000006 {
  reader @0 :Reader;
  genericReader @1 :GenericReader(Text, Data);
}
