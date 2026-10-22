#!/usr/bin/env bash
# usage: cmp2.sh <tree> <outdir> <evidence-id> <change-slug>
tree=$1; out=$2; ev=$3; ch=$4; mkdir -p "$out"
MB=/opt/.cargo/target/main-933cc592/debug/cargo-workplan
HB=/opt/.cargo/target/adv-6d310e02/debug/cargo-workplan
i=0
run() {
  i=$((i+1))
  for b in M H; do
    bin=$MB; [[ $b == H ]] && bin=$HB
    (cd "$tree" && "$bin" workplan --root "$tree" "$@" > "$out/$i.$b.out" 2> "$out/$i.$b.err"; echo $? > "$out/$i.$b.code")
  done
  if cmp -s "$out/$i.M.out" "$out/$i.H.out" && cmp -s "$out/$i.M.err" "$out/$i.H.err" && cmp -s "$out/$i.M.code" "$out/$i.H.code"; then
    echo "SAME $i $* (exit $(cat $out/$i.M.code))"
  else
    echo "DIFF $i $* (exit $(cat $out/$i.M.code)/$(cat $out/$i.H.code))"
  fi
}
for fmt in text json; do
  if [[ $fmt == json ]]; then f="--format json"; else f=""; fi
  run $f check
  run $f -q check
  run $f status
  run $f coverage code
  run $f coverage tests
  run $f evidence claims "$ev"
  run $f change impact "$ch"
  run $f change reconcile "$ch"
  run $f docs preview "$ch"
done
