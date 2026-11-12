#!/usr/bin/env bash
# ABOUTME: Stage installed release binaries in one stable cache directory.
# ABOUTME: Reuse restored binaries without depending on the installer's paths.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# -ge 2 ]] || fail 'Usage: bash cache-release-tools.sh CACHE_DIRECTORY TOOL [TOOL...]'
mkdir -p -- "$1"
cache_directory=$(cd -- "$1" && pwd -P)
shift
export PATH="$cache_directory:$PATH"
for tool in "$@"; do
    executable=$(type -P "$tool") || fail "Release tool not found: $tool"
    if [[ $executable != "$cache_directory/$tool" ]]; then
        install -m 755 "$executable" "$cache_directory/$tool"
    fi
done
printf '%s\n' "$cache_directory"
