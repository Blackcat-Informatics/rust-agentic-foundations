#!/usr/bin/env bash
set -u
name=$1; msg=$2
log=/tmp/raf-recheck-scripts/probe-$name.log
git commit -m "$msg" >"$log" 2>&1; rc=$?
echo "[$name] commit rc=$rc"
grep -E 'Running tests|test result|pre-commit:|panicked|profile violations|issue-refs:|  [A-Za-z./_-]+(:[0-9]+)?: |symlink' "$log" | head -20
echo "HEAD now: $(git log -1 --format='%h %G? %s')"
