#!/usr/bin/env bash
set -euo pipefail

# Shared with scripts sourcing this file; standalone ShellCheck cannot see them.
# shellcheck disable=SC2034
release_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
fail() { printf '%s\n' "$*" >&2; exit 1; }
# Releases are calendar versions: an eight-digit YYYYMMDD date and a same-day
# counter starting at 1, compared numerically so publication order is version order.
version_check() {
    [[ $1 =~ ^[1-9][0-9]{7}\.[1-9][0-9]*$ ]] || fail 'Version must be YYYYMMDD.COUNTER (no v prefix, counter from 1, no leading zeros)'
}
# iOS reserves the last two digits of the build number for the daily counter.
ios_version_check() {
    version_check "$1"
    local counter=${1#*.}
    [[ ${#counter} -le 2 ]] || fail 'iOS counter must be 1 through 99'
}
new_output() {
    [[ ! -e $1 && ! -L $1 ]] || fail "Output already exists: $1"
    [[ -d $(dirname -- "$1") ]] || fail "Output parent must exist: $1"
}
