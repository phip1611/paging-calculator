#!/usr/bin/env bash

set -e

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

function fn_main() {
    test_cmd "test_res/0xdeadbeef_x86.stdout.txt" \
        "cargo run 2>/dev/null -- 0xdeadbeef x86"

    test_cmd "test_res/0xdeadbeef_x86_pae.stdout.txt" \
        "cargo run 2>/dev/null -- 0xdeadbeef x86 --pae"

    test_cmd "test_res/0xdeadbeef_x86_64.stdout.txt" \
        "cargo run 2>/dev/null -- 0xdeadbeef x86_64"

    test_cmd "test_res/0xdeadbeef_x86_64_5level.stdout.txt" \
        "cargo run 2>/dev/null -- 0xdeadbeef x86_64 --five-level"
}

function test_cmd() {
    FILE=$1
    CMD=$2

    ACTUAL=$(eval "$CMD")
    EXPECTED=$(cat "$FILE")

    if [ "$ACTUAL" != "$EXPECTED" ];
    then
        echo "Unexpected output! CMD is: '$CMD'"
        diff  <(echo "$EXPECTED" ) <(echo "$ACTUAL")
        exit 1
    fi
}

fn_main
