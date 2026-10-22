#!/usr/bin/env bash
# usage: cmp.sh <tree> <outdir>
set -u
tree=$1; out=$2; mkdir -p "$out"
MB=/opt/.cargo/target/main-933cc592/debug/cargo-workplan
HB=/opt/.cargo/target/adv-6d310e02/debug/cargo-workplan
slugs=$(ls "$tree"/docs/capabilities | grep -v index.md | sed 's/\.md$//')
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
  f=(); [[ $fmt == json ]] && f=(--format json)
  run "${f[@]}" check
  run "${f[@]}" -q check
  run "${f[@]}" status
  run "${f[@]}" coverage code
  run "${f[@]}" coverage tests
  run "${f[@]}" evidence claims
  run "${f[@]}" docs sync --check
  for s in $slugs; do run "${f[@]}" docs preview "$s"; done
done
