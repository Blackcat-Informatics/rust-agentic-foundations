# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xa7c2bc02a14af2d0;

annotation required @0xe91447cf636b013a (field) :Void;

struct Record @0xf1804b9b18ae6505 {
  union {
    none @0 :Void;
    overlay :group {
      details :group {
        nested @1 :Text $required;
      }
    }
  }
}
