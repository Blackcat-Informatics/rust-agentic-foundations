# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xa8aa72ae232a19ab;
# import "comment.capnp"
const importCount :UInt32 = 1;
const exampleText :Text = "quote \" and # import \"ordinary-string.capnp\"";
const exampleLine :Text = `embed "literal-line.bin"
;
struct Entry @0xfe3dcd67079854c3 {
  value @0 :UInt32 = import # The compiler embeds the resolved scalar.
    "capture-defaults.capnp".limit;
}
