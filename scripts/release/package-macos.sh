#!/usr/bin/env bash
# ABOUTME: Assemble Spaceport previews with the app, chat adapter and pinned updater.
# ABOUTME: Validate architecture and compiled identity before writing a bundle.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# == 5 || $# == 6 ]] || fail 'Usage: bash package-macos.sh VERSION ARM64_BINARY X86_64_BINARY OUTPUT_DIR THIRD_PARTY_NOTICES [debug|release]'
version_check "$1"
[[ $(uname -s) == Darwin ]] || fail 'macOS required'
[[ -f $2 && ( -f $3 || $3 == - ) && -d $4 ]] || fail 'ARM64 binary, optional Intel binary (- for none), and an existing output directory required'
[[ -f $5 && -s $5 ]] || fail 'Nonempty third-party notices file required'
profile=${6:-release}
[[ $profile == debug || $profile == release ]] || fail 'Profile must be debug or release'
binaries=("$2")
archs=(arm64)
bundle_architecture=arm64
if [[ $3 != - ]]; then binaries+=("$3"); archs+=(x86_64); bundle_architecture=universal; fi
for binary in "${binaries[@]}"; do
    python3 "$release_root/scripts/release/preview-identity.py" "$1" "$profile" "$binary"
done
# The app starts chats with sheprd-acp beside its executable.
companions=(sheprd-acp)
for companion in "${companions[@]}"; do
    [[ -f $(dirname -- "$2")/$companion && ( $3 == - || -f $(dirname -- "$3")/$companion ) ]] || fail "A $companion beside each binary required"
done
[[ -n ${SPARKLE_FRAMEWORK_PATH:-} ]] || fail 'SPARKLE_FRAMEWORK_PATH must name the pinned Sparkle SDK directory'
python3 "$release_root/scripts/release/sparkle_framework.py" validate-sdk "$SPARKLE_FRAMEWORK_PATH"
out=$(cd -- "$4" && pwd)/Spaceport.app
new_output "$out"
icons=$(python3 "$release_root/scripts/release/build-icon.py" macos "${binaries[@]}")
icns=${icons%$'\n'*}
catalog=${icons##*$'\n'}
[[ -f $icns && -f $catalog ]] || fail 'Could not select the macOS icons'
[[ $(lipo -archs "$2") == arm64 ]] || fail 'First binary must be arm64 only'
[[ $3 == - || $(lipo -archs "$3") == x86_64 ]] || fail 'Second binary must be x86_64 only'
for companion in "${companions[@]}"; do
    [[ $(lipo -archs "$(dirname -- "$2")/$companion") == arm64 ]] || fail "First $companion must be arm64 only"
    [[ $3 == - || $(lipo -archs "$(dirname -- "$3")/$companion") == x86_64 ]] || fail "Second $companion must be x86_64 only"
done
tmp=$(mktemp -d "${out%/*}/.herdr-assembly.XXXXXX")
trap 'rm -rf -- "$tmp"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
app=$tmp/Spaceport.app
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
if [[ $3 == - ]]; then
    cp "$2" "$app/Contents/MacOS/spaceport"
else
    lipo -create "$2" "$3" -output "$app/Contents/MacOS/spaceport"
fi
for companion in "${companions[@]}"; do
    if [[ $3 == - ]]; then
        cp "$(dirname -- "$2")/$companion" "$app/Contents/MacOS/$companion"
    else
        lipo -create "$(dirname -- "$2")/$companion" "$(dirname -- "$3")/$companion" -output "$app/Contents/MacOS/$companion"
    fi
done
for executable in spaceport "${companions[@]}"; do
    for arch in "${archs[@]}"; do
        lipo "$app/Contents/MacOS/$executable" -verify_arch "$arch"
    done
    # Rust/linker outputs can contain ad-hoc signatures; distribution signing is separate.
    if codesign -d "$app/Contents/MacOS/$executable" 2>/dev/null; then
        codesign --remove-signature "$app/Contents/MacOS/$executable"
    fi
    chmod 755 "$app/Contents/MacOS/$executable"
done
cp "$release_root/assets/macos/Info.plist" "$app/Contents/Info.plist"
# Each translation's InfoPlist.strings (docs/i18n.md); English is Info.plist itself.
for lproj in "$release_root"/assets/macos/*.lproj; do
    if [[ -d "$lproj" ]]; then cp -R "$lproj" "$app/Contents/Resources/"; fi
done
plutil -insert CFBundleIdentifier -string "$(cat "$release_root/crates/sheprd-gpui/macos-preview-app-id")" "$app/Contents/Info.plist"
plutil -insert SpaceportBuildProfile -string "$profile" "$app/Contents/Info.plist"
plutil -replace CFBundleShortVersionString -string "$1" "$app/Contents/Info.plist"
plutil -replace CFBundleVersion -string "$1" "$app/Contents/Info.plist"
plutil -insert LSMinimumSystemVersion -string 14.2 "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
mkdir "$app/Contents/Frameworks"
# Copy the complete SDK framework, including version links and nested services.
ditto "$SPARKLE_FRAMEWORK_PATH/Sparkle.framework" "$app/Contents/Frameworks/Sparkle.framework"
python3 "$release_root/scripts/release/sparkle_framework.py" configure "$app"
python3 "$release_root/scripts/release/sparkle_framework.py" validate-bundle "$app" --architecture "$bundle_architecture"
cp "$icns" "$app/Contents/Resources/Sheprd.icns"
cp "$catalog" "$app/Contents/Resources/Assets.car"
cp "$release_root/crates/sheprd-protocol/LICENSE-APACHE" "$app/Contents/Resources/"
cp "$release_root/crates/sheprd-protocol/NOTICE.md" "$app/Contents/Resources/"
cp "$release_root/LICENSE" "$release_root/NOTICE" "$release_root/assets/icons/LICENSE-octicons" "$app/Contents/Resources/"
cp "$SPARKLE_FRAMEWORK_PATH/LICENSE" "$app/Contents/Resources/Sparkle-LICENSE.txt"
cp "$5" "$app/Contents/Resources/THIRD-PARTY-NOTICES.txt"
cp "$release_root/crates/sheprd-gpui/SOUND-NOTICE.md" "$app/Contents/Resources/"
mv "$app" "$out"
printf '%s\n' "$out"
