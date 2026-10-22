#!/usr/bin/env bash
# usage: cmp.sh <tree> <outdir>
set -u
tree=$1; out=$2; mkdir -p "$out"
OB=/opt/.cargo/target/t4old-808dcc17/debug/cargo-workplan
HB=/opt/.cargo/target/t4review-2f781a04/debug/cargo-workplan
i=0
run() {
  i=$((i+1))
  for b in O H; do
    bin=$OB; [[ $b == H ]] && bin=$HB
    (cd "$tree" && "$bin" workplan --root "$tree" "$@" > "$out/$i.$b.out" 2> "$out/$i.$b.err"; echo $? > "$out/$i.$b.code")
  done
  if cmp -s "$out/$i.O.out" "$out/$i.H.out" && cmp -s "$out/$i.O.err" "$out/$i.H.err" && cmp -s "$out/$i.O.code" "$out/$i.H.code"; then
    echo "SAME $i $* (exit $(cat $out/$i.O.code))"
  else
    echo "DIFF $i $* (exit $(cat $out/$i.O.code)/$(cat $out/$i.H.code))"
  fi
}
for fmt in text json; do
  f=(); [[ $fmt == json ]] && f=(--format json)
  run "${f[@]}" check
  run "${f[@]}" -q check
  run "${f[@]}" status
  run "${f[@]}" coverage code
  run "${f[@]}" coverage tests
  run "${f[@]}" coverage claims
  run "${f[@]}" evidence claims gate-kills-a-test-past-its-timeout
  run "${f[@]}" change impact workplan-strict
  run "${f[@]}" change reconcile workplan-strict
  run "${f[@]}" change reconcile nosuch
  run "${f[@]}" docs preview workplan-strict

  run "${f[@]}" bogus-command
done
