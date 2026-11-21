# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

@0xa7c2bc02a14af2d0;

annotation pairedPresence @0xbfff7293108cab62 (field, group, union) :UInt16;
annotation required @0xe91447cf636b013a (field) :Void;

struct ScalarSource @0xb000000000000001 {
  count @0 :UInt16 $pairedPresence(1);
  pointer @1 :Data;
}

struct ScalarPeer @0xb000000000000002 {
  pointer @0 :Data $pairedPresence(1);
  count @1 :UInt16;
}

struct MissingPeer @0xb000000000000003 {
  pointer @0 :Data $pairedPresence(1);
}

struct SelfPair @0xb000000000000004 {
  pointer @0 :Data $pairedPresence(0);
}

struct ReversedPair @0xb000000000000005 {
  first @0 :Data;
  second @1 :Data $pairedPresence(0);
}

struct UnionSource @0xb000000000000006 {
  union {
    first @0 :Data $pairedPresence(2);
    second @1 :Data;
  }
  third @2 :Data;
}

struct UnionPeer @0xb000000000000007 {
  first @0 :Data $pairedPresence(1);
  union {
    second @1 :Data;
    third @2 :Data;
  }
}

struct GroupSource @0xb000000000000008 {
  nested :group {
    first @0 :Data $pairedPresence(1);
  }
  second @1 :Data;
}

struct GroupPeer @0xb000000000000009 {
  first @0 :Data $pairedPresence(1);
  nested :group {
    second @1 :Data;
  }
}

struct GroupApplication @0xb00000000000000a {
  nested :group $pairedPresence(1) {
    first @0 :Data;
  }
  second @1 :Data;
}

struct DuplicateApplication @0xb00000000000000b {
  first @0 :Data $pairedPresence(1) $pairedPresence(2);
  second @1 :Data;
  third @2 :Data;
}
