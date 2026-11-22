# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xf69153ee49b07300;

annotation route @0xf69153ee49b07301 (method) :Text;
annotation label @0xf69153ee49b07302 (param) :Text;
annotation fieldLabel @0xf69153ee49b07303 (field) :Text;

struct Tags @0xf69153ee49b07304 (T) {
  annotation marker @0xf69153ee49b07305 (method, param) :Void;
}

struct Named @0xf69153ee49b07306 {
  text @0 :Text $fieldLabel("data field");
  note @1 :Text;
}

interface Service @0xf69153ee49b07307 (T) {
  send @0 [U] (message :U $label("input") $Tags(T).marker)
      -> (reply :T $label("output") $Tags(T).marker)
      $route("first") $route("second") $Tags(T).marker;
  named @1 Named -> Named;
}
