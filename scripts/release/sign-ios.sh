#!/usr/bin/env bash
# ABOUTME: Sign, verify and upload the Spaceport Mobile app to TestFlight.
# ABOUTME: Hold distribution credentials in a temporary keychain and upload only a verified build.
# Disable tracing before reading credentials, including when invoked with bash -x.
set +x
set -euo pipefail
# This script must never run under `set -x`, so a failure otherwise reports only
# a tool's message with no indication of which command produced it. Report the
# line number, which is safe: it reveals no credential material.
set -E
trap 'printf "sign-ios.sh: failed at line %d\n" "$LINENO" >&2' ERR
source "$(dirname -- "$0")/common.sh"
[[ $# -eq 2 ]] || fail 'Usage: bash sign-ios.sh VERSION OUTPUT_DIR'
version_check "$1"
[[ $(uname -s) == Darwin ]] || fail 'macOS required'
[[ -d $2 ]] || fail 'Existing output directory required'
for tool in base64 codesign jq openssl plutil security shasum unzip xcodebuild xcrun; do
    command -v "$tool" >/dev/null || fail "Required tool: $tool"
done
# Match the targets in this release checkout. The app and extensions share one App Group.
bundle_id=ai.2389.spaceport.mobile
extension_id=ai.2389.spaceport.mobile.agent-status
app_group=group.ai.2389.spaceport.mobile.widget
# The build number follows the Android versionCode rule in android-request.py:
# YYYYMMDD*100+COUNTER, so every later tag uploads a greater TestFlight build.
# MARKETING_VERSION stays project.yml's, keeping the App Store Connect record.
build_number=$((10#${1%.*} * 100 + 10#${1#*.}))
project=$release_root/mobile/apple/SpaceportMobile.xcodeproj
[[ -f $project/project.pbxproj ]] || fail 'Run ./mobile/build-apple --release before signing'
[[ -f $release_root/mobile/.build/SpaceportMobileCore.xcframework/Info.plist ]] || fail 'Run ./mobile/build-apple --release before signing; the Rust library is missing'
# Reject a checkout whose Release configuration no longer builds the identifiers
# this route verifies, before any credential is read. Anchored at the end of the
# line so a Debug value, which appends to each identifier, cannot satisfy it.
bundle_ids=("$bundle_id" "$extension_id")
profile_settings=(SPACEPORT_MOBILE_PROFILE SPACEPORT_ACTIVITY_PROFILE)
if grep -Eq '^  ShareFile:[[:space:]]*$' "$release_root/mobile/apple/project.yml"; then
    bundle_ids+=("$bundle_id.share")
    profile_settings+=(SPACEPORT_SHARE_PROFILE)
fi
facts=("SPACEPORT_WIDGET_GROUP: $app_group")
for id in "${bundle_ids[@]}"; do facts+=("PRODUCT_BUNDLE_IDENTIFIER: $id"); done
for fact in "${facts[@]}"; do
    grep -Eq "^[[:space:]]*${fact//./\\.}[[:space:]]*$" "$release_root/mobile/apple/project.yml" \
        || fail "mobile/apple/project.yml no longer declares the Release $fact"
done
team=${SPACEPORT_APPLE_TEAM_ID:-}
[[ $team =~ ^[A-Z0-9]{10}$ ]] || fail 'SPACEPORT_APPLE_TEAM_ID must be the 10-character Apple Developer Team ID'
for name in APPLE_API_ISSUER_ID APPLE_API_KEY_ID APPLE_API_PRIVATE_KEY \
            IOS_DISTRIBUTION_P12_BASE64 IOS_DISTRIBUTION_P12_PASSWORD; do
    [[ -n ${!name:-} ]] || fail "Required environment variable: $name"
done
# Base64 profiles are written for this build and installed under the UUID they
# carry. UUIDs alone are accepted instead, so xcodebuild can fetch the profiles
# through the API key; both paths still need the API key to upload.
missing_written=()
missing_uuids=()
for setting in "${profile_settings[@]}"; do
    written=${setting}_B64
    uuid=${setting}_UUID
    [[ -n ${!written:-} ]] || missing_written+=("$written")
    [[ -n ${!uuid:-} ]] || missing_uuids+=("$uuid")
done
if [[ -z ${missing_written[*]:-} ]]; then
    profile_mode='written'
elif [[ -z ${missing_uuids[*]:-} ]]; then
    profile_mode='uuid'
else
    fail "Missing profiles. Supply ${missing_written[*]} for base64 signing, or ${missing_uuids[*]} for UUID signing."
fi
out=$(cd -- "$2" && pwd)
ipa=$out/Spaceport-$1-ios.ipa
new_output "$ipa"
new_output "$ipa.sha256"
new_output "$ipa.json"
umask 077
tmp=$(mktemp -d "$out/.spaceport-ios.XXXXXX")
keychain=$tmp/signing.keychain-db
api_key=$tmp/private_keys/AuthKey_$APPLE_API_KEY_ID.p8
# Xcode finds a profile named by UUID only here, so this route installs the
# profiles it was given and removes the ones it added on exit.
profiles_dir="$HOME/Library/MobileDevice/Provisioning Profiles"
original_keychains=()
installed_profiles=()
search_list_touched=0
complete=0
cleanup_keychain() {
    local failed=0 profile
    for profile in ${installed_profiles[@]+"${installed_profiles[@]}"}; do
        rm -f -- "$profile" || failed=1
    done
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
write_profile() {  # $1 base64 profile, $2 label, $3 installed-profile directory
    local decoded=$tmp/$2.mobileprovision
    printf '%s' "$1" | base64 -D > "$decoded"
    security cms -D -i "$decoded" > "$tmp/$2.plist" || fail "The $2 profile is not a readable provisioning profile"
    uuid=$(plutil -extract UUID raw "$tmp/$2.plist") || fail "The $2 profile has no UUID"
    [[ $uuid =~ ^[0-9A-Fa-f]{8}(-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}$ ]] || fail "The $2 profile UUID is malformed"
    # A profile already installed under this UUID is the same profile: leave it,
    # and remove only what this run installs.
    if [[ ! -e $3/$uuid.mobileprovision ]]; then
        cp "$decoded" "$3/$uuid.mobileprovision"
        installed_profiles+=("$3/$uuid.mobileprovision")
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$tmp/private_keys"
printf '%s' "$IOS_DISTRIBUTION_P12_BASE64" | base64 -D > "$tmp/certificate.p12"
printf '%s\n' "$APPLE_API_PRIVATE_KEY" > "$api_key"
chmod 600 "$api_key"
unset IOS_DISTRIBUTION_P12_BASE64 APPLE_API_PRIVATE_KEY
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
search_list_touched=1
security create-keychain -p "$password" "$keychain"
# xcodebuild and codesign search the user keychain list, not this path alone
# (sign-macos.sh records the same behavior on macOS 15).
security list-keychains -d user -s ${original_keychains[@]+"${original_keychains[@]}"} "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$password" "$keychain"
security import "$tmp/certificate.p12" -k "$keychain" -P "$IOS_DISTRIBUTION_P12_PASSWORD" -T /usr/bin/codesign >/dev/null
unset IOS_DISTRIBUTION_P12_PASSWORD
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$password" "$keychain" >/dev/null
unset password
rm "$tmp/certificate.p12"
# Reject a P12 holding any other identity, or a certificate for another Team.
identities=$(security find-identity -v -p codesigning "$keychain")
identity_count=$(printf '%s\n' "$identities" | awk '/valid identities found$/ {print $1}')
[[ $identity_count == 1 ]] || fail 'Imported P12 must hold exactly one signing identity'
subject=$(printf '%s\n' "$identities" | sed -n 's/^ *[0-9]*) [0-9A-F]\{40\} "\(.*\)"$/\1/p')
[[ $subject == "Apple Distribution: "*"($team)" ]] || fail "Imported identity '$subject' is not an Apple Distribution certificate for Team $team"
profile_specifiers=()
archive_settings=()
provisioning_args=()
if [[ $profile_mode == written ]]; then
    mkdir -p "$profiles_dir"
else
    provisioning_args=(-allowProvisioningUpdates
        -authenticationKeyPath "$api_key"
        -authenticationKeyID "$APPLE_API_KEY_ID"
        -authenticationKeyIssuerID "$APPLE_API_ISSUER_ID")
fi
for setting in "${profile_settings[@]}"; do
    if [[ $profile_mode == written ]]; then
        encoded=${setting}_B64
        # Keep installation and cleanup in the same shell. write_profile also sets uuid.
        write_profile "${!encoded}" "$setting" "$profiles_dir"
    else
        variable=${setting}_UUID
        uuid=${!variable}
        [[ $uuid =~ ^[0-9A-Fa-f]{8}(-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}$ ]] || fail "Malformed profile UUID in $variable"
    fi
    profile_specifiers+=("$uuid")
    archive_settings+=("$setting=$uuid")
done
# Xcode project scripts inherit this environment, even on checkouts without ShareFile.
unset SPACEPORT_MOBILE_PROFILE_B64 SPACEPORT_ACTIVITY_PROFILE_B64 SPACEPORT_SHARE_PROFILE_B64
printf 'archive\n' >&2
xcodebuild archive -project "$project" -scheme SpaceportMobile -configuration Release \
    -destination 'generic/platform=iOS' -archivePath "$tmp/SpaceportMobile.xcarchive" \
    CODE_SIGN_STYLE=Manual CODE_SIGN_IDENTITY='Apple Distribution' \
    DEVELOPMENT_TEAM="$team" CURRENT_PROJECT_VERSION="$build_number" \
    OTHER_CODE_SIGN_FLAGS="--keychain $keychain" \
    "${archive_settings[@]}" \
    ${provisioning_args[@]+"${provisioning_args[@]}"}
# Manual export signed by the profiles this run was given, with Xcode's own
# build-number rewrite disabled so the uploaded build number is the derived one.
# The export methods this Xcode accepts still name app-store for a store build.
cat > "$tmp/exportOptions.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store</string>
    <key>signingStyle</key>
    <string>manual</string>
    <key>signingCertificate</key>
    <string>Apple Distribution</string>
    <key>teamID</key>
    <string>$team</string>
    <key>manageAppVersionAndBuildNumber</key>
    <false/>
    <key>provisioningProfiles</key>
    <dict>
PLIST
for i in "${!bundle_ids[@]}"; do
    printf '        <key>%s</key><string>%s</string>\n' "${bundle_ids[i]}" "${profile_specifiers[i]}"
done >> "$tmp/exportOptions.plist"
cat >> "$tmp/exportOptions.plist" <<'PLIST'
    </dict>
</dict>
</plist>
PLIST
plutil -lint "$tmp/exportOptions.plist" >/dev/null || fail 'Generated export options are not a valid plist'
printf 'export\n' >&2
mkdir "$tmp/export"
xcodebuild -exportArchive -archivePath "$tmp/SpaceportMobile.xcarchive" \
    -exportPath "$tmp/export" -exportOptionsPlist "$tmp/exportOptions.plist" \
    ${provisioning_args[@]+"${provisioning_args[@]}"}
shopt -s nullglob
exported=("$tmp/export/"*.ipa)
shopt -u nullglob
[[ ${#exported[@]} == 1 ]] || fail 'Expected exactly one exported .ipa'
unzip -q "${exported[0]}" -d "$tmp/verify" || fail 'Exported .ipa is not a readable archive'
app=$tmp/verify/Payload/SpaceportMobile.app
bash "$release_root/scripts/release/verify-ios-app.sh" "$app" "$team" "$build_number" "$app_group" "${bundle_ids[@]}"
marketing_version=$(plutil -extract CFBundleShortVersionString raw "$app/Info.plist")
new_output "$ipa"
mv -n "${exported[0]}" "$ipa"
[[ ! -e ${exported[0]} ]] || fail "Could not promote output: $ipa"
digest=$(shasum -a 256 "$ipa" | cut -d ' ' -f 1)
printf '%s  %s\n' "$digest" "${ipa##*/}" > "$ipa.sha256"
chmod 644 "$ipa" "$ipa.sha256"
# altool reads the API key from ./private_keys, so it runs from the temporary
# directory that holds the one key this build may use.
printf 'upload\n' >&2
status=0
(cd -- "$tmp" && xcrun altool --upload-app --type ios --file "$ipa" \
    --apiKey "$APPLE_API_KEY_ID" --apiIssuer "$APPLE_API_ISSUER_ID" \
    --output-format json) > "$tmp/upload.json" 2> "$tmp/upload.log" || status=$?
cat -- "$tmp/upload.log" >&2 || true
if [[ $status != 0 ]]; then
    cat -- "$tmp/upload.json" >&2 || true
    fail "TestFlight upload failed; do not upload build $build_number again. Retained: $ipa"
fi
# altool's exit status is the upload result. Its message is reported when Apple
# supplies one, so a changed envelope cannot turn an accepted upload into a failure.
upload_message=$(jq -r '."success-message" // empty' "$tmp/upload.json" 2>/dev/null) || upload_message=''
jq -n --arg artifact "${ipa##*/}" --arg sha256 "$digest" --arg bundle_id "$bundle_id" \
    --arg extension_bundle_id "$extension_id" --arg marketing_version "$marketing_version" \
    --arg build_number "$build_number" --arg team_id "$team" --arg app_group "$app_group" \
    --arg message "$upload_message" \
    '{artifact: $artifact, sha256: $sha256, bundle_id: $bundle_id, extension_bundle_id: $extension_bundle_id,
      marketing_version: $marketing_version, build_number: $build_number, team_id: $team_id,
      app_group: $app_group, testflight: "accepted", message: $message, kind: "ios-testflight"}' > "$tmp/receipt.json"
new_output "$ipa.json"
mv -n "$tmp/receipt.json" "$ipa.json"
[[ ! -e $tmp/receipt.json ]] || fail "Could not promote receipt: $ipa.json"
complete=1
printf 'ipa=%s\nsha256=%s\nbundle_id=%s\nmarketing_version=%s\nbuild_number=%s\nteam_id=%s\ntestflight=accepted\n' \
    "$ipa" "$digest" "$bundle_id" "$marketing_version" "$build_number" "$team"
