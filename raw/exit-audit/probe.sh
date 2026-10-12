#!/usr/bin/env bash
# usage: probe.sh <name> <commit-msg>  (changes already staged)
set -u
name=$1; msg=$2
log=/tmp/raf-audit-scripts/probe-$name.log
git commit -m "$msg" >"$log" 2>&1; rc=$?
echo "[$name] commit rc=$rc"
grep -E 'Running tests|test result|pre-commit:|panicked|violation|issue-refs:|licenses:|citation:|FAILED|failures:|^    [a-z_]+$|profile violations|^  [a-zA-Z./_-]+:|hits' "$log" | head -40
echo "HEAD now: $(git log -1 --format='%h %G? %s')"
