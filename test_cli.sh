#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
cd "$script_dir"

function fn_main() {
    test_cmd "test_res/0xdeadbeef_x86.stdout.txt" \
        0xdeadbeef x86

    test_cmd "test_res/0xdeadbeef_x86_pae.stdout.txt" \
        0xdeadbeef x86 --pae

    test_cmd "test_res/0xdeadbeef_x86_64.stdout.txt" \
        0xdeadbeef x86_64

    test_cmd "test_res/0xdeadbeef_x86_64_5level.stdout.txt" \
        0xdeadbeef x86_64 --five-level

    # Global options should also be accepted after a subcommand.
    test_cmd "test_res/0xdeadbeef_x86.stdout.txt" \
        0xdeadbeef x86 --color never

    test_success 0x00007fffffffffff x86_64
    test_success 0xffff800000000000 x86_64
    test_error \
        "error: 0x0000800000000000 is not a canonical 48-bit x86_64 virtual address" \
        0x0000800000000000 x86_64
    test_error \
        "error: 0xffff7fffffffffff is not a canonical 48-bit x86_64 virtual address" \
        0xffff7fffffffffff x86_64

    test_success 0x00ffffffffffffff x86_64 --five-level
    test_success 0xff00000000000000 x86_64 --five-level
    test_error \
        "error: 0x0100000000000000 is not a canonical 57-bit x86_64 virtual address" \
        0x0100000000000000 x86_64 --five-level
    test_error \
        "error: 0xfeffffffffffffff is not a canonical 57-bit x86_64 virtual address" \
        0xfeffffffffffffff x86_64 --five-level

    test_error \
        "error: invalid value '0xnothex' for '<VIRTUAL_ADDRESS>': virtual address could not be parsed as number of type \`u64\`" \
        0xnothex x86
}

function test_cmd() {
    local expected_file=$1
    shift

    local actual
    local expected
    actual=$(cargo run --quiet -- "$@" 2>&1)
    expected=$(<"$expected_file")

    if [[ "$actual" != "$expected" ]]; then
        echo "Unexpected output for: cargo run -- $*"
        diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual")
        exit 1
    fi
}

function test_success() {
    if ! cargo run --quiet -- "$@" >/dev/null; then
        echo "Expected success for: cargo run -- $*"
        exit 1
    fi
}

function test_error() {
    local expected_first_line=$1
    shift

    local actual
    local status
    set +e
    actual=$(cargo run --quiet -- "$@" 2>&1)
    status=$?
    set -e

    if ((status != 2)); then
        echo "Expected exit code 2 but got $status for: cargo run -- $*"
        exit 1
    fi

    local actual_first_line=${actual%%$'\n'*}
    if [[ "$actual_first_line" != "$expected_first_line" ]]; then
        echo "Unexpected error for: cargo run -- $*"
        diff \
            <(printf '%s\n' "$expected_first_line") \
            <(printf '%s\n' "$actual_first_line")
        exit 1
    fi
}

fn_main
