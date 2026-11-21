# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
@0xefc1329300000050;

annotation label @0xefc1329300000051 (field) :Text;
annotation flag @0xefc1329300000052 (field) :Void;
annotation parameter @0xefc1329300000056 (param) :Text;

struct Tags @0xefc1329300000053 (T) {
  annotation value @0xefc1329300000054 (field) :T;
}

struct Record @0xefc1329300000055 {
  name @0 :Text $label("first") $flag $label("second");
  value @1 :Text $Tags(Text).value("alpha");
  names @2 :List(Text) $Tags(List(Text)).value(["one", "two"]);
  branded @3 :Text $Tags(Text).value("a");
  repeated @4 :Text $label("same") $label("same");
}

interface Service @0xefc1329300000057 {
  read @0 (source :Text $parameter("old")) -> Record;
}
