# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
@0xfa42ee5500000080;

struct Tags @0xfa42ee5500000081 (T) {
  annotation marker @0xfa42ee5500000082 (struct, enum, interface) :Void;
}

struct Record @0xfa42ee5500000083 (U) $Tags(U).marker {}

enum Shade @0xfa42ee5500000084 $Tags(Text).marker {
  red @0;
}

interface Service @0xfa42ee5500000085
    $Tags(List(Text)).marker $labels(["base"]) $labels(["second"]) {}

annotation labels @0xfa42ee5500000086 (interface) :List(Text);

struct Envelope @0xfa42ee5500000087 {
  record @0 :Record(Text);
  shade @1 :Shade;
}
