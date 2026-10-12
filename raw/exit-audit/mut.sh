#!/usr/bin/env bash
# usage: mut.sh <name> <python-mutation-file>   run inside scratch clone; stages and runs docs_contract, then resets
set -u
name=$1; mut=$2
python3 -I "$mut" || { echo "[$name] mutation script failed"; exit 9; }
git add -A
log=/tmp/raf-audit-scripts/mut-$name.log
cargo test --locked --test docs_contract >"$log" 2>&1; rc=$?
echo "[$name] docs_contract rc=$rc"
sed -n '/profile violations:/,/^$/p' "$log" | head -12
grep -E '^test result' "$log"
git reset -q --hard HEAD; git clean -qfd
