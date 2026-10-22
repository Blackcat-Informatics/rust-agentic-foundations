---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Architecture
id: arch:row-keys
title: Row keys
description: "How a row key is derived."
timestamp: 2026-01-02T03:04:05Z
---
# Row keys

The key is the digest of the row text.

~~~text
row: alpha
key: 0f3a
~~~

A key never changes when a row moves.