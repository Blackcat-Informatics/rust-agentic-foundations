#!/usr/bin/env bash
set -u
name=$1; msg=$2
log=/tmp/raf-p3-scripts/probe-$name.log
git commit -m "$msg" >"$log" 2>&1; rc=$?
echo "[$name] commit rc=$rc"
grep -E 'test result|pre-commit:|profile violations|issue-refs:|  [A-Za-z./_-]+(:[0-9]+)?: |symlinks are not' "$log" | head -12
echo "HEAD now: $(git log -1 --format='%h %G? %s')"
