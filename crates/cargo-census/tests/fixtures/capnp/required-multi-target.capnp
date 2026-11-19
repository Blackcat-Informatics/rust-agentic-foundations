# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xb337fa2d7cf4f8bc;

annotation required @0xe91447cf636b013a (field, param) :Void;

struct Record @0xf1804b9b18ae6505 {
  label @0 :Text $required;
}

interface Service {
  set @0 (label :Text $required) -> ();
}
