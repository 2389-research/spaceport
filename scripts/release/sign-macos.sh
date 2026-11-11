#!/usr/bin/env bash
# ABOUTME: Sign and notarize an explicitly built Spaceport macOS preview.
# ABOUTME: Retain submission receipts and isolate temporary signing credentials.
# Disable tracing before reading credentials, including when invoked with bash -x.
set +x
set -euo pipefail
# This script must never run under `set -x`, so a failure otherwise reports only
# a tool's message with no indication of which command produced it. Report the
# line number, which is safe: it reveals no credential material.
set -E
trap 'printf "sign-macos.sh: failed at line %d\n" "$LINENO" >&2' ERR
source "$(dirname -- "$0")/common.sh"
[[ $# -ge 3 && $# -le 5 ]] || fail 'Usage: bash sign-macos.sh VERSION UNSIGNED_APP OUTPUT_DIR [arm64|universal] [installed|import]'
version_check "$1"
[[ $(uname -s) == Darwin ]] || fail 'macOS required'
[[ -d $2 && -d $3 ]] || fail 'App and existing output directory required'
architecture=${4:-universal}
mode=${5:-import}
case $architecture in arm64) expected_archs=arm64 ;; universal) expected_archs='x86_64 arm64' ;; *) fail 'Architecture must be arm64 or universal' ;; esac
[[ $mode == installed || $mode == import ]] || fail 'Signing mode must be installed or import'
command -v jq >/dev/null || fail 'jq required'
out=$(cd -- "$3" && pwd)/Spaceport-$1-$architecture-apple-darwin.dmg
new_output "$out"
new_output "$out.json"
# Validate the complete artifact before reading or importing credentials.
app=$2
for field in CFBundleShortVersionString CFBundleVersion; do
    [[ $(plutil -extract "$field" raw "$app/Contents/Info.plist") == "$1" ]] || fail 'App version mismatch'
done
[[ $(plutil -extract LSMinimumSystemVersion raw "$app/Contents/Info.plist") == 14.2 ]] || fail 'App minimum OS mismatch'
[[ $(plutil -extract CFBundleExecutable raw "$app/Contents/Info.plist") == spaceport ]] || fail 'App executable mismatch'
app_id=$(cat "$release_root/crates/sheprd-gpui/macos-preview-app-id")
[[ $(plutil -extract CFBundleIdentifier raw "$app/Contents/Info.plist") == "$app_id" ]] || fail 'App signing identity mismatch'
[[ $(plutil -extract CFBundleName raw "$app/Contents/Info.plist") == Spaceport ]] || fail 'App name mismatch'
profile=$(plutil -extract SpaceportBuildProfile raw "$app/Contents/Info.plist")
python3 "$release_root/scripts/release/preview-identity.py" "$1" "$profile" "$app/Contents/MacOS/spaceport"
helpers=(sheprd-acp)
shopt -s nullglob dotglob
entries=("$app/Contents/MacOS/"*)
[[ ${#entries[@]} == 2 ]] || fail 'Unexpected bundled executables'
for entry in "${entries[@]}"; do
    [[ -f $entry && ! -L $entry ]] || fail 'Bundled executable must be a regular file, not a symlink'
    [[ ${entry##*/} == spaceport || ${entry##*/} == sheprd-acp ]] || fail 'Unexpected bundled executable'
done
shopt -u nullglob dotglob
for executable in spaceport "${helpers[@]}"; do
    archs=$(lipo -archs "$app/Contents/MacOS/$executable")
    [[ $archs == "$expected_archs" || ( $architecture == universal && $archs == 'arm64 x86_64' ) ]] || fail 'App architecture mismatch'
done
python3 "$release_root/scripts/release/sparkle_framework.py" validate-unsigned-bundle "$app" --architecture "$architecture"
team=HD9NM9NSMK
# The local installed certificate may be selected by its unambiguous public SHA1.
[[ ${MACOS_SIGNING_IDENTITY:-} == 'Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)' || ${MACOS_SIGNING_IDENTITY:-} == FF599C9C778C879ECB7DC8AEF258890A1268F7E6 ]] || fail 'Wrong Developer ID identity or Team'
for name in APPLE_API_KEY_ID APPLE_API_ISSUER_ID; do
    [[ -n ${!name:-} ]] || fail "Required environment variable: $name"
done
if [[ $mode == import ]]; then
    for name in MACOS_CERTIFICATE_P12_BASE64 MACOS_CERTIFICATE_PASSWORD APPLE_API_PRIVATE_KEY; do
        [[ -n ${!name:-} ]] || fail "Required environment variable: $name"
    done
else
    [[ -f ${APPLE_API_KEY_PATH:-} ]] || fail 'APPLE_API_KEY_PATH must name an existing private key'
fi
receipts=${SPACEPORT_NOTARY_RECEIPTS:-$release_root/target/notarization}
[[ $receipts == /* ]] || fail 'SPACEPORT_NOTARY_RECEIPTS must be an absolute path'
receipts="$receipts/$app_id-$1-$architecture"
[[ ! -e $receipts/app ]] || fail "Notarization receipt already exists: $receipts/app; this preview version is reserved"
apple_retry_delay=${APPLE_RETRY_DELAY:-30}
[[ $apple_retry_delay =~ ^[0-9]{1,3}$ ]] || fail 'APPLE_RETRY_DELAY must be whole seconds'
umask 077
tmp=$(mktemp -d "${out%/*}/.herdr-sign.XXXXXX")
keychain=$tmp/signing.keychain-db
sign_options=(--force --sign "$MACOS_SIGNING_IDENTITY")
complete=0
search_list_touched=0
original_keychains=()
cleanup_keychain() {
    local failed=0
    if [[ $search_list_touched == 1 ]]; then
        security list-keychains -d user -s ${original_keychains[@]+"${original_keychains[@]}"} || failed=1
    fi
    if [[ -e $keychain ]]; then
        security delete-keychain "$keychain" >/dev/null 2>&1 || failed=1
    fi
    return "$failed"
}
cleanup() {
    status=$?
    [[ $status != 0 || $complete == 1 ]] || status=1
    trap - EXIT
    cleanup_keychain || status=1
    rm -rf -- "$tmp" || status=1
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
if [[ $mode == import ]]; then
# Creation may add the keychain to the user search list on some macOS versions,
# and may not. Set the list explicitly below so the signing keychain is present
# exactly once; cleanup_keychain restores the original list.
search_list=$(security list-keychains -d user)
while IFS= read -r line; do
    [[ -z $line ]] && continue
    [[ $line =~ ^[[:space:]]*\"(.*)\"[[:space:]]*$ ]] || fail 'Cannot parse keychain search list'
    original_keychains+=("${BASH_REMATCH[1]}")
done <<< "$search_list"
password=$(openssl rand -hex 32)
printf '%s' "$MACOS_CERTIFICATE_P12_BASE64" | base64 -D > "$tmp/certificate.p12"
printf '%s\n' "$APPLE_API_PRIVATE_KEY" > "$tmp/AuthKey.p8"
unset MACOS_CERTIFICATE_P12_BASE64 APPLE_API_PRIVATE_KEY
search_list_touched=1
security create-keychain -p "$password" "$keychain"
# codesign does not honour --keychain for identity lookup on macOS 15: it searches
# the user keychain search list, so the signing keychain must be in it. Verified on
# a macos-15 runner, where an identity that `security find-identity` reports as
# valid is otherwise rejected with "The specified item could not be found in the
# keychain". cleanup_keychain restores the original list.
security list-keychains -d user -s ${original_keychains[@]+"${original_keychains[@]}"} "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$password" "$keychain"
security import "$tmp/certificate.p12" -k "$keychain" -P "$MACOS_CERTIFICATE_PASSWORD" -T /usr/bin/codesign >/dev/null
unset MACOS_CERTIFICATE_PASSWORD
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" >/dev/null
unset password
rm "$tmp/certificate.p12"
# Reject a P12 containing additional signing identities.
identities=$(security find-identity -v -p codesigning "$keychain")
identity_count=$(printf '%s\n' "$identities" | awk '/valid identities found$/ {print $1}')
[[ $identity_count == 1 && $identities == *'Developer ID Application: 2389 Research, Inc (HD9NM9NSMK)'* ]] || fail 'Imported identity does not match the selected Team'
sign_options+=(--keychain "$keychain")
notary_key=$tmp/AuthKey.p8
else
    notary_key=$APPLE_API_KEY_PATH
fi
mkdir "$tmp/image"
app=$tmp/image/Spaceport.app
ditto "$2" "$app"
# Sign known Sparkle code inside out, before the app; never discover signing targets.
python3 "$release_root/scripts/release/sparkle_framework.py" sign "$app" --architecture "$architecture" -- "${sign_options[@]}"
# Nested chat helpers need no app entitlements and are signed before the app.
for executable in "${helpers[@]}"; do
    codesign "${sign_options[@]}" --options runtime --timestamp "$app/Contents/MacOS/$executable"
done
codesign "${sign_options[@]}" --options runtime --entitlements "$release_root/assets/macos/Sheprd.entitlements" --timestamp "$app/Contents/MacOS/spaceport"
codesign "${sign_options[@]}" --options runtime --entitlements "$release_root/assets/macos/Sheprd.entitlements" --timestamp "$app"
codesign --verify --deep --strict --verbose=2 "$app"
# Only idempotent ticket operations use retries. Submit is deliberately excluded.
apple() {
    local attempt
    for attempt in 1 2 3 4 5 6 7 8 9 10; do
        xcrun "$@" && return 0
        ((attempt < 10)) || break
        echo "xcrun $1 $2 failed (attempt $attempt of 10); retrying in ${apple_retry_delay}s" >&2
        sleep "$apple_retry_delay"
    done
    return 1
}
notarize() {
    local digest receipt uuid status
    digest=$(shasum -a 256 "$1" | cut -d ' ' -f 1)
    receipt="$receipts/$2"
    mkdir -p "$receipts"
    mkdir "$receipt" 2>/dev/null || fail "Notarization receipt already exists: $receipt; inspect it before taking further action"
    printf '%s\n' "$digest" > "$receipt/artifact.sha256"
    printf '%s\n' "$(basename -- "$1")" > "$receipt/artifact-name.txt"
    # A lost response cannot establish that Apple did not accept the upload.
    # Keep the intent and output even on cancellation; never submit it again.
    xcrun notarytool submit "$1" --key "$notary_key" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" --output-format json > "$receipt/submission.json" 2> "$receipt/submit.stderr" || true
    uuid=$(jq -er -s 'if length == 1 then .[0].id else empty end | select(type == "string" and test("^[0-9a-fA-F]{8}(-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}$"))' "$receipt/submission.json") || fail "Notarization submit outcome unknown; do not resubmit. Receipt: $receipt"
    printf '%s\n' "$uuid" > "$receipt/submission-id.txt"
    xcrun notarytool wait "$uuid" --key "$notary_key" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" --timeout 20m --output-format json > "$receipt/status.json" 2> "$receipt/wait.stderr" || true
    status=$(jq -er -s 'if length == 1 then .[0].status else empty end | select(type == "string")' "$receipt/status.json") || fail "Notarization status unknown; retained UUID and receipt: $receipt"
    if [[ $status != Accepted ]]; then
        if [[ $status == Invalid || $status == Rejected ]]; then
            xcrun notarytool log "$uuid" --key "$notary_key" --key-id "$APPLE_API_KEY_ID" --issuer "$APPLE_API_ISSUER_ID" "$receipt/notary-log.json" > "$receipt/log.stdout" 2> "$receipt/log.stderr" || true
        fi
        fail "Notarization $status; retained UUID and receipt: $receipt"
    fi
}
# Verify the actual signing result, including the certificate's Team, not just its label.
signature=$(codesign -d --verbose=4 "$app" 2>&1)
signed_team=$(printf '%s\n' "$signature" | sed -n 's/^TeamIdentifier=//p')
signed_id=$(printf '%s\n' "$signature" | sed -n 's/^Identifier=//p')
[[ $signed_team == "$team" && $signed_id == "$app_id" ]] || fail 'Signed app Team or identifier mismatch'
ditto -c -k --keepParent "$app" "$tmp/Spaceport.zip"
notarize "$tmp/Spaceport.zip" app
apple stapler staple "$app"
apple stapler validate "$app"
codesign --verify --deep --strict --verbose=2 "$app"
spctl --assess --type execute --verbose=2 "$app"
ln -s /Applications "$tmp/image/Applications"
hdiutil create -volname Spaceport -srcfolder "$tmp/image" -format UDZO -ov "$tmp/Spaceport.dmg"
codesign "${sign_options[@]}" --timestamp "$tmp/Spaceport.dmg"
notarize "$tmp/Spaceport.dmg" dmg
apple stapler staple "$tmp/Spaceport.dmg"
apple stapler validate "$tmp/Spaceport.dmg"
codesign --verify --strict --verbose=2 "$tmp/Spaceport.dmg"
spctl --assess --type open --context context:primary-signature --verbose=2 "$tmp/Spaceport.dmg"
chmod 644 "$tmp/Spaceport.dmg"
cleanup_keychain
search_list_touched=0
new_output "$out"
mv -n "$tmp/Spaceport.dmg" "$out"
[[ ! -e $tmp/Spaceport.dmg ]] || fail "Could not promote output: $out"
jq -n --arg artifact "$(basename -- "$out")" --arg sha256 "$(shasum -a 256 "$out" | cut -d ' ' -f 1)" \
    --arg app_id "$app_id" --arg version "$1" --arg profile "$profile" --arg architecture "$architecture" \
    --arg team_id "$team" '{artifact: $artifact, sha256: $sha256, app_id: $app_id, version: $version, profile: $profile, architecture: $architecture, team_id: $team_id, kind: "signed-macos-preview"}' > "$out.json"
complete=1
printf '%s\n' "$out"
