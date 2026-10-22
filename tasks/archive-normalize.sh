#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
set -euo pipefail

authority=/home/paudley/Active/rust-agentic-foundations/.worktrees/16-raf-workplan-strict-mode-force-coverage
stage="$authority/.stage/raf-workplan-strict-mode-force-coverage"
delivery_head=$(git --no-optional-locks -C "$authority" rev-parse HEAD)
current_index=
cleanup() {
    if [[ -n "$current_index" ]]; then
        rm -f -- "$current_index" "$current_index.lock"
    fi
}
trap cleanup EXIT
clones=(
    t2review/mut t3review/head t3review/main t3review/adv t3review/mut
    t4-states t4review t4old t4adv t5-base t5-states t5-states3
    t5review t5review-base t5review-adv t4r2 t4adv2 t5fix-state
    t6review t7review
)
[[ ${#clones[@]} -eq 20 ]]
actual=$(find "$stage/scratch" -type d -name .git -prune -print | wc -l)
[[ "$actual" -eq 20 ]]
mkdir -p "$stage/raw/archive-reconstruction"
: > "$stage/raw/archive-reconstruction/validated.tsv"
find "$stage/scratch" -type d -name .git -prune -o \( -type f -o -type l \) -printf '%y %m %s %T@ %p -> %l\n' |
    LC_ALL=C sort > "$stage/raw/archive-reconstruction/ordinary-before.txt"

symref() {
    local clone=$1 name=$2 output=$3 status
    if git --no-optional-locks -C "$clone" symbolic-ref -q "$name" > "$output"; then
        return
    else
        status=$?
        [[ "$status" -eq 1 ]] || return "$status"
        printf 'ABSENT_OR_DETACHED\n' > "$output"
    fi
}

for relative in "${clones[@]}"; do
    clone="$stage/scratch/$relative"
    [[ "$(realpath -e "$clone")" == "$clone" ]]
    [[ -d "$clone/.git" && ! -L "$clone/.git" ]]
    out="$stage/raw/archive-reconstruction/$relative"
    mkdir -p "$out"
    git --no-optional-locks -C "$clone" rev-parse --verify HEAD > "$out/head.txt"
    head=$(< "$out/head.txt")
    git --no-optional-locks -C "$authority" cat-file -e "$head^{commit}"
    git --no-optional-locks -C "$authority" merge-base --is-ancestor "$head" "$delivery_head"
    symref "$clone" HEAD "$out/HEAD.txt"
    symref "$clone" refs/remotes/origin/HEAD "$out/refs-remotes-origin-HEAD.txt"
    git --no-optional-locks -c core.quotePath=true -C "$clone" ls-files --stage > "$out/index.txt"
    if awk '$3 != 0 {bad=1} END {exit !bad}' "$out/index.txt"; then
        printf 'Unmerged index in %s\n' "$relative" >&2
        exit 1
    fi
    git --no-optional-locks -c core.quotePath=true -C "$clone" ls-files --others --exclude-standard > "$out/untracked.txt"
    git --no-optional-locks -c core.quotePath=true -C "$clone" status --porcelain=v1 --untracked-files=all > "$out/status.txt"
    git --no-optional-locks -C "$clone" diff --cached --binary --full-index --no-color --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ HEAD -- > "$out/staged.patch"
    git --no-optional-locks -C "$clone" diff --binary --full-index --no-color --no-ext-diff --no-textconv --src-prefix=a/ --dst-prefix=b/ -- > "$out/working.patch"
    : > "$out/used-refs.tsv"
    for ref in refs/remotes/origin/main refs/remotes/origin/HEAD; do
        leaf=${ref//\//-}
        if git --no-optional-locks -C "$clone" show-ref --verify --quiet "$ref"; then
            git --no-optional-locks -C "$clone" rev-parse --verify "$ref^{commit}" > "$out/$leaf-revision.txt"
            revision=$(< "$out/$leaf-revision.txt")
            git --no-optional-locks -C "$authority" cat-file -e "$revision^{commit}"
            git --no-optional-locks -C "$authority" merge-base --is-ancestor "$revision" "$delivery_head"
            git --no-optional-locks -C "$clone" merge-base --all HEAD "$ref" > "$out/$leaf-merge-base.txt"
            while IFS= read -r merge_base; do
                git --no-optional-locks -C "$authority" cat-file -e "$merge_base^{commit}"
                git --no-optional-locks -C "$authority" merge-base --is-ancestor "$merge_base" "$delivery_head"
            done < "$out/$leaf-merge-base.txt"
            printf '%s\t%s\n' "$ref" "$revision" >> "$out/used-refs.tsv"
        else
            status=$?
            [[ "$status" -eq 1 ]] || exit "$status"
            printf 'ABSENT\n' > "$out/$leaf-revision.txt"
            printf 'ABSENT\n' > "$out/$leaf-merge-base.txt"
            printf '%s\tABSENT\n' "$ref" >> "$out/used-refs.tsv"
        fi
    done
    current_index="$out/reconstruction.index"
    rm -f -- "$current_index" "$current_index.lock"
    GIT_INDEX_FILE="$current_index" git --no-optional-locks -C "$authority" read-tree "$head"
    if [[ -s "$out/staged.patch" ]]; then
        GIT_INDEX_FILE="$current_index" git --no-optional-locks -C "$authority" apply --cached "$out/staged.patch"
    fi
    GIT_INDEX_FILE="$current_index" git --no-optional-locks -c core.quotePath=true -C "$authority" ls-files --stage > "$out/replayed-index.txt"
    cmp "$out/index.txt" "$out/replayed-index.txt"
    cleanup
    current_index=
    printf '%s\t%s\tINDEX_EXACT\tUSED_REFS_AND_BASES_IN_DELIVERY_HISTORY\n' "$relative" "$head" >> "$stage/raw/archive-reconstruction/validated.tsv"
    printf 'VALIDATED %s\n' "$relative"
done
[[ "$(wc -l < "$stage/raw/archive-reconstruction/validated.tsv")" -eq 20 ]]
# No database removal occurs until every clone has passed exact replay.
for relative in "${clones[@]}"; do
    clone="$stage/scratch/$relative"
    [[ "$(realpath -e "$clone")" == "$clone" ]]
    [[ -d "$clone/.git" && ! -L "$clone/.git" ]]
    rm -rf -- "$clone/.git"
    printf 'REMOVED_REDUNDANT_DATABASE %s\n' "$relative"
done
find "$stage/scratch" -type d -name .git -prune -o \( -type f -o -type l \) -printf '%y %m %s %T@ %p -> %l\n' |
    LC_ALL=C sort > "$stage/raw/archive-reconstruction/ordinary-after.txt"
cmp "$stage/raw/archive-reconstruction/ordinary-before.txt" "$stage/raw/archive-reconstruction/ordinary-after.txt"
remaining=$(find "$stage" -type d -name .git -prune -print | wc -l)
[[ "$remaining" -eq 0 ]]
printf 'TERMINAL PASS: 20 exact index replays; all used refs/base commits in delivery ancestry; ordinary scratch inventory unchanged; zero nested Git databases; temporary indexes removed.\n'
