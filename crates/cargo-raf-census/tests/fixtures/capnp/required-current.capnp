# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xa7c2bc02a14af2d0;

annotation required @0xe91447cf636b013a (field) :Void;

struct Record @0xf1804b9b18ae6505 {
  name @0 :Text = "anonymous" $required;
  keep @1 :Data $required;
  release @2 :Text;
  options :group {
    destination @3 :Text $required;
  }
  token @4 :Data $required;
  extra @5 :Extra;

  struct Extra {
    nested @0 :Text $required;
  }
}
