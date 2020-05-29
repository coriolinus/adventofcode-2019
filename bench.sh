#!/usr/bin/env bash

set -e 

cargo fmt

input="$1"
shift 1

if [ -z "$input" ]; then
    echo "usage: $0 INPUT"
    exit 1
fi

set -x

hyperfine "target/release/aoc2019 $input $*" "target/release/aoc2019 $input --no-part1 --part2 $*"