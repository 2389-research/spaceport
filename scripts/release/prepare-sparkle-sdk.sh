#!/usr/bin/env bash
# ABOUTME: Acquire a pinned SDK on misses and validate existing SDK bytes on hits.
# ABOUTME: Preserve invalid cache contents and print only the validated SDK path.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# -ge 1 && $# -le 2 ]] || fail 'Usage: bash prepare-sparkle-sdk.sh SDK_DIRECTORY [PINNED_ARCHIVE]'
sdk_directory=$1
if [[ -L $sdk_directory || ( -e $sdk_directory && ! -d $sdk_directory ) ]]; then
    fail "Expected a real SDK directory: $sdk_directory"
elif [[ -d $sdk_directory ]]; then
    python3 "$release_root/scripts/release/sparkle_framework.py" validate-sdk "$sdk_directory"
elif [[ $# == 2 ]]; then
    python3 "$release_root/scripts/release/sparkle_framework.py" acquire "$sdk_directory" --archive "$2" > /dev/null
else
    python3 "$release_root/scripts/release/sparkle_framework.py" acquire "$sdk_directory" > /dev/null
fi
(cd -- "$sdk_directory" && pwd -P)
