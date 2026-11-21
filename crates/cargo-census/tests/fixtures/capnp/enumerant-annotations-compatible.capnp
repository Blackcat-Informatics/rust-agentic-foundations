# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
@0xfa42ee5500000070;

annotation label @0xfa42ee5500000071 (enumerant) :Text;

struct Tags @0xfa42ee5500000072 (T) {
  annotation value @0xfa42ee5500000073 (enumerant) :T;
  annotation marker @0xfa42ee5500000076 (enumerant) :Text;
}

struct Scope @0xfa42ee5500000074 (T) {
  annotation family @0xfa42ee5500000077 (enumerant) :Text;
  enum Shade @0xfa42ee5500000075 {
    red @0 $Tags(Text).value("warm") $label("first") $label("second");
    blue @1 $Tags(List(Text)).value(["cool"]);
    green @2 $Tags(Text).marker("steady") $family("local");
    violet @3 $label("new");
  }
}
