# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
@0xfa42ee5500000090;
$Tags(Text).fileTag("schema");
$fileItems(["one"]);

struct Tags @0xfa42ee5500000091 (T) {
  annotation fileTag @0xfa42ee5500000092 (file) :T;
  annotation constTag @0xfa42ee5500000093 (const) :T;
  annotation declarationTag @0xfa42ee5500000094 (annotation) :Void;
}

const version @0xfa42ee5500000095 :UInt32 = 1 $Tags(Text).constTag("version");

struct Outer @0xfa42ee5500000096 (U) {
  annotation marker @0xfa42ee5500000097 (field) :Void $Tags(U).declarationTag;
}

annotation fileItems @0xfa42ee5500000098 (file) :List(Text);
