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
    red @0 $label("second") $label("first") $Tags(Text).value("warm");
    blue @1 $Tags(List(Text)).value(["cold"]);
    green @2 $Tags(List(Text)).marker("steady") $family("local");
  }
}
