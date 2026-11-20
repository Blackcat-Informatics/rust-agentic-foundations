# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0


@0xeec4000000000000;

interface Top @0xeec4000000000001 {
  read @0 (count :UInt16 = 2) -> (value :UInt16);
}

interface Left @0xeec4000000000002 extends(Top) {}
interface Right @0xeec4000000000003 extends(Top) {}
interface Extra @0xeec4000000000004 extends(Left) {}
interface Leaf @0xeec4000000000005 extends(Right, Extra) {}

struct Record @0xeec4000000000006 {
  service @0 :Leaf;
}
