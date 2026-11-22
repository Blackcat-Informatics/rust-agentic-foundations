# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xa7c2bc02a14af2d0;

annotation pairedPresence @0xbfff7293108cab62 (field, group, union) :UInt16;
annotation required @0xe91447cf636b013a (field) :Void;

struct Record @0xf1804b9b18ae6505 {
  profile @0 :Text = "checkpoint" $pairedPresence(1);
  profileSince @1 :Data;
  vault @2 :Data $required;
  feedHeld @3 :Data;
}
