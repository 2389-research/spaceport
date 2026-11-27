#!/usr/bin/env bash
# ABOUTME: Verify the exported TestFlight app and each expected extension before upload.
# ABOUTME: Check the shipped identities, build numbers, signatures and embedded App Group grants.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# -ge 6 ]] || fail 'Usage: verify-ios-app.sh APP TEAM BUILD_NUMBER APP_GROUP APP_ID EXTENSION_ID...'
app=$1 team=$2 build_number=$3 app_group=$4
shift 4
bundle_ids=("$@")
bundles=("$app")
[[ -d $app ]] || fail 'Exported .ipa is missing its app'
shopt -s nullglob
for extension in "$app"/PlugIns/*.appex; do
    id=$(plutil -extract CFBundleIdentifier raw "$extension/Info.plist") || fail 'Cannot identify exported extension'
    found=false
    for ((i=1; i<${#bundle_ids[@]}; i++)); do
        if [[ $id == "${bundle_ids[i]}" ]]; then
            [[ -z ${bundles[i]:-} ]] || fail "Duplicate exported extension $id"
            bundles[i]=$extension
            found=true
            break
        fi
    done
    [[ $found == true ]] || fail "Unexpected exported extension $id"
done
for i in "${!bundle_ids[@]}"; do
    [[ -n ${bundles[i]:-} ]] || fail "Missing exported extension ${bundle_ids[i]}"
done

codesign --verify --deep --strict --verbose=2 "$app"
for i in "${!bundle_ids[@]}"; do
    bundle=${bundles[i]}
    id=${bundle_ids[i]}
    [[ $(plutil -extract CFBundleIdentifier raw "$bundle/Info.plist") == "$id" ]] || fail "Exported identifier mismatch for $id"
    [[ $(plutil -extract CFBundleVersion raw "$bundle/Info.plist") == "$build_number" ]] || fail "Exported build number mismatch for $id"
    signature=$(codesign -d --verbose=4 "$bundle" 2>&1) || fail "Cannot read signature for $id"
    grep -Fxq "TeamIdentifier=$team" <<< "$signature" || fail "Signed Team mismatch for $id"
    grep -Fxq "Identifier=$id" <<< "$signature" || fail "Signed identifier mismatch for $id"
    codesign -d --entitlements - --xml "$bundle" 2>/dev/null | plutil -convert json -o - - |
        jq -e --arg group "$app_group" '."com.apple.security.application-groups" | type == "array" and index($group) != null' >/dev/null ||
        fail "Signed $id does not grant $app_group"
    # Distribution profiles also contain DER certificates and dates. Only the entitlements are JSON values.
    security cms -D -i "$bundle/embedded.mobileprovision" | plutil -extract Entitlements json -o - - |
        jq -e --arg app_id "$team.$id" --arg group "$app_group" '
            .["application-identifier"] == $app_id and
            .["get-task-allow"] == false and
            (.["com.apple.security.application-groups"] | type == "array" and index($group) != null)' >/dev/null ||
        fail "Invalid distribution profile identifier, debugging entitlement or App Group for $id"
done
