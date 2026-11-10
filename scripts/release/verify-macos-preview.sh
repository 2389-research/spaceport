#!/usr/bin/env bash
# ABOUTME: Verify the final macOS preview image and smoke its mounted executable.
# ABOUTME: Write a checksum only after signatures, tickets, identity and CLI checks pass.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# == 3 ]] || fail "Usage: verify-macos-preview.sh VERSION PROFILE OUTPUT_DIR"
version_check "$1"
case "$2" in debug|release) ;; *) fail "Expected debug or release profile" ;; esac
[[ $(uname -s) == Darwin ]] || fail "Preview verification requires macOS"
directory=$(cd -- "$3" && pwd)
file="Spaceport-$1-arm64-apple-darwin.dmg"
image="$directory/$file"
[[ -f "$image" && ! -L "$image" ]] || fail "Missing regular preview image: $image"
new_output "$directory/SHA256SUMS"
digest=$(shasum -a 256 "$image" | cut -d ' ' -f 1)
jq -e --arg file "$file" --arg digest "$digest" --arg version "$1" --arg profile "$2" '
    .artifact == $file and .sha256 == $digest and .version == $version and
    .profile == $profile and .architecture == "arm64" and
    .app_id == "ai.2389.spaceport" and .team_id == "HD9NM9NSMK" and
    .kind == "signed-macos-preview"' "$image.json" > /dev/null || fail "Preview metadata or checksum mismatch: $image.json"
codesign --verify --strict --verbose=2 "$image"
xcrun stapler validate "$image"
spctl --assess --type open --context context:primary-signature --verbose=2 "$image"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/spaceport-verify.XXXXXX")
mounted=false
cleanup() {
    if [[ "$mounted" == true ]]; then
        hdiutil detach "$tmp/mount" || return 1
    fi
    rm -rf -- "$tmp"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
# Mark intent first: a failed attach may still have mounted the exact private path.
mounted=true
hdiutil attach -readonly -nobrowse -mountpoint "$tmp/mount" "$image" > "$tmp/attach.log"
app="$tmp/mount/Spaceport.app"
codesign --verify --deep --strict --verbose=2 "$app"
signature=$(codesign -dv --verbose=4 "$app" 2>&1)
grep -qx 'Identifier=ai.2389.spaceport' <<< "$signature" || fail "Signed app identifier mismatch"
grep -qx 'TeamIdentifier=HD9NM9NSMK' <<< "$signature" || fail "Signed app Team mismatch"
xcrun stapler validate "$app"
spctl --assess --type execute --verbose=2 "$app"
python3 "$release_root/scripts/release/preview-identity.py" "$1" "$2" "$app/Contents/MacOS/spaceport"
python3 "$release_root/scripts/release/smoke-cli.py" "$app/Contents/MacOS/spaceport"
"$app/Contents/MacOS/sheprd-acp" --help > /dev/null
hdiutil detach "$tmp/mount"
mounted=false
(cd -- "$directory" && shasum -a 256 "$file" > SHA256SUMS)
