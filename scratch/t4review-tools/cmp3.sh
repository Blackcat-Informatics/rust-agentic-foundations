#!/usr/bin/env bash
# usage: cmp3.sh <tree> <outdir> <text|json> <from> <to>
set -u
tree=$1; out=$2; fmt=$3; mkdir -p "$out"
declare -A BIN=([O]=/opt/.cargo/target/t4old-808dcc17/debug/cargo-workplan [H]=/opt/.cargo/target/t4review-2f781a04/debug/cargo-workplan [N]=/opt/.cargo/target/t4r2-d3267c45/debug/cargo-workplan)
f=(); [[ $fmt == json ]] && f=(--format json)
from=$4; to=$5
i=0
run() {
  i=$((i+1))
  if (( i < from || i > to )); then return; fi
  for b in O H N; do
    (cd "$tree" && "${BIN[$b]}" workplan --root "$tree" "${f[@]}" "$@" > "$out/$fmt.$i.$b.out" 2> "$out/$fmt.$i.$b.err"; echo $? > "$out/$fmt.$i.$b.code")
  done
  same() { cmp -s "$out/$fmt.$i.$1.out" "$out/$fmt.$i.$2.out" && cmp -s "$out/$fmt.$i.$1.err" "$out/$fmt.$i.$2.err" && cmp -s "$out/$fmt.$i.$1.code" "$out/$fmt.$i.$2.code"; }
  r1=DIFF; same O N && r1=SAME; r2=DIFF; same H N && r2=SAME
  echo "7d35393:$r1 da83cdc:$r2 $fmt $i $* (exit $(cat $out/$fmt.$i.O.code)/$(cat $out/$fmt.$i.H.code)/$(cat $out/$fmt.$i.N.code))"
}
run check
run -q check
run status
run coverage code
run coverage tests
run coverage claims
run evidence claims gate-kills-a-test-past-its-timeout
run change impact workplan-strict
run change reconcile workplan-strict
run change reconcile nosuch
run docs preview workplan-strict
run bogus-command
